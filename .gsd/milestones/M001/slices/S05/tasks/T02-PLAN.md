# T02: Audio Preprocessing

**Goal:** Implement mel-spectrogram computation for Moonshine encoder input

## Plan

1. **Understand Moonshine audio requirements**
   - Review `assets/models/moonshine/README.md` for preprocessing specs
   - Sample rate: 16000 Hz
   - Window size: 25ms (400 samples)
   - Hop length: 10ms (160 samples)
   - Mel bins: 80
   - FFT size: 400

2. **Implement mel-spectrogram computation**
   - Add `mel_spectrogram.rs` module to `src/core/stt/`
   - Implement Short-Time Fourier Transform (STFT)
   - Apply mel filterbank (80 bins)
   - Apply log compression: `log(mel + ε)`
   - Optional: per-speaker normalization

3. **Integrate with MoonshineEngine**
   - Update `MoonshineEngine::preprocess_audio()` in `src/core/stt/engine.rs`
   - Replace placeholder with actual mel-spectrogram computation
   - Output format: `[batch, 80 mel_bins, time_frames]`

4. **Handle audio duration validation**
   - Check audio length against `SttConfig::max_duration_seconds`
   - Return `SttError::AudioTooLong` if exceeds 30s
   - Calculate audio duration from sample count

5. **Write unit tests**
   - Test mel-spectrogram output shape
   - Test various audio lengths
   - Test duration validation
   - Test edge cases (empty audio, very short audio)

## Files to Create/Modify

- `src/core/stt/mel_spectrogram.rs` (new) — Mel-spectrogram computation
- `src/core/stt/engine.rs` — Update `preprocess_audio()` implementation
- `src/core/stt/mod.rs` — Export new module
- `.gsd/milestones/M001/slices/S05/tasks/T02-PLAN.md` — This file

## Verification

- [ ] `cargo check --lib` passes
- [ ] Mel-spectrogram produces correct output shape
- [ ] STFT parameters match Moonshine requirements
- [ ] Duration validation works (rejects >30s audio)
- [ ] 5+ unit tests pass

## Dependencies

- Consumes: Raw audio from T01 (`AudioResult::samples`)
- Produces: Preprocessed audio for T03 (mel-spectrogram)
