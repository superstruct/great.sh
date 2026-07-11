# Spec 0050 — Statusline width guard counts display columns

## Summary

`visible_len` (src/cli/statusline.rs:505) and `truncate_to_width`
(src/cli/statusline.rs:526) count Unicode scalar values, not display columns.
Double-width glyphs (⚡ U+26A1 in some terminals, CJK, powerline glyphs) count
as one column, so the overflow guard undercounts and the rendered line can wrap
in narrow terminals. Count display columns instead.

## Interfaces (unchanged signatures)

- `fn visible_len(s: &str) -> usize` — returns display columns, ANSI escapes
  stripped
- `fn truncate_to_width(s: &str, max_visible: usize) -> String` — truncates at
  the column budget, preserving ANSI escapes and appending `\x1b[0m` when a
  colored string is truncated

Call sites (no changes needed — semantics only): statusline.rs:909–910 (line
overflow guard) and :1002–1005 (agents-segment budget).

## Implementation

1. Add `unicode-width = "0.2"` to `[dependencies]` in Cargo.toml (already in
   Cargo.lock at 0.2.2 as a transitive dep — no new supply chain).
2. In both helpers, replace the per-char `+= 1` with
   `UnicodeWidthChar::width(c).unwrap_or(0)`. `None` (control chars) and
   `Some(0)` (combining marks, ZWJ) count 0 columns.
3. In `truncate_to_width`, a char must be dropped when
   `visible + char_width > max_visible` — check BEFORE pushing, so a
   double-width glyph never straddles the budget boundary. Keep the existing
   escape-sequence and reset-append behavior byte-for-byte.

## Edge cases

- Empty string → 0 columns (existing behavior)
- String of ANSI escapes only → 0 columns (existing behavior)
- Double-width char exactly at the boundary: budget 5, input 3×U+3042 (あ, 2
  cols each) → keep 2 chars (4 cols), drop the third
- Combining mark (e.g. U+0301) and ZWJ (U+200D) → 0 columns
- Control chars other than ESC → 0 columns (width() returns None)

## Testing strategy

- New: N double-width glyphs measure 2N columns (`visible_len`)
- New: truncation cuts at the column budget, not char count (boundary case
  above), and still appends the reset when escapes were seen
- New: combining marks count 0
- Existing ASCII/ANSI tests must pass unchanged

## Acceptance criteria (from task 0050)

- A line of N double-width glyphs measures 2N columns
- Truncation cuts at column budget, not char count
- Existing ASCII tests unchanged

## Security considerations

None — pure formatting change, no input crosses a trust boundary. New direct
dependency is already in the lockfile (audited transitively).

Self-review: alternatives (hand-rolled width table) rejected — unicode-width is
the ecosystem standard and already in the dependency tree; no platform
variance (terminal-side rendering differs, but column accounting per UAX #11
is the correct portable approximation); success criteria are the three
testable acceptance criteria above; no open questions.
