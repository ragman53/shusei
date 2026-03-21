//! Vocabulary management
//!
//! This module handles word extraction using morphological analysis (lindera)
//! and vocabulary list management.

use serde::{Deserialize, Serialize};

use crate::core::db::Word;
use crate::core::error::Result;

/// Vocabulary entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VocabularyEntry {
    pub id: i64,
    pub word: String,
    pub meaning: Option<String>,
    pub example_sentence: Option<String>,
    pub source_book: Option<String>,
    pub source_page: Option<i32>,
    pub tags: Option<String>,
    pub created_at: String,
    pub review_count: i32,
    pub last_reviewed_at: Option<String>,
}

/// New vocabulary entry (for creation)
#[derive(Debug, Clone, Default)]
pub struct NewVocabularyEntry {
    pub word: String,
    pub meaning: Option<String>,
    pub example_sentence: Option<String>,
    pub source_book: Option<String>,
    pub source_page: Option<i32>,
    pub tags: Option<String>,
}

/// Word extractor using morphological analysis
pub struct WordExtractor {
    /// Lindera tokenizer (placeholder - will be initialized later)
    _tokenizer: Option<()>,
}

impl WordExtractor {
    /// Create a new word extractor
    pub fn new() -> Result<Self> {
        // TODO: Initialize lindera tokenizer
        // This requires the ipadic feature to be enabled
        
        log::info!("Initializing word extractor");
        
        Ok(Self {
            _tokenizer: None,
        })
    }
    
    /// Extract words from text
    pub fn extract_words(&self, text: &str) -> Vec<ExtractedWord> {
        // TODO: Implement using lindera
        // For now, use simple whitespace splitting for English
        
        if text.chars().any(|c| c.is_ascii()) {
            // English text - use whitespace splitting
            self.extract_english_words(text)
        } else {
            // Japanese text - would need lindera
            self.extract_japanese_words_placeholder(text)
        }
    }
    
    /// Extract words from English text
    fn extract_english_words(&self, text: &str) -> Vec<ExtractedWord> {
        text.split_whitespace()
            .filter(|word| word.len() > 2)  // Filter short words
            .filter(|word| word.chars().all(|c| c.is_alphabetic()))  // Only letters
            .map(|word| ExtractedWord {
                word: word.to_lowercase(),
                pos: PartOfSpeech::Unknown,
                is_foreign: true,
            })
            .collect()
    }
    
    /// Placeholder for Japanese word extraction
    fn extract_japanese_words_placeholder(&self, text: &str) -> Vec<ExtractedWord> {
        // TODO: Implement using lindera
        // For now, just return empty
        log::warn!("Japanese word extraction not yet implemented");
        Vec::new()
    }
    
    /// Extract sentence containing a word
    pub fn extract_sentence(&self, text: &str, word: &str) -> Option<String> {
        // Find sentence containing the word
        let sentences: Vec<&str> = text.split(|c| c == '。' || c == '.' || c == '！' || c == '!')
            .collect();
        
        for sentence in sentences {
            if sentence.contains(word) {
                return Some(sentence.trim().to_string());
            }
        }
        
        None
    }
}

impl Default for WordExtractor {
    fn default() -> Self {
        Self::new().expect("Failed to create WordExtractor")
    }
}

/// An extracted word with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedWord {
    /// The word text
    pub word: String,
    
    /// Part of speech
    pub pos: PartOfSpeech,
    
    /// Whether this is a foreign (English) word
    pub is_foreign: bool,
}

/// Part of speech classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Particle,
    Unknown,
}

impl std::fmt::Display for PartOfSpeech {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartOfSpeech::Noun => write!(f, "名詞"),
            PartOfSpeech::Verb => write!(f, "動詞"),
            PartOfSpeech::Adjective => write!(f, "形容詞"),
            PartOfSpeech::Adverb => write!(f, "副詞"),
            PartOfSpeech::Particle => write!(f, "助詞"),
            PartOfSpeech::Unknown => write!(f, "不明"),
        }
    }
}

