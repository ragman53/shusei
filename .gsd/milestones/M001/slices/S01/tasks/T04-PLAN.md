# T04: 01-core-infrastructure 04

**Slice:** S01 — **Milestone:** M001

## Description

Implement Android lifecycle handling with state persistence and JNI memory management

Purpose: Ensure app handles background transitions gracefully without memory leaks
Output: Lifecycle-aware app with state restoration and clean JNI patterns

## Must-Haves

- [ ] "App survives background/foreground transition"
- [ ] "State restored after app killed and reopened"
- [ ] "No JNI memory leaks during normal operation"

## Files

- `src/platform/android.rs`
- `src/core/state.rs`
- `src/app.rs`
