//! Camera page component
//!
//! This is the main UI component for the camera capture and OCR functionality.
//! It demonstrates the critical JNI camera PoC.

use dioxus::prelude::*;
use base64::{Engine as _, engine::general_purpose};

use crate::platform::{get_platform_api, CameraResult, PlatformApi};
use crate::core::ocr::{NdlocrEngine, OcrEngine};
use crate::core::db::{Database, NewBookPage};
use crate::core::storage::StorageService;

/// Camera page component
#[component]
pub fn CameraPage(#[props(into)] book_id: Option<String>) -> Element {
    // Clone book_id for use in closures
    let book_id_for_save = book_id.clone();
    
    // State for camera capture
    let mut captured_image = use_signal(|| None::<Vec<u8>>);
    let mut image_dimensions = use_signal(|| None::<(u32, u32)>);
    let mut is_capturing = use_signal(|| false);
    let mut is_processing_ocr = use_signal(|| false);
    let mut is_saving_page = use_signal(|| false);
    let mut ocr_result = use_signal(|| None::<String>);
    let mut error_message = use_signal(|| None::<String>);
    
    // NEW: Page number state
    let mut page_number = use_signal(|| 1u32);
    
    // NEW: OCR engine state
    let mut ocr_engine = use_signal(|| None::<NdlocrEngine>);
    let mut is_engine_ready = use_signal(|| false);
    let mut is_engine_loading = use_signal(|| true);
    
    // Log book_id on mount
    if let Some(ref id) = book_id {
        log::debug!("Camera page mounted for book_id={}", id);
    }
    
    // NEW: Initialize OCR engine on mount
    use_effect(move || {
        spawn(async move {
            is_engine_loading.set(true);
            is_engine_ready.set(false);
            
            // Get app data directory
            let app_data_dir = std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            
            let model_dir = app_data_dir.join("models");
            
            // Create and initialize OCR engine
            let mut engine = NdlocrEngine::new(&model_dir, "ja");
            match engine.initialize().await {
                Ok(()) => {
                    log::info!("OCR engine initialized, ready={}", engine.is_ready());
                    ocr_engine.set(Some(engine));
                    is_engine_ready.set(true);
                }
                Err(e) => {
                    log::error!("OCR engine initialization failed: {}", e);
                    error_message.set(Some(format!("OCR engine failed to load: {}", e)));
                }
            }
            
            is_engine_loading.set(false);
        });
    });
    
    // Capture image from camera
    let capture = move |_| {
        spawn(async move {
            is_capturing.set(true);
            error_message.set(None);
            
            let platform = get_platform_api();
            
            // Request camera permission first
            if !platform.has_camera_permission().await {
                match platform.request_camera_permission().await {
                    Ok(granted) => {
                        if !granted {
                            error_message.set(Some("Camera permission denied".to_string()));
                            is_capturing.set(false);
                            return;
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Permission error: {}", e)));
                        is_capturing.set(false);
                        return;
                    }
                }
            }
            
            // Capture image
            match platform.capture_image().await {
                Ok(result) => {
                    log::info!("Image captured: {}x{} ({} bytes)", 
                        result.width, result.height, result.image_data.len());
                    captured_image.set(Some(result.image_data));
                    image_dimensions.set(Some((result.width, result.height)));
                    ocr_result.set(None);
                }
                Err(e) => {
                    log::error!("Capture failed: {}", e);
                    error_message.set(Some(format!("Capture failed: {}", e)));
                }
            }
            
            is_capturing.set(false);
        });
    };
    
    // Run OCR on captured image
    let run_ocr = move |_| {
        spawn(async move {
            let image = captured_image();
            if image.is_none() {
                error_message.set(Some("No image captured".to_string()));
                return;
            }
            
            // Check engine readiness
            let engine = ocr_engine();
            if engine.is_none() || !is_engine_ready() {
                error_message.set(Some("OCR engine not ready".to_string()));
                return;
            }
            
            is_processing_ocr.set(true);
            error_message.set(None);
            
            // Use actual OCR engine
            let engine = engine.unwrap();
            match engine.process_image(&image.unwrap()).await {
                Ok(result) => {
                    log::info!("OCR completed: {} chars, confidence {:.2}", 
                        result.plain_text.len(), result.confidence);
                    ocr_result.set(Some(format!(
                        "OCR Result (confidence: {:.1}%):\n{}",
                        result.confidence * 100.0,
                        result.plain_text
                    )));
                }
                Err(e) => {
                    log::error!("OCR processing failed: {}", e);
                    error_message.set(Some(format!("OCR failed: {}", e)));
                }
            }
            
            is_processing_ocr.set(false);
        });
    };
    
    // Generate base64 data URI for image display
    let get_image_uri = || {
        captured_image().map(|data| {
            let base64_data = general_purpose::STANDARD.encode(&data);
            format!("data:image/jpeg;base64,{}", base64_data)
        })
    };
    
    rsx! {
        div { class: "flex flex-col h-full",
            // Header
            header { class: "bg-green-600 text-white p-4 flex items-center",
                Link {
                    to: crate::app::Route::Home,
                    class: "mr-4 text-white",
                    "←"
                }
                h1 { class: "text-xl font-bold", "📷 Capture Page" }
            }
            
            // Main content
            div { class: "flex-1 p-4 flex flex-col items-center justify-center",
                // Error message
                if let Some(error) = error_message() {
                    div { class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4 w-full",
                        "{error}"
                    }
                }
                
                // NEW: Engine loading indicator
                if is_engine_loading() {
                    div { class: "bg-blue-50 border border-blue-200 text-blue-700 px-4 py-3 rounded mb-4 w-full flex items-center",
                        div { class: "animate-spin mr-3", "⏳" }
                        span { "Loading OCR engine..." }
                    }
                }
                
                // Camera preview / captured image
                if let Some(image_uri) = get_image_uri() {
                    // Show captured image
                    div { class: "w-full max-w-md",
                        // Display image preview using base64 data URI
                        div { class: "bg-gray-200 h-64 flex items-center justify-center rounded-lg mb-4 overflow-hidden",
                            img {
                                src: "{image_uri}",
                                class: "max-w-full max-h-full object-contain",
                                alt: "Captured image"
                            }
                        }
                        
                        // Show dimensions if available
                        if let Some((width, height)) = image_dimensions() {
                            p { class: "text-sm text-gray-500 text-center mb-2",
                                "{width} x {height} pixels"
                            }
                        }
                        
                        // NEW: Page number input
                        div { class: "mb-4",
                            label { 
                                class: "block text-sm font-medium text-gray-700 mb-1",
                                "Page Number"
                            }
                            input {
                                type: "number",
                                min: "1",
                                value: "{page_number()}",
                                oninput: move |e| {
                                    if let Ok(num) = e.value().parse::<u32>() {
                                        if num >= 1 {
                                            page_number.set(num);
                                        }
                                    }
                                },
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-green-500"
                            }
                        }
                        
                        // Action buttons
                        div { class: "flex gap-2",
                            button {
                                class: "flex-1 bg-blue-600 text-white p-3 rounded-lg disabled:bg-gray-400",
                                onclick: run_ocr,
                                disabled: is_processing_ocr() || !is_engine_ready(),
                                title: if !is_engine_ready() { "OCR engine loading..." } else { "" },
                                if is_engine_loading() {
                                    "Loading..."
                                } else if is_processing_ocr() {
                                    "Processing..."
                                } else if !is_engine_ready() {
                                    "Engine Loading..."
                                } else {
                                    "🔍 Run OCR"
                                }
                            }
                            button {
                                class: "flex-1 bg-gray-600 text-white p-3 rounded-lg",
                                onclick: move |_| {
                                    captured_image.set(None);
                                    image_dimensions.set(None);
                                    ocr_result.set(None);
                                },
                                "🔄 Retake"
                            }
                        }
                    }
                } else {
                    // Capture button
                    if is_capturing() {
                        div { class: "text-center",
                            div { class: "animate-spin text-4xl mb-4", "⏳" }
                            p { "Opening camera..." }
                        }
                    } else {
                        button {
                            class: "bg-green-600 text-white px-8 py-4 rounded-lg text-xl",
                            onclick: capture,
                            "📷 Take Photo"
                        }
                    }
                }
                
                // OCR result
                if let Some(ref result) = ocr_result() {
                    div { class: "mt-4 w-full max-w-md",
                        h2 { class: "text-lg font-semibold mb-2", "OCR Result" }
                        div { class: "bg-gray-100 p-4 rounded-lg",
                            p { "{result}" }
                        }
                        
                        // Save button
                        button {
                            class: "mt-4 w-full bg-purple-600 text-white p-3 rounded-lg disabled:bg-gray-400",
                            onclick: move |_| {
                                let book_id = book_id_for_save.clone();
                                let page_num = page_number();
                                let image_data = captured_image();
                                let ocr_text = ocr_result();
                                
                                if book_id.is_none() {
                                    error_message.set(Some("No book ID - cannot save page".to_string()));
                                    return;
                                }
                                
                                if image_data.is_none() {
                                    error_message.set(Some("No image data to save".to_string()));
                                    return;
                                }
                                
                                spawn(async move {
                                    is_saving_page.set(true);
                                    error_message.set(None);
                                    
                                    // Get app data directory
                                    let app_data_dir = std::env::current_exe()
                                        .ok()
                                        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                                        .unwrap_or_else(|| std::path::PathBuf::from("."));
                                    
                                    let book_id = book_id.unwrap();
                                    let image_data = image_data.unwrap();
                                    
                                    // Step 1: Save image via StorageService
                                    let storage = match StorageService::new(app_data_dir.clone()) {
                                        Ok(s) => s,
                                        Err(e) => {
                                            log::error!("Storage initialization failed: {}", e);
                                            error_message.set(Some(format!("Storage error: {}", e)));
                                            is_saving_page.set(false);
                                            return;
                                        }
                                    };
                                    let image_path = match storage.save_page_image(&image_data, &book_id) {
                                        Ok(path) => {
                                            log::info!("Page image saved: {}", path);
                                            path
                                        }
                                        Err(e) => {
                                            log::error!("Storage save failed: {}", e);
                                            error_message.set(Some(format!("Failed to save image: {}", e)));
                                            is_saving_page.set(false);
                                            return;
                                        }
                                    };
                                    
                                    // Step 2: Save metadata to database
                                    let db_path = app_data_dir.join("shusei.db");
                                    let db = match Database::open(&db_path) {
                                        Ok(db) => db,
                                        Err(e) => {
                                            log::error!("Database open failed: {}", e);
                                            error_message.set(Some(format!("Database error: {}", e)));
                                            is_saving_page.set(false);
                                            return;
                                        }
                                    };
                                    
                                    // Parse OCR result to extract text and confidence
                                    let ocr_text = ocr_text.unwrap_or_default();
                                    let (plain_text, confidence) = if ocr_text.is_empty() {
                                        (String::new(), None)
                                    } else {
                                        // Extract confidence from OCR result display format
                                        let lines: Vec<&str> = ocr_text.lines().collect();
                                        let text = lines.get(1..).unwrap_or(&[]).join("\n");
                                        
                                        // Parse confidence from first line: "OCR Result (confidence: 85.3%):"
                                        let conf = lines.first().and_then(|line| {
                                            line.split("confidence: ").nth(1)
                                                .and_then(|s| s.trim_end_matches('%').parse::<f32>().ok())
                                                .map(|c| c / 100.0)
                                        });
                                        
                                        (text, conf)
                                    };
                                    
                                    // Create NewBookPage struct
                                    let new_page = NewBookPage {
                                        book_id: book_id.clone(),
                                        page_number: page_num as i32,
                                        image_path,
                                        ocr_markdown: plain_text.clone(),
                                        ocr_text_plain: plain_text.clone(),
                                        confidence,
                                    };
                                    
                                    // Step 3: Save to database
                                    match db.save_page(&new_page) {
                                        Ok(page_id) => {
                                            log::info!("Page saved: book_id={}, page={}, path={}, db_id={}", 
                                                book_id, page_num, new_page.image_path, page_id);
                                            
                                            // Update pages_captured count in books table
                                            if let Ok(book) = db.get_book(&book_id) {
                                                if let Some(mut book) = book {
                                                    book.pages_captured += 1;
                                                    let _ = db.update_book(&book);
                                                }
                                            }
                                            
                                            error_message.set(None);
                                            ocr_result.set(Some(format!("✅ Page {} saved successfully!\n\n{}", page_num, ocr_text)));
                                        }
                                        Err(e) => {
                                            // Check for UNIQUE constraint violation (page already exists)
                                            let error_msg = e.to_string();
                                            if error_msg.contains("UNIQUE constraint failed") {
                                                log::warn!("Page {} already exists for book {}", page_num, book_id);
                                                error_message.set(Some(
                                                    format!("Page {} already exists for this book. Please use a different page number or overwrite.", page_num)
                                                ));
                                            } else {
                                                log::error!("Database save failed: {}", e);
                                                error_message.set(Some(format!("Failed to save to database: {}", e)));
                                            }
                                        }
                                    }
                                    
                                    is_saving_page.set(false);
                                });
                            },
                            disabled: is_saving_page() || ocr_result().is_none(),
                            if is_saving_page() {
                                "💾 Saving..."
                            } else if ocr_result().is_none() {
                                "💾 Save Page (run OCR first)"
                            } else {
                                "💾 Save Page"
                            }
                        }
                    }
                }
                
                // NEW: Debug info showing book_id
                if let Some(ref id) = book_id {
                    div { class: "mt-4 p-3 bg-gray-50 rounded-lg border border-gray-200",
                        p { class: "text-xs text-gray-500", "Book ID: {id}" }
                        p { class: "text-xs text-gray-500", "Engine Ready: {is_engine_ready()}" }
                    }
                }
            }
            
            // Status bar
            div { class: "bg-gray-100 p-2 text-center text-sm text-gray-600",
                "Week 1 PoC: JNI Camera Capture"
            }
        }
    }
}
