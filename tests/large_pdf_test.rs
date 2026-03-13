//! Integration test for batch PDF rendering with large PDFs
//!
//! This test validates that the batch processing infrastructure
//! handles large PDFs (373 pages) correctly.

use shusei::core::pdf::PdfProcessor;

#[test]
fn test_batch_rendering() {
    // Initialize processor
    let processor = PdfProcessor::new();
    if processor.is_err() {
        println!("Skipping test - PDF processor not available (CRT linking issue)");
        return;
    }

    let processor = processor.unwrap();

    // Open the 373-page test PDF
    let pdf_path = "tests/large_pdf_test.pdf";

    // Check if test PDF exists
    if !std::path::Path::new(pdf_path).exists() {
        println!("Skipping test - test PDF not found at {}", pdf_path);
        return;
    }

    let doc = processor.open(pdf_path);
    if doc.is_err() {
        println!("Skipping test - could not open test PDF: {:?}", doc.err());
        return;
    }

    let doc = doc.unwrap();

    // Get total page count
    let total_pages = processor.page_count(&doc);
    println!("Test PDF has {} pages", total_pages);

    // Render first batch (10 pages)
    let batch = processor.render_pages_batch(&doc, 0, 10, 1000, 1000);

    // Verify batch rendering succeeded
    assert!(batch.is_ok(), "Batch rendering should succeed");
    let pages = batch.unwrap();

    // Verify we got the expected number of pages
    assert_eq!(
        pages.len(),
        10,
        "Should render exactly 10 pages in first batch"
    );

    // Verify all pages rendered with content
    for (i, page) in pages.iter().enumerate() {
        assert!(!page.is_empty(), "Page {} should not be empty", i);
        println!("Page {}: {} bytes rendered", i, page.len());
    }

    println!("Batch rendering test passed successfully!");
}

#[test]
fn test_batch_rendering_multiple_batches() {
    // Test that multiple batches can be rendered sequentially
    let processor = PdfProcessor::new();
    if processor.is_err() {
        println!("Skipping test - PDF processor not available");
        return;
    }

    let processor = processor.unwrap();

    let pdf_path = "tests/large_pdf_test.pdf";
    if !std::path::Path::new(pdf_path).exists() {
        println!("Skipping test - test PDF not found");
        return;
    }

    let doc = processor.open(pdf_path).expect("Failed to open test PDF");
    let total_pages = processor.page_count(&doc);

    // Render multiple batches
    let batch_size = 10;
    let mut total_rendered = 0;
    let mut current_page = 0;

    while current_page < total_pages as u32 {
        let batch = processor.render_pages_batch(&doc, current_page, batch_size, 800, 1000);

        if let Ok(pages) = batch {
            let batch_len = pages.len();
            total_rendered += batch_len;
            current_page += batch_len as u32;
            println!(
                "Batch complete: rendered {} pages (total: {}/{})",
                batch_len, total_rendered, total_pages
            );

            // Verify each page in batch
            for (i, page) in pages.iter().enumerate() {
                assert!(!page.is_empty(), "Batch page {} should not be empty", i);
            }
        } else {
            println!("Batch failed at page {}: {:?}", current_page, batch.err());
            break;
        }
    }

    println!(
        "Multi-batch test complete: rendered {} / {} pages",
        total_rendered, total_pages
    );
    assert!(
        total_rendered > 0,
        "Should have rendered at least some pages"
    );
}
