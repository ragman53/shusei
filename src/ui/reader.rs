//! Reader page component
//!
//! This component provides the PDF reading experience with reflow support.

use dioxus::prelude::*;
use std::sync::Arc;
use crate::app::Route;
use crate::core::db::{Book, BookPage, Database};
use crate::core::pdf::{PdfConversionService, ConversionProgress, ConversionStage};
use crate::core::ocr::NdlocrEngine;
use crate::core::storage::StorageService;
use crate::ui::components::{PageJumpModal, ConversionProgressDisplay};

/// Reader page component - shows library of PDF books
#[component]
pub fn ReaderPage() -> Element {
    rsx! {
        div { class: "flex flex-col h-full",
            header { class: "bg-purple-600 text-white p-4",
                div { class: "flex items-center",
                    Link {
                        to: Route::Home,
                        class: "mr-4 text-white",
                        "←"
                    }
                    h1 { class: "text-xl font-bold", "📖 My Library" }
                }
            }
            
            // Navigate to library screen for full functionality
            div { class: "flex-1 flex items-center justify-center p-4",
                div { class: "text-center",
                    p { class: "text-gray-600 mb-4", "Visit the main library to manage your books" }
                    Link {
                        to: Route::BookList,
                        class: "bg-purple-600 text-white px-6 py-2 rounded-lg",
                        "Go to Library"
                    }
                }
            }
        }
    }
}

