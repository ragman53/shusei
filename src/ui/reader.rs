//! Reader page component
//!
//! This component provides the PDF reading experience with reflow support.

use dioxus::prelude::*;

/// Reader page component
#[component]
pub fn ReaderPage() -> Element {
    // State for books list
    let mut books = use_signal(|| Vec::<BookInfo>::new());
    let mut is_loading = use_signal(|| true);
    
    // Load books on mount
    use_effect(move || {
        spawn(async move {
            // TODO: Load from actual database
            books.set(Vec::new());
            is_loading.set(false);
        });
    });
    
    rsx! {
        div { class: "flex flex-col h-full",
            // Header
            header { class: "bg-purple-600 text-white p-4",
                div { class: "flex items-center",
                    Link {
                        to: crate::app::Route::Home,
                        class: "mr-4 text-white",
                        "←"
                    }
                    h1 { class: "text-xl font-bold", "📖 Library" }
                }
            }
            
            // Books list
            div { class: "flex-1 overflow-auto p-4",
                if is_loading() {
                    div { class: "text-center py-8",
                        p { class: "text-gray-500", "Loading library..." }
                    }
                } else if books().is_empty() {
                    div { class: "text-center py-8",
                        p { class: "text-gray-500", "No books in library" }
                        p { class: "text-sm text-gray-400 mt-2", "Import a PDF to start reading!" }
                        
                        // Import button
                        button {
                            class: "mt-4 bg-purple-600 text-white px-6 py-2 rounded-lg",
                            onclick: move |_| {
                                // TODO: Open file picker
                                log::info!("Import PDF clicked");
                            },
                            "📥 Import PDF"
                        }
                    }
                } else {
                    for book in books() {
                        BookCard { book }
                    }
                }
            }
            
            // Import FAB
            button {
                class: "fixed bottom-6 right-6 bg-purple-600 text-white w-14 h-14 rounded-full flex items-center justify-center text-2xl shadow-lg",
                onclick: move |_| {
                    // TODO: Open file picker
                    log::info!("Import PDF FAB clicked");
                },
                "+"
            }
        }
    }
}

/// Book info (placeholder)
#[derive(Clone, PartialEq)]
struct BookInfo {
    id: i64,
    title: String,
    total_pages: i32,
    converted_pages: i32,
}

/// Book card component
#[component]
fn BookCard(book: BookInfo) -> Element {
    let progress = if book.total_pages > 0 {
        book.converted_pages as f32 / book.total_pages as f32 * 100.0
    } else {
        0.0
    };
    
    rsx! {
        Link {
            to: crate::app::Route::ReaderBook { book_id: book.id },
            class: "block bg-white border rounded-lg p-4 mb-3 shadow-sm hover:shadow-md transition-shadow",
            h3 { class: "font-semibold text-gray-800", "{book.title}" }
            
            // Progress bar
            div { class: "mt-2",
                div { class: "bg-gray-200 h-2 rounded-full overflow-hidden",
                    div {
                        class: "bg-purple-600 h-full",
                        style: "width: {progress}%"
                    }
                }
                p { class: "text-xs text-gray-500 mt-1",
                    "{book.converted_pages} / {book.total_pages} pages"
                }
            }
        }
    }
}