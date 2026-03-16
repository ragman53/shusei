---
id: T03
slice: S04
milestone: M002-dbrk2n
title: Wire Export Functionality
estimated_steps: 3
estimated_files: 2
---

# T03: Wire Export Functionality

**Slice:** S04 — Word Collection
**Milestone:** M002-dbrk2n

## Description

Wire up the export buttons (Markdown/CSV) to the existing export_vocabulary() functions in src/core/vocab.rs. The export logic is already implemented - this task connects the UI buttons to generate and download/export the vocabulary list.

## Steps

1. Import `export_vocabulary()` and `ExportFormat` from `src/core/vocab.rs`
2. Wire Markdown button to call `export_vocabulary(&words, ExportFormat::Markdown)`
3. Wire CSV button to call `export_vocabulary(&words, ExportFormat::Csv)`
4. Add JSON export button (optional, already implemented in export_json())
5. Show toast notification on export success ("Exported X words as Markdown/CSV")
6. For Android: Use file picker JNI to save exported file (or copy to clipboard as fallback)

## Must-Haves

- [ ] Markdown button generates markdown-formatted vocabulary list
- [ ] CSV button generates CSV-formatted vocabulary list
- [ ] Export uses actual word data from current list (respects search filter)
- [ ] Success toast shows number of words exported
- [ ] Export handles empty list gracefully (show info toast: "No words to export")

## Verification

- `cargo test --lib vocab::test_export_functions` — Test passes
- Manual test: Click Markdown export → generates markdown output → toast shown
- Manual test: Click CSV export → generates CSV output → toast shown
- Verify export format matches existing export_markdown()/export_csv() output

## Observability Impact

- Signals added/changed: Export event logged with format and word count
- How a future agent inspects this: Runtime logs show "Exported {count} words as {format}"
- Failure state exposed: Error toast if export fails (e.g., serialization error)

## Inputs

- `src/ui/vocab.rs` — Current export buttons with TODO handlers
- `src/core/vocab.rs` — export_vocabulary(), export_markdown(), export_csv(), export_json() functions

## Expected Output

- `src/ui/vocab.rs` — Export buttons wired to export functions with toast feedback
- `src/core/vocab.rs` — No changes needed (export functions already exist)
