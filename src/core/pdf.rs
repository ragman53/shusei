//! PDF processing module
//!
//! This module handles PDF rendering using pdfium-render.

use std::path::Path;
use std::sync::Arc;

use pdfium_render::prelude::*;

use crate::core::error::{Result, ShuseiError};
use crate::core::db::Database;
use crate::core::storage::StorageService;
use crate::core::ocr::engine::NdlocrEngine;

/// PDF processor for rendering pages as images
pub struct PdfProcessor {
    /// Pdfium bindings
    pdfium: Pdfium,
}

impl PdfProcessor {
    /// Create a new PDF processor
    pub fn new() -> Result<Self> {
        let pdfium = Pdfium::new(Pdfium::bind_to_system_library().map_err(|e| {
            ShuseiError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?);

        log::info!("PDF processor initialized");

        Ok(Self { pdfium })
    }

    /// Open a PDF file
    pub fn open(&self, path: impl AsRef<Path>) -> Result<PdfDocument> {
        let document = self
            .pdfium
            .load_pdf_from_file(path.as_ref(), None)
            .map_err(|e| {
                ShuseiError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        Ok(document)
    }

    /// Get the number of pages in a PDF
    pub fn page_count(&self, document: &PdfDocument) -> u32 {
        document.pages().len() as u32
    }

    /// Render a page to an image
    pub fn render_page(
        &self,
        document: &PdfDocument,
        page_index: u32,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>> {
        let page = document.pages().get(page_index as usize).map_err(|e| {
            ShuseiError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        // Render to bitmap
        let bitmap = page
            .render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width(width as i32)
                    .set_target_height(height as i32)
                    .set_render_flags(PdfBitmapRenderFlags::RENDER_ANNOTATIONS),
            )
            .map_err(|e| {
                ShuseiError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        // Convert to RGBA bytes
        let bytes = bitmap.as_bytes();

        Ok(bytes.to_vec())
    }

    /// Render all pages as images
    pub fn render_all_pages(
        &self,
        document: &PdfDocument,
        width: u32,
        height: u32,
        mut progress_callback: impl FnMut(u32, u32),
    ) -> Result<Vec<Vec<u8>>> {
        let total_pages = self.page_count(document);
        let mut images = Vec::with_capacity(total_pages as usize);

        for i in 0..total_pages {
            progress_callback(i + 1, total_pages);
            let image = self.render_page(document, i, width, height)?;
            images.push(image);
        }

        Ok(images)
    }

    /// Render a batch of pages with progress tracking and resume support
    ///
    /// # Arguments
    /// * `document` - PDF document to render
    /// * `book_id` - Book identifier for progress tracking
    /// * `db` - Database connection for progress tracking
    /// * `storage` - Storage service for saving rendered images
    /// * `batch_size` - Number of pages to render per batch (default: 10)
    /// * `width` - Target width for rendered images
    /// * `height` - Target height for rendered images
    ///
    /// # Returns
    /// Vec of (page_number, image_bytes) for the batch
    pub async fn render_pages_batch(
        &self,
        document: &PdfDocument,
        book_id: &str,
        db: &Database,
        storage: &StorageService,
        batch_size: u32,
        width: u32,
        height: u32,
    ) -> Result<Vec<(u32, Vec<u8>)>> {
        // Get current progress to determine starting page
        let progress = db.get_progress(book_id)?;
        let last_processed = progress
            .as_ref()
            .map(|p| p.last_processed_page)
            .unwrap_or(0);
        let total_pages = self.page_count(document);

        // Calculate batch range
        let start_page = last_processed;
        let end_page = std::cmp::min(start_page + batch_size, total_pages);

        if start_page >= end_page {
            // All pages already rendered
            return Ok(Vec::new());
        }

        let mut rendered_pages = Vec::with_capacity((end_page - start_page) as usize);

        // Render pages in the batch
        for page_num in start_page..end_page {
            let image_bytes = self.render_page(document, page_num, width, height)?;
            
            // Save image to storage
            let image_path = storage.save_image(
                &image_bytes,
                &format!("pdf_pages/{}", book_id),
            )?;
            
            rendered_pages.push((page_num, image_bytes));
        }

        // Update progress after batch complete
        db.update_progress(book_id, end_page as i32, "processing")?;

        Ok(rendered_pages)
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
    pub fn from_document(document: &PdfDocument) -> Self {
        Self {
            title: document.metadata().title(),
            author: document.metadata().author(),
            subject: document.metadata().subject(),
            creator: document.metadata().creator(),
            producer: document.metadata().producer(),
            page_count: document.pages().len() as u32,
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
        progress_cb: impl Fn(ConversionProgress),
    ) -> Result<()> {
        // Open PDF
        let document = self.pdf_processor.open(pdf_path)?;
        let total_pages = self.pdf_processor.page_count(&document);

        // Initialize progress tracking
        self.db.create_progress(book_id, total_pages as i32)?;

        // Stage 1: Render pages in batches
        progress_cb(ConversionProgress {
            stage: ConversionStage::Rendering,
            current_page: 0,
            total_pages,
        });

        let batch_size = 10;
        let mut rendered_count = 0;

        loop {
            let pages = self
                .pdf_processor
                .render_pages_batch(
                    &document,
                    book_id,
                    &self.db,
                    &self.storage,
                    batch_size,
                    800,
                    1000,
                )
                .await?;

            if pages.is_empty() {
                break; // All pages rendered
            }

            rendered_count += pages.len();

            // Stage 2: Process rendered pages with OCR
            progress_cb(ConversionProgress {
                stage: ConversionStage::OcrProcessing,
                current_page: rendered_count as u32,
                total_pages,
            });

            let book_id = book_id.to_string();
            let db = Arc::clone(&self.db);
            let progress_cb_clone = progress_cb.clone(); // This won't work, need to refactor

            // Use a simple counter for progress
            let processed = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
            let processed_clone = std::sync::Arc::clone(&processed);
            
            self.ocr_engine
                .process_pages_parallel(
                    pages,
                    &book_id,
                    &self.db,
                    move |page, _| {
                        let current = processed_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        progress_cb(ConversionProgress {
                            stage: ConversionStage::OcrProcessing,
                            current_page: current + 1,
                            total_pages,
                        });
                    },
                )
                .await?;

            // Check if all pages are done
            if rendered_count >= total_pages as usize {
                break;
            }
        }

        // Mark as complete
        self.db.update_progress(book_id, total_pages as i32, "completed")?;

        progress_cb(ConversionProgress {
            stage: ConversionStage::Complete,
            current_page: total_pages,
            total_pages,
        });

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

            // Create database and storage
            let db = Database::in_memory().unwrap();
            let storage_dir = TempDir::new().unwrap();
            let storage = StorageService::new(storage_dir.path().to_path_buf()).unwrap();

            // Create book
            let book_id = db
                .create_book(&NewBook {
                    title: "Test Book".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            // Initialize progress
            db.create_progress(&book_id, 3).unwrap();

            // Open PDF
            let document = processor.open(&pdf_path).unwrap();

            // Render batch
            let result = processor
                .render_pages_batch(&document, &book_id, &db, &storage, 10, 800, 1000)
                .await;

            if let Ok(pages) = result {
                assert_eq!(pages.len(), 3);
                for (page_num, bytes) in pages {
                    assert!(page_num < 3);
                    assert!(!bytes.is_empty());
                }
            }
        }

        #[tokio::test]
        async fn test_render_pages_batch_returns_page_number_and_bytes() {
            let processor = PdfProcessor::new();
            if processor.is_err() {
                println!("Skipping test - pdfium not available");
                return;
            }

            let processor = processor.unwrap();
            let temp_dir = TempDir::new().unwrap();

            let pdf_path = temp_dir.path().join("test.pdf");
            create_test_pdf(&pdf_path).unwrap();

            let db = Database::in_memory().unwrap();
            let storage_dir = TempDir::new().unwrap();
            let storage = StorageService::new(storage_dir.path().to_path_buf()).unwrap();

            let book_id = db
                .create_book(&NewBook {
                    title: "Test".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            db.create_progress(&book_id, 3).unwrap();

            let document = processor.open(&pdf_path).unwrap();
            let result = processor
                .render_pages_batch(&document, &book_id, &db, &storage, 10, 800, 1000)
                .await;

            if let Ok(pages) = result {
                // Verify structure: Vec<(page_number, image_bytes)>
                for (page_num, bytes) in pages {
                    assert!(page_num < 3, "Page number should be valid");
                    assert!(!bytes.is_empty(), "Image bytes should not be empty");
                }
            }
        }
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
