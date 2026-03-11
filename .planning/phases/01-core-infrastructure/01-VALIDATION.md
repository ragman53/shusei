---
phase: 01
slug: core-infrastructure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 01 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (test configuration in package section) |
| **Quick run command** | `cargo test --lib --no-fail-fast` |
| **Full suite command** | `cargo test --all-targets` |
| **Estimated runtime** | ~30-60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib --no-fail-fast`
- **After every plan wave:** Run `cargo test --all-targets`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds

---

## Per-task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | CORE-02 | unit | `cargo test --lib db::tests` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | CORE-02 | unit | `cargo test --lib models::tests` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | CORE-03 | integration | `cargo test --test file_storage` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | CORE-02 | unit | `cargo test --lib db::books::crud` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 2 | CORE-01 | component | `cargo test --lib ui::library::tests` | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 2 | CORE-01 | component | `cargo test --lib ui::add_book::tests` | ❌ W0 | ⬜ pending |
| 01-04-01 | 04 | 3 | CORE-04 | integration | `cargo test --test lifecycle` | ❌ W0 | ⬜ pending |
| 01-04-02 | 04 | 3 | CORE-05 | unit | `cargo test --lib platform::jni::tests` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/core/db.rs` — add `#[cfg(test)] mod tests` with book CRUD tests
- [ ] `src/core/models.rs` — add Book struct with serialization tests
- [ ] `tests/file_storage.rs` — integration test for cover photo storage
- [ ] `tests/lifecycle.rs` — Android lifecycle simulation tests
- [ ] `src/ui/library.rs` — component tests for library screen
- [ ] `src/ui/add_book.rs` — component tests for add book form

*Wave 0 will create test scaffolds for all CORE requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| App launches and shows library screen | CORE-01 | Requires Android device/emulator | 1. Build APK: `cargo ndk build --release`<br>2. Install on device<br>3. Launch app<br>4. Verify library screen visible |
| UI layout and styling | CORE-01 | Visual verification needed | 1. Run on Android emulator<br>2. Check list view layout<br>3. Verify "Add Book" button visible<br>4. Test empty state message |
| Background/foreground transition | CORE-04 | Requires actual Android lifecycle | 1. Launch app<br>2. Press home button (background)<br>3. Wait 5 seconds<br>4. Reopen app<br>5. Verify same screen restored |
| JNI memory leak detection | CORE-05 | Requires profiling tools | 1. Run app with Android Studio profiler<br>2. Perform 20+ add/book operations<br>3. Monitor native memory<br>4. Verify no continuous growth |
| Cover photo capture flow | CORE-03 | Requires camera hardware | 1. Click "Add Book"<br>2. Click "Add cover photo"<br>3. Take photo with camera<br>4. Verify preview shows<br>5. Complete book add |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending (after plan creation)

---

## Test Commands Reference

**Unit tests (fast, no Android):**
```bash
cargo test --lib --no-fail-fast
```

**Integration tests (requires Android emulator):**
```bash
cargo ndk test --release
```

**Single test module:**
```bash
cargo test --lib db::tests::test_book_crud
```

**With output:**
```bash
cargo test --lib -- --nocapture
```

**Coverage (optional):**
```bash
cargo tarpaulin --out Html
```
