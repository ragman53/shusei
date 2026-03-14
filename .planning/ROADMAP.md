# Roadmap: 読書アプリ (Reading App)

**Core Value:** 紙の本と PDF をシームレスに統合し、読書中の思考を逃さず記録できる完全オフライン環境。

**Granularity:** Standard | **Phases:** 7 | **Requirements:** 30/30 mapped ✓

---

## Phases

- [ ] **Phase 1: Core Infrastructure** - Android app shell with book library, SQLite, and file storage
- [ ] **Phase 2: Paper Book Capture** - Camera → OCR workflow for physical books
- [ ] **Phase 3: PDF Support** - PDF import, OCR conversion, and reflow reading
- [ ] **Phase 4: Annotation Foundation** - Sticky notes, bookmarks, and word collection
- [ ] **Phase 5: Voice Memos** - Real-time voice-to-text with Moonshine
- [ ] **Phase 6: AI Enhancement** - AI-generated word definitions with on-device LLM
- [ ] **Phase 7: Performance Polish** - Low-RAM optimization and final refinements

---

## Phase Details

### Phase 1: Core Infrastructure
**Goal:** Working Android app with book library, SQLite database, and file storage system

**Depends on:** Nothing (first phase)

**Requirements:** CORE-01, CORE-02, CORE-03, CORE-04, CORE-05

**Success Criteria** (what must be TRUE):
1. User can launch the app and see a book library screen
2. User can add a book with title, author, and optional cover photo
3. Book metadata persists after app restart
4. App handles background/foreground transitions without crashes
5. No memory leaks from JNI usage during normal operation

**Research Flags:** None - standard Dioxus + SQLite patterns well-documented

**Plans:** 4 plans

Plans:
- [x] 01-01-PLAN.md — Database foundation (models + schema + CRUD)
- [x] 01-02-PLAN.md — File storage for cover photos
- [x] 01-03-PLAN.md — Library UI (list + add book form)
- [x] 01-04-PLAN.md — Android lifecycle + state persistence

---

### Phase 2: Paper Book Capture
**Goal:** Complete camera → OCR workflow for digitizing physical book pages

**Depends on:** Phase 1

**Requirements:** PAPER-01, PAPER-02, PAPER-03, PAPER-04, PAPER-05

**Success Criteria** (what must be TRUE):
1. User can open camera and capture a book page
2. Captured image is automatically downscaled to 2MP or less
3. User can run OCR and see extracted text within 5 seconds (on mid-range device)
4. OCR text is saved and linked to the page image
5. User can view both the original image and extracted text together

**Research Flags:** 
- NDLOCR-Lite Rust integration with ort needs validation
- Complex ONNX model loading on Android requires testing

**Plans:** TBD

---

### Phase 3: PDF Support
**Goal:** Import PDFs, convert to readable text via OCR, and display with reflow

**Depends on:** Phase 1, Phase 2 (shares OCR service)

**Requirements:** PDF-01, PDF-02, PDF-03, PDF-04

**Success Criteria** (what must be TRUE):
1. User can select a PDF file from device storage
2. PDF pages are processed via OCR with visible progress indicator
3. Converted content displays in reflow mode with adjustable font size
4. Large PDFs (100+ pages) process without crashing low-RAM devices
5. User can navigate between pages smoothly

**Research Flags:** None - uses same OCR infrastructure as Phase 2

**Plans:** TBD

---

### Phase 03.4: PDF Database Integration for Human Verification (INSERTED)

**Goal:** Fix PDF import flow by adding pdf_path column to books table, enabling end-to-end PDF import → conversion → reading workflow
**Requirements**: CORE-02, CORE-03, PDF-01
**Depends on:** Phase 3, Phase 03.3

**Success Criteria** (what must be TRUE):
1. User can import a PDF and it appears in the library
2. User can tap the PDF book and see the reader view
3. Reader can find the PDF file for conversion
4. PDF conversion completes successfully
5. User can read the converted pages

**Research Flags:** None - straightforward schema migration

**Plans:** 1 plan

Plans:
- [ ] 03.4-01-PLAN.md — Add pdf_path column, wire through import and reader flows

### Phase 03.3: load pdf function for human verification on Android real devices (INSERTED)

**Goal:** Verify PDF loading functionality works correctly on real Android devices (not emulator)
**Requirements:** Verification phase (no new requirements)
**Depends on:** Phase 3, Phase 03.2

**Success Criteria** (what must be TRUE):
1. App launches and shows Library screen on real Android device
2. Load Demo PDF button is visible and tappable
3. PDF loads without crashes on real Android device
4. OCR processing completes without OOM
5. UI remains responsive during processing

**Research Flags:** None - verification of existing functionality

**Plans:** 1/1 plans complete

Plans:
- [ ] 03.3-01-PLAN.md — Human verification of PDF loading on Android device

