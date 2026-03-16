# T04: Library Navigation

**Slice:** S02 — Camera Book Capture
**Milestone:** M002-dbrk2n

## Description

Add "Capture Pages" button to book list items in `LibraryScreen` so users can navigate from their book library directly to camera capture with the correct `book_id`. Also display page count badge on each book card.

## Steps

1. Read `src/ui/library.rs` to understand current `LibraryScreen` structure
2. Add "Capture Pages" button to each book card:
   - Button text: "📷 Capture Pages" or "➕ Add Pages"
   - OnClick: navigate to `/camera/:book_id` using `use_navigator()`
   - Style: secondary button (green background to match camera theme)
3. Add page count display to book card:
   - Query database for page count: `SELECT COUNT(*) FROM book_pages WHERE book_id = ?`
   - Display badge: "X pages" or "No pages yet"
   - Update badge after successful page save (via app state or re-fetch)
4. Handle empty book list state:
   - If no books exist, show "Create a book first" message with link to add book
5. Add "View Pages" button for books with existing pages:
   - Navigate to book detail view (future feature) or show page list
6. Ensure navigation flow is smooth:
   - Book List → Capture Pages → Camera → Save → Back to Book List
   - Page count should update after return (may require state refresh)

## Must-Haves

- [ ] "Capture Pages" button visible on each book card
- [ ] Button navigates to `/camera/:book_id` with correct book_id
- [ ] Page count badge shows current number of pages
- [ ] Empty book list handled gracefully
- [ ] Navigation flow works end-to-end (list → camera → save → list)

## Verification

- Desktop test: `dx serve`, create book, verify "Capture Pages" button appears
- Navigation test: Click button → verify URL is `/camera/{book_id}`
- Page count test: Save a page → return to list → verify count updated
- Code inspection: `LibraryScreen` uses `use_navigator()` for navigation

## Observability Impact

- **Signals added/changed:**
  - `log::debug!("Navigating to camera for book_id={}", book_id)` — Navigation event
  - `log::debug!("Book card rendered: {} pages", page_count)` — Card render logging
- **How a future agent inspects this:**
  - Check navigation history in Dioxus devtools
  - Monitor logcat for navigation events
- **Failure state exposed:**
  - Button disabled if book_id is missing
  - Error toast if navigation fails (unlikely)

## Inputs

- T01: Book creation flow (creates books to display in list)
- T02/T03: Camera page with book_id parameter (navigation target)
- `src/core/db.rs` — Database query for page count
- `src/ui/library.rs` — Current library screen implementation

## Expected Output

- `src/ui/library.rs` — Modified with "Capture Pages" button and page count badge
- Working navigation flow from book list to camera capture
- Page count visible on each book card
