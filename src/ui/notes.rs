//! Notes page component
//!
//! This component displays the list of sticky notes and provides search functionality.

use dioxus::prelude::*;

use crate::core::db::{Database, StickyNote};

/// Notes page component
#[component]
pub fn NotesPage() -> Element {
    // State for notes list
    let mut notes = use_signal(|| Vec::<StickyNote>::new());
    let mut search_query = use_signal(|| String::new());
    let mut is_loading = use_signal(|| true);
    
    // Load notes on mount
    use_effect(move || {
        spawn(async move {
            // TODO: Load from actual database
            // For now, show empty list
            notes.set(Vec::new());
            is_loading.set(false);
        });
    });
    
    // Search handler
    let search = move |_| {
        spawn(async move {
            let query = search_query();
            if query.is_empty() {
                // Load all notes
                notes.set(Vec::new());
            } else {
                // Search notes
                // TODO: Implement FTS search
                notes.set(Vec::new());
            }
        });
    };
    
    rsx! {
        div { class: "flex flex-col h-full",
            // Header
            header { class: "bg-blue-600 text-white p-4",
                div { class: "flex items-center mb-2",
                    Link {
                        to: crate::app::Route::Home,
                        class: "mr-4 text-white",
                        "←"
                    }
                    h1 { class: "text-xl font-bold", "📝 Sticky Notes" }
                }
                
                // Search bar
                div { class: "mt-2",
                    input {
                        class: "w-full p-2 rounded text-black",
                        placeholder: "Search notes...",
                        value: search_query(),
                        oninput: move |e| search_query.set(e.value()),
                        onchange: search,
                    }
                }
            }
            
            // Notes list
            div { class: "flex-1 overflow-auto p-4",
                if is_loading() {
                    div { class: "text-center py-8",
                        p { class: "text-gray-500", "Loading..." }
                    }
                } else if notes().is_empty() {
                    div { class: "text-center py-8",
                        p { class: "text-gray-500", "No notes yet" }
                        p { class: "text-sm text-gray-400 mt-2", "Capture a page to create your first note!" }
                    }
                } else {
                    for note in notes() {
                        NoteCard { note }
                    }
                }
            }
            
            // FAB
            Link {
                to: crate::app::Route::Camera,
                class: "fixed bottom-6 right-6 bg-green-600 text-white w-14 h-14 rounded-full flex items-center justify-center text-2xl shadow-lg",
                "+"
            }
        }
    }
}

/// Note card component
#[component]
fn NoteCard(note: StickyNote) -> Element {
    rsx! {
        Link {
            to: crate::app::Route::NoteDetail { id: note.id },
            class: "block bg-white border rounded-lg p-4 mb-3 shadow-sm hover:shadow-md transition-shadow",
            // Book title
            if let Some(book) = &note.book_title {
                p { class: "text-sm text-gray-500 mb-1", "{book}" }
            }
            
            // Content preview
            if let Some(content) = &note.ocr_markdown {
                p { class: "text-gray-800 line-clamp-3",
                    // Truncate to first 100 characters
                    "{content.chars().take(100).collect::<String>()}..."
                }
            }
            
            // Date
            p { class: "text-xs text-gray-400 mt-2",
                "{note.created_at}"
            }
        }
    }
}