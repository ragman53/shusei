# Feature Landscape: Offline Reading App

**Domain:** Reading/Annotation Apps (読書アプリ - Reading App)  
**Researched:** 2026-03-11  
**Focus:** Physical books + PDFs with sticky notes, voice memos, word collection

## Overview

This research categorizes features for an **offline-first reading app** that uniquely combines physical book workflow (OCR-based) with PDF reading, featuring sticky notes (付箋), voice memos (ボイスメモ), and word collection (単語採集). The app targets Japanese/English readers who value privacy and want to capture reading thoughts without cloud dependencies.

---

## Table Stakes (Must-Have)

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Book Library Management** | Users need to organize their reading materials | Medium | Shelf/list view, manual metadata entry (title, author), cover photo |
| **Page Navigation** | Core reading functionality | Low | Page numbers, progress indicator, jump to page |
| **Text Display/Reflow** | Reading experience foundation | Medium | PDF: reflow display; Physical: OCR text display |
| **Bookmarking** | Save reading position | Low | Essential for both PDF and physical book workflows |
| **Basic Annotation** | Note-taking is core promise | Medium | Text selection → add note (sticky note) |
| **Local Data Storage** | Offline requirement | Low | SQLite for structured data, file system for images/audio |
| **Dark/Light Mode** | Eye strain reduction | Low | Standard reading app expectation |
| **Font Size Adjustment** | Accessibility | Low | Text reflow capability required |
| **Search Within Book** | Find information quickly | Medium | Full-text search on OCR/PDF text |

### Physical Book Specific Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Camera Capture** | Physical book digitization | Medium | Page photo → OCR workflow |
| **OCR Text Extraction** | Convert images to readable text | High | NDLOCR-Lite integration, Japanese/English support |
| **Photo + Text Pairing** | Verify OCR accuracy | Low | Save both original photo and extracted text |
| **Manual Page Number Entry** | Physical books lack digital metadata | Low | User inputs current page |

### PDF Specific Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **File Import** | Get PDFs into app | Low | File picker, import from storage |
| **PDF Rendering** | Display PDF content | Medium | Convert to Markdown via NDLOCR, reflow view |
| **Page Thumbnails** | Quick navigation | Medium | Generate from PDF or captured photos |

---

## Differentiators (Competitive Advantage)

Features that set this product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Voice Memo Integration** | Capture thoughts hands-free while reading | High | Moonshine Voice ASR, real-time speech-to-text |
| **AI-Generated Word Definitions** | Learn vocabulary in context | High | On-device Qwen3.5-08B, 100% offline |
| **Word Collection System** | Build personal dictionary | Medium | Word + context sentence + page + AI definition |
| **Collection Frequency Visualization** | See learning patterns | Low | Show how many times a word appears across books |
| **Unified Physical + Digital** | Seamless experience across book types | High | Same annotation tools for both workflows |
| **100% Offline Operation** | Privacy, no network dependency | Medium | All AI/ML models on-device |
| **Low RAM Optimization** | Works on budget Android devices | High | Memory-conscious design for Android Go |
| **Cross-Reference Word Usage** | See words in different contexts | Medium | Link word occurrences across books |
| **Export Annotations** | Data portability | Medium | Markdown/text export of notes and words |
| **Reading Session Timer** | Track reading habits | Low | Simple timer with session logging |

### Key Differentiator Analysis

**Voice Memos (ボイスメモ)**
- **Why it matters:** Readers often have thoughts while hands are occupied holding a book
- **Implementation:** Moonshine Voice offers streaming ASR optimized for live speech
- **Unique value:** Real-time transcription while reading, not post-hoc recording

**Word Collection with AI (単語採集)**
- **Why it matters:** Language learners need context-aware definitions
- **Implementation:** Qwen3.5-08B generates definitions based on sentence context
- **Unique value:** AI understands nuance that dictionary lookups miss

**Physical Book Bridge**
- **Why it matters:** Most apps treat physical and digital as separate worlds
- **Implementation:** OCR + photo workflow makes paper books "digital-native"
- **Unique value:** Single app for entire reading life

---

