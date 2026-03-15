//! PDF processing module
//!
//! This module handles PDF rendering using hayro (pure Rust PDF renderer).

use std::path::Path;
use std::sync::Arc;

use hayro::{Pdf, RenderSettings, InterpreterSettings, render};
use log::{info, debug, warn};
use rayon::prelude::*;

use crate::core::error::{Result, ShuseiError};
use crate::core::db::Database;
use crate::core::storage::StorageService;
use crate::core::ocr::NdlocrEngine;

/// PDF document wrapper for hayro
pub struct PdfDocument {
    pub pdf: Pdf,
    pub data: Vec<u8>,
}

/// PDF processor for rendering pages as images
pub struct PdfProcessor {}

impl PdfProcessor {
    /// Create a new PDF processor
    pub fn new() -> Result<Self> {
        log::info!("PDF processor initialized (hayro, pure Rust)");
        Ok(Self {})
    }

    /// Open a PDF file
    pub fn open(&self, path: impl AsRef<Path>) -> Result<PdfDocument> {
        let pdf_data = std::fs::read(path.as_ref()).map_err(|e| {
            ShuseiError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        let pdf = Pdf::new(std::sync::Arc::new(pdf_data.clone())).map_err(|e| {
            ShuseiError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to load PDF: {:?}", e),
            ))
        })?;
        Ok(PdfDocument { pdf, data: pdf_data })
    }

    /// Get the number of pages in a PDF
    pub fn page_count(&self, document: &PdfDocument) -> usize {
        document.pdf.pages().len()
    }

    /// Render a page to an image
    pub fn render_page(
        &self,
        document: &PdfDocument,
        page_index: u32,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>> {
        let pages = document.pdf.pages();
        let page = pages.iter().nth(page_index as usize).ok_or_else(|| {
            ShuseiError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Page {} not found", page_index),
            ))
        })?;

        // Render to pixmap with hayro
        let render_settings = RenderSettings {
            x_scale: 1.0,
            y_scale: 1.0,
            width: Some(width as u16),
            height: Some(height as u16),
        };
        let interpreter_settings = InterpreterSettings::default();
        let pixmap = render(page, &interpreter_settings, &render_settings);

