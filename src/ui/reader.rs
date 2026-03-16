//! Reader page component
//!
//! This component provides the PDF reading experience with reflow support.

use dioxus::prelude::*;
use std::sync::Arc;
use crate::app::Route;
use crate::core::db::{Book, BookPage, Database, NewWord};
use crate::core::pdf::{PdfConversionService, ConversionProgress};
use crate::core::ocr::NdlocrEngine;
use crate::core::storage::StorageService;
use crate::core::vocab::WordExtractor;
use crate::ui::components::{PageJumpModal, ConversionProgressDisplay};
use web_sys::{window, HtmlElement, ScrollIntoViewOptions, ScrollBehavior};

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

/// Helper to get localStorage
fn get_local_storage() -> Option<web_sys::Storage> {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
}

/// Save font size preference to localStorage
fn save_font_size_preference(size: i32) {
    if let Some(storage) = get_local_storage() {
        let _ = storage.set("reader_font_size", &size.to_string());
        log::info!("Font size preference saved: {}px", size);
    }
}

/// Load font size preference from localStorage
fn load_font_size_preference() -> i32 {
    if let Some(storage) = get_local_storage() {
        if let Ok(Some(size_str)) = storage.get("reader_font_size") {
            if let Ok(size) = size_str.parse::<i32>() {
                log::info!("Font size preference loaded: {}px", size);
                return size;
            }
        }
    }
    18 // default
}

/// Save reading progress to localStorage (for quick restore)
fn save_last_read_position(book_id: i64, page: i32) {
    if let Some(storage) = get_local_storage() {
        let key = format!("last_read_book_{}", book_id);
        let _ = storage.set(&key, &page.to_string());
        log::info!("Last read position saved: book={}, page={}", book_id, page);
    }
}

/// Load reading progress from localStorage
fn load_last_read_position(book_id: i64) -> Option<i32> {
    if let Some(storage) = get_local_storage() {
        let key = format!("last_read_book_{}", book_id);
        if let Ok(Some(page_str)) = storage.get(&key) {
            if let Ok(page) = page_str.parse::<i32>() {
                log::info!("Last read position loaded: book={}, page={}", book_id, page);
                return Some(page);
            }
        }
    }
    None
}

/// Word span component with tap handler
#[component]
fn WordSpan(word: String, page: BookPage, on_tap: EventHandler<(String, BookPage)>) -> Element {
    rsx! {
        span {
            class: "cursor-pointer hover:bg-yellow-200 transition-colors duration-150 rounded px-0.5",
            "data-word": word.clone(),
            onclick: move |_| {
                on_tap.call((word.clone(), page.clone()));
            },
            "{word} "
        }
    }
}

/// Paragraph component with word tap support
#[component]
fn TapParagraph(text: String, page: BookPage, on_tap: EventHandler<(String, BookPage)>) -> Element {
    let words: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
    let page_clone = page.clone();
    let on_tap_clone = on_tap.clone();
    
    rsx! {
        p { class: "mb-2",
            {words.into_iter().map(move |word| {
                let w = word.clone();
                let p = page_clone.clone();
                let h = on_tap_clone.clone();
                rsx! { WordSpan { word: w, page: p, on_tap: h } }
            })}
        }
    }
}

