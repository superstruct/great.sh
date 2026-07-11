# 0056 — Statusline ANSI parser only terminates escapes on 'm'

| Field | Value |
|---|---|
| Priority | P3 |
| Type | bug |
| Module | `src/cli/statusline.rs` |
| Status | backlog |
| Estimated Complexity | S |

## Problem

`visible_len` and `truncate_to_width` end an ANSI escape sequence only when
they see `m` (SGR terminator). A non-SGR escape (e.g. cursor movement ending
in `A`–`H`) would be mis-parsed: its bytes after the introducer count toward
visible width. Pre-existing behavior, surfaced during the 0050 review; no
practical impact today because the statusline only emits SGR color codes.

## Acceptance Criteria

- Non-SGR CSI sequences (final byte in `@`–`~`) are treated as zero-width
- Existing SGR handling and truncation reset behavior unchanged
- Regression test with a cursor-movement escape mixed into colored text
