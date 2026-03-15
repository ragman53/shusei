---
id: S05
milestone: M001
status: ready
---

# S05: Voice Memos — Context

<!-- Slice-scoped context. Milestone-only sections (acceptance criteria, completion class,
     milestone sequence) do not belong here — those live in the milestone context. -->

## Goal

Implement voice-to-text transcription for hands-free note capture, integrated with the existing annotation flow, achieving >80% accuracy on clear speech for 30-second memos.

## Why this Slice

S05 delivers the voice memo capability (ボイスメモ) that differentiates this app from basic note-taking tools. Readers can capture thoughts hands-free while holding a physical book, then edit the transcript before saving. This slice builds on S04's annotation foundation by using the `user_note` field to store voice memo transcripts. The order matters: annotation schema (S04) must exist before voice memos can attach to pages, and voice→text (S05) must work before AI keyword extraction (S06) can analyze transcripts.

## Scope

### In Scope

- **Voice memo recording** — 30 second max duration, initiated from note creation flow
- **Moonshine Tiny integration** — ONNX model (~34MB) for Japanese/English speech-to-text
- **Post-recording transcript display** — Show transcribed text after user stops recording (not live streaming)
- **Transcript editing** — Users can review and edit transcript before saving to database
- **Page-level attachment** — Voice memos linked to specific page number via `annotations.user_note` field
- **Lazy model loading** — Load Moonshine on first voice memo use, retain until app backgrounded
- **Accuracy target** — >80% transcription accuracy on clear speech, tested on 4GB+ RAM devices
- **Unit tests** — Prove voice memo pipeline works end-to-end (record → transcribe → edit → save)

### Out of Scope

- **Audio playback** — Store transcript only, no raw audio files saved (deferred to later slice)
- **Live streaming transcript** — No real-time text display during recording (deferred)
- **Text selection attachment** — Voice memos attach to page only, not highlighted text ranges (deferred)
- **AI keyword extraction** — No automatic topic tagging or keyword analysis of transcripts (belongs to S06)
- **Speaker diarization** — No multi-speaker detection (single speaker assumed)
- **Voice commands** — No "bookmark this page" or "highlight this" via voice (deferred)
- **2GB RAM optimization** — Target 4GB+ devices first; low-RAM optimization deferred
- **60+ second recordings** — Hard 30 second limit enforced

## Constraints

- **Uses existing annotation schema** — Voice memo transcript stored in `annotations.user_note` field (S04), no schema changes
- **Moonshine Tiny model only** — 34MB model fits memory budget; Small (123M) or Large (245M) variants not used
- **30 second hard limit** — Enforced in UI (stop button auto-triggers at 30s) and backend (reject longer audio)
- **Lazy model loading** — Model loaded on first use, retained until `onPause()` lifecycle event
- **Transcript-only storage** — No audio files saved; users cannot replay original recording
- **Japanese + English support** — Moonshine must handle both languages per project requirements
- **Offline operation** — No cloud APIs; 100% on-device transcription

## Integration Points

### Consumes

- `src/core/stt/engine.rs` — `SttEngine` trait and `MoonshineEngine` skeleton (pre-existing)
- `src/core/stt/decoder.rs` — KV cache and decoder state management (pre-existing)
- `src/core/stt/tokenizer.rs` — Token-to-text conversion (pre-existing)
- `src/core/db.rs` — `annotations` table with `user_note` field (S04)
- `src/core/error.rs` — `SttError` type for transcription failures (pre-existing)
- `src/platform/android.rs` — JNI lifecycle handlers for model unloading on `onPause()` (S01)

### Produces

- `src/core/stt/engine.rs` (modified) — Full `transcribe()` implementation with tract ONNX runtime
- `src/ui/components.rs` (modified) — `VoiceMemoInput` component with record/stop/edit flow
- `src/ui/reader.rs` (modified) — Voice memo button in note creation dialog
- `assets/stt/models/` — Moonshine Tiny ONNX models (encoder.onnx, decoder.onnx)
- `.gsd/milestones/M001/slices/S05/S05-TEST.md` (new) — Test procedure for accuracy validation

## Open Questions

- **Model file sourcing** — Moonshine Tiny ONNX models not yet bundled; need to download or convert from PyTorch. Current thinking: use Moonshine's official export script to generate ONNX, bundle in `assets/stt/models/`
- **Audio capture implementation** — Android `AudioRecord` via JNI not yet implemented. Current thinking: use `android-activity` crate + JNI wrapper for `android.media.AudioRecord`
- **Error handling for poor accuracy** — What threshold triggers "transcription may be inaccurate" warning? Current thinking: if Moonshine returns confidence <0.7, show warning banner above transcript editor
- **Model loading time** — How long to load 34MB model on cold start? Current thinking: benchmark during implementation; if >2 seconds, add loading spinner with "Preparing voice input..." message

---

*Context gathered: 2026-03-15*
*Status: Ready for planning*