### Phase 4: Annotation Foundation
**Goal:** Rich annotation system for captured content including notes, bookmarks, and word collection

**Depends on:** Phase 1, Phase 2, Phase 3

**Requirements:** ANNO-01, ANNO-02, ANNO-03, ANNO-04

**Success Criteria** (what must be TRUE):
1. User can add, edit, and delete sticky notes on any page
2. User can bookmark pages and see a list of all bookmarks
3. User can tap any word to collect it with surrounding context sentence
4. Collected words show count of how many times each word was collected

**Research Flags:** None - standard CRUD operations

**Plans:** TBD

---

### Phase 5: Voice Memos
**Goal:** Real-time voice-to-text transcription using Moonshine for hands-free note capture

**Depends on:** Phase 1, Phase 4 (links to annotations)

**Requirements:** VOICE-01, VOICE-02, VOICE-03, VOICE-04

**Success Criteria** (what must be TRUE):
1. User can start voice recording and see real-time transcription
2. Voice memos are saved and linked to the current page
3. User can view list of all voice memos and play them back
4. App automatically selects appropriate Moonshine model based on device capability

**Research Flags:**
- Moonshine Rust bindings don't exist yet - need to create JNI wrapper around C++ core
- Audio pipeline requires Android-specific implementation

**Plans:** TBD

---

### Phase 6: AI Enhancement
**Goal:** AI-generated word definitions using on-device Qwen3.5 LLM

**Depends on:** Phase 1, Phase 4 (word collection), Phase 5 (memory management patterns)

**Requirements:** AI-01, AI-02, AI-03, AI-04

**Success Criteria** (what must be TRUE):
1. User can generate AI definition for any collected word (offline)
2. Generated definitions are saved and persist with the word
3. When selecting a previously collected word, user sees how many times it was collected
4. Only one AI model loads at a time (never simultaneously with OCR or Voice)

**Research Flags:**
- Qwen3.5-08B quantization and on-device performance needs testing
- Memory pressure handling for 2GB RAM devices requires validation

**Plans:** TBD

---

### Phase 7: Performance Polish
**Goal:** Optimized performance for low-RAM Android devices and final refinements

**Depends on:** Phase 1-6

**Requirements:** PERF-01, PERF-02, PERF-03

**Success Criteria** (what must be TRUE):
1. App launches and operates smoothly on Android Go devices (2GB RAM)
2. Image processing (OCR) runs in background without freezing the UI
3. Voice recording handles audio streaming without buffer overflow or dropouts
4. All features work reliably without crashes during extended use

**Research Flags:** None - performance testing and optimization

**Plans:** TBD

---

## Coverage Validation

| Category | Requirements | Phase | Status |
|----------|---------------|-------|--------|
| Core Infrastructure | 5 | 1 | Pending |
| Paper Book Capture | 5 | 2 | Pending |
| PDF Support | 4 | 3 | Pending |
| Annotation | 4 | 4 | Pending |
| Voice Memos | 4 | 5 | Pending |
| AI Enhancement | 4 | 6 | Pending |
| Performance | 3 | 7 | Pending |
| **Total** | **30** | **7 phases** | **Pending** |

✓ All 30 v1 requirements mapped to exactly one phase
✓ No orphaned requirements
✓ No duplicate mappings

---

## Dependencies Graph

```
Phase 1 (Core)
    │
    ├──→ Phase 2 (Paper Capture)
    │       │
    │       └──→ Phase 4 (Annotation) ──→ Phase 5 (Voice) ──→ Phase 6 (AI)
    │
    └──→ Phase 3 (PDF)
            │
            └──→ Phase 4 (Annotation)

Phase 7 (Polish) depends on all previous phases
```

**Critical Path:** 1 → 2 → 4 → 5 → 6 → 7 (with 3 as parallel branch)

---

## Risk Mitigation

| Risk | Phase | Mitigation |
|------|-------|------------|
| Memory OOM loading multiple AI models | 6 | Sequential loading only, INT8 quantization |
| OCR memory spikes on large images | 2 | Pre-downscale to 2MP max before processing |
| JNI reference leaks | 1 | Establish patterns early with explicit cleanup |
| Audio buffer overflow | 5 | Adaptive buffering, chunk-based processing |
| Database bloat | 1 | Store images in filesystem, paths in SQLite |

---

## Progress Tracking

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core Infrastructure | 0/TBD | Not started | - |
| 2. Paper Book Capture | 0/TBD | Not started | - |
| 3. PDF Support | 0/TBD | Not started | - |
| 4. Annotation Foundation | 0/TBD | Not started | - |
| 5. Voice Memos | 0/TBD | Not started | - |
| 6. AI Enhancement | 0/TBD | Not started | - |
| 7. Performance Polish | 0/TBD | Not started | - |

### Phase 8: 4

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 7
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 8 to break down)

---

*Roadmap created: 2026-03-11*
*Granularity: Standard | Mode: yolo*
