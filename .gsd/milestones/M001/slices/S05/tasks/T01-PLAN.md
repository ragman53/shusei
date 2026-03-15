# T01: Audio Recording Pipeline

**Goal:** Implement audio capture from microphone with 30s limit

## Plan

1. **Investigate existing platform audio API**
   - Review `src/platform/mod.rs` `PlatformApi::record_audio()` trait
   - Review `src/platform/android.rs` stub implementation
   - Understand `AudioResult` structure (samples, sample_rate, duration)

2. **Implement Android audio recording via JNI**
   - Use `android-activity` crate for Android lifecycle
   - JNI wrapper for `android.media.AudioRecord` class
   - Configure AudioRecord for 16kHz mono PCM
   - Implement 30-second hard limit with auto-stop

3. **Implement permission handling**
   - Add `request_microphone_permission()` implementation
   - Handle permission denial gracefully
   - Show user-facing error message if permission denied

4. **Add error handling**
   - Handle audio hardware unavailable
   - Handle recording interrupted
   - Return proper `ShuseiError::AudioProcessing` errors

5. **Write unit tests**
   - Test audio capture returns correct format
   - Test 30s limit is enforced
   - Test error cases (permission denied, hardware unavailable)

## Files to Create/Modify

- `src/platform/android.rs` — Implement `record_audio()` with JNI AudioRecord
- `src/platform/mod.rs` — May need to refine `AudioResult` structure
- `.gsd/milestones/M001/slices/S05/tasks/T01-PLAN.md` — This file

## Verification

- [ ] `cargo check --lib` passes
- [ ] Audio recording returns 16kHz mono PCM as `Vec<f32>`
- [ ] 30-second limit enforced (auto-stop at 30s)
- [ ] Permission request flow works
- [ ] Error cases handled properly
- [ ] 3+ unit tests pass

## Dependencies

- Consumes: `src/platform/mod.rs` trait definition
- Produces: Working audio capture for STT pipeline
