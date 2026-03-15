# T01: 01-core-infrastructure 01

**Slice:** S01 — **Milestone:** M001

## Description

Create database foundation with Book model and books table schema

Purpose: Establish the data layer that all book operations depend on
Output: Working database schema with tested CRUD operations

## Must-Haves

- [ ] "Book metadata can be saved to database"
- [ ] "Book data persists after app restart"
- [ ] "No memory leaks from database operations"

## Files

- `src/core/models.rs`
- `src/core/db.rs`