/// Reader page component
#[component]
pub fn ReaderPage() -> Element {
    rsx! {
        div { class: "flex flex-col h-full",
            header { class: "bg-purple-600 text-white p-4",
                div { class: "flex items-center",
                    Link { to: Route::Home, class: "mr-4 text-white", "←" }
                    h1 { class: "text-xl font-bold", "📖 My Library" }
                }
            }
            div { class: "flex-1 flex items-center justify-center p-4",
                div { class: "text-center",
                    p { class: "text-gray-600 mb-4", "Visit the main library to manage your books" }
                    Link { to: Route::BookList, class: "bg-purple-600 text-white px-6 py-2 rounded-lg", "Go to Library" }
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
    let mut font_size = use_signal(|| 18);
    let mut show_page_jump = use_signal(|| false);
    let mut current_page = use_signal(|| 1);
    let mut is_converting = use_signal(|| false);
    let mut conversion_progress = use_signal(|| Option::<ConversionProgress>::None);
    let mut conversion_error = use_signal(|| Option::<String>::None);
    
    // Toast notification state
    let mut show_toast = use_signal(|| false);
    let mut toast_message = use_signal(|| String::new());
    let mut toast_type = use_signal(|| ToastType::Success);
    
    // Load book, pages, font size preference, and restore position on mount
    use_effect(move || {
        // Load font size preference from localStorage
        font_size.set(load_font_size_preference());
        
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                match Database::open("shusei.db") {
                    Ok(db) => {
                        let book_result = db.get_book(&book_id.to_string());
                        let pages_result = db.get_pages_by_book(&book_id.to_string());
                        
                        // Also load progress
                        let progress_result = db.get_progress(&book_id.to_string());
                        
                        match (book_result, pages_result, progress_result) {
                            (Ok(Some(b)), Ok(p), progress) => {
                                let last_page = progress.ok().flatten()
                                    .and_then(|prog| if prog.last_processed_page > 0 { 
                                        Some(prog.last_processed_page) 
                                    } else { None })
                                    .or_else(|| load_last_read_position(book_id));
                                Some((b, p, last_page))
                            }
                            _ => None,
                        }
                    }
                    Err(_) => None,
                }
            }).await;
            
            match result {
                Ok(Some((b, p, last_page_opt))) => { 
                    book.set(Some(b)); 
                    pages.set(p);
                    
                    // Restore position after render
                    if let Some(last_page) = last_page_opt {
                        current_page.set(last_page);
                        // Scroll to the page after DOM is ready
                        spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            if let Some(win) = window() {
                                if let Some(document) = win.document() {
                                    let element_id = format!("page-{}", last_page);
                                    if let Some(element) = document.get_element_by_id(&element_id) {
                                        // Use scrollIntoView directly on Element
                                        element.scroll_into_view();
                                        log::info!("Restored position to page {}", last_page);
                                    }
                                }
                            }
                        });
                    }
                }
                _ => error.set(Some("Failed to load book data".to_string())),
            }
            is_loading.set(false);
        });
    });
    
    // Handle word tap - callback that saves word to database
    let mut toast_msg = toast_message.clone();
    let mut toast_t = toast_type.clone();
    let mut show_t = show_toast.clone();
    
    let handle_word_save = move |word: String, page: BookPage| {
        let book_id_str = book_id.to_string();
        let page_number = page.page_number;
        let page_text = page.ocr_text_plain.clone();
        let mut t_msg = toast_msg.clone();
        let mut t_type = toast_t.clone();
        let mut s_toast = show_t.clone();
        
        spawn(async move {
            let db_result = tokio::task::spawn_blocking(move || {
                match Database::open("shusei.db") {
                    Ok(db) => {
                        match db.get_word_by_text(&word) {
                            Ok(Some(_)) => {
                                log::info!("Word '{}' already saved", word);
                                Err("already_exists".to_string())
                            }
                            Ok(None) => {
                                let extractor = WordExtractor::new().ok();
                                let sentence = extractor.as_ref().and_then(|ext| ext.extract_sentence(&page_text, &word));
                                
                                let new_word = NewWord {
                                    word: word.clone(),
                                    definition: None,
                                    ai_generated: false,
                                    source_book_id: Some(book_id_str.clone()),
                                    source_page: Some(page_number),
                                    context_text: sentence,
                                };
                                
                                match db.create_word(&new_word) {
                                    Ok(id) => {
                                        log::info!("Word '{}' saved with id={}", word, id);
                                        Ok(())
                                    }
                                    Err(e) => Err(format!("Save failed: {}", e))
                                }
                            }
                            Err(e) => Err(format!("DB error: {}", e))
                        }
                    }
                    Err(e) => Err(format!("DB open failed: {}", e))
                }
            }).await;
            
            match db_result {
                Ok(Ok(())) => { t_msg.set("Word saved!".to_string()); t_type.set(ToastType::Success); }
                Ok(Err(err)) if err == "already_exists" => { t_msg.set("Already saved".to_string()); t_type.set(ToastType::Info); }
                Ok(Err(err)) => { t_msg.set(err); t_type.set(ToastType::Error); }
                Err(e) => { t_msg.set(format!("Error: {}", e)); t_type.set(ToastType::Error); }
            }
            s_toast.set(true);
            spawn(async move { tokio::time::sleep(tokio::time::Duration::from_secs(3)).await; s_toast.set(false); });
        });
    };
    
    // Create event handler for word taps
    let on_word_tap = EventHandler::new(move |(word, page): (String, BookPage)| {
        handle_word_save(word, page);
    });
    
    // Handle font size change - save to localStorage
    let mut handle_font_size_change = move |size: i32| {
        font_size.set(size);
        save_font_size_preference(size);
    };
    
    rsx! {
        div { class: "flex flex-col h-full bg-gray-50",
            header { class: "bg-purple-600 text-white p-4 shadow-md",
                div { class: "flex items-center justify-between",
                    div { class: "flex items-center space-x-3",
                        Link { to: Route::Reader, class: "text-white hover:text-purple-200", "←" }
                        if let Some(b) = book() { h1 { class: "text-xl font-bold", "{b.title}" } }
                        if !pages().is_empty() {
                            button { class: "bg-purple-500 hover:bg-purple-400 px-3 py-1 rounded text-sm",
                                onclick: move |_| show_page_jump.set(true), "#{current_page()}"
                            }
                        }
                    }
                    div { class: "flex items-center space-x-4",
                        if !pages().is_empty() { span { class: "text-sm", "Page {current_page()} of {pages().len()}" } }
                        div { class: "flex items-center space-x-2",
                            span { class: "text-sm", "{font_size()}px" }
                            input { r#type: "range", min: "12", max: "32", value: "{font_size()}",
                                oninput: move |e| { if let Ok(size) = e.value().parse::<i32>() { handle_font_size_change(size); } },
                                class: "w-32 h-2 bg-purple-300 rounded-lg appearance-none cursor-pointer"
                            }
                        }
                    }
                }
            }
            
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
                    div { class: "flex items-center justify-center h-full p-4",
                        div { class: "text-center max-w-md",
                            p { class: "text-4xl mb-4", "📄" }
                            p { class: "text-gray-600 text-lg mb-2", "This PDF hasn't been converted yet" }
                            p { class: "text-gray-500 text-sm mb-4", "Convert pages to start reading." }
                            if is_converting() {
                                div { class: "mb-4",
                                    if let Some(progress) = conversion_progress() {
                                        ConversionProgressDisplay { stage: progress.stage, current_page: progress.current_page, total_pages: progress.total_pages }
                                    } else {
                                        div { class: "text-center",
                                            div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600 mx-auto mb-2" }
                                            p { class: "text-sm text-gray-600", "Starting conversion..." }
                                        }
                                    }
                                }
                            } else {
                                button { class: "bg-purple-600 text-white px-6 py-2 rounded-lg hover:bg-purple-700",
                                    onclick: move |_| {
                                        spawn(async move {
                                            is_converting.set(true); conversion_error.set(None);
                                            let app_data_dir = std::env::current_exe().ok().and_then(|exe| exe.parent().map(|p| p.to_path_buf())).unwrap_or_else(|| std::path::PathBuf::from("."));
                                            let pdf_path = match book().as_ref().and_then(|b| b.pdf_path.as_ref()) {
                                                Some(path) => app_data_dir.join(path),
                                                None => { conversion_error.set(Some("PDF path not stored".to_string())); is_converting.set(false); return; }
                                            };
                                            if !pdf_path.exists() { conversion_error.set(Some(format!("PDF file not found: {}", pdf_path.display()))); is_converting.set(false); return; }
                                            let ocr = NdlocrEngine::new(&app_data_dir, "en");
                                            match (Database::open("shusei.db"), StorageService::new(app_data_dir.clone())) {
                                                (Ok(db), Ok(storage)) => {
                                                    match PdfConversionService::new(ocr, Arc::new(db), Arc::new(storage)) {
                                                        Ok(conv_service) => {
                                                            match conv_service.convert_pdf(&book_id.to_string(), &pdf_path, |_| {}).await {
                                                                Ok(_) => {
                                                                    log::info!("Conversion complete");
                                                                    if let Ok(db) = Database::open("shusei.db") {
                                                                        if let Ok(loaded_pages) = db.get_pages_by_book(&book_id.to_string()) { pages.set(loaded_pages); }
                                                                    }
                                                                }
                                                                Err(e) => { log::error!("Conversion failed: {:?}", e); conversion_error.set(Some(format!("Conversion failed: {}", e))); }
                                                            }
                                                        }
                                                        Err(e) => { conversion_error.set(Some(format!("Failed to initialize: {}", e))); }
                                                    }
                                                }
                                                _ => conversion_error.set(Some("Failed to initialize services".to_string())),
                                            }
                                            is_converting.set(false);
                                        });
                                    },
                                    "Convert"
                                }
                            }
                            if let Some(err) = conversion_error() { p { class: "text-red-600 text-sm mt-2", "{err}" } }
                        }
                    }
                } else {
                    div { class: "max-w-2xl mx-auto p-4 space-y-6", style: "font-size: {font_size()}px",
                        onscroll: move |e| {
                            // Simple scroll handler - update current page
                            let pages_len = pages().len();
                            if pages_len == 0 {
                                return;
                            }
                            
                            // Get scroll position from event data
                            let scroll_data = e.data();
                            let scroll_y = scroll_data.scroll_top() as f32;
                            let total_height = scroll_data.scroll_height() as f32;
                            let page_height = total_height / pages_len as f32;
                            let estimated_page = ((scroll_y / page_height) + 1.0) as i32;
                            let new_page = estimated_page.min(pages_len as i32).max(1);
                            current_page.set(new_page);
                            
                            // Debounced save to database
                            let book_id_for_save = book_id;
                            let new_page_for_save = new_page;
                            let _timer_handle = spawn(async move {
                                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                
                                // Save to database
                                let db_result = tokio::task::spawn_blocking(move || {
                                    match Database::open("shusei.db") {
                                        Ok(db) => {
                                            match db.update_progress(&book_id_for_save.to_string(), new_page_for_save, "reading") {
                                                Ok(_) => {
                                                    log::info!("Progress saved: book={}, page={}", book_id_for_save, new_page_for_save);
                                                    Ok(())
                                                }
                                                Err(e) => Err(format!("Failed to save progress: {}", e))
                                            }
                                        }
                                        Err(e) => Err(format!("DB open failed: {}", e))
                                    }
                                }).await;
                                
                                // Also save to localStorage for quick restore
                                save_last_read_position(book_id_for_save, new_page_for_save);
                                
                                if let Err(e) = db_result {
                                    log::warn!("Progress save error (non-fatal): {}", e);
                                }
                            });
                        },
                        for page in pages().into_iter() {
                            div { class: "bg-white rounded-lg shadow-sm p-6", id: "page-{page.page_number}",
                                div { class: "prose max-w-none",
                                    // Render page content with word tap support
                                    {render_page_content(&page.ocr_markdown, page.clone(), on_word_tap.clone())}
                                }
                            }
                            div { class: "flex items-center justify-center",
                                div { class: "flex items-center space-x-4",
                                    div { class: "h-px bg-gray-300 w-16" }
                                    span { class: "text-gray-500 text-sm font-medium", "Page {page.page_number}" }
                                    div { class: "h-px bg-gray-300 w-16" }
                                }
                            }
                        }
                    }
                }
            }
            
            PageJumpModal { show: show_page_jump(), total_pages: pages().len() as i32,
                on_close: move |_| show_page_jump.set(false),
                on_submit: move |page_num| { current_page.set(page_num); }
            }
            
            if show_toast() {
                ToastNotification { message: toast_message(), toast_type: toast_type(), on_close: move |_| show_toast.set(false) }
            }
        }
    }
}

