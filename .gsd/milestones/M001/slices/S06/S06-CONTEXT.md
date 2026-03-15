---
id: S06
milestone: M001
status: ready
---

# S06: AI Enhancement — Context

<!-- Slice-scoped context. Milestone-only sections (acceptance criteria, completion class,
     milestone sequence) do not belong here — those live in the milestone context. -->

## Goal

Implement on-device AI word definitions using Qwen3.5-0.8B, enabling tap-to-define functionality with <5 second latency and >85% accuracy, cached in the existing words table.

## Why this Slice

S06 delivers the AI-generated word definitions (単語採集) that differentiate this app from basic dictionary lookups. The AI understands context and nuance that static dictionaries miss, especially for Japanese polysemous words. This slice builds on S04's annotation foundation (words table with `ai_generated` flag) and S05's sequential model loading patterns. The order matters: word collection schema (S04) must exist before AI can populate definitions, and voice memos (S05) establishes the model loading/unloading lifecycle that S06 reuses. After S06, only performance polish (S07) remains before MVP launch.

## Scope

### In Scope

- **Tap-to-define interaction** — User taps any word in reader view → inline popup shows AI-generated definition
- **Qwen3.5-0.8B integration** — Candle-based on-device LLM inference (~800MB RAM budget)
- **Inline popup display** — Definition appears directly under tapped word, dismissible tap-outside
- **Definition caching** — Every AI definition stored in `words` table with `ai_generated=true`, never regenerated
- **Graceful fallback** — If model fails to load, show "AI unavailable — manual definition only" state
- **Success metrics** — <5 second definition generation AND >85% accuracy vs dictionary (validated on 50 test words)
- **4GB+ RAM target** — Optimize for mid-range devices; 2GB optimization deferred
- **Unit tests** — Prove AI definition pipeline works end-to-end (tap → load model → generate → cache → display)

### Out of Scope

- **Word frequency visualization** — Show how often word appears across books (deferred to later slice)
- **Cross-reference word usage** — See same word in different contexts across library (deferred)
- **Batch AI processing** — Pre-generate definitions for all collected words at once (deferred)
- **AI-powered search** — Semantic search using AI understanding of content (deferred)
- **Reading recommendations** — Vocabulary-level based book suggestions (deferred)
- **2GB RAM optimization** — Target 4GB+ devices first; low-RAM optimization deferred to S07
- **Model selection UI** — Users cannot choose between 0.5B/3B/8B variants (single 0.8B model)
- **Definition editing** — Users cannot modify AI-generated definitions (view-only)

## Constraints

- **Sequential model loading** — Never load OCR + Voice + AI models simultaneously (per research pitfalls)
- **Qwen3.5-0.8B only** — Single model size; no adaptive selection based on device or confidence
- **Uses existing words table** — Store definitions via `words` table with `ai_generated` boolean (S04), no schema changes
- **100% offline operation** — No cloud APIs; all inference on-device via Candle
- **Lazy model loading** — Model loaded on first tap-to-define, retained until app backgrounded
- **Japanese + English support** — Qwen must handle both languages per project requirements
- **<5 second target** — Hard latency budget: model load (if cold) + inference + display

## Integration Points

### Consumes

- `src/core/db.rs` — `words` table with `ai_generated` field (S04)
- `src/core/models.rs` — Word model structure (pre-existing)
- `src/ui/reader.rs` — Reader view with text display for tap detection (S03)
- `src/platform/android.rs` — JNI lifecycle handlers for model unloading on `onPause()` (S01)
- Candle 0.8.2 — Hugging Face's pure Rust ML framework for Qwen inference (research-confirmed)

### Produces

- `src/core/ai/engine.rs` (new) — `AiEngine` trait and `QwenEngine` implementation with Candle
- `src/core/ai/mod.rs` (new) — AI module with word definition service
- `src/ui/components.rs` (modified) — `DefinitionPopup` component for inline display
- `src/ui/reader.rs` (modified) — Tap handler for word selection and definition trigger
- `assets/ai/models/` — Qwen3.5-0.8B quantized GGUF model files
- `.gsd/milestones/M001/slices/S06/S06-TEST.md` (new) — Accuracy validation test procedure (50 words)

## Open Questions

- **Model file sourcing** — Qwen3.5-0.8B GGUF quantized model not yet bundled; need to download from Hugging Face. Current thinking: use `Qwen/Qwen3.5-0.8B-GGUF` official quant, download on first run or bundle if <100MB
- **Word tokenization** — How to detect word boundaries in Japanese text (no spaces)? Current thinking: use MeCab or Jieba for Japanese word segmentation before sending to Qwen
- **Context window** — How much surrounding text to include for context-aware definitions? Current thinking: ±50 characters from tapped word, or full sentence if shorter
- **Cold start handling** — First tap triggers model load (2-3 seconds); how to communicate this to user? Current thinking: show "Preparing AI..." loading spinner on first tap only, then instant thereafter

---

*Context gathered: 2026-03-15*
*Status: Ready for planning*
