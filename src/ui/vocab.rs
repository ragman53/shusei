//! Vocabulary page component
//!
//! This component displays the vocabulary list and provides word extraction functionality.

use dioxus::prelude::*;

use crate::core::vocab::VocabularyEntry;

/// Vocabulary page component
#[component]
pub fn VocabPage() -> Element {
    // State for vocabulary list
    let mut words = use_signal(|| Vec::<VocabularyEntry>::new());
    let mut search_query = use_signal(|| String::new());
    let mut is_loading = use_signal(|| true);
    
    // Load vocabulary on mount
    use_effect(move || {
        spawn(async move {
            // TODO: Load from actual database
            words.set(Vec::new());
            is_loading.set(false);
        });
    });
    
    rsx! {
        div { class: "flex flex-col h-full",
            // Header
            header { class: "bg-orange-600 text-white p-4",
                div { class: "flex items-center mb-2",
                    Link {
                        to: crate::app::Route::Home,
                        class: "mr-4 text-white",
                        "←"
                    }
                    h1 { class: "text-xl font-bold", "📚 Vocabulary" }
                }
                
                // Search bar
                div { class: "mt-2",
                    input {
                        class: "w-full p-2 rounded text-black",
                        placeholder: "Search words...",
                        value: search_query(),
                        oninput: move |e| search_query.set(e.value()),
                    }
                }
            }
            
            // Words list
            div { class: "flex-1 overflow-auto p-4",
                if is_loading() {
                    div { class: "text-center py-8",
                        p { class: "text-gray-500", "Loading vocabulary..." }
                    }
                } else if words().is_empty() {
                    div { class: "text-center py-8",
                        p { class: "text-gray-500", "No words saved" }
                        p { class: "text-sm text-gray-400 mt-2", "Tap on words while reading to add them!" }
                    }
                } else {
                    for word in words() {
                        WordCard { word }
                    }
                }
            }
            
            // Export button
            div { class: "p-4 border-t",
                div { class: "flex gap-2",
                    button {
                        class: "flex-1 bg-gray-200 text-gray-800 p-2 rounded-lg",
                        onclick: move |_| {
                            // TODO: Export as Markdown
                            log::info!("Export MD clicked");
                        },
                        "📄 Markdown"
                    }
                    button {
                        class: "flex-1 bg-gray-200 text-gray-800 p-2 rounded-lg",
                        onclick: move |_| {
                            // TODO: Export as CSV
                            log::info!("Export CSV clicked");
                        },
                        "📊 CSV"
                    }
                }
            }
        }
    }
}

/// Word card component
#[component]
fn WordCard(word: VocabularyEntry) -> Element {
    rsx! {
        div { class: "bg-white border rounded-lg p-4 mb-3 shadow-sm",
            div { class: "flex justify-between items-start",
                div {
                    h3 { class: "font-semibold text-lg", "{word.word}" }
                    if let Some(meaning) = &word.meaning {
                        p { class: "text-gray-600 mt-1", "{meaning}" }
                    }
                }
                button {
                    class: "text-gray-400 hover:text-red-500",
                    onclick: move |_| {
                        // TODO: Delete word
                        log::info!("Delete word: {}", word.word);
                    },
                    "🗑️"
                }
            }
            
            // Example sentence
            if let Some(example) = &word.example_sentence {
                p { class: "text-sm text-gray-500 mt-2 italic", "\"{example}\"" }
            }
            
            // Source
            if let Some(book) = &word.source_book {
                p { class: "text-xs text-gray-400 mt-2",
                    "From: {book} (p.{word.source_page.unwrap_or(0)})"
                }
            }
        }
    }
}