/// Vocabulary list export format
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Markdown,
    Csv,
    Json,
}

/// Export vocabulary list
pub fn export_vocabulary(entries: &[VocabularyEntry], format: ExportFormat) -> String {
    match format {
        ExportFormat::Markdown => export_markdown(entries),
        ExportFormat::Csv => export_csv(entries),
        ExportFormat::Json => export_json(entries),
    }
}

/// Export vocabulary list (Word struct version)
pub fn export_vocabulary_words(words: &[Word], format: ExportFormat) -> String {
    match format {
        ExportFormat::Markdown => export_markdown_words(words),
        ExportFormat::Csv => export_csv_words(words),
        ExportFormat::Json => export_json_words(words),
    }
}

fn export_markdown(entries: &[VocabularyEntry]) -> String {
    let mut md = String::from("# Vocabulary List\n\n");
    
    for entry in entries {
        md.push_str(&format!("## {}\n", entry.word));
        if let Some(meaning) = &entry.meaning {
            md.push_str(&format!("**Meaning**: {}\n", meaning));
        }
        if let Some(example) = &entry.example_sentence {
            md.push_str(&format!("**Example**: {}\n", example));
        }
        if let Some(book) = &entry.source_book {
            md.push_str(&format!("**Source**: {} (p.{})\n", book, entry.source_page.unwrap_or(0)));
        }
        md.push('\n');
    }
    
    md
}

fn export_markdown_words(words: &[Word]) -> String {
    let mut md = String::from("# Vocabulary List\n\n");
    
    for word in words {
        md.push_str(&format!("## {}\n", word.word));
        if let Some(definition) = &word.definition {
            md.push_str(&format!("**Definition**: {}\n", definition));
        }
        if let Some(context) = &word.context_text {
            md.push_str(&format!("**Example**: {}\n", context));
        }
        if let Some(book_id) = &word.source_book_id {
            if let Some(page) = word.source_page {
                md.push_str(&format!("**Source**: Book {} (p.{})\n", book_id, page));
            } else {
                md.push_str(&format!("**Source**: Book {}\n", book_id));
            }
        }
        md.push('\n');
    }
    
    md
}

fn export_csv(entries: &[VocabularyEntry]) -> String {
    let mut csv = String::from("word,meaning,example_sentence,source_book,source_page,tags\n");
    
    for entry in entries {
        let meaning = entry.meaning.as_deref().unwrap_or("");
        let example = entry.example_sentence.as_deref().unwrap_or("");
        let book = entry.source_book.as_deref().unwrap_or("");
        let page = entry.source_page.unwrap_or(0);
        let tags = entry.tags.as_deref().unwrap_or("");
        
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            entry.word, meaning, example, book, page, tags
        ));
    }
    
    csv
}

fn export_csv_words(words: &[Word]) -> String {
    let mut csv = String::from("word,definition,example_sentence,source_book_id,source_page\n");
    
    for word in words {
        let definition = word.definition.as_deref().unwrap_or("");
        let example = word.context_text.as_deref().unwrap_or("");
        let book_id = word.source_book_id.as_deref().unwrap_or("");
        let page = word.source_page.unwrap_or(0);
        
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            word.word, definition, example, book_id, page
        ));
    }
    
    csv
}

fn export_json(entries: &[VocabularyEntry]) -> String {
    serde_json::to_string_pretty(entries).unwrap_or_else(|_| "[]".to_string())
}

