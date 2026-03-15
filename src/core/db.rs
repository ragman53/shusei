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
                created_at      INTEGER NOT NULL,
                updated_at      INTEGER NOT NULL,
                is_pdf          BOOLEAN DEFAULT FALSE,
                pdf_path        TEXT
            );
            
            CREATE INDEX IF NOT EXISTS idx_books_title ON books(title);
            
            -- Enable WAL mode for concurrent reads
            PRAGMA journal_mode=WAL;
            
            -- Processing progress table for PDF conversion tracking
            CREATE TABLE IF NOT EXISTS processing_progress (
                book_id TEXT PRIMARY KEY REFERENCES books(id),
                last_processed_page INTEGER DEFAULT 0,
                total_pages INTEGER,
                status TEXT DEFAULT 'pending',
                updated_at INTEGER NOT NULL
            );
            
            -- Book pages table
            CREATE TABLE IF NOT EXISTS book_pages (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                book_id         TEXT NOT NULL REFERENCES books(id),
                page_number     INTEGER NOT NULL,
                image_path      TEXT NOT NULL,
                ocr_markdown    TEXT NOT NULL,
                ocr_text_plain  TEXT NOT NULL,
                confidence      REAL,
                created_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                UNIQUE(book_id, page_number)
            );
            
            CREATE INDEX IF NOT EXISTS idx_book_pages_book ON book_pages(book_id);
            CREATE INDEX IF NOT EXISTS idx_book_pages_number ON book_pages(page_number);
            
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
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
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
            INSERT INTO books (id, title, author, cover_path, pages_captured, total_pages, last_opened_at, created_at, updated_at, is_pdf, pdf_path)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
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
                book.is_pdf,
                book.pdf_path,
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
                last_opened_at = ?7,
                is_pdf = ?8,
                pdf_path = ?9
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
                book.is_pdf,
                book.pdf_path,
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

    // ==================== Book Pages ====================

    /// Save a book page with OCR results
    pub fn save_page(&self, page: &NewBookPage) -> Result<i64> {
        let result = self.conn.execute(
            r#"
            INSERT INTO book_pages (
                book_id, page_number, image_path, ocr_markdown, 
                ocr_text_plain, confidence
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                page.book_id,
                page.page_number,
                page.image_path,
                page.ocr_markdown,
                page.ocr_text_plain,
                page.confidence,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get a book page by ID
    pub fn get_page(&self, id: i64) -> Result<Option<BookPage>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM book_pages WHERE id = ?1")?;

        let page = stmt
            .query_row(params![id], |row| BookPage::from_row(row))
            .optional()?;

        Ok(page)
    }

    /// Get all pages for a book, ordered by page number
    pub fn get_pages_by_book(&self, book_id: &str) -> Result<Vec<BookPage>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM book_pages WHERE book_id = ?1 ORDER BY page_number ASC")?;

        let pages = stmt
            .query_map(params![book_id], |row| BookPage::from_row(row))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(pages)
    }

    // ==================== Processing Progress ====================

    /// Create progress tracking record for a book
    pub fn create_progress(&self, book_id: &str, total_pages: i32) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO processing_progress (book_id, last_processed_page, total_pages, status, updated_at)
            VALUES (?1, 0, ?2, 'pending', ?3)
            "#,
            params![book_id, total_pages, now],
        )?;

        Ok(())
    }

    /// Update progress for a book
    pub fn update_progress(&self, book_id: &str, last_page: i32, status: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            r#"
            UPDATE processing_progress SET
                last_processed_page = ?2,
                status = ?3,
                updated_at = ?4
            WHERE book_id = ?1
            "#,
            params![book_id, last_page, status, now],
        )?;

        Ok(())
    }

    /// Get progress for a book
    pub fn get_progress(&self, book_id: &str) -> Result<Option<ProcessingProgress>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM processing_progress WHERE book_id = ?1")?;

        let progress = stmt
            .query_row(params![book_id], |row| ProcessingProgress::from_row(row))
            .optional()?;

        Ok(progress)
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
    pub updated_at: i64,
    pub is_pdf: bool,
    pub pdf_path: Option<String>,
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
            updated_at: row.get(8)?,
            is_pdf: row.get(9)?,
            pdf_path: row.get(10)?,
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
    pub is_pdf: bool,
    pub pdf_path: Option<String>,
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
    pub is_pdf: Option<bool>,
}

// ==================== Book Pages ====================

/// Book page model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookPage {
    pub id: i64,
    pub book_id: String,
    pub page_number: i32,
    pub image_path: String,
    pub ocr_markdown: String,
    pub ocr_text_plain: String,
    pub confidence: Option<f32>,
    pub created_at: i64,
}

impl BookPage {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            book_id: row.get(1)?,
            page_number: row.get(2)?,
            image_path: row.get(3)?,
            ocr_markdown: row.get(4)?,
            ocr_text_plain: row.get(5)?,
            confidence: row.get(6)?,
            created_at: row.get(7)?,
        })
    }
}

/// New book page (for creation)
#[derive(Debug, Clone)]
pub struct NewBookPage {
    pub book_id: String,
    pub page_number: i32,
    pub image_path: String,
    pub ocr_markdown: String,
    pub ocr_text_plain: String,
    pub confidence: Option<f32>,
}

/// Processing progress for PDF conversion
#[derive(Debug, Clone)]
pub struct ProcessingProgress {
    pub book_id: String,
    pub last_processed_page: i32,
    pub total_pages: i32,
    pub status: String,
    pub updated_at: i64,
}

