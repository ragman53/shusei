//! Reader page component
//!
//! This component provides the PDF reading experience with reflow support.

use dioxus::prelude::*;
use crate::app::Route;
use crate::core::db::{Book, BookPage, Database};
use crate::ui::components::PageJumpModal;

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
                            p { class: "text-gray-600 text-lg mb-2", "No pages converted yet" }
                            p { class: "text-gray-500 text-sm mb-4", 
                                "This book hasn't been processed. Convert pages to start reading."
                            }
                            Link {
                                to: Route::BookList,
                                class: "inline-block bg-purple-600 text-white px-6 py-2 rounded-lg",
                                "Go to Library"
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