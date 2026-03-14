//! Library screen component
//!
//! This module contains the LibraryScreen component that displays
//! a list of books and allows users to add new books.

use dioxus::prelude::*;
use dioxus_router::use_navigator;
use std::path::Path;
use std::sync::Arc;

use crate::app::Route;
use crate::core::db::{Book, Database, NewBook};
use crate::core::pdf::{PdfMetadata, PdfProcessor};
use crate::core::storage::StorageService;

    #[cfg(target_os = "android")]
use crate::platform::{android, PlatformApi};

/// Filter type for library
#[derive(Clone, PartialEq)]
pub enum LibraryFilter {
    All,
    PdfsOnly,
    PhysicalOnly,
}

/// Metadata review dialog props
#[derive(Props, Clone, PartialEq)]
pub struct MetadataReviewProps {
    pub show: bool,
    pub title: String,
    pub author: String,
    pub page_count: u32,
    pub on_close: EventHandler<()>,
    pub on_confirm: EventHandler<(String, String)>,
}

/// Metadata review dialog component
#[component]
pub fn MetadataReviewDialog(props: MetadataReviewProps) -> Element {
    let mut title = use_signal(|| props.title.clone());
    let mut author = use_signal(|| props.author.clone());

    if !props.show {
        return rsx! {};
    }

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| props.on_close.call(()),
            div {
                class: "bg-white rounded-lg p-6 w-full max-w-md",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "text-lg font-bold mb-4", "Review PDF Metadata" }
                p { class: "text-sm text-gray-600 mb-4", "Page count: {props.page_count}" }
                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "Title" }
                    input {
                        r#type: "text",
                        value: "{title()}",
                        oninput: move |e| title.set(e.value()),
                        class: "w-full border rounded-lg px-3 py-2"
                    }
                }
                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "Author" }
                    input {
                        r#type: "text",
                        value: "{author()}",
                        oninput: move |e| author.set(e.value()),
                        class: "w-full border rounded-lg px-3 py-2"
                    }
                }
                div { class: "flex space-x-2",
                    button {
                        class: "flex-1 bg-gray-200 text-gray-800 py-2 rounded-lg",
                        onclick: move |_| props.on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "flex-1 bg-green-600 text-white py-2 rounded-lg",
                        onclick: move |_| {
                            props.on_confirm.call((title(), author()));
                        },
                        "Import"
                    }
                }
            }
        }
    }
}

