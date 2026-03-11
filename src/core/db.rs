//! SQLite database operations
//!
//! This module handles all database operations including CRUD operations
//! and full-text search using FTS5.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::core::error::{Result, ShuseiError};
use crate::core::storage::StorageService;

/// Database connection wrapper
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create database at the specified path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path.as_ref())?;

        let db = Self { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    /// Create an in-memory database (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        let db = Self { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            -- Sticky notes table
            CREATE TABLE IF NOT EXISTS sticky_notes (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
                image_path  TEXT,
                ocr_markdown    TEXT,
                voice_transcript TEXT,
                book_title  TEXT,
                page_number INTEGER,
                user_memo   TEXT,
                tags        TEXT,
                ocr_text_plain TEXT
            );
            
            -- FTS5 virtual table for full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS sticky_notes_fts USING fts5(
                ocr_text_plain, user_memo, book_title, voice_transcript,
                content='sticky_notes', content_rowid='id'
            );
            
            -- Books table for library management
            CREATE TABLE IF NOT EXISTS books (
                id              TEXT PRIMARY KEY,
                title           TEXT NOT NULL,
                author          TEXT NOT NULL,
                cover_path      TEXT,
                pages_captured  INTEGER DEFAULT 0,
                total_pages     INTEGER,
                last_opened_at  INTEGER,
                created_at      INTEGER NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_books_title ON books(title);
            
            -- Enable WAL mode for concurrent reads
            PRAGMA journal_mode=WAL;
            
            -- Book pages table
            CREATE TABLE IF NOT EXISTS book_pages (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                book_id     INTEGER NOT NULL REFERENCES books(id),
                page_number INTEGER NOT NULL,
                markdown    TEXT NOT NULL,
                confidence  REAL,
                UNIQUE(book_id, page_number)
            );
            
            -- Vocabulary table
            CREATE TABLE IF NOT EXISTS vocabulary (
                id               INTEGER PRIMARY KEY AUTOINCREMENT,
                word             TEXT NOT NULL,
                meaning          TEXT,
                example_sentence TEXT,
                source_book      TEXT,
                source_page      INTEGER,
                tags             TEXT,
                created_at       TEXT NOT NULL DEFAULT (datetime('now')),
                review_count     INTEGER DEFAULT 0,
                last_reviewed_at TEXT
            );
            
            -- Indexes for better query performance
            CREATE INDEX IF NOT EXISTS idx_sticky_notes_book ON sticky_notes(book_title);
            CREATE INDEX IF NOT EXISTS idx_sticky_notes_created ON sticky_notes(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_vocab_word ON vocabulary(word);
            "#,
        )?;

        log::info!("Database schema initialized");
        Ok(())
    }

    // ==================== Sticky Notes ====================

    /// Create a new sticky note
    pub fn create_sticky_note(&self, note: &NewStickyNote) -> Result<i64> {
        let result = self.conn.execute(
            r#"
            INSERT INTO sticky_notes (
                image_path, ocr_markdown, voice_transcript, book_title,
                page_number, user_memo, tags, ocr_text_plain
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                note.image_path,
                note.ocr_markdown,
                note.voice_transcript,
                note.book_title,
                note.page_number,
                note.user_memo,
                note.tags,
                note.ocr_text_plain,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get a sticky note by ID
    pub fn get_sticky_note(&self, id: i64) -> Result<Option<StickyNote>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM sticky_notes WHERE id = ?1")?;

        let note = stmt
            .query_row(params![id], |row| StickyNote::from_row(row))
            .optional()?;

        Ok(note)
    }

    /// Get all sticky notes
    pub fn get_all_sticky_notes(&self) -> Result<Vec<StickyNote>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM sticky_notes ORDER BY created_at DESC")?;

        let notes = stmt
            .query_map([], |row| StickyNote::from_row(row))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(notes)
    }

    /// Search sticky notes using FTS
    pub fn search_sticky_notes(&self, query: &str) -> Result<Vec<StickyNote>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT s.* FROM sticky_notes s
            JOIN sticky_notes_fts fts ON s.id = fts.rowid
            WHERE sticky_notes_fts MATCH ?1
            ORDER BY rank
            "#,
        )?;

        let notes = stmt
            .query_map(params![query], |row| StickyNote::from_row(row))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(notes)
    }

    /// Update a sticky note
    pub fn update_sticky_note(&self, id: i64, note: &UpdateStickyNote) -> Result<bool> {
        let result = self.conn.execute(
            r#"
            UPDATE sticky_notes SET
                image_path = COALESCE(?2, image_path),
                ocr_markdown = COALESCE(?3, ocr_markdown),
                voice_transcript = COALESCE(?4, voice_transcript),
                book_title = COALESCE(?5, book_title),
                page_number = COALESCE(?6, page_number),
                user_memo = COALESCE(?7, user_memo),
                tags = COALESCE(?8, tags),
                ocr_text_plain = COALESCE(?9, ocr_text_plain),
                updated_at = datetime('now')
            WHERE id = ?1
            "#,
            params![
                id,
                note.image_path,
                note.ocr_markdown,
                note.voice_transcript,
                note.book_title,
                note.page_number,
                note.user_memo,
                note.tags,
                note.ocr_text_plain,
            ],
        )?;

        Ok(result > 0)
    }

    /// Delete a sticky note
    pub fn delete_sticky_note(&self, id: i64) -> Result<bool> {
        let result = self
            .conn
            .execute("DELETE FROM sticky_notes WHERE id = ?1", params![id])?;

        Ok(result > 0)
    }

    // ==================== Books ====================

    /// Save cover photo for a book
    ///
    /// This method:
    /// 1. Saves the image to filesystem using StorageService
    /// 2. Updates the book's cover_path in the database
    /// 3. Returns the stored path
    pub fn save_cover_photo(
        &self,
        book_id: &str,
        image_data: &[u8],
        storage: &StorageService,
    ) -> Result<String> {
        // Save image to filesystem
        let relative_path = storage.save_image(image_data, "cover")?;

        // Update database with cover_path
        self.conn.execute(
            "UPDATE books SET cover_path = ?1 WHERE id = ?2",
            params![relative_path.clone(), book_id],
        )?;

        Ok(relative_path)
    }

    /// Remove cover photo from a book
    ///
    /// This method:
    /// 1. Deletes the image file from filesystem
    /// 2. Clears the cover_path in the database
    pub fn remove_cover_photo(&self, book_id: &str, storage: &StorageService) -> Result<()> {
        // Get current cover_path
        let cover_path = self.conn.query_row(
            "SELECT cover_path FROM books WHERE id = ?1",
            params![book_id],
            |row| row.get::<_, Option<String>>(0),
        )?;

        // Delete file if it exists
        if let Some(path) = cover_path {
            storage.delete_image(&path)?;
        }

        // Clear database field
        self.conn.execute(
            "UPDATE books SET cover_path = NULL WHERE id = ?1",
            params![book_id],
        )?;

        Ok(())
    }

    /// Get a book by ID
    pub fn get_book(&self, id: &str) -> Result<Option<Book>> {
        let mut stmt = self.conn.prepare("SELECT * FROM books WHERE id = ?1")?;

        let book = stmt
            .query_row(params![id], |row| Book::from_row(row))
            .optional()?;

        Ok(book)
    }

    /// Get all books sorted by title
    pub fn get_all_books(&self) -> Result<Vec<Book>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM books ORDER BY title ASC")?;

        let books = stmt
            .query_map([], |row| Book::from_row(row))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(books)
    }

    /// Create a new book
    pub fn create_book(&self, book: &NewBook) -> Result<String> {
        let id = book.id.clone().unwrap_or_else(|| {
            use std::collections::hash_map::RandomState;
            use std::hash::{BuildHasher, Hasher};
            use std::time::{SystemTime, UNIX_EPOCH};

            let hasher = RandomState::new().build_hasher();
            let hash = hasher.finish();
            format!(
                "{:016x}-{}",
                hash,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            )
        });

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            r#"
            INSERT INTO books (id, title, author, cover_path, pages_captured, total_pages, last_opened_at, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                id,
                book.title,
                book.author,
                book.cover_path,
                book.pages_captured,
                book.total_pages,
                book.last_opened_at,
                now, // created_at
                now, // updated_at
            ],
        )?;

        Ok(id)
    }

    /// Update a book
    pub fn update_book(&self, book: &Book) -> Result<bool> {
        let result = self.conn.execute(
            r#"
            UPDATE books SET
                title = ?2,
                author = ?3,
                cover_path = ?4,
                pages_captured = ?5,
                total_pages = ?6,
                last_opened_at = ?7
            WHERE id = ?1
            "#,
            params![
                book.id,
                book.title,
                book.author,
                book.cover_path,
                book.pages_captured,
                book.total_pages,
                book.last_opened_at,
            ],
        )?;

        Ok(result > 0)
    }

    /// Delete a book by ID
    pub fn delete_book(&self, id: &str) -> Result<bool> {
        let result = self
            .conn
            .execute("DELETE FROM books WHERE id = ?1", params![id])?;

        Ok(result > 0)
    }
}

