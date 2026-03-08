//! Vocabulary management
//!
//! This module handles word extraction using morphological analysis (lindera)
//! and vocabulary list management.

use serde::{Deserialize, Serialize};

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

fn export_json(entries: &[VocabularyEntry]) -> String {
    serde_json::to_string_pretty(entries).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
}