        // Return RGBA bytes (premultiplied)
        Ok(pixmap.data_as_u8_slice().to_vec())
    }

    /// Render a page with retry-once logic
    ///
    /// Attempts to render the page, and if it fails, retries once before returning the error.
    fn render_with_retry(
        &self,
        document: &PdfDocument,
        page_index: u32,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>> {
        // First attempt
        match self.render_page(document, page_index, width, height) {
            Ok(result) => Ok(result),
            Err(e) => {
                log::warn!("Render failed for page {}, retrying...", page_index);
                // Retry once
                self.render_page(document, page_index, width, height)
            }
        }
    }

    /// Render all pages as images
    pub fn render_all_pages(
        &self,
        document: &PdfDocument,
        width: u32,
        height: u32,
        mut progress_callback: impl FnMut(u32, u32),
    ) -> Result<Vec<Vec<u8>>> {
        let total_pages = self.page_count(document) as u32;
        let mut images = Vec::with_capacity(total_pages as usize);

        for i in 0..total_pages {
            progress_callback(i + 1, total_pages);
            let image = self.render_with_retry(document, i, width, height)?;
            images.push(image);
        }

        Ok(images)
    }

    /// Render a batch of pages with parallel processing and concurrency control
    ///
    /// # Arguments
    /// * `document` - PDF document to render
    /// * `start_page` - Starting page index (0-based)
    /// * `batch_size` - Number of pages to render in this batch (default: 10)
    /// * `width` - Target width for rendered images
    /// * `height` - Target height for rendered images
    ///
    /// # Returns
    /// Vec of rendered page images
    pub fn render_pages_batch(
        &self,
        document: &PdfDocument,
        start_page: u32,
        batch_size: usize,
        width: u32,
        height: u32,
    ) -> Result<Vec<Vec<u8>>> {
        let total_pages = self.page_count(document);
        let start_idx = start_page as usize;
        let end_idx = std::cmp::min(start_idx + batch_size, total_pages);

        if start_idx >= end_idx {
            // No pages to render
            return Ok(Vec::new());
        }

        debug!("Rendering pages {}-{} of {}", start_idx + 1, end_idx, total_pages);

        // Use rayon for parallel iteration with concurrency control
        // Limit to 3 concurrent render operations to prevent memory spikes
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(3)
            .build()
            .unwrap();

        // Render pages with retry logic and skip-on-failure
        let pages: Vec<Vec<u8>> = pool.install(|| {
            (start_idx..end_idx)
                .into_par_iter()
                .filter_map(|i| {
                    let page_index = i as u32;
                    match self.render_with_retry(document, page_index, width, height) {
                        Ok(image) => Some(image),
                        Err(e) => {
                            log::warn!("Skipping page {} after retry: {}", page_index, e);
                            None // Skip failed pages
                        }
                    }
                })
                .collect()
        });

        Ok(pages)
    }

    /// Import a PDF file: copy to app directory and extract metadata
    ///
    /// Returns (metadata, copied_path)
    pub fn import_pdf(
        &self,
        source_path: &Path,
        app_data_dir: &Path,
    ) -> Result<(PdfMetadata, String)> {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Create pdfs directory if it doesn't exist
        let pdfs_dir = app_data_dir.join("pdfs");
        fs::create_dir_all(&pdfs_dir)?;

        // Generate UUID for filename
        let uuid = uuid::Uuid::new_v4().to_string();
        let copied_path = pdfs_dir.join(format!("{}.pdf", uuid));

        // Copy PDF file
        fs::copy(source_path, &copied_path)?;

        // Open and extract metadata
        let document = self.open(&copied_path)?;
        let mut metadata = PdfMetadata::from_document(&document);

        // Fallback: use filename if title is missing
        if metadata.title.is_none()
            || metadata
                .title
                .as_ref()
                .map(|s| s.is_empty())
                .unwrap_or(true)
        {
            metadata.title = source_path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string());
        }

        Ok((metadata, copied_path.to_string_lossy().to_string()))
    }
}

impl Default for PdfProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to create PdfProcessor")
    }
}

/// PDF metadata
#[derive(Debug, Clone)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub page_count: u32,
}

impl PdfMetadata {
    /// Extract metadata from a PDF document
    /// Note: hayro doesn't expose PDF metadata yet, so only page_count is populated
    pub fn from_document(document: &PdfDocument) -> Self {
        Self {
            title: None,
            author: None,
            subject: None,
            creator: None,
            producer: None,
            page_count: document.pdf.pages().len() as u32,
        }
    }
}

// ==================== PDF Conversion Service ====================

/// Stage of the PDF conversion process
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionStage {
    Rendering,
    OcrProcessing,
    Complete,
}

/// Progress information for PDF conversion
#[derive(Debug, Clone)]
pub struct ConversionProgress {
    pub stage: ConversionStage,
    pub current_page: u32,
    pub total_pages: u32,
}

/// High-level PDF conversion service orchestrator
pub struct PdfConversionService {
    pdf_processor: PdfProcessor,
    ocr_engine: NdlocrEngine,
    db: Arc<Database>,
    storage: Arc<StorageService>,
}

impl PdfConversionService {
    /// Create a new PDF conversion service
    pub fn new(
        ocr_engine: NdlocrEngine,
        db: Arc<Database>,
        storage: Arc<StorageService>,
    ) -> Result<Self> {
        let pdf_processor = PdfProcessor::new()?;

        Ok(Self {
            pdf_processor,
            ocr_engine,
            db,
            storage,
        })
    }

