---
estimated_steps: 5
estimated_files: 3
---

# T02: Verify APK structure and model assets

**Slice:** S01 — Android Build + Deploy
**Milestone:** M002-dbrk2n

## Description

Ensure APK includes assets/models/ directory with NDLOCR and Moonshine model files. Verify bundle configuration in Dioxus.toml.

## Steps

1. Check if model files exist: `ls -la assets/models/`
2. If models missing, document acquisition steps (download from source)
3. Verify Dioxus.toml bundle config includes models: `resources = ["assets/models/*"]`
4. After build, unzip APK and inspect: `unzip -l app-debug.apk | grep models`
5. Confirm model files are in APK assets directory

## Must-Haves

- [ ] NDLOCR model file present (detection + recognition)
- [ ] Moonshine model file present (STT encoder + decoder)
- [ ] Dioxus.toml bundle config includes assets/models/*
- [ ] APK contains model files in assets/ directory

## Verification

- `unzip -l target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk | grep -E "\.onnx$"` shows model files
- Model files total size < 50MB (NDLOCR ~5-10MB, Moonshine ~20-30MB)

## Observability Impact

- Signals added/changed: None (verification task)
- How a future agent inspects this: unzip APK, check assets/models/ contents
- Failure state exposed: Missing model files in APK, build succeeds but inference fails at runtime

## Inputs

- M001 inference engines — require model files at runtime
- Dioxus.toml — bundle configuration

## Expected Output

- Verified model files in APK
- Updated Dioxus.toml if bundle config was incomplete
- Documentation of model acquisition if files were missing
