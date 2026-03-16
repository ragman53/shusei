# M002-dbrk2n: Android Prototype

**Vision:** Deploy working Android prototype on Moto G66j 5G — users can capture book pages with OCR, read PDFs with progress tracking, and save vocabulary with example sentences.

## Success Criteria

- APK installs and launches on Moto G66j 5G without crashes
- User can create book → capture 2+ pages → OCR extracts text → pages saved with book linkage
- User can import PDF → convert to markdown → scroll with font control + progress tracking
- User can tap 3+ words → save with example sentences → data persists after app restart
- NDLOCR + Moonshine models bundled and load successfully

## Key Risks / Unknowns

- **Android Gradle build compatibility** — Dioxus 0.7.3 generates obsolete Java 8 config; patch script required
- **Model bundling** — Confirm model files exist and load correctly on device
- **JNI camera stability** — Mid-range device may have memory constraints during capture + OCR
- **Performance** — Inference speed unknown on Moto G66j 5G hardware

## Proof Strategy

- **Gradle build compatibility** → retire in S01 by successfully building debug APK with patch script
- **Model bundling** → retire in S05 by loading models on device and running inference
- **Camera stability** → retire in S02 by capturing 5+ pages without crashes
- **Performance** → retire in S05 by measuring OCR latency on device (<5s acceptable for prototype)

## Verification Classes

- Contract verification: Unit tests for new UI components, integration tests for camera → OCR → save flow
- Integration verification: Real device testing on Moto G66j 5G; verify JNI, model loading, SQLite persistence
- Operational verification: App survives background/restore; data persists across restarts
- UAT / human verification: User can complete camera capture flow, PDF reading flow, word collection flow

## Milestone Definition of Done

This milestone is complete only when all are true:

- All 5 slices are complete with passing verification
- Shared components (database, OCR, models) are wired together and functional
- Android APK builds successfully with patch script
- APK installs and launches on Moto G66j 5G
- Success criteria re-checked against live device behavior
- Final integrated acceptance scenarios pass (camera, PDF, word collection all work end-to-end)

## Requirement Coverage

- Covers: R001, R002, R003, R004, R005, R006, R007
- Partially covers: none
- Leaves for later: R008 (Qwen), R009 (Dictionary), R010 (Voice memo)
- Orphan risks: none

## Slices

- [x] **S01: Android Build + Deploy** `risk:high` `depends:[]`
  > After this: Debug APK builds with Gradle patch script, installs on Moto G66j 5G, app launches without crashes, SQLite persists data

- [x] **S02: Camera Book Capture** `risk:high` `depends:[S01]`
  > After this: User can create book (title/author) → capture pages via camera → OCR runs → pages saved with book linkage + page number

- [x] **S03: PDF Reflow Reader** `risk:medium` `depends:[S01]`
  > After this: User can import PDF → convert to markdown → read with continuous scroll, font control (12-32px), progress tracking, last-read position sync

- [ ] **S04: Word Collection** `risk:medium` `depends:[S03]`
  > After this: User can tap word in PDF/OCR text → save word + full example sentence → definition shows "coming soon" placeholder → data persists

- [ ] **S05: Model Bundling + Integration** `risk:low` `depends:[S02, S03, S04]`
  > After this: NDLOCR + Moonshine models bundled in APK, end-to-end flows (camera→OCR, PDF→read, word→save) work smoothly on device

## Boundary Map

### S01 → S02

Produces:
- Working debug APK with Gradle patch script
- SQLite database accessible on Android
- JNI platform API functional (camera, file picker)

Consumes:
- nothing (first slice)

### S01 → S03

Produces:
- Working debug APK with Gradle patch script
- SQLite database accessible on Android
- File picker JNI for PDF import

Consumes:
- nothing (first slice)

### S02 → S05

Produces:
- Camera capture flow with book linkage
- OCR integration with NDLOCR
- Pages saved to book_pages table

Consumes:
- S01: SQLite persistence, JNI camera API

### S03 → S04

Produces:
- PDF reflow reader with progress tracking
- Word tap detection in rendered text
- Last-read position storage

Consumes:
- S01: SQLite persistence, file picker

### S03 → S05

Produces:
- PDF conversion pipeline (NDLOCR)
- Continuous scroll view with font control

Consumes:
- S01: SQLite persistence

### S04 → S05

Produces:
- Word + example sentence save flow
- Vocabulary list UI

Consumes:
- S03: Word tap detection in PDF
- S01: SQLite words table

### S05 → Final Integration

Produces:
- Bundled models (NDLOCR, Moonshine) in APK assets
- End-to-end verification on Moto G66j 5G
- Performance measurements (OCR latency, memory usage)

Consumes:
- S02: Camera → OCR flow
- S03: PDF → reflow flow
- S04: Word → save flow
