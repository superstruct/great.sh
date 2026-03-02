# 0022: Align `great diff` Counter Buckets with Visual Markers

**Priority:** P2
**Type:** enhancement
**Module:** `src/cli/diff.rs`
**Status:** pending
**Estimated Complexity:** S
**Source:** Rams visual review, iteration 013

## Context

Missing MCP commands display a `+` (green, "install") marker but are tallied in `configure_count`. The summary shows "N to install, M to configure" where "configure" includes items visually marked as needing installation. CI consumers parsing the summary get counts that don't match the visible markers.

Additionally, duplicate secrets (appearing in both `secrets.required` and `find_secret_refs`) are counted twice in `secrets_count`.

## Requirements

1. Define clear classification rules: what counts as "install" vs "configure" vs "secrets".
2. Ensure each diff line's visual marker matches its summary bucket.
3. Deduplicate secrets that appear in both `secrets.required` and `find_secret_refs`.
4. Consider unifying section header style (some have action verbs, "Tools" does not).

## Acceptance Criteria

- [ ] Every `+` marker line increments `install_count`, every `~` increments `configure_count`
- [ ] Duplicate secrets counted only once
- [ ] Section headers use consistent style
- [ ] Integration tests updated

## Dependencies

None.