// ==================== Data Models ====================

/// Sticky note model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StickyNote {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub image_path: Option<String>,
    pub ocr_markdown: Option<String>,
    pub voice_transcript: Option<String>,
    pub book_title: Option<String>,
    pub page_number: Option<i32>,
    pub user_memo: Option<String>,
    pub tags: Option<String>,
    pub ocr_text_plain: Option<String>,
}

impl StickyNote {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            created_at: row.get(1)?,
            updated_at: row.get(2)?,
            image_path: row.get(3)?,
            ocr_markdown: row.get(4)?,
            voice_transcript: row.get(5)?,
            book_title: row.get(6)?,
            page_number: row.get(7)?,
            user_memo: row.get(8)?,
            tags: row.get(9)?,
            ocr_text_plain: row.get(10)?,
        })
    }
}

/// New sticky note (for creation)
#[derive(Debug, Clone, Default)]
pub struct NewStickyNote {
    pub image_path: Option<String>,
    pub ocr_markdown: Option<String>,
    pub voice_transcript: Option<String>,
    pub book_title: Option<String>,
    pub page_number: Option<i32>,
    pub user_memo: Option<String>,
    pub tags: Option<String>,
    pub ocr_text_plain: Option<String>,
}

/// Update sticky note (for partial updates)
#[derive(Debug, Clone, Default)]
pub struct UpdateStickyNote {
    pub image_path: Option<String>,
    pub ocr_markdown: Option<String>,
    pub voice_transcript: Option<String>,
    pub book_title: Option<String>,
    pub page_number: Option<i32>,
    pub user_memo: Option<String>,
    pub tags: Option<String>,
    pub ocr_text_plain: Option<String>,
}