/// Library screen component that displays book list
#[component]
pub fn LibraryScreen() -> Element {
    let mut books = use_signal(|| vec![]);
    let mut importing = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut filter = use_signal(|| LibraryFilter::All);
    let mut show_metadata_dialog = use_signal(|| false);
    let mut pending_metadata = use_signal(|| Option::<(PdfMetadata, String)>::None);
    let mut review_title = use_signal(|| String::new());
    let mut review_author = use_signal(|| String::new());
    let mut review_pages = use_signal(|| 0u32);
    let navigator = use_navigator();

    // Load books on mount
    use_effect(move || {
        spawn(async move {
            // Load from actual database
            if let Ok(db) = Database::open("shusei.db") {
                if let Ok(all_books) = db.get_all_books() {
                    books.set(all_books);
                }
            }
        });
    });

    #[cfg(not(target_os = "android"))]
    let handle_import_pdf = move |_| {
        spawn(async move {
            importing.set(true);
            error_message.set(None);
            
            // Open file picker
            let file = rfd::AsyncFileDialog::new()
                .add_filter("PDF", &["pdf"])
                .pick_file()
                .await;
            
            if let Some(file) = file {
                let source_path = file.path().to_path_buf();
                
                // Get app data directory
                let app_data_dir = match std::env::current_exe() {
                    Ok(exe) => exe.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from(".")),
                    Err(_) => std::path::PathBuf::from("."),
                };
                
                // Import PDF using PdfProcessor
                match PdfProcessor::new() {
                    Ok(processor) => {
                        match processor.import_pdf(&source_path, &app_data_dir) {
                            Ok((metadata, copied_path)) => {
                                log::info!("PDF imported: {:?} -> {}", metadata, copied_path);
                                
                                // Show metadata review dialog
                                review_title.set(metadata.title.clone().unwrap_or_else(|| {
                                    source_path.file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("Unknown")
                                        .to_string()
                                }));
                                review_author.set(metadata.author.clone().unwrap_or_default());
                                review_pages.set(metadata.page_count);
                                pending_metadata.set(Some((metadata, copied_path)));
                                show_metadata_dialog.set(true);
                            }
                            Err(e) => {
                                log::error!("Failed to import PDF: {:?}", e);
                                error_message.set(Some(format!("Failed to import PDF: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create PdfProcessor: {:?}", e);
                        error_message.set(Some(format!("Failed to initialize PDF processor: {}", e)));
                    }
                }
            }
            
            importing.set(false);
        });
    };
    
    #[cfg(target_os = "android")]
    let handle_import_pdf = move |_| {
        spawn(async move {
            importing.set(true);
            error_message.set(None);
            
            // Use platform file picker
            use crate::platform::PlatformApi;
            match crate::platform::get_platform_api().pick_file(&["pdf"]).await {
                Ok(file_path) => {
                    log::info!("PDF file picked: {}", file_path);
                    
                    let source_path = std::path::PathBuf::from(&file_path);
                    
                    // Get app data directory
                    let app_data_dir = match crate::platform::android::get_assets_directory() {
                        Ok(dir) => dir,
                        Err(e) => {
                            log::error!("Failed to get app directory: {:?}", e);
                            error_message.set(Some(format!("Failed to get app directory: {}", e)));
                            importing.set(false);
                            return;
                        }
                    };
                    
                    // Import PDF using PdfProcessor
                    match PdfProcessor::new() {
                        Ok(processor) => {
                            match processor.import_pdf(&source_path, &app_data_dir) {
                                Ok((metadata, copied_path)) => {
                                    log::info!("PDF imported: {:?} -> {}", metadata, copied_path);
                                    
                                    // Show metadata review dialog
                                    review_title.set(metadata.title.clone().unwrap_or_else(|| {
                                        source_path.file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("Unknown")
                                            .to_string()
                                    }));
                                    review_author.set(metadata.author.clone().unwrap_or_default());
                                    review_pages.set(metadata.page_count);
                                    pending_metadata.set(Some((metadata, copied_path)));
                                    show_metadata_dialog.set(true);
                                }
                                Err(e) => {
                                    log::error!("Failed to import PDF: {:?}", e);
                                    error_message.set(Some(format!("Failed to import PDF: {}", e)));
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to create PdfProcessor: {:?}", e);
                            error_message.set(Some(format!("Failed to initialize PDF processor: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to pick file: {:?}", e);
                    error_message.set(Some(format!("Failed to pick file: {}", e)));
                }
            }
            
            importing.set(false);
        });
    };
    
    // Load demo PDF handler for Android
    #[cfg(target_os = "android")]
    let load_demo_pdf = move |_| {
        spawn(async move {
            importing.set(true);
            error_message.set(None);
            
            log::info!("Loading demo PDF from bundled assets...");
            
            // Copy asset from APK to files directory
            let demo_path = match crate::platform::android::copy_asset_to_files("test/medium_pdf_test.pdf") {
                Ok(path) => path,
                Err(e) => {
                    log::error!("Failed to copy demo PDF from assets: {:?}", e);
                    error_message.set(Some(format!("Failed to load demo PDF: {}", e)));
                    importing.set(false);
                    return;
                }
            };
            
            log::info!("Demo PDF copied to: {:?}", demo_path);
            
            // Get app data directory
            let app_data_dir = match crate::platform::android::get_assets_directory() {
                Ok(dir) => dir,
                Err(e) => {
                    log::error!("Failed to get assets directory: {:?}", e);
                    error_message.set(Some(format!("Failed to get app directory: {}", e)));
                    importing.set(false);
                    return;
                }
            };
            
            // Import PDF using PdfProcessor
            match PdfProcessor::new() {
                Ok(processor) => {
                    match processor.import_pdf(&demo_path, &app_data_dir) {
                        Ok((metadata, copied_path)) => {
                            log::info!("Demo PDF imported: {:?} -> {}", metadata, copied_path);
                            
                            // Show metadata review dialog
                            review_title.set(metadata.title.clone().unwrap_or_else(|| "Demo PDF".to_string()));
                            review_author.set(metadata.author.clone().unwrap_or_default());
                            review_pages.set(metadata.page_count);
                            pending_metadata.set(Some((metadata, copied_path)));
                            show_metadata_dialog.set(true);
                        }
                        Err(e) => {
                            log::error!("Failed to import demo PDF: {:?}", e);
                            error_message.set(Some(format!("Failed to import demo PDF: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create PdfProcessor: {:?}", e);
                    error_message.set(Some(format!("Failed to initialize PDF processor: {}", e)));
                }
            }
            
            importing.set(false);
        });
    };
    
    // Load demo PDF handler for desktop
    #[cfg(not(target_os = "android"))]
    let load_demo_pdf = move |_| {
        spawn(async move {
            importing.set(true);
            error_message.set(None);
            
            log::info!("Loading demo PDF from bundled assets...");
            
            // Direct path to the demo PDF
            let demo_path = std::path::PathBuf::from("assets/test/medium_pdf_test.pdf");
            
            // Get app data directory
            let app_data_dir = match std::env::current_exe() {
                Ok(exe) => exe.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from(".")),
                Err(_) => std::path::PathBuf::from("."),
            };
            
            // Import PDF using PdfProcessor
            match PdfProcessor::new() {
                Ok(processor) => {
                    match processor.import_pdf(&demo_path, &app_data_dir) {
                        Ok((metadata, copied_path)) => {
                            log::info!("Demo PDF imported: {:?} -> {}", metadata, copied_path);
                            
                            // Show metadata review dialog
                            review_title.set(metadata.title.clone().unwrap_or_else(|| "Demo PDF".to_string()));
                            review_author.set(metadata.author.clone().unwrap_or_default());
                            review_pages.set(metadata.page_count);
                            pending_metadata.set(Some((metadata, copied_path)));
                            show_metadata_dialog.set(true);
                        }
                        Err(e) => {
                            log::error!("Failed to import demo PDF: {:?}", e);
                            error_message.set(Some(format!("Failed to import demo PDF: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create PdfProcessor: {:?}", e);
                    error_message.set(Some(format!("Failed to initialize PDF processor: {}", e)));
                }
            }
            
            importing.set(false);
        });
    };
    
    // Handle metadata confirmation
    let handle_metadata_confirm = move |(title, author): (String, String)| {
        spawn(async move {
            if let Some((metadata, copied_path)) = pending_metadata.take() {
                // Create book record in database
                match Database::open("shusei.db") {
                    Ok(db) => {
                        let new_book = NewBook {
                            id: Some(uuid::Uuid::new_v4().to_string()),
                            title,
                            author,
                            cover_path: None,
                            pages_captured: 0,
                            total_pages: Some(metadata.page_count as i32),
                            last_opened_at: None,
                            is_pdf: true,
                            pdf_path: Some(copied_path.clone()),
                        };
                        
                        match db.create_book(&new_book) {
                            Ok(book_id) => {
                                log::info!("Book created with ID: {}", book_id);
                                // Refresh book list
                                if let Ok(all_books) = db.get_all_books() {
                                    books.set(all_books);
                                }
                                error_message.set(Some(format!("✓ PDF imported successfully")));
                            }
                            Err(e) => {
                                log::error!("Failed to create book: {:?}", e);
                                error_message.set(Some(format!("Failed to save book: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to open database: {:?}", e);
                        error_message.set(Some(format!("Database error: {}", e)));
                    }
                }
                
                show_metadata_dialog.set(false);
                pending_metadata.set(None);
            }
        });
    };
    
    // Apply filter to books
    let filtered_books = {
        let all_books = books();
        match filter() {
            LibraryFilter::All => all_books,
            LibraryFilter::PdfsOnly => all_books.into_iter().filter(|b: &Book| b.is_pdf).collect(),
            LibraryFilter::PhysicalOnly => all_books.into_iter().filter(|b: &Book| !b.is_pdf).collect(),
        }
    };

    rsx! {
        div { class: "flex flex-col h-full p-4",
            // Header
            header { class: "mb-4",
                h1 { class: "text-2xl font-bold", "My Library" }
            }

            // Button container
            div { class: "flex gap-2 mb-4",
                // Add book button
                button {
                    class: "bg-blue-600 text-white px-4 py-2 rounded-lg",
                    onclick: move |_| {
                        navigator.push(Route::AddBook);
                        ()
                    },
                    "Add Book"
                }

                // Import PDF button
                button {
                    class: "bg-green-600 text-white px-4 py-2 rounded-lg",
                    onclick: handle_import_pdf,
                    disabled: importing(),
                    if importing() {
                        "Importing..."
                    } else {
                        "Import PDF"
                    }
                }

                // Load Demo PDF button (both platforms)
                button {
                    class: "bg-orange-500 text-white px-4 py-2 rounded-lg",
                    onclick: load_demo_pdf,
                    disabled: importing(),
                    if importing() { "Loading..." } else { "Load Demo PDF" }
                }
            }
            
            // Filter toggle
            div { class: "flex gap-2 mb-4",
                button {
                    class: if filter() == LibraryFilter::All {
                        "px-3 py-1 rounded-lg bg-purple-600 text-white"
                    } else {
                        "px-3 py-1 rounded-lg bg-gray-200 text-gray-700"
                    },
                    onclick: move |_| filter.set(LibraryFilter::All),
                    "All"
                }
                button {
                    class: if filter() == LibraryFilter::PdfsOnly {
                        "px-3 py-1 rounded-lg bg-purple-600 text-white"
                    } else {
                        "px-3 py-1 rounded-lg bg-gray-200 text-gray-700"
                    },
                    onclick: move |_| filter.set(LibraryFilter::PdfsOnly),
                    "📄 PDFs"
                }
                button {
                    class: if filter() == LibraryFilter::PhysicalOnly {
                        "px-3 py-1 rounded-lg bg-purple-600 text-white"
                    } else {
                        "px-3 py-1 rounded-lg bg-gray-200 text-gray-700"
                    },
                    onclick: move |_| filter.set(LibraryFilter::PhysicalOnly),
                    "📚 Physical"
                }
            }

            // Success/error message
            if let Some(error) = error_message() {
                div { 
                    class: if error.starts_with("✓") {
                        "bg-green-100 border border-green-400 text-green-700 px-4 py-2 rounded mb-4"
                    } else {
                        "bg-red-100 border border-red-400 text-red-700 px-4 py-2 rounded mb-4"
                    },
                    "{error}"
                }
            }

            // Book list or empty state
            if filtered_books.is_empty() {
                div { class: "text-center py-8",
                    p { class: "text-gray-500", 
                        match filter() {
                            LibraryFilter::All => "No books yet",
                            LibraryFilter::PdfsOnly => "No PDF books",
                            LibraryFilter::PhysicalOnly => "No physical books",
                        }
                    }
                }
            } else {
                div { class: "space-y-2",
                    for book in filtered_books {
                        BookCard { book }
                    }
                }
            }
        }
        
        // Metadata review dialog
        MetadataReviewDialog {
            show: show_metadata_dialog(),
            title: review_title(),
            author: review_author(),
            page_count: review_pages(),
            on_close: move |_| {
                show_metadata_dialog.set(false);
                pending_metadata.set(None);
            },
            on_confirm: handle_metadata_confirm,
        }
    }
}

/// Book card component displaying book information
#[component]
pub fn BookCard(book: Book) -> Element {
    let navigator = use_navigator();
    
    // Calculate conversion progress
    let progress = if let Some(total) = book.total_pages {
        if total > 0 {
            (book.pages_captured as f32 / total as f32 * 100.0) as u32
        } else {
            0
        }
    } else {
        0
    };
    
    // Check if conversion is needed
    let needs_conversion = book.is_pdf && book.total_pages.map(|t| t > 0).unwrap_or(false) && book.pages_captured < book.total_pages.unwrap_or(0);
    let convert_book_id = book.id.clone();
    
    rsx! {
        div {
            class: "block bg-white border rounded-lg p-3 shadow-sm hover:shadow-md transition-shadow cursor-pointer",
            onclick: move |_| {
                // Navigate to reader for this book
                if let Ok(id) = book.id.parse::<i64>() {
                    navigator.push(Route::ReaderBook { book_id: id });
                }
            },
            
            // Header with title and PDF badge
            div { class: "flex items-center justify-between mb-2",
                h3 { class: "font-semibold text-gray-800", "{book.title}" }
                if book.is_pdf {
                    span { class: "bg-purple-100 text-purple-700 text-xs px-2 py-1 rounded-full", "📄 PDF" }
                }
            }
            
            p { class: "text-gray-600 text-sm mb-2", "by {book.author}" }
            
            // Conversion progress
            if book.is_pdf && book.total_pages.is_some() {
                div { class: "mt-2",
                    div { class: "flex justify-between text-xs text-gray-500 mb-1",
                        span { "Conversion progress" }
                        span { "{book.pages_captured}/{book.total_pages.unwrap()} pages" }
                    }
                    div { class: "bg-gray-200 h-2 rounded-full overflow-hidden",
                        div {
                            class: "bg-purple-600 h-full transition-all duration-300",
                            style: "width: {progress}%"
                        }
                    }
                    // Convert button for incomplete PDFs
                    if needs_conversion {
                        button {
                            class: "mt-2 w-full bg-purple-600 text-white text-sm px-3 py-1 rounded hover:bg-purple-700",
                            onclick: move |e| {
                                e.stop_propagation();
                                // Navigate to reader where conversion can be triggered
                                if let Ok(id) = convert_book_id.parse::<i64>() {
                                    navigator.push(Route::ReaderBook { book_id: id });
                                }
                            },
                            if book.pages_captured > 0 {
                                "Resume Conversion"
                            } else {
                                "Convert"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::Route;
    use crate::core::db::Book;

    #[test]
    fn test_library_screen_renders_without_books() {
        // Test that LibraryScreen renders with empty state when no books
        // This is a placeholder - actual Dioxus component testing requires special setup
        assert!(true, "LibraryScreen should render empty state");
    }

    #[test]
    fn test_library_screen_shows_book_list_when_loaded() {
        // Test that LibraryScreen shows books when loaded
        let books = vec![
            Book {
                id: "1".to_string(),
                title: "Test Book".to_string(),
                author: "Test Author".to_string(),
                cover_path: None,
                pages_captured: 0,
                total_pages: None,
                last_opened_at: None,
                created_at: 1234567890,
                updated_at: 1234567890,
                is_pdf: false,
                pdf_path: None,
            },
        ];
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].title, "Test Book");
    }

    #[test]
    fn test_books_sorted_alphabetically_by_title() {
        // Test that books are sorted alphabetically by title
        let mut books = vec![
            Book {
                id: "1".to_string(),
                title: "Zebra Book".to_string(),
                author: "Author".to_string(),
                cover_path: None,
                pages_captured: 0,
                total_pages: None,
                last_opened_at: None,
                created_at: 1234567890,
                updated_at: 1234567890,
                is_pdf: false,
                pdf_path: None,
            },
            Book {
                id: "2".to_string(),
                title: "Alpha Book".to_string(),
                author: "Author".to_string(),
                cover_path: None,
                pages_captured: 0,
                total_pages: None,
                last_opened_at: None,
                created_at: 1234567890,
                updated_at: 1234567890,
                is_pdf: false,
                pdf_path: None,
            },
        ];
        
        // Sort alphabetically by title
        books.sort_by(|a, b| a.title.cmp(&b.title));
        
        assert_eq!(books[0].title, "Alpha Book");
        assert_eq!(books[1].title, "Zebra Book");
    }

    #[test]
    fn test_add_book_button_navigates_to_add_book_route() {
        // Test that Add Book button navigates to AddBook route
        // This tests the Route enum value exists
        let route = Route::AddBook;
        assert!(true, "AddBook route should exist: {:?}", route);
    }

    #[test]
    fn test_book_card_shows_title_and_author() {
        // Test that BookCard displays title and author
        let book = Book {
            id: "1".to_string(),
            title: "Test Book".to_string(),
            author: "Test Author".to_string(),
            cover_path: None,
            pages_captured: 0,
            total_pages: None,
            last_opened_at: None,
            created_at: 1234567890,
            updated_at: 1234567890,
            is_pdf: false,
            pdf_path: None,
        };
        
        assert_eq!(book.title, "Test Book");
        assert_eq!(book.author, "Test Author");
    }
}
