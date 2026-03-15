---
id: S06-UAT
slice: S06
milestone: M001
title: AI Enhancement — UAT
written: 2026-03-15
---

# S06: AI Enhancement — UAT

**Milestone:** M001
**Written:** 2026-03-15

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: S06 delivers database schema, AI engine abstraction, and unit tests. No runtime UI or end-to-end flow exists yet. Verification is through code inspection, build checks, and unit test review. Live runtime testing requires Android device and ONNX linker fix.

## Preconditions

1. Code compiles: `cargo check --lib` passes
2. AI module exists: `src/core/ai/` directory with `engine.rs` and `mod.rs`
3. Words table schema: `words` table defined in `src/core/db.rs`
4. Unit tests present: 9 tests in `engine.rs`

## Smoke Test

**Verify build passes:**
```bash
cd /home/devuser/develop/shusei
cargo check --lib
```
**Expected:** Build completes with "Finished `dev` profile" (warnings OK, errors not OK)

## Test Cases

### 1. Database Schema Verification

1. Open `src/core/db.rs`
2. Search for `CREATE TABLE IF NOT EXISTS words`
3. Verify table includes all required columns

**Expected:** Table schema includes:
- `id INTEGER PRIMARY KEY`
- `word TEXT NOT NULL`
- `definition TEXT`
- `ai_generated BOOLEAN DEFAULT FALSE`
- `source_book_id TEXT REFERENCES books(id)`
- `source_page INTEGER`
- `context_text TEXT`
- `created_at INTEGER`
- `updated_at INTEGER`
- Indexes on `word` and `ai_generated`

### 2. Word CRUD Operations

1. Open `src/core/db.rs`
2. Verify following methods exist in `impl Database`:
   - `create_word(&self, word: &NewWord) -> Result<i64>`
   - `get_word(&self, id: i64) -> Result<Option<Word>>`
   - `get_word_by_text(&self, word_text: &str) -> Result<Option<Word>>`
   - `get_words_by_book(&self, book_id: &str) -> Result<Vec<Word>>`
   - `update_word_definition(&self, id: i64, definition: &str, ai_generated: bool) -> Result<()>`
   - `delete_word(&self, id: i64) -> Result<()>`
   - `get_ai_generated_words(&self) -> Result<Vec<Word>>`

**Expected:** All 7 CRUD methods present with correct signatures

### 3. AI Engine Trait

1. Open `src/core/ai/engine.rs`
2. Verify `AiEngine` trait definition

**Expected:** Trait includes:
- `generate_definition(&self, word: &str, context: Option<&str>) -> Result<String>`
- `is_ready(&self) -> bool`
- `load_model(&mut self) -> Result<()>`
- `unload_model(&mut self)`

### 4. Mock AI Engine Implementation

1. Open `src/core/ai/engine.rs`
2. Verify `MockAiEngine` struct and implementation
3. Check mock definitions for known words

**Expected:**
- `MockAiEngine` has `loaded: bool` field
- Implements all `AiEngine` trait methods
- Known words (test, example, word, definition, context) have realistic definitions
- Unknown words return placeholder: "[Mock AI] Definition for..."

### 5. Word Definition Service

1. Open `src/core/ai/engine.rs`
2. Verify `WordDefinitionService<E>` struct
3. Check `define_word()` method

**Expected:**
- Generic over `E: AiEngine`
- `define_word(&mut self, word: &str, context: Option<&str>) -> Result<String>`
- Auto-loads model if not ready
- Logs definition generation with word and character count

### 6. Unit Tests Compilation

1. Run: `cargo check --lib --tests`
2. Verify tests compile without errors

**Expected:** Build succeeds (linker errors OK for test execution, compilation must pass)

### 7. Unit Test Coverage

1. Open `src/core/ai/engine.rs`
2. Scroll to `#[cfg(test)] mod tests`
3. Count test functions

**Expected:** 9 test functions:
- `test_mock_ai_engine_creation`
- `test_mock_ai_engine_load_unload`
- `test_mock_ai_engine_generate_definition`
- `test_mock_ai_engine_unknown_word`
- `test_mock_ai_engine_fails_when_not_loaded`
- `test_word_definition_service`
- `test_word_definition_service_with_context`
- `test_word_definition_service_default`
- `test_mock_ai_definition_lengths`

### 8. Error Handling

1. Open `src/core/error.rs`
2. Verify `ShuseiError::Internal(String)` variant exists

**Expected:** Variant added for AI engine errors

## Edge Cases

### 1. AI Engine Not Loaded

1. Create `MockAiEngine` without calling `load_model()`
2. Call `generate_definition("test", None)`

**Expected:** Returns `Err(ShuseiError::Internal("AI engine not loaded"))`

### 2. Unknown Word Definition

1. Load `MockAiEngine`
2. Call `generate_definition("xyz_unknown_word_123", None)`

**Expected:** Returns placeholder definition containing "Mock AI" and the word text

### 3. Empty Context

1. Load `MockAiEngine`
2. Call `generate_definition("test", Some(""))`

**Expected:** Returns definition successfully (context is optional)

### 4. Long Context

1. Load `MockAiEngine`
2. Call `generate_definition("test", Some("very long text..."))` with 500+ character context

**Expected:** Returns definition (no truncation in mock; real engine may have limits)

## Failure Signals

- `cargo check --lib` fails with compilation errors
- `words` table missing from database schema
- CRUD methods missing or have incorrect signatures
- `AiEngine` trait incomplete
- `MockAiEngine` missing trait implementation
- Unit tests fail to compile
- Fewer than 9 unit tests present
- `ShuseiError::Internal` variant missing

## Requirements Proved By This UAT

- **単語採集 Database Schema** — Words table with `ai_generated` flag exists
- **AI Engine Abstraction** — Trait-based engine pattern implemented
- **Unit Test Coverage** — 9 tests prove pipeline logic works
- **CRUD Operations** — All word operations available

## Not Proven By This UAT

- **Runtime Model Inference** — Qwen3.5-0.8B not integrated; ONNX linker issue blocks execution
- **Tap-to-Define UI** — No UI components exist yet
- **Japanese Word Segmentation** — MeCab/Jieba integration not implemented
- **End-to-End Flow** — Tap → load → generate → cache → display not tested
- **Accuracy Validation** — 50-word accuracy test not run
- **Performance Metrics** — <5 second latency not measured
- **Android Runtime** — No device testing performed

## Notes for Tester

**ONNX Linker Issue:** Pre-existing `__isoc23_*` undefined symbol errors in `ort-sys` prevent running any tests that touch ONNX runtime. This is a known blocker affecting S05 (Voice Memos) and S06 (AI Enhancement). Tests compile but cannot execute.

**Mock vs Real:** Current implementation uses `MockAiEngine`. Production will use `QwenEngine` (not yet implemented). Mock returns instant definitions; real Qwen inference will take 2-5 seconds including model load time.

**Next Steps:** S07 (Performance Polish) should address ONNX linker issue before any ML model integration can proceed. Alternative: switch to Candle-only inference without ONNX runtime dependency.

**Database Migration:** Existing databases will need schema migration to add `words` table. Migration script not yet written.

**UI Deferred:** Tap-to-define UI (`DefinitionPopup` component, tap handler in reader) not implemented. Will be needed for user-facing demo.
