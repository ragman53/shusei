//! Vocabulary page component
//!
//! This component displays the vocabulary list and provides word extraction functionality.

use dioxus::prelude::*;

use crate::core::db::{Database, Word};
use crate::core::vocab::{export_vocabulary_words, ExportFormat};

/// Toast notification type
#[derive(Clone, Copy, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Info,
}

/// Toast notification component
#[component]
pub fn ToastNotification(message: String, toast_type: ToastType, on_close: EventHandler<()>) -> Element {
    let bg_color = match toast_type {
        ToastType::Success => "bg-green-500",
        ToastType::Error => "bg-red-500",
        ToastType::Info => "bg-blue-500",
    };
    
    let icon = match toast_type {
        ToastType::Success => "✓",
        ToastType::Error => "✗",
        ToastType::Info => "ℹ",
    };
    
    rsx! {
        div {
            class: "fixed bottom-4 left-1/2 transform -translate-x-1/2 z-50",
            div {
                class: "{bg_color} text-white px-6 py-3 rounded-lg shadow-lg flex items-center space-x-3",
                span { class: "text-lg", "{icon}" }
                span { class: "font-medium", "{message}" }
                button {
                    class: "ml-2 hover:opacity-75",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }
        }
    }
}

/// Vocabulary page component
#[component]
pub fn VocabPage() -> Element {
    // State for vocabulary list
    let mut words = use_signal(|| Vec::<Word>::new());
    let mut search_query = use_signal(|| String::new());
    let mut is_loading = use_signal(|| true);
    
    // State for delete confirmation dialog
    let mut show_delete_dialog = use_signal(|| Option::<i64>::None);
    
    // State for toast notifications
    let mut show_toast = use_signal(|| false);
    let mut toast_message = use_signal(|| String::new());
    let mut toast_type = use_signal(|| ToastType::Info);
    
    // Helper to show toast
    let mut show_toast_fn = move |msg: String, t: ToastType| {
        toast_message.set(msg);
        toast_type.set(t);
        show_toast.set(true);
        spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            show_toast.set(false);
        });
    };
    
    // Delete handler
    let delete_word_handler = move |word_id: i64| {
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                match Database::open("shusei.db") {
                    Ok(db) => {
                        db.delete_word(word_id).map(|_| word_id)
                    }
                    Err(e) => Err(e),
                }
            }).await;
            
            match result {
                Ok(Ok(id)) => {
                    log::info!("Deleted word {}", id);
                    // Remove word from local state
                    words.write().retain(|w| w.id != id);
                    show_toast_fn("Word deleted".to_string(), ToastType::Success);
                }
                Ok(Err(e)) => {
                    log::error!("Failed to delete word: {}", e);
                    show_toast_fn(format!("Failed to delete: {}", e), ToastType::Error);
                }
                Err(e) => {
                    log::error!("Task failed: {}", e);
                    show_toast_fn("Failed to delete word".to_string(), ToastType::Error);
                }
            }
            show_delete_dialog.set(None);
        });
    };
    
    // Filter words based on search query
    let filtered_words: Vec<Word> = {
        let words = words();
        let query = search_query().to_lowercase();
        if query.is_empty() {
            words.clone()
        } else {
            words.into_iter().filter(|w| w.word.to_lowercase().contains(&query)).collect()
        }
    };
    
    // Export handlers
    let mut export_markdown_handler = {
        let words = filtered_words.clone();
        move |_| {
            if words.is_empty() {
                show_toast_fn("No words to export".to_string(), ToastType::Info);
                return;
            }
            
            let markdown = export_vocabulary_words(&words, ExportFormat::Markdown);
            log::info!("Exported {} words as Markdown", words.len());
            show_toast_fn(format!("Exported {} words as Markdown", words.len()), ToastType::Success);
            
            // For now, log the output (in a real app, this would save to file or clipboard)
            log::debug!("Markdown output:\n{}", markdown);
        }
    };
    
    let mut export_csv_handler = {
        let words = filtered_words.clone();
        move |_| {
            if words.is_empty() {
                show_toast_fn("No words to export".to_string(), ToastType::Info);
                return;
            }
            
            let csv = export_vocabulary_words(&words, ExportFormat::Csv);
            log::info!("Exported {} words as CSV", words.len());
            show_toast_fn(format!("Exported {} words as CSV", words.len()), ToastType::Success);
            
            // For now, log the output (in a real app, this would save to file or clipboard)
            log::debug!("CSV output:\n{}", csv);
        }
    };
    
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
                        WordCard { 
                            word,
                            on_delete: move |id| show_delete_dialog.set(Some(id)),
                        }
                    }
                }
            }
            
            // Export button
            div { class: "p-4 border-t",
                div { class: "flex gap-2",
                    button {
                        class: "flex-1 bg-gray-200 text-gray-800 p-2 rounded-lg",
                        onclick: move |_| export_markdown_handler(()),
                        "📄 Markdown"
                    }
                    button {
                        class: "flex-1 bg-gray-200 text-gray-800 p-2 rounded-lg",
                        onclick: move |_| export_csv_handler(()),
                        "📊 CSV"
                    }
                }
            }
            
            // Delete confirmation dialog
            if let Some(word_id) = show_delete_dialog() {
                div {
                    class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                    onclick: move |_| show_delete_dialog.set(None),
                    div {
                        class: "bg-white rounded-lg p-6 max-w-sm mx-4",
                        onclick: move |e| e.stop_propagation(),
                        h3 { class: "text-lg font-semibold mb-4", "Delete Word?" }
                        p { class: "text-gray-600 mb-6", "This action cannot be undone." }
                        div { class: "flex gap-3 justify-end",
                            button {
                                class: "px-4 py-2 text-gray-600 hover:bg-gray-100 rounded",
                                onclick: move |_| show_delete_dialog.set(None),
                                "Cancel"
                            }
                            button {
                                class: "px-4 py-2 bg-red-500 text-white hover:bg-red-600 rounded",
                                onclick: move |_| delete_word_handler(word_id),
                                "Delete"
                            }
                        }
                    }
                }
            }
            
            // Toast notification
            if show_toast() {
                ToastNotification { 
                    message: toast_message(), 
                    toast_type: toast_type(), 
                    on_close: move |_| show_toast.set(false) 
                }
            }
        }
    }
}

