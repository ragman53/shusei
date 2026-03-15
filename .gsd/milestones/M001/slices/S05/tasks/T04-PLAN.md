# T04: Voice Memo UI Component

**Goal:** Create voice memo recording component with record/stop/edit flow

## Plan

1. **Design VoiceMemoInput component structure**
   - Record button (starts recording)
   - Stop button (appears during recording)
   - Timer display (shows elapsed time up to 30s)
   - Transcript editor (textarea for reviewing/editing)
   - Save/Cancel buttons
   - Loading spinner (during transcription)

2. **Implement component state machine**
   - States: `Idle` → `Recording` → `Transcribing` → `Editing` → `Saved`
   - Track elapsed time during recording
   - Store transcript text
   - Handle state transitions

3. **Implement recording flow**
   - Call `platform_api.record_audio(30)` on record button click
   - Show timer updating in real-time
   - Auto-stop at 30 seconds
   - Handle permission errors gracefully

4. **Implement transcription flow**
   - Call `stt_engine.transcribe()` with audio samples
   - Show loading spinner during transcription
   - Display transcript in editable textarea
   - Show confidence warning if <0.7

5. **Implement save flow**
   - User can edit transcript
   - Save button emits transcript string
   - Cancel button discards recording
   - Return transcript to parent component

6. **Add error handling**
   - Permission denied: show user-friendly message
   - Recording failed: retry option
   - Transcription failed: allow manual text entry
   - Timeout handling

## Files to Create/Modify

- `src/ui/components.rs` — Add `VoiceMemoInput` component
- `src/ui/mod.rs` — Export new component
- `.gsd/milestones/M001/slices/S05/tasks/T04-PLAN.md` — This file

## Verification

- [ ] `cargo check --lib` passes
- [ ] Component renders correctly in all states
- [ ] Recording starts/stops properly
- [ ] Timer displays correctly (0:00 to 0:30)
- [ ] Transcript editor shows transcribed text
- [ ] Save/Cancel buttons work
- [ ] Error states handled gracefully

## Dependencies

- Consumes: Audio recording from T01, transcription from T03
- Produces: Transcript string for T05 (saves to annotations)
