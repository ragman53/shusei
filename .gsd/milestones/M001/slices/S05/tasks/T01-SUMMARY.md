---
id: T01
slice: S05
milestone: M001
title: Audio Recording Pipeline
status: completed
started_at: 2026-03-15
completed_at: 2026-03-15
observability_surfaces:
  - JNI callback logs in LogCat (tag: "Shusei")
  - AudioResult struct in src/platform/mod.rs for error inspection
  - Permission state visible via has_microphone_permission()
---

# T01: Audio Recording Pipeline — Summary

**Goal:** Implement audio capture from microphone with 30s limit

## What Happened

Task T01 implemented the Android audio recording infrastructure via JNI. The implementation adds:

1. **AudioRecordState management** — New static state for tracking audio recording callbacks (following the existing camera/file picker pattern)
2. **record_audio() implementation** — Complete JNI integration that:
   - Enforces 30-second hard limit
   - Requests microphone permission automatically
   - Calls Java `startAudioRecording(int maxSeconds)` method
   - Waits for callback with timeout (max_seconds + 5s buffer)
3. **Permission handling** — Implemented `has_microphone_permission()` and `request_microphone_permission()` using JNI calls to MainActivity
4. **JNI callbacks** — Added four callback functions:
   - `Java_com_shusei_app_MainActivity_onAudioRecorded` — Receives audio data (float array, sample rate, duration)
   - `Java_com_shusei_app_MainActivity_onAudioRecordFailed` — Handles recording failures
   - `Java_dev_dioxus_main_MainActivity_onAudioRecorded` — Same for dev.dioxus.main package
   - `Java_dev_dioxus_main_MainActivity_onAudioRecordFailed` — Same for dev.dioxus.main package
5. **send_audio_result()** — Helper function to send results back to async caller via oneshot channel

## Verification

**Build Verification:**
- `cargo check --lib` passes with no errors
- Code follows existing patterns for camera/file picker
- Type signatures match `PlatformApi` trait

**Code Quality:**
- Proper error handling for JNI failures
- Timeout handling for recording
- 30-second hard limit enforced in Rust code (defense in depth with Java side)
- Both package namespaces supported (com.shusei.app and dev.dioxus.main)

**Pending Verification:**
- Runtime testing on Android device (requires hardware)
- Integration with STT pipeline (T03)
- End-to-end voice memo flow (T05)

## Files Created/Modified

- `src/platform/android.rs` (+250 lines) — Audio recording implementation, permission handling, JNI callbacks

## Follow-ups

1. **Java Side Implementation** — MainActivity.java needs:
   - `startAudioRecording(int maxSeconds)` method
   - `hasMicrophonePermission()` method
   - `requestMicrophonePermission()` method
   - `nativeOnAudioRecorded(float[], int, float)` callback
   - `nativeOnAudioRecordFailed(String)` callback
   - Android `AudioRecord` integration for 16kHz mono PCM capture

2. **Audio Preprocessing (T02)** — Raw PCM needs conversion to mel-spectrogram

3. **Error Handling Refinement** — Consider retry logic for transient failures

## Forward Intelligence

### What the next task should know
- Audio format: 16kHz mono PCM as `Vec<f32>` (normalized -1.0 to 1.0)
- Max duration: 30 seconds enforced at Rust layer
- Sample rate: 16000 Hz (standard for speech recognition)
- Duration: Provided by Java side (more accurate than calculating from sample count)

### What's fragile
- **JNI Callback Pattern** — Relies on Java side calling correct callback; no verification that callback will fire
- **Timeout Logic** — 5s buffer may not be enough if Java side has delays; may need tuning
- **Permission Flow** — Assumes permission granted immediately; Android runtime permission may show dialog

### Authoritative diagnostics
- **src/platform/android.rs:100-140** — `record_audio()` implementation
- **src/platform/android.rs:550-700** — JNI callback functions
- **src/platform/mod.rs:45-52** — `AudioResult` structure definition

### What assumptions changed
- **Original:** Audio recording would use byte array (PCM bytes)
- **Actual:** Using float array for normalized samples (easier for DSP processing)

## Diagnostics

**Check recording status:**
```bash
adb logcat -s Shusei | grep -E "AudioRecord|onAudioRecorded|onAudioRecordFailed"
```

**Verify permission state:**
- Call `has_microphone_permission()` from Rust debugger or add temporary log statement
- Check AndroidManifest.xml has `<uses-permission android:name="android.permission.RECORD_AUDIO" />`

**Common failure modes:**
- `AudioRecord callback timeout` — Java side didn't call callback; check LogCat for exceptions
- `Permission denied` — User denied microphone permission; check Android app settings
- `JNI method not found` — MainActivity.java missing `startAudioRecording()` method