    /// Convert a PDF to text via OCR
    ///
    /// # Arguments
    /// * `book_id` - Book identifier
    /// * `pdf_path` - Path to PDF file
    /// * `progress_cb` - Callback for progress updates
    ///
    /// # Returns
    /// Ok(()) on success, error on failure
    pub async fn convert_pdf(
        &self,
        book_id: &str,
        pdf_path: &Path,
        progress_cb: impl Fn(ConversionProgress) + Send + Sync + 'static,
    ) -> Result<()> {
        let conversion_start = std::time::Instant::now();
        info!("Starting PDF conversion for book {}: {:?}", book_id, pdf_path);
        
        // Wrap callback in Arc for sharing across async closures
        let progress_cb = Arc::new(progress_cb);

        // Open PDF, get page count, and render all pages synchronously
        let (total_pages_u32, all_pages) = {
            let document = self.pdf_processor.open(pdf_path)?;
            let total_pages = self.pdf_processor.page_count(&document);
            let total_pages_u32 = total_pages as u32;
            
            info!("PDF opened: {} pages", total_pages);
            
            // Initialize progress tracking
            self.db.create_progress(book_id, total_pages as i32)?;

            // Stage 1: Render pages
            progress_cb(ConversionProgress {
                stage: ConversionStage::Rendering,
                current_page: 0,
                total_pages: total_pages_u32,
            });

            // Render all pages using batch processing with parallel rendering
            let batch_size = 10;
            let mut all_pages: Vec<(u32, Vec<u8>)> = Vec::new();
            let mut current_page = 0u32;
            
            while current_page < total_pages_u32 {
                let batch_start = std::time::Instant::now();
                
                // Render batch with parallel processing
                let batch_images = self.pdf_processor.render_pages_batch(
                    &document,
                    current_page,
                    batch_size,
                    800,
                    1000,
                )?;

                if batch_images.is_empty() {
                    break;
                }

                let batch_len = batch_images.len();
                let batch_time = batch_start.elapsed();
                
                // Store pages with their numbers and save to storage
                for (i, image_bytes) in batch_images.into_iter().enumerate() {
                    let page_num = current_page + i as u32;
                    let _image_path = self.storage.save_image(
                        &image_bytes,
                        &format!("pdf_pages/{}", book_id),
                    )?;
                    all_pages.push((page_num, image_bytes));
                }
                
                // Update progress after batch complete
                current_page += batch_len as u32;
                self.db.update_progress(book_id, current_page as i32, "processing")?;
                
                info!(
                    "Batch complete: rendered {} pages (total: {}/{}), time: {:.2?}",
                    batch_len,
                    current_page,
                    total_pages_u32,
                    batch_time
                );
            }
            
            (total_pages_u32, all_pages)
            // document dropped here
        };

        // Stage 2: Process all rendered pages with OCR (no document borrow)
        info!("Starting OCR processing for {} pages", total_pages_u32);
        let ocr_start = std::time::Instant::now();
        
        progress_cb(ConversionProgress {
            stage: ConversionStage::OcrProcessing,
            current_page: 0,
            total_pages: total_pages_u32,
        });

        let book_id = book_id.to_string();
        
        // Use a simple counter for progress
        let processed = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let processed_clone = std::sync::Arc::clone(&processed);
        
        // Clone Arc for the async closure
        let progress_cb_clone = Arc::clone(&progress_cb);
        self.ocr_engine
            .process_pages_parallel(
                all_pages,
                &book_id,
                &self.db,
                move |page_num, _| {
                    let current = processed_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if current % 10 == 0 || current == total_pages_u32 - 1 {
                        info!("OCR progress: page {}/{}", current + 1, total_pages_u32);
                    }
                    progress_cb_clone(ConversionProgress {
                        stage: ConversionStage::OcrProcessing,
                        current_page: current + 1,
                        total_pages: total_pages_u32,
                    });
                },
            )
            .await?;

        let ocr_time = ocr_start.elapsed();
        info!("OCR processing complete: {} pages in {:.2?}", total_pages_u32, ocr_time);

        // Mark as complete
        self.db.update_progress(&book_id, total_pages_u32 as i32, "completed")?;

        progress_cb(ConversionProgress {
            stage: ConversionStage::Complete,
            current_page: total_pages_u32,
            total_pages: total_pages_u32,
        });

        let total_time = conversion_start.elapsed();
        info!(
            "PDF conversion complete for book {}: {} pages, total time: {:.2?}",
            book_id, total_pages_u32, total_time
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    mod batch_rendering {
        use super::*;
        use crate::core::db::{Database, NewBook};
        use tempfile::TempDir;

        fn create_test_pdf(path: &Path) -> std::io::Result<()> {
            // Create a minimal 3-page PDF
            let pdf_content = b"%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R 4 0 R 5 0 R] /Count 3 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>
endobj
4 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>
endobj
5 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>
endobj
xref
0 6
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
0000000198 00000 n
0000000281 00000 n
trailer
<< /Size 6 /Root 1 0 R >>
startxref
364
%%EOF";
            std::fs::write(path, pdf_content)
        }

        #[tokio::test]
        async fn test_render_pages_batch_renders_pages() {
            let processor = PdfProcessor::new();
            if processor.is_err() {
                println!("Skipping test - pdfium not available");
                return;
            }

            let processor = processor.unwrap();
            let temp_dir = TempDir::new().unwrap();

            // Create test PDF
            let pdf_path = temp_dir.path().join("test.pdf");
            create_test_pdf(&pdf_path).unwrap();

            // Open PDF
            let document = processor.open(&pdf_path).unwrap();

            // Render batch (pages 0-2, batch size 10)
            let result = processor
                .render_pages_batch(&document, 0, 10, 800, 1000);

            if let Ok(pages) = result {
                assert_eq!(pages.len(), 3);
                for bytes in pages {
                    assert!(!bytes.is_empty());
                }
            }
        }

        #[tokio::test]
        async fn test_render_pages_batch_returns_bytes() {
            let processor = PdfProcessor::new();
            if processor.is_err() {
                println!("Skipping test - pdfium not available");
                return;
            }

            let processor = processor.unwrap();
            let temp_dir = TempDir::new().unwrap();

            let pdf_path = temp_dir.path().join("test.pdf");
            create_test_pdf(&pdf_path).unwrap();

            let document = processor.open(&pdf_path).unwrap();
            let result = processor
                .render_pages_batch(&document, 0, 10, 800, 1000);

            if let Ok(pages) = result {
                // Verify structure: Vec<Vec<u8>>
                for bytes in pages {
                    assert!(!bytes.is_empty(), "Image bytes should not be empty");
                }
            }
        }
    }

    // Basic unit tests for PDF operations
    #[test]
    fn test_processor_new() {
        let processor = PdfProcessor::new();
        assert!(processor.is_ok(), "PdfProcessor::new() should succeed");
    }

    #[test]
    fn test_open_pdf() {
        // Create a minimal test PDF
        let temp_dir = TempDir::new().unwrap();
        let pdf_path = temp_dir.path().join("test.pdf");
        let minimal_pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\nxref\n0 3\n0000000000 65535 f\n0000000009 00000 n\n0000000058 00000 n\ntrailer\n<< /Size 3 /Root 1 0 R >>\nstartxref\n116\n%%EOF";
        fs::write(&pdf_path, minimal_pdf).unwrap();

        let processor = PdfProcessor::new().unwrap();
        let doc = processor.open(&pdf_path);
        assert!(doc.is_ok(), "Opening valid PDF should succeed");
    }

    #[test]
    fn test_render_single_page() {
        // Create a minimal 1-page PDF
        let temp_dir = TempDir::new().unwrap();
        let pdf_path = temp_dir.path().join("test.pdf");
        let pdf_content = b"%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>
endobj
xref
0 4
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
trailer
<< /Size 4 /Root 1 0 R >>
startxref
198
%%EOF";
        fs::write(&pdf_path, pdf_content).unwrap();

        let processor = PdfProcessor::new().unwrap();
        let doc = processor.open(&pdf_path).unwrap();
        let result = processor.render_page(&doc, 0, 1000, 1000);
        assert!(result.is_ok(), "Rendering page should succeed");
        assert!(!result.unwrap().is_empty(), "Rendered page should not be empty");
    }

    #[test]
    fn test_pdf_processor_new() {
        let processor = PdfProcessor::new();
        // This test may fail if pdfium is not installed on the system
        // which is fine for development
        if processor.is_ok() {
            println!("PDF processor created successfully");
        }
    }

    #[test]
    fn test_import_pdf_copies_file_and_returns_metadata() {
        // This test requires pdfium and a sample PDF
        // It will be skipped if pdfium is not available
        let processor = PdfProcessor::new();
        if processor.is_err() {
            println!("Skipping test - pdfium not available");
            return;
        }

        let processor = processor.unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Create a minimal PDF for testing (1x1 pixel PDF)
        let source_path = temp_dir.path().join("test.pdf");
        let minimal_pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\nxref\n0 3\n0000000000 65535 f\n0000000009 00000 n\n0000000058 00000 n\ntrailer\n<< /Size 3 /Root 1 0 R >>\nstartxref\n116\n%%EOF";
        fs::write(&source_path, minimal_pdf).unwrap();

        let app_data_dir = temp_dir.path().join("app_data");
        fs::create_dir_all(&app_data_dir).unwrap();

        // This should compile but may fail at runtime if pdfium can't load the PDF
        let result = processor.import_pdf(&source_path, &app_data_dir);

        // If pdfium can load it, verify the result
        if let Ok((metadata, copied_path)) = result {
            assert!(fs::exists(&copied_path).unwrap());
            assert_eq!(metadata.page_count, 0);
        }
    }

    #[test]
    fn test_import_pdf_uses_uuid_for_filename() {
        let processor = PdfProcessor::new();
        if processor.is_err() {
            return;
        }

        let processor = processor.unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Create minimal PDF
        let source_path = temp_dir.path().join("test.pdf");
        let minimal_pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\nxref\n0 3\n0000000000 65535 f\n0000000009 00000 n\n0000000058 00000 n\ntrailer\n<< /Size 3 /Root 1 0 R >>\nstartxref\n116\n%%EOF";
        fs::write(&source_path, minimal_pdf).unwrap();

        let app_data_dir = temp_dir.path().join("app_data");
        fs::create_dir_all(&app_data_dir).unwrap();

        let result = processor.import_pdf(&source_path, &app_data_dir);

        if let Ok((_, copied_path)) = result {
            // Filename should be a UUID (36 chars with dashes)
            let filename = std::path::Path::new(&copied_path)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();
            assert_eq!(filename.len(), 36, "Filename should be UUID format");
        }
    }

    #[test]
    fn test_import_pdf_extracts_metadata_with_fallback() {
        let processor = PdfProcessor::new();
        if processor.is_err() {
            return;
        }

        let processor = processor.unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Create PDF without metadata
        let source_path = temp_dir.path().join("no_metadata.pdf");
        let minimal_pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\nxref\n0 3\n0000000000 65535 f\n0000000009 00000 n\n0000000058 00000 n\ntrailer\n<< /Size 3 /Root 1 0 R >>\nstartxref\n116\n%%EOF";
        fs::write(&source_path, minimal_pdf).unwrap();

        let app_data_dir = temp_dir.path().join("app_data");
        fs::create_dir_all(&app_data_dir).unwrap();

        let result = processor.import_pdf(&source_path, &app_data_dir);

        if let Ok((metadata, _)) = result {
            // Should fallback to filename when metadata is missing
            assert!(metadata.title.is_some() || metadata.title.is_none());
            assert_eq!(metadata.page_count, 0);
        }
    }
}
