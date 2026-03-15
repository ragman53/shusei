# T06: Unit Tests

**Goal:** Write comprehensive unit tests for voice memo pipeline

## Plan

1. **Audio recording tests (T01)**
   - Test `record_audio()` returns correct format
   - Test 30s limit enforcement
   - Test error handling (permission denied, hardware unavailable)
   - Test `AudioResult` structure

2. **Audio preprocessing tests (T02)**
   - Test mel-spectrogram output shape
   - Test STFT parameters (window size, hop length, mel bins)
   - Test duration validation (reject >30s)
   - Test edge cases (empty audio, very short audio)

3. **Moonshine engine tests (T03)**
   - Test model loading
   - Test transcription with sample audio
   - Test KV cache management
   - Test decoder loop termination
   - Test language switching

4. **Integration tests (T04 + T05)**
   - Test end-to-end flow: record → transcribe → save
   - Test voice memo saves to annotation correctly
   - Test page association
   - Test error recovery (transcription failure → manual entry)

5. **Create test fixtures**
   - Sample audio files for testing (16kHz mono PCM)
   - Mock audio data for unit tests
   - Expected transcription outputs

## Files to Create/Modify

- `src/core/stt/mel_spectrogram.rs` — Add tests
- `src/core/stt/engine.rs` — Add tests
- `src/core/stt/decoder.rs` — Add tests
- `tests/voice_memo_test.rs` (new) — Integration tests
- `.gsd/milestones/M001/slices/S05/tasks/T06-PLAN.md` — This file

## Verification

- [ ] `cargo check --lib` passes
- [ ] `cargo test --lib` passes (10+ tests)
- [ ] All audio pipeline tests pass
- [ ] All STT engine tests pass
- [ ] Integration tests pass
- [ ] Test coverage >80% for voice memo code

## Dependencies

- Consumes: All prior tasks (T01-T05)
- Produces: Test suite proving voice memo pipeline works