/// Book model
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

impl Book {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            title: row.get(1)?,
            author: row.get(2)?,
            cover_path: row.get(3)?,
            pages_captured: row.get(4)?,
            total_pages: row.get(5)?,
            last_opened_at: row.get(6)?,
            created_at: row.get(7)?,
        })
    }
}

/// New book (for creation)
#[derive(Debug, Clone, Default)]
pub struct NewBook {
    pub id: Option<String>,
    pub title: String,
    pub author: String,
    pub cover_path: Option<String>,
    pub pages_captured: i32,
    pub total_pages: Option<i32>,
    pub last_opened_at: Option<i64>,
}

/// Update book (for partial updates)
#[derive(Debug, Clone, Default)]
pub struct UpdateBook {
    pub title: Option<String>,
    pub author: Option<String>,
    pub cover_path: Option<String>,
    pub pages_captured: Option<i32>,
    pub total_pages: Option<i32>,
    pub last_opened_at: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_in_memory() {
        let db = Database::in_memory().unwrap();
        let note = db.get_sticky_note(1).unwrap();
        assert!(note.is_none());
    }

    #[test]
    fn test_create_and_get_sticky_note() {
        let db = Database::in_memory().unwrap();

        let new_note = NewStickyNote {
            ocr_markdown: Some("# Test\nHello world".to_string()),
            ocr_text_plain: Some("Test Hello world".to_string()),
            ..Default::default()
        };

        let id = db.create_sticky_note(&new_note).unwrap();
        assert!(id > 0);

        let note = db.get_sticky_note(id).unwrap().unwrap();
        assert_eq!(note.ocr_markdown, Some("# Test\nHello world".to_string()));
    }

    mod books_schema {
        use super::*;

        #[test]
        fn table_exists() {
            let db = Database::in_memory().unwrap();

            let mut stmt = db
                .conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='books'")
                .unwrap();
            let exists = stmt.exists([]).unwrap();
            assert!(exists, "books table should exist");
        }

        #[test]
        fn index_exists() {
            let db = Database::in_memory().unwrap();

            let mut stmt = db
                .conn
                .prepare(
                    "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_books_title'",
                )
                .unwrap();
            let exists = stmt.exists([]).unwrap();
            assert!(exists, "idx_books_title index should exist");
        }

        #[test]
        fn wal_mode_supported() {
            let db = Database::in_memory().unwrap();

            let result = db.conn.pragma_update(None, "journal_mode", "WAL");
            assert!(result.is_ok(), "WAL mode should be supported");
        }

