//! AI module for on-device word definitions
//!
//! This module provides AI-powered word definition generation using
//! on-device LLM models (Qwen3.5-0.8B).

pub mod engine;

pub use engine::{AiEngine, MockAiEngine, WordDefinitionService};
