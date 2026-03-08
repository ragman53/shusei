//! SQLite database operations
//!
//! This module handles all database operations including CRUD operations
//! and full-text search using FTS5.

use std::path::Path;

use rusqlite::{Connection, params, Row, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::core::error::{ShuseiError, Result};

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
            
            -- Books table for PDF reading
            CREATE TABLE IF NOT EXISTS books (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                title           TEXT NOT NULL,
                file_path       TEXT,
                total_pages     INTEGER,
                converted_pages INTEGER DEFAULT 0,
                last_read_pos   REAL DEFAULT 0.0,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );
            
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
            CREATE INDEX IF NOT EXISTS idx_books_updated ON books(updated_at DESC);
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
        let mut stmt = self.conn.prepare(
            "SELECT * FROM sticky_notes WHERE id = ?1"
        )?;
        
        let note = stmt
            .query_row(params![id], |row| StickyNote::from_row(row))
            .optional()?;
        
        Ok(note)
    }
    
    /// Get all sticky notes
    pub fn get_all_sticky_notes(&self) -> Result<Vec<StickyNote>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM sticky_notes ORDER BY created_at DESC"
        )?;
        
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
            "#
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
        let result = self.conn.execute(
            "DELETE FROM sticky_notes WHERE id = ?1",
            params![id],
        )?;
        
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
}