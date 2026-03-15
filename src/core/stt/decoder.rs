//! Autoregressive decoder with KV cache for Moonshine
//!
//! This module implements the decoder loop with KV cache management
//! for efficient autoregressive token generation.

use std::collections::HashMap;

use crate::core::error::{SttError, Result};

/// KV Cache for efficient autoregressive decoding
#[derive(Debug, Clone)]
pub struct KvCache {
    /// Cache layers
    layers: Vec<CacheLayer>,
    
    /// Maximum sequence length
    max_seq_len: usize,
}

/// A single cache layer
#[derive(Debug, Clone)]
pub struct CacheLayer {
    /// Key cache
    pub key: Vec<Vec<f32>>,
    
    /// Value cache
    pub value: Vec<Vec<f32>>,
}

impl KvCache {
    /// Create a new KV cache
    pub fn new(num_layers: usize, num_heads: usize, head_dim: usize, max_seq_len: usize) -> Self {
        let layers = (0..num_layers)
            .map(|_| CacheLayer {
                key: vec![vec![0.0; head_dim * num_heads]; max_seq_len],
                value: vec![vec![0.0; head_dim * num_heads]; max_seq_len],
            })
            .collect();
        
        Self {
            layers,
            max_seq_len,
        }
    }
    
    /// Get the current sequence length
    pub fn seq_len(&self) -> usize {
        // Return the first non-zero position
        self.layers
            .first()
            .map(|layer| layer.key.len())
            .unwrap_or(0)
    }
    
    /// Append new key-value pairs to the cache
    pub fn append(&mut self, layer_idx: usize, keys: &[Vec<f32>], values: &[Vec<f32>]) -> Result<()> {
        if layer_idx >= self.layers.len() {
            return Err(SttError::KvCache(format!(
                "Invalid layer index: {} >= {}",
                layer_idx,
                self.layers.len()
            )).into());
        }
        
        let layer = &mut self.layers[layer_idx];
        
        for (i, key) in keys.iter().enumerate() {
            if layer.key.len() < self.max_seq_len {
                layer.key.push(key.clone());
                if i < values.len() {
                    layer.value.push(values[i].clone());
                }
            }
        }
        
        Ok(())
    }
    
    /// Clear the cache
    pub fn clear(&mut self) {
        for layer in &mut self.layers {
            layer.key.clear();
            layer.value.clear();
        }
    }
    
    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.layers.iter().all(|layer| layer.key.is_empty())
    }
}

/// Decoder state for autoregressive generation
#[derive(Debug)]
pub struct DecoderState {
    /// Current position in the sequence
    pub position: usize,
    
    /// KV cache
    pub kv_cache: KvCache,
    
    /// Generated tokens so far
    pub tokens: Vec<i32>,
    
    /// Whether EOS has been generated
    pub finished: bool,
}

impl DecoderState {
    /// Create a new decoder state
    pub fn new(kv_cache: KvCache) -> Self {
        Self {
            position: 0,
            kv_cache,
            tokens: Vec::new(),
            finished: false,
        }
    }
    
    /// Append a new token
    pub fn append_token(&mut self, token: i32) {
        self.tokens.push(token);
        self.position += 1;
    }
    
    /// Check if decoding should stop
    pub fn should_stop(&self, eos_token: i32, max_steps: usize) -> bool {
        self.finished || 
            self.tokens.last().map(|&t| t == eos_token).unwrap_or(false) ||
            self.position >= max_steps
    }
}

/// Autoregressive decoder step result
#[derive(Debug)]
pub struct DecoderStepResult {
    /// Next token
    pub next_token: i32,
    
    /// Token probability
    pub probability: f32,
    
    /// Whether this is the EOS token
    pub is_eos: bool,
}

/// Run a single decoder step
pub fn decoder_step(
    _state: &mut DecoderState,
    _encoder_output: &[f32],
) -> Result<DecoderStepResult> {
    // TODO: Implement decoder step using tract
    // This will be implemented in Week 9-10
    
    // Placeholder implementation
    Ok(DecoderStepResult {
        next_token: 0,
        probability: 1.0,
        is_eos: false,
    })
}

/// Sampling strategies for token selection
#[derive(Debug, Clone, Copy)]
pub enum SamplingStrategy {
    /// Greedy: always select the highest probability token
    Greedy,
    
    /// Top-k sampling: sample from top k tokens
    TopK(usize),
    
    /// Top-p (nucleus) sampling
    TopP(f32),
}

impl Default for SamplingStrategy {
    fn default() -> Self {
        SamplingStrategy::Greedy
    }
}

/// Select a token based on logits and sampling strategy
pub fn select_token(logits: &[f32], strategy: SamplingStrategy) -> (i32, f32) {
    match strategy {
        SamplingStrategy::Greedy => {
            let (idx, &prob) = logits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));
            (idx as i32, prob)
        }
        SamplingStrategy::TopK(k) => {
            // TODO: Implement top-k sampling
            let (idx, &prob) = logits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));
            (idx as i32, prob)
        }
        SamplingStrategy::TopP(_p) => {
            // TODO: Implement top-p sampling
            let (idx, &prob) = logits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));
            (idx as i32, prob)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_kv_cache_new() {
        let cache = KvCache::new(4, 8, 64, 512);
        assert!(cache.is_empty());
        assert_eq!(cache.seq_len(), 0);
    }
    
    #[test]
    fn test_decoder_state() {
        let cache = KvCache::new(4, 8, 64, 512);
        let state = DecoderState::new(cache);
        assert_eq!(state.position, 0);
        assert!(state.tokens.is_empty());
        assert!(!state.finished);
    }
    
    #[test]
    fn test_sampling_strategy_greedy() {
        let logits = vec![0.1, 0.5, 0.3, 0.9, 0.2];
        let (token, prob) = select_token(&logits, SamplingStrategy::Greedy);
        assert_eq!(token, 3);
        assert!((prob - 0.9).abs() < 1e-6);
    }
}