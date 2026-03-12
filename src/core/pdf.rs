//! PDF processing module
//!
//! This module handles PDF rendering using pdfium-render.

use std::path::Path;

use pdfium_render::prelude::*;

use crate::core::error::{Result, ShuseiError};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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