/// Reader view for a specific book with reflow text display
#[component]
pub fn ReaderBookView(book_id: i64) -> Element {
    let mut book = use_signal(|| Option::<Book>::None);
    let mut pages = use_signal(|| Vec::<BookPage>::new());
    let mut is_loading = use_signal(|| true);
    let mut error = use_signal(|| Option::<String>::None);
    let mut font_size = use_signal(|| 18); // Default 18px, range 12-32px
    let mut show_page_jump = use_signal(|| false);
    let mut current_page = use_signal(|| 1);
    let mut is_converting = use_signal(|| false);
    let mut conversion_progress = use_signal(|| Option::<ConversionProgress>::None);
    let mut conversion_error = use_signal(|| Option::<String>::None);
    
    // Load book and pages on mount
    use_effect(move || {
        spawn(async move {
            // Run blocking DB operations in a separate task
            let result = tokio::task::spawn_blocking(move || {
                match Database::open("shusei.db") {
                    Ok(db) => {
                        // Load book
                        let book_result = db.get_book(&book_id.to_string());
                        // Load pages
                        let pages_result = db.get_pages_by_book(&book_id.to_string());
                        
                        match (book_result, pages_result) {
                            (Ok(Some(b)), Ok(p)) => Some((b, p)),
                            _ => None,
                        }
                    }
                    Err(_) => None,
                }
            }).await;
            
            match result {
                Ok(Some((b, p))) => {
                    book.set(Some(b));
                    pages.set(p);
                }
                _ => {
                    error.set(Some("Failed to load book data".to_string()));
                }
            }
            is_loading.set(false);
        });
    });
    
    rsx! {
        div { class: "flex flex-col h-full bg-gray-50",
            // Header
            header { class: "bg-purple-600 text-white p-4 shadow-md",
                div { class: "flex items-center justify-between",
                    div { class: "flex items-center space-x-3",
                        Link {
                            to: Route::Reader,
                            class: "text-white hover:text-purple-200",
                            "←"
                        }
                        if let Some(b) = book() {
                            h1 { class: "text-xl font-bold", "{b.title}" }
                        }
                        // Page jump button
                        if !pages().is_empty() {
                            button {
                                class: "bg-purple-500 hover:bg-purple-400 px-3 py-1 rounded text-sm",
                                onclick: move |_| show_page_jump.set(true),
                                "#{current_page()}"
                            }
                        }
                    }
                    div { class: "flex items-center space-x-4",
                        // Progress indicator
                        if !pages().is_empty() {
                            span { class: "text-sm", 
                                "Page {current_page()} of {pages().len()}"
                            }
                        }
                        // Font size control
                        div { class: "flex items-center space-x-2",
                            span { class: "text-sm", "{font_size()}px" }
                            input {
                                r#type: "range",
                                min: "12",
                                max: "32",
                                value: "{font_size()}",
                                oninput: move |e| {
                                    if let Ok(size) = e.value().parse::<i32>() {
                                        font_size.set(size);
                                    }
                                },
                                class: "w-32 h-2 bg-purple-300 rounded-lg appearance-none cursor-pointer"
                            }
                        }
                    }
                }
            }
            
            // Content
            div { class: "flex-1 overflow-y-auto",
                if is_loading() {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600" }
                    }
                } else if let Some(err) = error() {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "bg-red-100 border border-red-400 text-red-700 px-6 py-4 rounded-lg",
                            p { class: "font-semibold", "Error Loading Book" }
                            p { class: "text-sm mt-1", "{err}" }
                        }
                    }
                } else if pages().is_empty() {
                    // Empty state - no pages converted yet
                    div { class: "flex items-center justify-center h-full p-4",
                        div { class: "text-center max-w-md",
                            p { class: "text-4xl mb-4", "📄" }
                            p { class: "text-gray-600 text-lg mb-2", "This PDF hasn't been converted yet" }
                            p { class: "text-gray-500 text-sm mb-4", 
                                "Convert pages to start reading."
                            }
                            if is_converting() {
                                div { class: "mb-4 max-w-sm",
                                    if let Some(progress) = conversion_progress() {
                                        ConversionProgressDisplay {
                                            stage: progress.stage,
                                            current_page: progress.current_page,
                                            total_pages: progress.total_pages,
                                        }
                                    } else {
                                        div { class: "text-center",
                                            div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600 mx-auto mb-2" }
                                            p { class: "text-sm text-gray-600", "Starting conversion..." }
                                        }
                                    }
                                }
                            } else {
                                button {
                                    class: "bg-purple-600 text-white px-6 py-2 rounded-lg hover:bg-purple-700",
                                    onclick: move |_| {
                                        spawn(async move {
                                            is_converting.set(true);
                                            conversion_error.set(None);
                                            
                                            // Get app data directory
                                            let app_data_dir = std::env::current_exe()
                                                .ok()
                                                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                                                .unwrap_or_else(|| std::path::PathBuf::from("."));
                                            
                                            // Construct PDF path from book ID (assuming stored as pdfs/{id}.pdf)
                                            let pdf_path = app_data_dir.join("pdfs").join(format!("{}.pdf", book_id));
                                            
                                            if !pdf_path.exists() {
                                                conversion_error.set(Some("PDF file not found".to_string()));
                                                is_converting.set(false);
                                                return;
                                            }
                                            
                                            // Initialize conversion service
                                            let ocr = NdlocrEngine::new(&app_data_dir);
                                            match (Database::open("shusei.db"), StorageService::new(app_data_dir.clone())) {
                                                (Ok(db), Ok(storage)) => {
                                                    let service = PdfConversionService::new(
                                                        ocr,
                                                        Arc::new(db),
                                                        Arc::new(storage),
                                                    );
                                                    
                                                    match service {
                                                        Ok(conv_service) => {
                                                            let book_id_str = book_id.to_string();
                                                            
                                                            // Simple conversion without progress callback (progress shown via is_converting state)
                                                            // Progress callback requires Send+Sync which conflicts with Dioxus signals
                                                            match conv_service.convert_pdf(&book_id_str, &pdf_path, |_| {
                                                                // No-op progress callback - UI shows generic "converting" state
                                                            }).await {
                                                                Ok(_) => {
                                                                    log::info!("Conversion complete");
                                                                    // Reload pages
                                                                    if let Ok(db) = Database::open("shusei.db") {
                                                                        if let Ok(loaded_pages) = db.get_pages_by_book(&book_id_str) {
                                                                            pages.set(loaded_pages);
                                                                        }
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    log::error!("Conversion failed: {:?}", e);
                                                                    conversion_error.set(Some(format!("Conversion failed: {}", e)));
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            log::error!("Failed to create conversion service: {:?}", e);
                                                            conversion_error.set(Some(format!("Failed to initialize: {}", e)));
                                                        }
                                                    }
                                                }
                                                _ => {
                                                    conversion_error.set(Some("Failed to initialize conversion services".to_string()));
                                                }
                                            }
                                            
                                            is_converting.set(false);
                                        });
                                    },
                                    "Convert"
                                }
                            }
                            if let Some(err) = conversion_error() {
                                p { class: "text-red-600 text-sm mt-2", "{err}" }
                            }
                        }
                    }
                } else {
                    // Continuous scroll view with all pages
                    div { 
                        class: "max-w-2xl mx-auto p-4 space-y-6",
                        style: "font-size: {font_size()}px",
                        onscroll: move |e| {
                            // Update current page based on scroll position
                            let scroll_y = e.scroll_top() as f32;
                            let pages_len = pages().len();
                            if pages_len > 0 {
                                // Simple heuristic: estimate current page based on scroll position
                                let total_height = e.scroll_height() as f32;
                                let page_height = total_height / pages_len as f32;
                                let estimated_page = ((scroll_y / page_height) + 1.0) as i32;
                                current_page.set(estimated_page.min(pages_len as i32).max(1));
                            }
                        },
                        for (idx, page) in pages().into_iter().enumerate() {
                            // Page content with id for scrolling
                            div { 
                                class: "bg-white rounded-lg shadow-sm p-6",
                                id: "page-{page.page_number}",
                                div { class: "prose max-w-none",
                                    // Render OCR markdown as HTML
                                    div { dangerous_inner_html: render_markdown(&page.ocr_markdown) }
                                }
                            }
                            
                            // Page separator with page number
                            div { class: "flex items-center justify-center",
                                div { class: "flex items-center space-x-4",
                                    div { class: "h-px bg-gray-300 w-16" }
                                    span { class: "text-gray-500 text-sm font-medium", 
                                        "Page {page.page_number}"
                                    }
                                    div { class: "h-px bg-gray-300 w-16" }
                                }
                            }
                        }
                    }
                }
            }
            
            // Page jump modal
            PageJumpModal {
                show: show_page_jump(),
                total_pages: pages().len() as i32,
                on_close: move |_| show_page_jump.set(false),
                on_submit: move |page_num| {
                    // Update current page (scroll handled by re-render)
                    current_page.set(page_num);
                }
            }
        }
    }
}

/// Simple markdown to HTML renderer
fn render_markdown(md: &str) -> String {
    // Basic markdown parsing for common elements
    let mut html = md.to_string();
    
    // Headers
    html = html.replace("\n# ", "\n<h1>").replace("\n", "</h1>\n");
    html = html.replace("\n## ", "\n<h2>").replace("\n", "</h2>\n");
    html = html.replace("\n### ", "\n<h3>").replace("\n", "</h3>\n");
    
    // Bold
    html = html.replace("**", "<strong>").replace("**", "</strong>");
    
    // Line breaks
    html = html.replace("\n\n", "</p><p>").replace("\n", "<br/>");
    
    format!("<p>{}</p>", html)
}