impl ProcessingProgress {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            book_id: row.get(0)?,
            last_processed_page: row.get(1)?,
            total_pages: row.get(2)?,
            status: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod processing_progress {
        use super::*;

        #[test]
        fn test_create_progress_inserts_record() {
            let db = Database::in_memory().unwrap();

            // Create a book first
            let book_id = db
                .create_book(&NewBook {
                    title: "Test Book".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            let result = db.create_progress(&book_id, 100);
            assert!(result.is_ok());
        }

        #[test]
        fn test_update_progress_modifies_last_processed_page() {
            let db = Database::in_memory().unwrap();

            let book_id = db
                .create_book(&NewBook {
                    title: "Test Book".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            db.create_progress(&book_id, 100).unwrap();
            db.update_progress(&book_id, 50, "processing").unwrap();

            let progress = db.get_progress(&book_id).unwrap().unwrap();
            assert_eq!(progress.last_processed_page, 50);
        }

        #[test]
        fn test_update_progress_changes_status() {
            let db = Database::in_memory().unwrap();

            let book_id = db
                .create_book(&NewBook {
                    title: "Test Book".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            db.create_progress(&book_id, 100).unwrap();
            db.update_progress(&book_id, 0, "processing").unwrap();

            let progress = db.get_progress(&book_id).unwrap().unwrap();
            assert_eq!(progress.status, "processing");
        }

        #[test]
        fn test_get_progress_returns_none_for_non_existent() {
            let db = Database::in_memory().unwrap();

            let result = db.get_progress("non-existent-book").unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn test_status_transitions_from_processing_to_completed() {
            let db = Database::in_memory().unwrap();

            let book_id = db
                .create_book(&NewBook {
                    title: "Test Book".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            db.create_progress(&book_id, 100).unwrap();
            db.update_progress(&book_id, 50, "processing").unwrap();
            db.update_progress(&book_id, 100, "completed").unwrap();

            let progress = db.get_progress(&book_id).unwrap().unwrap();
            assert_eq!(progress.status, "completed");
            assert_eq!(progress.last_processed_page, 100);
        }

        #[test]
        fn test_status_transitions_from_processing_to_failed() {
            let db = Database::in_memory().unwrap();

            let book_id = db
                .create_book(&NewBook {
                    title: "Test Book".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            db.create_progress(&book_id, 100).unwrap();
            db.update_progress(&book_id, 30, "processing").unwrap();
            db.update_progress(&book_id, 30, "failed").unwrap();

            let progress = db.get_progress(&book_id).unwrap().unwrap();
            assert_eq!(progress.status, "failed");
            assert_eq!(progress.last_processed_page, 30);
        }
    }

    mod book_pages {
        use super::*;

        #[test]
        fn test_save_page_inserts_and_returns_id() {
            let db = Database::in_memory().unwrap();

            // Create a book first
            let book_id = db
                .create_book(&NewBook {
                    title: "Test Book".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            let new_page = NewBookPage {
                book_id: book_id.clone(),
                page_number: 1,
                image_path: "pages/test/image1.jpg".to_string(),
                ocr_markdown: "# Page 1\nTest content".to_string(),
                ocr_text_plain: "Page 1 Test content".to_string(),
                confidence: Some(0.95),
            };

            let id = db.save_page(&new_page).unwrap();
            assert!(id > 0);
        }

        #[test]
        fn test_get_page_retrieves_by_id() {
            let db = Database::in_memory().unwrap();

            let book_id = db
                .create_book(&NewBook {
                    title: "Test Book".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            let new_page = NewBookPage {
                book_id: book_id.clone(),
                page_number: 1,
                image_path: "pages/test/image1.jpg".to_string(),
                ocr_markdown: "# Page 1".to_string(),
                ocr_text_plain: "Page 1".to_string(),
                confidence: Some(0.9),
            };
            let page_id = db.save_page(&new_page).unwrap();

            let page = db.get_page(page_id).unwrap().unwrap();
            assert_eq!(page.id, page_id);
            assert_eq!(page.book_id, book_id);
            assert_eq!(page.page_number, 1);
            assert_eq!(page.ocr_markdown, "# Page 1");
        }

        #[test]
        fn test_get_pages_by_book_returns_sorted_pages() {
            let db = Database::in_memory().unwrap();

            let book_id = db
                .create_book(&NewBook {
                    title: "Test Book".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            // Save pages in non-sequential order
            db.save_page(&NewBookPage {
                book_id: book_id.clone(),
                page_number: 3,
                image_path: "pages/test/img3.jpg".to_string(),
                ocr_markdown: "Page 3".to_string(),
                ocr_text_plain: "Page 3".to_string(),
                confidence: None,
            })
            .unwrap();

            db.save_page(&NewBookPage {
                book_id: book_id.clone(),
                page_number: 1,
                image_path: "pages/test/img1.jpg".to_string(),
                ocr_markdown: "Page 1".to_string(),
                ocr_text_plain: "Page 1".to_string(),
                confidence: None,
            })
            .unwrap();

            db.save_page(&NewBookPage {
                book_id: book_id.clone(),
                page_number: 2,
                image_path: "pages/test/img2.jpg".to_string(),
                ocr_markdown: "Page 2".to_string(),
                ocr_text_plain: "Page 2".to_string(),
                confidence: None,
            })
            .unwrap();

            let pages = db.get_pages_by_book(&book_id).unwrap();
            assert_eq!(pages.len(), 3);
            assert_eq!(pages[0].page_number, 1);
            assert_eq!(pages[1].page_number, 2);
            assert_eq!(pages[2].page_number, 3);
        }

        #[test]
        fn test_get_page_returns_none_for_non_existent() {
            let db = Database::in_memory().unwrap();

            let result = db.get_page(999).unwrap();
            assert!(result.is_none());
        }
    }
}