fn export_json_words(words: &[Word]) -> String {
    serde_json::to_string_pretty(words).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::{Database, NewBook, NewWord};
    
    #[test]
    fn test_extract_english_words() {
        let extractor = WordExtractor::new().unwrap();
        let words = extractor.extract_words("Hello world this is a test");
        
        assert!(!words.is_empty());
        assert!(words.iter().any(|w| w.word == "hello"));
        assert!(words.iter().any(|w| w.word == "world"));
    }
    
    #[test]
    fn test_extract_sentence() {
        let extractor = WordExtractor::new().unwrap();
        let text = "This is a test. Another sentence here.";
        
        let sentence = extractor.extract_sentence(text, "test");
        assert_eq!(sentence, Some("This is a test".to_string()));
    }
    
    #[test]
    fn test_export_markdown_words() {
        // Create test words
        let words = vec![
            Word {
                id: 1,
                word: "test".to_string(),
                definition: Some("A procedure intended to establish the quality".to_string()),
                ai_generated: false,
                source_book_id: Some("book1".to_string()),
                source_page: Some(10),
                context_text: Some("This is a test sentence.".to_string()),
                created_at: 1000,
                updated_at: 1000,
            },
            Word {
                id: 2,
                word: "vocabulary".to_string(),
                definition: Some("A body of words".to_string()),
                ai_generated: false,
                source_book_id: None,
                source_page: None,
                context_text: Some("Building vocabulary is important.".to_string()),
                created_at: 2000,
                updated_at: 2000,
            },
        ];
        
        let md = export_markdown_words(&words);
        
        assert!(md.contains("# Vocabulary List"));
        assert!(md.contains("## test"));
        assert!(md.contains("## vocabulary"));
        assert!(md.contains("**Definition**: A procedure intended to establish the quality"));
        assert!(md.contains("**Example**: This is a test sentence."));
        assert!(md.contains("**Source**: Book book1 (p.10)"));
        assert!(md.contains("**Definition**: A body of words"));
    }
    
    #[test]
    fn test_export_csv_words() {
        let words = vec![
            Word {
                id: 1,
                word: "test".to_string(),
                definition: Some("A procedure".to_string()),
                ai_generated: false,
                source_book_id: Some("book1".to_string()),
                source_page: Some(10),
                context_text: Some("Test sentence.".to_string()),
                created_at: 1000,
                updated_at: 1000,
            },
        ];
        
        let csv = export_csv_words(&words);
        
        assert!(csv.contains("word,definition,example_sentence,source_book_id,source_page"));
        assert!(csv.contains("\"test\",\"A procedure\",\"Test sentence.\",\"book1\",\"10\""));
    }
    
    #[test]
    fn test_export_json_words() {
        let words = vec![
            Word {
                id: 1,
                word: "test".to_string(),
                definition: Some("A procedure".to_string()),
                ai_generated: true,
                source_book_id: None,
                source_page: None,
                context_text: None,
                created_at: 1000,
                updated_at: 1000,
            },
        ];
        
        let json = export_json_words(&words);
        
        assert!(json.contains("\"word\": \"test\""));
        assert!(json.contains("\"definition\": \"A procedure\""));
        assert!(json.contains("\"ai_generated\": true"));
    }
    
    #[test]
    fn test_export_vocabulary_words() {
        let words = vec![
            Word {
                id: 1,
                word: "test".to_string(),
                definition: Some("Definition".to_string()),
                ai_generated: false,
                source_book_id: None,
                source_page: None,
                context_text: None,
                created_at: 1000,
                updated_at: 1000,
            },
        ];
        
        // Test all formats
        let md = export_vocabulary_words(&words, ExportFormat::Markdown);
        assert!(md.contains("## test"));
        
        let csv = export_vocabulary_words(&words, ExportFormat::Csv);
        assert!(csv.contains("\"test\""));
        
        let json = export_vocabulary_words(&words, ExportFormat::Json);
        assert!(json.contains("\"word\": \"test\""));
    }
    
    #[test]
    fn test_export_empty_list() {
        let words: Vec<Word> = vec![];
        
        let md = export_markdown_words(&words);
        assert_eq!(md, "# Vocabulary List\n\n");
        
        let csv = export_csv_words(&words);
        assert_eq!(csv, "word,definition,example_sentence,source_book_id,source_page\n");
        
        let json = export_json_words(&words);
        assert_eq!(json, "[]");
    }
}