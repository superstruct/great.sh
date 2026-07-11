# 0050 — Statusline width guard counts chars, not display columns

| Field | Value |
|---|---|
| Priority | P3 |
| Type | bug |
| Module | `src/cli/statusline.rs` |
| Status | backlog |
| Estimated Complexity | S |

## Problem

`visible_len` and `truncate_to_width` count Unicode scalar values, not display
columns. Double-width glyphs (⚡ U+26A1, powerline glyphs, CJK) are counted as
one column, so the overflow guard can undercount and let the rendered line wrap
in narrow terminals.

## Proposed Fix

Use the `unicode-width` crate (`UnicodeWidthChar::width`) in both helpers.
Zero-width joiners and combining marks should count as 0.

## Acceptance Criteria

- A line of N double-width glyphs measures 2N columns
- Truncation cuts at column budget, not char count
- Existing ASCII tests unchanged
