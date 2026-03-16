//! Vocabulary page component
//!
//! This component displays the vocabulary list and provides word extraction functionality.

use dioxus::prelude::*;

use crate::core::db::{Database, Word};

/// Vocabulary page component
#[component]
pub fn VocabPage() -> Element {
    // State for vocabulary list
    let mut words = use_signal(|| Vec::<Word>::new());
    let mut search_query = use_signal(|| String::new());
    let mut is_loading = use_signal(|| true);
    
    // Load vocabulary on mount
    use_effect(move || {
        spawn(async move {
            // Load words from database
            let result = tokio::task::spawn_blocking(|| {
                match Database::open("shusei.db") {
                    Ok(db) => db.get_all_words(),
                    Err(e) => Err(e),
                }
            }).await;
            
            match result {
                Ok(Ok(loaded_words)) => {
                    log::info!("Loaded {} words from database", loaded_words.len());
                    words.set(loaded_words);
                }
                Ok(Err(e)) => {
                    log::error!("Failed to load vocabulary: {}", e);
                }
                Err(e) => {
                    log::error!("Task failed: {}", e);
                }
            }
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
fn WordCard(word: Word) -> Element {
    rsx! {
        div { class: "bg-white border rounded-lg p-4 mb-3 shadow-sm",
            div { class: "flex justify-between items-start",
                div {
                    h3 { class: "font-semibold text-lg", "{word.word}" }
                    // Definition placeholder (per D007)
                    p { class: "text-gray-600 mt-1", "Definition coming soon" }
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
            
            // Example sentence (context_text)
            if let Some(context) = &word.context_text {
                p { class: "text-sm text-gray-500 mt-2 italic", "\"{context}\"" }
            }
            
            // Source reference
            if let Some(book_id) = &word.source_book_id {
                if let Some(page) = word.source_page {
                    p { class: "text-xs text-gray-400 mt-2",
                        "From: Book {book_id} (p.{page})"
                    }
                } else {
                    p { class: "text-xs text-gray-400 mt-2",
                        "From: Book {book_id}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::{Database, NewWord};

    #[test]
    fn test_vocab_loads_from_database() {
        // Create in-memory database
        let db = Database::in_memory().unwrap();
        
        // Create a test book
        let book_id = db.create_book(&crate::core::db::NewBook {
            title: "Test Book".to_string(),
            author: "Test Author".to_string(),
            ..Default::default()
        }).unwrap();
        
        // Add some words to the database
        let word1 = NewWord {
            word: "test".to_string(),
            definition: None,
            ai_generated: false,
            source_book_id: Some(book_id.clone()),
            source_page: Some(1),
            context_text: Some("This is a test sentence.".to_string()),
        };
        
        let word2 = NewWord {
            word: "vocabulary".to_string(),
            definition: None,
            ai_generated: false,
            source_book_id: Some(book_id.clone()),
            source_page: Some(2),
            context_text: Some("Building vocabulary is important.".to_string()),
        };
        
        db.create_word(&word1).unwrap();
        db.create_word(&word2).unwrap();
        
        // Verify get_all_words returns the words
        let words = db.get_all_words().unwrap();
        assert_eq!(words.len(), 2);
        
        // Verify both words are present (order may vary if timestamps are identical)
        let word_texts: Vec<&String> = words.iter().map(|w| &w.word).collect();
        assert!(word_texts.contains(&&"test".to_string()));
        assert!(word_texts.contains(&&"vocabulary".to_string()));
        
        // Verify word fields for vocabulary word
        let vocab_word = words.iter().find(|w| w.word == "vocabulary").unwrap();
        assert_eq!(vocab_word.context_text, Some("Building vocabulary is important.".to_string()));
        assert_eq!(vocab_word.source_book_id, Some(book_id.clone()));
        assert_eq!(vocab_word.source_page, Some(2));
        
        log::info!("✅ test_vocab_loads_from_database passed: loaded {} words", words.len());
    }
    
    #[test]
    fn test_vocab_empty_database() {
        let db = Database::in_memory().unwrap();
        
        // Verify get_all_words returns empty vector for empty database
        let words = db.get_all_words().unwrap();
        assert_eq!(words.len(), 0);
        
        log::info!("✅ test_vocab_empty_database passed: empty database returns empty list");
    }
}