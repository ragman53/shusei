//! Camera page component
//!
//! This is the main UI component for the camera capture and OCR functionality.
//! It demonstrates the critical JNI camera PoC.

use dioxus::prelude::*;
use base64::{Engine as _, engine::general_purpose};

use crate::platform::{get_platform_api, CameraResult, PlatformApi};

/// Camera page component
#[component]
pub fn CameraPage() -> Element {
    // State for camera capture
    let mut captured_image = use_signal(|| None::<Vec<u8>>);
    let mut image_dimensions = use_signal(|| None::<(u32, u32)>);
    let mut is_capturing = use_signal(|| false);
    let mut is_processing = use_signal(|| false);
    let mut ocr_result = use_signal(|| None::<String>);
    let mut error_message = use_signal(|| None::<String>);
    
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
                return;
            }
            
            is_processing.set(true);
            error_message.set(None);
            
            // TODO: Call OCR engine
            // For now, just set a placeholder
            ocr_result.set(Some("OCR processing will be implemented in Week 3-5".to_string()));
            
            is_processing.set(false);
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
                        
                        // Action buttons
                        div { class: "flex gap-2",
                            button {
                                class: "flex-1 bg-blue-600 text-white p-3 rounded-lg",
                                onclick: run_ocr,
                                disabled: is_processing(),
                                if is_processing() {
                                    "Processing..."
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
                            class: "mt-4 w-full bg-purple-600 text-white p-3 rounded-lg",
                            onclick: move |_| {
                                // TODO: Save to database
                                log::info!("Save note clicked");
                            },
                            "💾 Save as Note"
                        }
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