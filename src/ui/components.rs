//! Shared UI components
//!
//! This module contains reusable UI components used across the application.

use dioxus::prelude::*;
use dioxus_router::Link;

use crate::app::Route;
use crate::core::pdf::ConversionStage as PdfConversionStage;

// Re-export ConversionStage for convenience
pub use crate::core::pdf::ConversionStage;

/// Loading spinner component
#[component]
pub fn LoadingSpinner() -> Element {
    rsx! {
        div { class: "flex items-center justify-center",
            div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" }
        }
    }
}

/// Error message component
#[component]
pub fn ErrorMessage(message: String) -> Element {
    rsx! {
        div { class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded",
            "{message}"
        }
    }
}

/// Button component
#[component]
pub fn Button(
    text: String,
    onclick: EventHandler<MouseEvent>,
    variant: Option<String>,
    disabled: Option<bool>,
) -> Element {
    let variant_class = match variant.as_deref() {
        Some("primary") => "bg-blue-600 text-white",
        Some("secondary") => "bg-gray-200 text-gray-800",
        Some("danger") => "bg-red-600 text-white",
        _ => "bg-blue-600 text-white",
    };

    let disabled_class = if disabled.unwrap_or(false) {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    };

    rsx! {
        button {
            class: "px-4 py-2 rounded-lg {variant_class} {disabled_class}",
            onclick: move |e| onclick.call(e),
            disabled: disabled.unwrap_or(false),
            "{text}"
        }
    }
}

/// Card component
#[component]
pub fn Card(children: Element) -> Element {
    rsx! {
        div { class: "bg-white border rounded-lg p-4 shadow-sm",
            {children}
        }
    }
}

/// Header component
#[component]
pub fn PageHeader(title: String, back_to: Option<Route>) -> Element {
    rsx! {
        header { class: "bg-blue-600 text-white p-4",
            div { class: "flex items-center",
                if let Some(route) = back_to {
                    Link {
                        to: route,
                        class: "mr-4 text-white",
                        "←"
                    }
                }
                h1 { class: "text-xl font-bold", "{title}" }
            }
        }
    }
}

/// Conversion progress display component
/// Shows stage-based progress with visual indicators
#[component]
pub fn ConversionProgressDisplay(
    stage: ConversionStage,
    current_page: u32,
    total_pages: u32,
) -> Element {
    // Calculate percentage
    let percentage = if total_pages > 0 {
        (current_page as f32 / total_pages as f32 * 100.0) as u32
    } else {
        0
    };

    // Stage-specific styling
    let (icon, color, message) = match stage {
        ConversionStage::Rendering => (
            "📄",
            "text-blue-600",
            format!("Rendering page {} of {}...", current_page, total_pages),
        ),
        ConversionStage::OcrProcessing => (
            "🔍",
            "text-purple-600",
            format!("Processing OCR page {} of {}...", current_page, total_pages),
        ),
        ConversionStage::Complete => ("✓", "text-green-600", "Conversion complete!".to_string()),
    };

    rsx! {
        div { class: "w-full",
            // Stage indicator
            div { class: "flex items-center justify-center mb-3",
                span { class: "text-2xl mr-2", "{icon}" }
                span { class: "font-semibold {color}", "{message}" }
            }

            // Progress bar
            div { class: "bg-gray-200 h-3 rounded-full overflow-hidden",
                div {
                    class: if stage == ConversionStage::Complete {
                        "bg-green-600 h-full transition-all duration-300"
                    } else if stage == ConversionStage::OcrProcessing {
                        "bg-purple-600 h-full transition-all duration-300"
                    } else {
                        "bg-blue-600 h-full transition-all duration-300"
                    },
                    style: "width: {percentage}%"
                }
            }

            // Page count
            p { class: "text-xs text-gray-500 mt-1 text-center",
                "Page {current_page} of {total_pages}"
            }
        }
    }
}

/// Empty state component
#[component]
pub fn EmptyState(
    icon: String,
    message: String,
    hint: Option<String>,
    action_label: Option<String>,
    on_action: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        div { class: "text-center py-8",
            p { class: "text-4xl mb-4", "{icon}" }
            p { class: "text-gray-500", "{message}" }

            if let Some(hint) = hint {
                p { class: "text-sm text-gray-400 mt-2", "{hint}" }
            }

            if let Some(label) = action_label {
                if let Some(handler) = on_action {
                    button {
                        class: "mt-4 bg-blue-600 text-white px-6 py-2 rounded-lg",
                        onclick: move |e| handler.call(e),
                        "{label}"
                    }
                }
            }
        }
    }
}

/// Progress bar component
#[component]
pub fn ProgressBar(progress: f32, label: Option<String>) -> Element {
    rsx! {
        div { class: "w-full",
            div { class: "bg-gray-200 h-2 rounded-full overflow-hidden",
                div {
                    class: "bg-blue-600 h-full transition-all duration-300",
                    style: "width: {progress}%"
                }
            }
            if let Some(text) = label {
                p { class: "text-xs text-gray-500 mt-1", "{text}" }
            }
        }
    }
}

/// Tab bar component
#[component]
pub fn TabBar(tabs: Vec<TabItem>, active_tab: usize, on_select: EventHandler<usize>) -> Element {
    rsx! {
        div { class: "flex border-b",
            for (index, tab) in tabs.into_iter().enumerate() {
                button {
                    class: if index == active_tab {
                        "flex-1 py-3 text-center border-b-2 border-blue-600 text-blue-600"
                    } else {
                        "flex-1 py-3 text-center text-gray-500"
                    },
                    onclick: move |_| on_select.call(index),
                    "{tab.label}"
                }
            }
        }
    }
}

/// Tab item for TabBar
#[derive(Clone, PartialEq)]
pub struct TabItem {
    pub label: String,
    pub icon: Option<String>,
}

/// Page jump modal component
#[component]
pub fn PageJumpModal(
    show: bool,
    total_pages: i32,
    on_close: EventHandler<()>,
    on_submit: EventHandler<i32>,
) -> Element {
    let mut input_value = use_signal(|| String::new());

    if !show {
        return rsx! {};
    }

    rsx! {
        // Modal backdrop
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| on_close.call(()),

            // Modal content
            div {
                class: "bg-white rounded-lg p-6 w-full max-w-xs",
                onclick: move |e| e.stop_propagation(),

                h2 { class: "text-lg font-bold mb-4", "Go to Page" }

                input {
                    r#type: "number",
                    min: "1",
                    max: "{total_pages}",
                    value: "{input_value()}",
                    oninput: move |e| input_value.set(e.value()),
                    placeholder: format!("1 - {total_pages}"),
                    class: "w-full border rounded-lg px-3 py-2 mb-4"
                }

                div { class: "flex space-x-2",
                    button {
                        class: "flex-1 bg-gray-200 text-gray-800 py-2 rounded-lg",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "flex-1 bg-purple-600 text-white py-2 rounded-lg",
                        onclick: move |_| {
                            if let Ok(page) = input_value().parse::<i32>() {
                                if page >= 1 && page <= total_pages {
                                    on_submit.call(page);
                                    on_close.call(());
                                }
                            }
                        },
                        "Go"
                    }
                }
            }
        }
    }
}
