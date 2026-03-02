# 0017: Bump Rust Version in Windows Cross-Compilation Dockerfile

**Priority:** P2
**Type:** bugfix
**Module:** `docker/cross-windows.Dockerfile`
**Status:** done (iteration 010, commit a117187)

## What Was Done

Bumped `FROM rust:1.83-slim` to `FROM rust:1.85-slim` on line 6. Updated usage comments to document `docker compose` workflow.

Root cause: `time-core 0.1.8` declares `edition = "2024"`, which requires Cargo >= 1.85.0. The `edition2024` feature was stabilized in Rust 1.85.0 (2025-02-20).

## Acceptance Criteria

- [x] `FROM` line pins to Rust >= 1.85
- [x] No floating `latest` or `stable` tags
- [x] `cargo clippy` and `cargo test` pass unchanged (no Rust source changes)
