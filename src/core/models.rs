//! Book model and serialization
//!
//! This module defines the Book struct used throughout the application
//! for representing book metadata.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a simple UUID-like string
fn generate_id() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    let hasher = RandomState::new().build_hasher();
    let hash = hasher.finish();
    format!("{:016x}", hash)
}

/// Get current Unix timestamp
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Book model representing a book in the library
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub cover_path: Option<String>,
    pub pages_captured: i32,
    pub total_pages: Option<i32>,
    pub last_opened_at: Option<i64>,
    pub created_at: i64,
}

impl Default for Book {
    fn default() -> Self {
        Self {
            id: generate_id(),
            title: String::new(),
            author: String::new(),
            cover_path: None,
            pages_captured: 0,
            total_pages: None,
            last_opened_at: None,
            created_at: current_timestamp(),
        }
    }
}

impl Book {
    /// Create a new book with required fields
    pub fn new(title: String, author: String) -> Self {
        Self {
            title,
            author,
            ..Default::default()
        }
    }

    /// Create a new book with all fields
    #[allow(clippy::too_many_arguments)]
    pub fn with_all_fields(
        id: String,
        title: String,
        author: String,
        cover_path: Option<String>,
        pages_captured: i32,
        total_pages: Option<i32>,
        last_opened_at: Option<i64>,
        created_at: i64,
    ) -> Self {
        Self {
            id,
            title,
            author,
            cover_path,
            pages_captured,
            total_pages,
            last_opened_at,
            created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_book_serialization_round_trip() {
        let book = Book::with_all_fields(
            "test-id-123".to_string(),
            "Test Book".to_string(),
            "Test Author".to_string(),
            Some("covers/test.jpg".to_string()),
            100,
            Some(200),
            Some(1234567890),
            1234567890,
        );

        // Serialize to JSON
        let json = serde_json::to_string(&book).unwrap();

        // Deserialize back
        let deserialized: Book = serde_json::from_str(&json).unwrap();

        // Verify round-trip
        assert_eq!(book, deserialized);
    }

    #[test]
    fn test_book_minimal_fields() {
        let book = Book::new("Minimal Book".to_string(), "Minimal Author".to_string());

        assert_eq!(book.title, "Minimal Book");
        assert_eq!(book.author, "Minimal Author");
        assert_eq!(book.pages_captured, 0);
        assert!(book.id.len() > 0);
        assert!(book.created_at > 0);
    }

    #[test]
    fn test_book_all_fields() {
        let book = Book::with_all_fields(
            "full-id".to_string(),
            "Full Book".to_string(),
            "Full Author".to_string(),
            Some("covers/full.jpg".to_string()),
            50,
            Some(100),
            Some(9876543210),
            1111111111,
        );

        assert_eq!(book.id, "full-id");
        assert_eq!(book.title, "Full Book");
        assert_eq!(book.author, "Full Author");
        assert_eq!(book.cover_path, Some("covers/full.jpg".to_string()));
        assert_eq!(book.pages_captured, 50);
        assert_eq!(book.total_pages, Some(100));
        assert_eq!(book.last_opened_at, Some(9876543210));
        assert_eq!(book.created_at, 1111111111);
    }

    #[test]
    fn test_book_default_trait() {
        let book = Book::default();

        assert!(book.id.len() > 0);
        assert_eq!(book.title, "");
        assert_eq!(book.author, "");
        assert_eq!(book.cover_path, None);
        assert_eq!(book.pages_captured, 0);
        assert_eq!(book.total_pages, None);
        assert_eq!(book.last_opened_at, None);
        assert!(book.created_at > 0);
    }
}
