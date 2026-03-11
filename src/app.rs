//! Root application component with routing

use dioxus::prelude::*;
use dioxus_router::{Link, Routable, Router};

use crate::ui::{CameraPage, NotesPage, ReaderPage, VocabPage};

/// Main application component
#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

/// Application routes
#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    #[route("/")]
    Home,

    #[route("/camera")]
    Camera,

    #[route("/notes")]
    Notes,

    #[route("/notes/:id")]
    NoteDetail { id: i64 },

    #[route("/reader")]
    Reader,

    #[route("/reader/:book_id")]
    ReaderBook { book_id: i64 },

    #[route("/vocab")]
    Vocab,

    #[route("/settings")]
    Settings,

    #[route("/books")]
    BookList,

    #[route("/add-book")]
    AddBook,
}

/// Home page - shows overview and quick actions
#[component]
fn Home() -> Element {
    rsx! {
        div { class: "flex flex-col h-full",
            // Header
            header { class: "bg-blue-600 text-white p-4",
                h1 { class: "text-2xl font-bold", "脳読" }
                p { class: "text-sm opacity-80", "Offline Reading Assistant" }
            }

            // Quick actions
            div { class: "p-4 space-y-4",
                h2 { class: "text-lg font-semibold mb-4", "Quick Actions" }

                Link {
                    to: Route::Camera,
                    class: "block bg-green-600 text-white p-4 rounded-lg text-center",
                    "📷 Capture Page"
                }

                Link {
                    to: Route::Notes,
                    class: "block bg-blue-600 text-white p-4 rounded-lg text-center",
                    "📝 Sticky Notes"
                }

                Link {
                    to: Route::Reader,
                    class: "block bg-purple-600 text-white p-4 rounded-lg text-center",
                    "📖 Read PDF"
                }

                Link {
                    to: Route::Vocab,
                    class: "block bg-orange-600 text-white p-4 rounded-lg text-center",
                    "📚 Vocabulary"
                }
            }

            // Recent items
            div { class: "p-4",
                h2 { class: "text-lg font-semibold mb-2", "Recent Notes" }
                p { class: "text-gray-500", "No recent notes" }
            }
        }
    }
}

/// Camera page wrapper
#[component]
fn Camera() -> Element {
    rsx! {
        CameraPage {}
    }
}

/// Notes page wrapper
#[component]
fn Notes() -> Element {
    rsx! {
        NotesPage {}
    }
}

/// Note detail page
#[component]
fn NoteDetail(id: i64) -> Element {
    rsx! {
        div { class: "p-4",
            h1 { class: "text-xl font-bold", "Note #{id}" }
            p { class: "text-gray-500", "Note detail view - coming soon" }
            Link {
                to: Route::Notes,
                class: "text-blue-600 mt-4 inline-block",
                "← Back to Notes"
            }
        }
    }
}

/// Reader page wrapper
#[component]
fn Reader() -> Element {
    rsx! {
        ReaderPage {}
    }
}

/// Reader book page
#[component]
fn ReaderBook(book_id: i64) -> Element {
    rsx! {
        div { class: "p-4",
            h1 { class: "text-xl font-bold", "Book #{book_id}" }
            p { class: "text-gray-500", "Book reader view - coming soon" }
            Link {
                to: Route::Reader,
                class: "text-blue-600 mt-4 inline-block",
                "← Back to Library"
            }
        }
    }
}

/// Vocab page wrapper
#[component]
fn Vocab() -> Element {
    rsx! {
        VocabPage {}
    }
}

/// Book list page - placeholder for library screen
#[component]
fn BookList() -> Element {
    rsx! {
        div { class: "p-4",
            h1 { class: "text-xl font-bold", "Library" }
            p { class: "text-gray-500", "Library view - coming soon" }
        }
    }
}

/// Add book page - placeholder for add book form
#[component]
fn AddBook() -> Element {
    rsx! {
        div { class: "p-4",
            h1 { class: "text-xl font-bold", "Add Book" }
            p { class: "text-gray-500", "Add book form - coming soon" }
        }
    }
}

/// Settings page
#[component]
fn Settings() -> Element {
    rsx! {
        div { class: "p-4",
            h1 { class: "text-xl font-bold mb-4", "Settings" }

            div { class: "space-y-4",
                div { class: "border-b pb-2",
                    h3 { class: "font-semibold", "OCR Model" }
                    p { class: "text-sm text-gray-500", "NDLOCR-Lite" }
                }

                div { class: "border-b pb-2",
                    h3 { class: "font-semibold", "STT Model" }
                    p { class: "text-sm text-gray-500", "Moonshine Tiny (Japanese)" }
                }

                div { class: "border-b pb-2",
                    h3 { class: "font-semibold", "Storage" }
                    p { class: "text-sm text-gray-500", "Local SQLite Database" }
                }
            }

            Link {
                to: Route::Home,
                class: "text-blue-600 mt-4 inline-block",
                "← Back to Home"
            }
        }
    }
}