## Anti-Features (Deliberately NOT Building)

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Cloud Sync** | Privacy promise, complexity | Local-only storage with optional manual export/import |
| **Social Features** | Scope creep, privacy concerns | Focus on personal knowledge management |
| **E-book Store Integration** | Licensing complexity, scope | User brings their own PDFs/books |
| **Automatic Cloud Backup** | Network dependency | Document manual backup process, user responsibility |
| **DRM Support** | Complexity, legal issues | Support DRM-free PDFs only |
| **Reading Statistics/Social** | Feature bloat | Simple personal reading log only |
| **Built-in Dictionary (licensed)** | Licensing costs | AI-generated definitions instead |
| **OCR Auto-Upload to Cloud** | Privacy violation | 100% on-device OCR processing |
| **Subscription Model** | Not aligned with offline ethos | One-time purchase or free/open source |
| **Multi-Device Sync** | Complexity, cloud dependency | Single-device focus, manual transfer if needed |

---

## Feature Dependencies

```
Core Infrastructure
├── SQLite Database (books, annotations, words)
├── File Storage (photos, audio, PDFs)
└── OCR Engine (NDLOCR-Lite)

Physical Book Workflow
├── Camera Access
├── Photo Storage
├── OCR Processing
└── Text Display
    ├── Sticky Notes
    ├── Voice Memos
    └── Word Collection

PDF Workflow
├── File Picker
├── PDF Import
├── Markdown Conversion (NDLOCR)
└── Reflow Display
    ├── Sticky Notes
    ├── Voice Memos
    └── Word Collection

Word Collection Feature
├── Text Selection
├── Word Extraction
├── Context Sentence Capture
├── AI Definition (Qwen3.5-08B)
├── Storage in Database
└── Frequency Tracking

Voice Memo Feature
├── Microphone Access
├── Audio Recording
├── ASR Processing (Moonshine)
└── Text Association with Page/Selection
```

---

## MVP Recommendation

**Prioritize for MVP:**

1. **Book Library Management** - Foundation, low complexity
2. **Physical Book Workflow** - Core differentiator
   - Camera capture → OCR → Text display
3. **Basic Sticky Notes** - Essential annotation
4. **PDF Import + Display** - Table stakes for digital
5. **Word Collection (basic)** - Key differentiator
   - Word + context + manual definition
   - AI definitions as Phase 2

**Defer:**
- **Voice Memos:** High complexity, can ship without initially
- **AI Definitions:** Requires ML model integration, can use manual input first
- **Export Features:** Important but not MVP-blocking
- **Advanced Search:** Basic text search sufficient initially

**Rationale:** The core value proposition is "unified reading notes for physical + PDF books." Get the basic workflows working first, then add the AI-enhanced features that differentiate from simple note-taking apps.

---

## Phase Recommendations

Based on feature dependencies and complexity:

### Phase 1: Core Reading Infrastructure
- Book library management
- Physical book capture + OCR
- PDF import + display
- Basic page navigation
- Local storage (SQLite)

### Phase 2: Annotation Foundation
- Sticky notes on text
- Basic word collection (manual definitions)
- Bookmarking
- Reading progress tracking

### Phase 3: AI Enhancement
- AI-generated word definitions (Qwen3.5)
- Voice memos (Moonshine integration)
- Word frequency visualization
- Advanced search

### Phase 4: Polish & Export
- Export annotations
- Reading statistics
- Performance optimizations
- UI refinements

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Table Stakes | **HIGH** | Standard reading app features well-understood |
| Physical Book OCR | **HIGH** | NDLOCR-Lite documented, proven technology |
| Voice Memos | **MEDIUM** | Moonshine Voice confirmed working on Android, integration complexity TBD |
| AI Definitions | **MEDIUM** | Qwen3.5-08B suitable for on-device, but Rust integration needs verification |
| Offline-First | **HIGH** | Clear scope, no external API dependencies |
| Low RAM Optimization | **MEDIUM** | Requires careful memory management, test on real devices |

---

## Sources

- [PROJECT.md](../PROJECT.md) - Project requirements and constraints
- [NDLOCR-Lite GitHub](https://github.com/ndl-lab/ndlocr-lite) - OCR technology details
- [Moonshine Voice GitHub](https://github.com/moonshine-ai/moonshine) - ASR capabilities
- Context7 Documentation for Dioxus and ONNX Runtime