        #[test]
        fn insert_valid_book_succeeds() {
            let db = Database::in_memory().unwrap();

            let result = db.conn.execute(
                "INSERT INTO books (id, title, author, pages_captured, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params!["test-id", "Test Book", "Test Author", 0, 1234567890]
            );

            if let Err(e) = &result {
                eprintln!("Insert error: {:?}", e);
            }
            assert!(
                result.is_ok(),
                "Should insert valid book: {:?}",
                result.err()
            );
        }

        #[test]
        fn reject_missing_title() {
            let db = Database::in_memory().unwrap();

            let result = db.conn.execute(
                "INSERT INTO books (id, author, pages_captured, created_at) VALUES (?1, ?2, ?3, ?4)",
                params!["test-id", "Test Author", 0, 1234567890]
            );

            assert!(result.is_err(), "Should reject book without title");
        }

        #[test]
        fn reject_missing_author() {
            let db = Database::in_memory().unwrap();

            let result = db.conn.execute(
                "INSERT INTO books (id, title, pages_captured, created_at) VALUES (?1, ?2, ?3, ?4)",
                params!["test-id", "Test Book", 0, 1234567890],
            );

            assert!(result.is_err(), "Should reject book without author");
        }
    }

    mod cover_photo {
        use super::*;
        use tempfile::TempDir;

        fn setup_db_and_storage() -> (Database, StorageService, TempDir) {
            let db = Database::in_memory().unwrap();
            let temp_dir = TempDir::new().unwrap();
            let storage = StorageService::new(temp_dir.path().to_path_buf()).unwrap();
            (db, storage, temp_dir)
        }

        #[test]
        fn test_save_cover_photo_saves_file_and_updates_database() {
            let (db, storage, _temp) = setup_db_and_storage();

            // Create a book first
            let new_book = NewBook {
                title: "Test Book".to_string(),
                author: "Test Author".to_string(),
                ..Default::default()
            };
            let book_id = db.create_book(&new_book).unwrap();

            // Save cover photo
            let image_data = b"fake image data";
            let result = db.save_cover_photo(&book_id, image_data, &storage);
            assert!(result.is_ok());

            // Verify database was updated
            let book = db.get_book(&book_id).unwrap().unwrap();
            assert!(book.cover_path.is_some());

            // Verify file exists
            let cover_path = book.cover_path.unwrap();
            let full_path = storage.assets_dir.join(&cover_path);
            assert!(full_path.exists());
        }

        #[test]
        fn test_save_cover_photo_returns_stored_path() {
            let (db, storage, _temp) = setup_db_and_storage();

            let new_book = NewBook {
                title: "Test Book".to_string(),
                author: "Test Author".to_string(),
                ..Default::default()
            };
            let book_id = db.create_book(&new_book).unwrap();

            let image_data = b"fake image data";
            let path = db.save_cover_photo(&book_id, image_data, &storage).unwrap();

            assert!(path.starts_with("images/"));
        }

        #[test]
        fn test_remove_cover_photo_deletes_file_and_clears_database() {
            let (db, storage, _temp) = setup_db_and_storage();

            // Create book with cover
            let new_book = NewBook {
                title: "Test Book".to_string(),
                author: "Test Author".to_string(),
                ..Default::default()
            };
            let book_id = db.create_book(&new_book).unwrap();

            let image_data = b"fake image data";
            let cover_path = db.save_cover_photo(&book_id, image_data, &storage).unwrap();

            // Remove cover photo
            db.remove_cover_photo(&book_id, &storage).unwrap();

            // Verify database field cleared
            let book = db.get_book(&book_id).unwrap().unwrap();
            assert!(book.cover_path.is_none());

            // Verify file deleted
            let full_path = storage.assets_dir.join(&cover_path);
            assert!(!full_path.exists());
        }

        #[test]
        fn test_get_book_returns_book_with_cover_path_after_save() {
            let (db, storage, _temp) = setup_db_and_storage();

            let new_book = NewBook {
                title: "Test Book".to_string(),
                author: "Test Author".to_string(),
                ..Default::default()
            };
            let book_id = db.create_book(&new_book).unwrap();

            let image_data = b"fake image data";
            db.save_cover_photo(&book_id, image_data, &storage).unwrap();

            let book = db.get_book(&book_id).unwrap().unwrap();
            assert!(book.cover_path.is_some());
            assert_eq!(book.title, "Test Book");
            assert_eq!(book.author, "Test Author");
        }
    }
}
