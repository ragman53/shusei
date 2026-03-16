# Decisions Register

<!-- Append-only. Never edit or remove existing rows.
     To reverse a decision, add a new row that supersedes it.
     Read this file at the start of any planning or research phase. -->

| # | When | Scope | Decision | Choice | Rationale | Revisable? |
|---|------|-------|----------|--------|-----------|------------|
| D001 | M002 | inference | Inference runtime for Android | tract-onnx (stay) | Qwen model size (500MB+) dominates APK anyway; 9MB runtime savings irrelevant; M001 already has 92 passing tests with tract | No |
| D002 | M002 | dictionary | Dictionary for word definitions | Defer to M003, show placeholder in M002 | Prototype priority is core flows (camera, PDF); word save + example sentence works without definition; reduces M002 complexity | No |
| D003 | M002 | models | Model bundling strategy | Bundle NDLOCR + Moonshine only (~30-40MB) | Qwen deferred; NDLOCR essential for OCR, Moonshine for future voice; keeps APK ~50-60MB for prototype | Yes — if device storage constraints discovered |
| D004 | M002 | build | Android build configuration | Debug APK sufficient for prototype | Release signing not required for device testing; simplifies M002; release build can be done in M003 | Yes — for production release |
| D005 | M002 | build | Gradle patch approach | Post-generation patch script (Java 21, skip lint, fix manifest) | Dioxus 0.7.3 generates obsolete Java 8 config; upstream fix not available; workaround confirmed in GitHub issue #5251 | Yes — if Dioxus fixes templates |
| D006 | M002 | device | Target device for testing | Motorola Moto G66j 5G (mid-range) | Real-world performance validation on mid-range hardware; ensures app works for typical users, not just flagship devices | No |
| D007 | M002 | word-collection | Word definition source | Placeholder in M002, dictionary/AI in M003 | Separates concerns: M002 proves word+sentence save works; M003 adds definition quality | No |
| D008 | M002 | build | Java/Kotlin target version | Java 17 (matching installed JDK) | System has OpenJDK 17; Java 21 not available; ensures build reproducibility without requiring JDK upgrade | Yes — if JDK 21 installed |
| D009 | M002 | sdk | Android SDK installation | Separate SDK from NDK | NDK-only ANDROID_HOME insufficient for gradle build; installed command-line tools + platform-tools + platform-33 + build-tools-34.0.0 | No |
