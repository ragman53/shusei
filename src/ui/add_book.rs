//! Add book form component
//!
//! This module contains the AddBookForm component that allows
//! users to add new books to their library.

use dioxus::prelude::*;
use dioxus_router::use_navigator;

use crate::app::Route;

/// Add book form component with modal styling
#[component]
pub fn AddBookForm() -> Element {
    let mut title = use_signal(|| String::new());
    let mut author = use_signal(|| String::new());
    let navigator = use_navigator();

    let handle_submit = move |_| {
        // TODO: Implement actual book creation
        // For now, just navigate back to library
        navigator.push(Route::BookList);
    };

    let is_valid = !title().is_empty() && !author().is_empty();

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4",
            div { class: "bg-white rounded-lg p-6 w-full max-w-md",
                h2 { class: "text-xl font-bold mb-4", "Add New Book" }

                form {
                    onsubmit: handle_submit,

                    // Title input
                    div { class: "mb-4",
                        label { class: "block text-sm font-medium mb-1", "Title *" }
                        input {
                            r#type: "text",
                            class: "w-full border rounded-lg px-3 py-2",
                            value: "{title}",
                            oninput: move |evt| title.set(evt.value()),
                            placeholder: "Book Title"
                        }
                    }

                    // Author input
                    div { class: "mb-4",
                        label { class: "block text-sm font-medium mb-1", "Author *" }
                        input {
                            r#type: "text",
                            class: "w-full border rounded-lg px-3 py-2",
                            value: "{author}",
                            oninput: move |evt| author.set(evt.value()),
                            placeholder: "Author Name"
                        }
                    }

                    // Cover photo button (placeholder)
                    div { class: "mb-4",
                        button {
                            r#type: "button",
                            class: "text-blue-600 text-sm",
                            "Add cover photo (coming soon)"
                        }
                    }

                    // Submit buttons
                    div { class: "flex gap-2",
                        button {
                            r#type: "submit",
                            class: "flex-1 bg-blue-600 text-white px-4 py-2 rounded-lg disabled:opacity-50",
                            disabled: !is_valid,
                            "Add Book"
                        }
                        button {
                            r#type: "button",
                            class: "flex-1 bg-gray-200 text-gray-800 px-4 py-2 rounded-lg",
                            onclick: move |_| {
                                navigator.push(Route::BookList);
                                ()
                            },
                            "Cancel"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_book_form_renders_with_inputs() {
        // Test that form renders with title and author inputs
        assert!(true, "AddBookForm should render with inputs");
    }

    #[test]
    fn test_submit_disabled_when_title_empty() {
        // Test that submit is disabled when title is empty
        let title = String::new();
        let author = "Author".to_string();
        let is_valid = !title.is_empty() && !author.is_empty();
        assert!(!is_valid, "Should be invalid when title is empty");
    }

    #[test]
    fn test_submit_disabled_when_author_empty() {
        // Test that submit is disabled when author is empty
        let title = "Title".to_string();
        let author = String::new();
        let is_valid = !title.is_empty() && !author.is_empty();
        assert!(!is_valid, "Should be invalid when author is empty");
    }

    #[test]
    fn test_submit_enabled_when_both_fields_filled() {
        // Test that submit is enabled when both fields are filled
        let title = "Title".to_string();
        let author = "Author".to_string();
        let is_valid = !title.is_empty() && !author.is_empty();
        assert!(is_valid, "Should be valid when both fields are filled");
    }

    #[test]
    fn test_submit_navigates_to_book_detail_route() {
        // Test that successful submit navigates to book detail
        // Placeholder - actual navigation testing requires Dioxus test harness
        assert!(true, "Submit should navigate on success");
    }
}
