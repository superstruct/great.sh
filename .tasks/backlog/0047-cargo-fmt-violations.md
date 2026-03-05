# 0047 — `cargo fmt` violations in detection.rs and cli_smoke.rs

| Field | Value |
|---|---|
| Priority | P3 |
| Type | chore |
| Module | `src/platform/detection.rs`, `tests/cli_smoke.rs` |
| Status | backlog |
| Estimated Complexity | XS |

## Problem

`cargo fmt --check` fails with formatting diffs in two files:

1. **`src/platform/detection.rs`** (lines ~684, ~738, ~757, ~814) — method chain formatting differences in test code
2. **`tests/cli_smoke.rs`** (lines ~516, ~540) — long assertion line formatting

## Fix

Run `cargo fmt` to auto-fix all violations.

## Evidence

Docker test run 2026-03-04. `cargo fmt --check` exits 1 with 6 formatting diffs.
