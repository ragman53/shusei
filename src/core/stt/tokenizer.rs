#![cfg(not(target_os = "android"))]

//! Tokenizer for STT models
//!
//! This module wraps the tokenizers crate for Moonshine tokenization.

use std::path::Path;

use tokenizers::Tokenizer as HfTokenizer;

use crate::core::error::{Result, SttError};

/// Tokenizer wrapper for Moonshine models
pub struct Tokenizer {
    /// HuggingFace tokenizer
    tokenizer: HfTokenizer,

    /// End-of-sequence token ID
    eos_token_id: i32,

    /// Start-of-sequence token ID
    bos_token_id: i32,
}

impl Tokenizer {
    /// Load tokenizer from file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let tokenizer = HfTokenizer::from_file(path.as_ref())
            .map_err(|e| SttError::Tokenization(format!("Failed to load tokenizer: {}", e)))?;

        // Moonshine uses specific special tokens
        // These may need to be adjusted based on the actual tokenizer
        let eos_token_id = 2; // Common EOS token ID
        let bos_token_id = 1; // Common BOS token ID

        Ok(Self {
            tokenizer,
            eos_token_id,
            bos_token_id,
        })
    }

    /// Encode text to token IDs
    pub fn encode(&self, text: &str) -> Result<Vec<i32>> {
        let encoding = self
            .tokenizer
            .encode(text, false)
            .map_err(|e| SttError::Tokenization(format!("Encoding failed: {}", e)))?;

        Ok(encoding.get_ids().iter().map(|&id| id as i32).collect())
    }

    /// Decode token IDs to text
    pub fn decode(&self, token_ids: &[i32]) -> Result<String> {
        let ids: Vec<u32> = token_ids.iter().map(|&id| id as u32).collect();

        let text = self
            .tokenizer
            .decode(&ids, true)
            .map_err(|e| SttError::Tokenization(format!("Decoding failed: {}", e)))?;

        Ok(text)
    }

    /// Get the EOS token ID
    pub fn eos_token_id(&self) -> i32 {
        self.eos_token_id
    }

    /// Get the BOS token ID
    pub fn bos_token_id(&self) -> i32 {
        self.bos_token_id
    }

    /// Get the vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.get_vocab_size(false)
    }

    /// Get a token by ID
    pub fn id_to_token(&self, id: i32) -> Option<String> {
        self.tokenizer.id_to_token(id as u32).map(|s| s.to_string())
    }

    /// Get a token ID by text
    pub fn token_to_id(&self, token: &str) -> Option<i32> {
        self.tokenizer.token_to_id(token).map(|id| id as i32)
    }
}

/// Simple tokenizer for testing without HuggingFace dependency
pub struct SimpleTokenizer {
    /// Simple character-level vocabulary
    vocab: std::collections::HashMap<char, i32>,

    /// Reverse vocabulary
    reverse_vocab: std::collections::HashMap<i32, char>,

    /// EOS token ID
    eos_token_id: i32,
}

impl SimpleTokenizer {
    /// Create a new simple tokenizer for Japanese
    pub fn new_japanese() -> Self {
        let mut vocab = std::collections::HashMap::new();
        let mut reverse_vocab = std::collections::HashMap::new();

        // Add common Japanese characters and symbols
        let mut id = 0i32;

        // Special tokens
        vocab.insert('<', id);
        reverse_vocab.insert(id, '<');
        id += 1;

        vocab.insert('>', id);
        reverse_vocab.insert(id, '>');
        id += 1;

        // Common characters
        for c in "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん".chars() {
            vocab.insert(c, id);
            reverse_vocab.insert(id, c);
            id += 1;
        }

        // Hiragana voiced
        for c in "がぎぐげござじずぜぞだぢづでどばびぶべぼぱぴぷぺぽ".chars()
        {
            vocab.insert(c, id);
            reverse_vocab.insert(id, c);
            id += 1;
        }

        // Common katakana
        for c in "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン".chars() {
            vocab.insert(c, id);
            reverse_vocab.insert(id, c);
            id += 1;
        }

        Self {
            vocab,
            reverse_vocab,
            eos_token_id: 2,
        }
    }

    /// Encode text to token IDs
    pub fn encode(&self, text: &str) -> Vec<i32> {
        text.chars()
            .filter_map(|c| self.vocab.get(&c).copied())
            .collect()
    }

    /// Decode token IDs to text
    pub fn decode(&self, token_ids: &[i32]) -> String {
        token_ids
            .iter()
            .filter_map(|&id| self.reverse_vocab.get(&id))
            .collect()
    }

    /// Get EOS token ID
    pub fn eos_token_id(&self) -> i32 {
        self.eos_token_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokenizer_japanese() {
        let tokenizer = SimpleTokenizer::new_japanese();

        let text = "こんにちは";
        let encoded = tokenizer.encode(text);
        let decoded = tokenizer.decode(&encoded);

        // Should be able to round-trip through encode/decode
        assert!(!encoded.is_empty() || text.is_empty());
        assert_eq!(decoded, "こんにちは");
    }

    #[test]
    fn test_simple_tokenizer_eos() {
        let tokenizer = SimpleTokenizer::new_japanese();
        assert_eq!(tokenizer.eos_token_id(), 2);
    }
}