/// Word card component
#[component]
fn WordCard(word: Word, on_delete: EventHandler<i64>) -> Element {
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
                    onclick: move |_| on_delete.call(word.id),
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
    
    #[test]
    fn test_word_delete() {
        // Create in-memory database
        let db = Database::in_memory().unwrap();
        
        // Create a test book
        let book_id = db.create_book(&crate::core::db::NewBook {
            title: "Test Book".to_string(),
            author: "Test Author".to_string(),
            ..Default::default()
        }).unwrap();
        
        // Add a word to the database
        let word = NewWord {
            word: "deletetest".to_string(),
            definition: None,
            ai_generated: false,
            source_book_id: Some(book_id.clone()),
            source_page: Some(1),
            context_text: Some("This word will be deleted.".to_string()),
        };
        
        let word_id = db.create_word(&word).unwrap();
        assert!(word_id > 0);
        
        // Verify word exists
        let words = db.get_all_words().unwrap();
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].id, word_id);
        
        // Delete the word
        let delete_result = db.delete_word(word_id);
        assert!(delete_result.is_ok());
        
        // Verify word is deleted
        let words_after = db.get_all_words().unwrap();
        assert_eq!(words_after.len(), 0);
        
        // Verify get_word returns None
        let retrieved = db.get_word(word_id).unwrap();
        assert!(retrieved.is_none());
        
        log::info!("✅ test_word_delete passed: word deleted successfully");
    }
    
    #[test]
    fn test_word_delete_with_multiple_words() {
        // Create in-memory database
        let db = Database::in_memory().unwrap();
        
        // Create a test book
        let book_id = db.create_book(&crate::core::db::NewBook {
            title: "Test Book".to_string(),
            author: "Test Author".to_string(),
            ..Default::default()
        }).unwrap();
        
        // Add multiple words
        let word1 = NewWord {
            word: "keep".to_string(),
            definition: None,
            ai_generated: false,
            source_book_id: Some(book_id.clone()),
            source_page: Some(1),
            context_text: Some("This word stays.".to_string()),
        };
        
        let word2 = NewWord {
            word: "delete".to_string(),
            definition: None,
            ai_generated: false,
            source_book_id: Some(book_id.clone()),
            source_page: Some(2),
            context_text: Some("This word goes.".to_string()),
        };
        
        let word1_id = db.create_word(&word1).unwrap();
        let word2_id = db.create_word(&word2).unwrap();
        
        // Verify both words exist
        let words = db.get_all_words().unwrap();
        assert_eq!(words.len(), 2);
        
        // Delete only word2
        db.delete_word(word2_id).unwrap();
        
        // Verify only word1 remains
        let words_after = db.get_all_words().unwrap();
        assert_eq!(words_after.len(), 1);
        assert_eq!(words_after[0].id, word1_id);
        assert_eq!(words_after[0].word, "keep");
        
        log::info!("✅ test_word_delete_with_multiple_words passed: selective delete works");
    }
    
    #[test]
    fn test_export_functions() {
        use crate::core::vocab::{export_vocabulary_words, ExportFormat};
        
        // Create test words
        let words = vec![
            Word {
                id: 1,
                word: "export_test".to_string(),
                definition: Some("Test definition".to_string()),
                ai_generated: false,
                source_book_id: Some("test_book".to_string()),
                source_page: Some(5),
                context_text: Some("This is an export test.".to_string()),
                created_at: 1000,
                updated_at: 1000,
            },
        ];
        
        // Test Markdown export
        let md = export_vocabulary_words(&words, ExportFormat::Markdown);
        assert!(md.contains("# Vocabulary List"));
        assert!(md.contains("## export_test"));
        assert!(md.contains("**Definition**: Test definition"));
        assert!(md.contains("**Example**: This is an export test."));
        log::info!("✅ Markdown export test passed");
        
        // Test CSV export
        let csv = export_vocabulary_words(&words, ExportFormat::Csv);
        assert!(csv.contains("word,definition,example_sentence,source_book_id,source_page"));
        assert!(csv.contains("\"export_test\",\"Test definition\",\"This is an export test.\",\"test_book\",\"5\""));
        log::info!("✅ CSV export test passed");
        
        // Test JSON export
        let json = export_vocabulary_words(&words, ExportFormat::Json);
        assert!(json.contains("\"word\": \"export_test\""));
        log::info!("✅ JSON export test passed");
        
        // Test empty list
        let empty: Vec<Word> = vec![];
        let md_empty = export_vocabulary_words(&empty, ExportFormat::Markdown);
        assert_eq!(md_empty, "# Vocabulary List\n\n");
        log::info!("✅ Empty export test passed");
    }
}