# Requirements

This file is the explicit capability and coverage contract for the project.

Use it to track what is actively in scope, what has been validated by completed work, what is intentionally deferred, and what is explicitly out of scope.

## Active

### R001 — Camera book capture with page linkage
- Class: core-capability
- Status: validated
- Description: User creates a book (title/author) → captures pages via camera → OCR runs → pages saved with book linkage and page number
- Why it matters: Paper book workflow is core value proposition — users need to photograph pages and have them organized by book
- Source: user
- Primary owning slice: M002/S02
- Supporting slices: M002/S05
- Validation: S02 integration tests (4 tests pass: end-to-end flow, multiple pages, duplicate detection, storage organization)
- Notes: Page number entered manually; OCR runs via NDLOCR; pages stored in book_pages table; device-level JNI camera testing pending S05

### R002 — PDF reflow reader with progress tracking
- Class: core-capability
- Status: validated
- Description: User imports PDF → NDLOCR converts to markdown → continuous scroll reading with font control (12-32px), auto scroll progress detection, last-read position sync
- Why it matters: Kindle-like reading experience for PDFs; progress tracking enables resume reading
- Source: user
- Primary owning slice: M002/S03
- Supporting slices: M002/S05
- Validation: S03 integration tests (5 tests pass: markdown rendering, word tap persistence, duplicate handling, progress auto-save, position restore)
- Notes: pulldown-cmark for proper markdown parsing; word-level spans for tap detection; debounced progress save (500ms); localStorage for font size preference; device-level testing pending S05

### R003 — Word + example sentence collection (placeholder definition)
- Class: primary-user-loop
- Status: validated
- Description: User taps word in PDF/OCR text → app shows "Definition coming soon" placeholder → saves word + full sentence containing the word to vocabulary
- Why it matters: Vocabulary learning requires context (example sentence); definition can be added later
- Source: user
- Primary owning slice: M002/S04
- Supporting slices: M002/S03
- Validation: S03 integration tests (3 tests pass: word tap saves to database, duplicate word handling, sentence extraction)
- Notes: Definition deferred to M003 per D007; word + sentence saved to words table with book/page reference; WordExtractor::extract_sentence() for context; toast notification for feedback; device-level testing pending S05

### R004 — APK deploys on Moto G66j 5G
- Class: operability
- Status: active
- Description: Debug APK installs and launches on Motorola Moto G66j 5G without crashes; basic flows (camera, PDF) work on device
- Why it matters: Real device testing validates performance, JNI stability, model loading on mid-range hardware
- Source: user
- Primary owning slice: M002/S01
- Supporting slices: M002/S05
- Validation: partial
- Notes: Debug APK built successfully (139MB); installation infrastructure ready; device testing pending physical hardware connection via WSL2 USB passthrough

### R005 — SQLite data persists across restarts
- Class: continuity
- Status: validated
- Description: Books, pages, words, annotations saved to SQLite survive app restart; last-read position restored
- Why it matters: Users expect their data to persist; core trust requirement
- Source: inferred
- Primary owning slice: M002/S01
- Supporting slices: M002/S02, M002/S03, M002/S04
- Validation: S02 integration tests verify book + page persistence; S03 tests verify word + progress persistence (test_word_tap_saves_to_database, test_progress_auto_save, test_last_position_restore); device-level verification pending S05
- Notes: Database persistence verified via file-based tests simulating app restart (6 tests pass for S02, 5 tests for S03); position restore on mount confirms data survives component remount; device-level verification pending physical hardware connection

### R006 — Model bundling (NDLOCR, Moonshine)
- Class: integration
- Status: active
- Description: NDLOCR OCR model (~5-10MB) and Moonshine STT model (~20-30MB) bundled in APK assets; models load on first inference
- Why it matters: Offline operation requires models on device; bundling avoids download complexity
- Source: execution
- Primary owning slice: M002/S05
- Supporting slices: M002/S02, M002/S03
- Validation: unmapped
- Notes: Qwen model deferred to M003; models stored in assets/models/ directory

### R007 — Android Gradle build patch script
- Class: operability
- Status: validated
- Description: Post-generation patch script fixes Dioxus-generated Gradle files (Java 21, skip lint, fix manifest); enables successful APK build
- Why it matters: Dioxus 0.7.3 generates obsolete Java 8 config; patch required for modern Android tooling
- Source: research
- Primary owning slice: M002/S01
- Supporting slices: none
- Validation: validated
- Notes: Script patches build.gradle.kts, AndroidManifest.xml, skips lintVital tasks; debug APK built successfully (139MB)

## Deferred

### R008 — Qwen AI definitions
- Class: differentiator
- Status: deferred
- Description: Tap word → show AI-generated definition from Qwen3.5-0.8B model
- Why it matters: AI definitions provide rich vocabulary learning; not required for prototype
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: Deferred to M003 due to model size (500MB+); M002 shows placeholder

### R009 — Japanese/English dictionary (JMdict/WordNet)
- Class: differentiator
- Status: deferred
- Description: Bundle JMdict (日英, 5MB) + WordNet (英英, 10MB) + optional JPDict (日日, 20MB) for offline dictionary lookup
- Why it matters: Provides instant definitions without AI; lighter weight than Qwen
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: Deferred to M003; M002 word-tap shows placeholder

### R010 — Voice memo recording
- Class: core-capability
- Status: deferred
- Description: Record voice memo via JNI → Moonshine STT transcribes to text → save with book/page reference
- Why it matters: Voice input is part of core vision; not critical for first prototype
- Source: execution
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: M001 has audio pipeline; UI + integration deferred

## Out of Scope

### R011 — Backup/sync functionality
- Class: anti-feature
- Status: out-of-scope
- Description: Cloud backup, cross-device sync, export/import
- Why it matters: Explicitly excluded to maintain offline-first, privacy-focused design
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: Data is user's responsibility; may add local export in future

### R012 — iOS support
- Class: constraint
- Status: out-of-scope
- Description: Build and deploy on iOS devices
- Why it matters: Focus on Android first; Dioxus enables future iOS port
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: M002 targets Android only

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| R001 | core-capability | validated | M002/S02 | M002/S05 | S02: 4 integration tests pass (end-to-end, multiple pages, duplicates, storage) |
| R002 | core-capability | validated | M002/S03 | M002/S05 | S03: 5 integration tests pass (markdown rendering, word tap, duplicate handling, progress save, position restore) |
| R003 | primary-user-loop | validated | M002/S04 | M002/S03 | S03: 3 integration tests pass (word save, duplicate detection, sentence extraction) |
| R004 | operability | active | M002/S01 | M002/S05 | partial: APK built, device testing pending |
| R005 | continuity | validated | M002/S01 | M002/S02, M002/S03, M002/S04 | S02: 6 tests verify book/page persistence; S03: 5 tests verify word/progress persistence |
| R006 | integration | active | M002/S05 | M002/S02, M002/S03 | unmapped |
| R007 | operability | validated | M002/S01 | none | S01: patch script + APK build success |
| R008 | differentiator | deferred | none | none | unmapped |
| R009 | differentiator | deferred | none | none | unmapped |
| R010 | core-capability | deferred | none | none | unmapped |
| R011 | anti-feature | out-of-scope | none | none | n/a |
| R012 | constraint | out-of-scope | none | none | n/a |

## Coverage Summary

- Active requirements: 7
- Mapped to slices: 7
- Validated: 5 (R001, R002, R003, R005, R007)
- Partial validation: 1 (R004 - pending device testing)
- Unmapped active requirements: 0
- Deferred: 3
- Out of scope: 2
