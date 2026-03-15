# T03: Moonshine Model Integration

**Goal:** Complete Moonshine engine implementation with tract ONNX runtime

## Plan

1. **Acquire Moonshine Tiny ONNX models**
   - Download from https://github.com/usefulsensors/moonshine or Hugging Face
   - Get both English and Japanese models (encoder + decoder for each)
   - Place in `assets/models/moonshine/` directory
   - Verify model files are valid ONNX format

2. **Complete MoonshineEngine::initialize()**
   - Load encoder.onnx using tract-onnx
   - Load decoder.onnx using tract-onnx
   - Initialize KV cache for decoder
   - Load tokenizer (existing `src/core/stt/tokenizer.rs`)
   - Handle model loading errors gracefully

3. **Implement transcribe() method**
   - Step 1: Run encoder on mel-spectrogram input
   - Step 2: Autoregressive decoder loop with KV cache
   - Step 3: Token-to-text conversion using tokenizer
   - Step 4: Return `SttResult` with text, confidence, timing

4. **Implement decoder loop**
   - Use existing `src/core/stt/decoder.rs` skeleton
   - Implement `decoder_step()` with tract inference
   - Manage KV cache updates
   - Handle EOS token detection
   - Enforce `max_decoder_steps` limit

5. **Add language support**
   - Support both Japanese and English models
   - Lazy load models based on `Language` config
   - Implement language switching if needed

6. **Write unit tests**
   - Test model loading
   - Test transcription with sample audio
   - Test KV cache management
   - Test decoder loop termination

## Files to Create/Modify

- `src/core/stt/engine.rs` — Complete `transcribe()` implementation
- `src/core/stt/decoder.rs` — Complete `decoder_step()` with tract
- `assets/models/moonshine/` — Add Moonshine Tiny ONNX models
- `.gsd/milestones/M001/slices/S05/tasks/T03-PLAN.md` — This file

## Verification

- [ ] `cargo check --lib` passes
- [ ] Models load successfully
- [ ] Transcription produces text output
- [ ] KV cache works correctly
- [ ] Decoder loop terminates properly
- [ ] 5+ unit tests pass

## Dependencies

- Consumes: Preprocessed audio from T02 (mel-spectrogram)
- Produces: Transcribed text for T04/T05 (SttResult::text)