/// Render page content with word tap support
fn render_page_content(content: &str, page: BookPage, on_tap: EventHandler<(String, BookPage)>) -> Element {
    let paragraphs: Vec<String> = content.split("\n\n").filter(|s| !s.trim().is_empty()).map(|s| s.to_string()).collect();
    let page_clone = page.clone();
    let on_tap_clone = on_tap.clone();
    
    rsx! {
        {paragraphs.into_iter().map(move |para| {
            let is_header = para.starts_with("# ");
            let text = if is_header { para.trim_start_matches("# ").trim().to_string() } else { para.clone() };
            let p = page_clone.clone();
            let h = on_tap_clone.clone();
            
            if is_header {
                rsx! {
                    h2 { class: "text-xl font-bold mb-2",
                        TapParagraph { text: text, page: p, on_tap: h }
                    }
                }
            } else {
                rsx! {
                    TapParagraph { text: text, page: p, on_tap: h }
                }
            }
        })}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::{Database, NewBook};
    
    #[test]
    fn test_word_extractor_extract_sentence() {
        let extractor = WordExtractor::new().unwrap();
        let text = "This is a test sentence. Another sentence here.";
        let sentence = extractor.extract_sentence(text, "test");
        assert_eq!(sentence, Some("This is a test sentence".to_string()));
    }
    
    #[test]
    fn test_word_tap_saves_to_database() {
        let db = Database::in_memory().unwrap();
        let book_id = db.create_book(&NewBook { title: "Test Book".to_string(), author: "Test".to_string(), ..Default::default() }).unwrap();
        
        let extractor = WordExtractor::new().unwrap();
        let text = "Hello world this is a test sentence.";
        let sentence = extractor.extract_sentence(text, "test");
        assert!(sentence.is_some());
        
        let new_word = NewWord {
            word: "test".to_string(), definition: None, ai_generated: false,
            source_book_id: Some(book_id.clone()), source_page: Some(1), context_text: sentence.clone(),
        };
        
        let word_id = db.create_word(&new_word).unwrap();
        assert!(word_id > 0);
        
        let word = db.get_word(word_id).unwrap().unwrap();
        assert_eq!(word.word, "test");
        assert_eq!(word.source_book_id, Some(book_id));
        assert_eq!(word.source_page, Some(1));
        assert!(!word.ai_generated);
    }
    
    #[test]
    fn test_duplicate_word_handling() {
        let db = Database::in_memory().unwrap();
        let book_id = db.create_book(&NewBook { title: "Test".to_string(), author: "Test".to_string(), ..Default::default() }).unwrap();
        
        let new_word = NewWord {
            word: "duplicate".to_string(), definition: None, ai_generated: false,
            source_book_id: Some(book_id.clone()), source_page: Some(1), context_text: Some("Test context".to_string()),
        };
        
        db.create_word(&new_word).unwrap();
        let existing = db.get_word_by_text("duplicate").unwrap();
        assert!(existing.is_some());
    }
    
    #[test]
    fn test_progress_auto_save() {
        let db = Database::in_memory().unwrap();
        let book_id = db.create_book(&NewBook { title: "Test Book".to_string(), author: "Test".to_string(), ..Default::default() }).unwrap();
        
        // Create initial progress
        db.create_progress(&book_id, 100).unwrap();
        
        // Simulate auto-save on scroll
        db.update_progress(&book_id, 5, "reading").unwrap();
        
        let progress = db.get_progress(&book_id).unwrap().unwrap();
        assert_eq!(progress.last_processed_page, 5);
        assert_eq!(progress.status, "reading");
    }
    
    #[test]
    fn test_last_position_restore() {
        let db = Database::in_memory().unwrap();
        let book_id = db.create_book(&NewBook { title: "Test Book".to_string(), author: "Test".to_string(), ..Default::default() }).unwrap();
        
        // Simulate saved progress
        db.create_progress(&book_id, 100).unwrap();
        db.update_progress(&book_id, 10, "reading").unwrap();
        
        // Restore position
        let progress = db.get_progress(&book_id).unwrap().unwrap();
        assert_eq!(progress.last_processed_page, 10);
        assert!(progress.last_processed_page > 0);
    }
}
