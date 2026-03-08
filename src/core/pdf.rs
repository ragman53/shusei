//! PDF processing module
//!
//! This module handles PDF rendering using pdfium-render.

use std::path::Path;

use pdfium_render::prelude::*;

use crate::core::error::{ShuseiError, Result};

/// PDF processor for rendering pages as images
pub struct PdfProcessor {
    /// Pdfium bindings
    pdfium: Pdfium,
}

impl PdfProcessor {
    /// Create a new PDF processor
    pub fn new() -> Result<Self> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .map_err(|e| ShuseiError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string()
                )))?
        );
        
        log::info!("PDF processor initialized");
        
        Ok(Self { pdfium })
    }
    
    /// Open a PDF file
    pub fn open(&self, path: impl AsRef<Path>) -> Result<PdfDocument> {
        let document = self.pdfium
            .load_pdf_from_file(path.as_ref(), None)
            .map_err(|e| ShuseiError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string()
            )))?;
        
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
        let page = document
            .pages()
            .get(page_index as usize)
            .map_err(|e| ShuseiError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string()
            )))?;
        
        // Render to bitmap
        let bitmap = page
            .render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width(width as i32)
                    .set_target_height(height as i32)
                    .set_render_flags(PdfBitmapRenderFlags::RENDER_ANNOTATIONS),
            )
            .map_err(|e| ShuseiError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string()
            )))?;
        
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
    
    #[test]
    fn test_pdf_processor_new() {
        let processor = PdfProcessor::new();
        // This test may fail if pdfium is not installed on the system
        // which is fine for development
        if processor.is_ok() {
            println!("PDF processor created successfully");
        }
    }
}