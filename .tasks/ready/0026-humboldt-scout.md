# 0026: `great diff` Output Channel Redesign -- Humboldt Scout Report

**Scout:** Alexander von Humboldt
**Spec:** `.tasks/ready/0026-diff-output-channel-spec.md`
**Review:** `.tasks/ready/0026-socrates-review.md` (APPROVED, 3 advisory-only bookkeeping corrections)
**Date:** 2026-02-27

---

## 1. Module Registration Confirmed

`/home/isaac/src/sh.great/src/cli/mod.rs`

Both modules are registered and wired to the CLI dispatch. No changes needed here.

| Line | Declaration |
|------|-------------|
| 3 | `pub mod diff;` |
| 8 | `pub mod output;` |
| 68 | `Diff(diff::Args),` in `Command` enum |

---

## 2. `src/cli/output.rs` -- Full Map

**Path:** `/home/isaac/src/sh.great/src/cli/output.rs`
**Total lines:** 43

### Existing functions

| Lines | Function | Channel | Signature |
|-------|----------|---------|-----------|
| 4-6 | `success` | stderr (`eprintln!`) | `pub fn success(msg: &str)` |
| 9-11 | `warning` | stderr (`eprintln!`) | `pub fn warning(msg: &str)` |
| 14-16 | `error` | stderr (`eprintln!`) | `pub fn error(msg: &str)` |
| 18-21 | `info` | stderr (`eprintln!`) | `pub fn info(msg: &str)` |
| 24-26 | `header` | stderr (`eprintln!`) | `pub fn header(msg: &str)` |
| 32-42 | `spinner` | N/A (indicatif) | `pub fn spinner(msg: &str) -> indicatif::ProgressBar` |

**Import:** Line 1 -- `use colored::Colorize;` (already present, covers `.bold()`, `.green()`, `.yellow()`, `.blue()`)

### Insertion point

Insert three new functions **after line 26** (closing brace of `header()`), before `spinner()` at line 28.

The insertion block:

```rust
/// Print a bold header/section title to stdout.
///
/// Use this in pipeline-oriented commands (e.g., `diff`) where all output
/// is data and belongs on stdout. Interactive commands should use `header()`.
pub fn header_stdout(msg: &str) {
    println!("{}", msg.bold());
}

/// Print an informational message to stdout with a blue info prefix.
///
/// Stdout variant of `info()` for pipeline-oriented commands.
pub fn info_stdout(msg: &str) {
    println!("{} {}", "ℹ".blue(), msg);
}

/// Print a success message to stdout with a green checkmark prefix.
///
/// Stdout variant of `success()` for pipeline-oriented commands.
pub fn success_stdout(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}
```

Note on Unicode literals: the spec uses `\u{2139}` (ℹ) and `\u{2713}` (✓) as escapes.
The existing `info()` and `success()` use the literal characters directly. Either form
is valid Rust; use literal characters to match the surrounding style.

---

## 3. `src/cli/diff.rs` -- Complete Call Site Map

**Path:** `/home/isaac/src/sh.great/src/cli/diff.rs`
**Total lines:** 243

### All `output::*` calls -- disposition table

| Line | Current call | Action | Reason |
|------|-------------|--------|--------|
| 40 | `output::error("No great.toml found. Run \`great init\` to create one.");` | **KEEP** on stderr | Fatal error before `exit(1)`; Unix convention |
| 49 | `output::header("great diff");` | Change to `header_stdout` | Data header |
| 50 | `output::info(&format!("Comparing {} against system state", config_path.display()));` | Change to `info_stdout` | Data info line |
| 123 | `output::header("Tools");` | Change to `header_stdout` | Data section header |
| 171 | `output::header("MCP Servers");` | Change to `header_stdout` | Data section header (Socrates confirmed line 171, not 170 as spec stated) |
| 218 | `output::header("Secrets");` | Change to `header_stdout` | Data section header |
| 226 | `output::success("Environment matches configuration — nothing to do.");` | Change to `success_stdout` | Data summary |
| 239 | `output::info(&format!("{} — run \`great apply\` to reconcile.", summary));` | Change to `info_stdout` | Data summary |

**Total changes:** 7 call sites. 1 preserved.

### `println!` calls already on stdout -- no change needed

| Lines | Context |
|-------|---------|
| 54 | `println!()` -- blank line after info |
| 125 | `println!("{}", diff)` -- tool diff lines |
| 127 | `println!()` -- blank line |
| 173 | `println!("{}", diff)` -- MCP diff lines |
| 175 | `println!()` -- blank line |
| 219 | `println!("{}", diff)` -- secret diff lines |
| 222 | `println!()` -- blank line |

No `eprintln!` calls exist outside of `output::*` wrappers in `diff.rs`.

---

## 4. `tests/cli_smoke.rs` -- Complete Diff Test Map

**Path:** `/home/isaac/src/sh.great/tests/cli_smoke.rs`

All 12 diff tests (Socrates corrected the spec's count of 11). 8 tests change assertions, 4 are unchanged.

### Tests requiring assertion channel changes (8 tests)

| # | Function | Fn line | Assertion lines | Current | After |
|---|----------|---------|-----------------|---------|-------|
| 1 | `diff_satisfied_config_exits_zero` | 134 | 154 | `.stderr(predicate::str::contains("nothing to do"))` | `.stdout(...)` |
| 2 | `diff_missing_tool_shows_plus` | 158 | 178 | `.stderr(predicate::str::contains("great apply"))` | `.stdout(...)` |
| 3 | `diff_with_custom_config_path` | 231 | 251 | `.stderr(predicate::str::contains("custom.toml"))` | `.stdout(...)` |
| 4 | `diff_summary_shows_counts` | 255 | 277 | `.stderr(predicate::str::contains("1 to install"))` | `.stdout(...)` |
| 4 | `diff_summary_shows_counts` | 255 | 278 | `.stderr(predicate::str::contains("1 secrets to resolve"))` | `.stdout(...)` |
| 4 | `diff_summary_shows_counts` | 255 | 279 | `.stderr(predicate::str::contains("great apply"))` | `.stdout(...)` |
| 5 | `diff_mcp_missing_command_counted_as_install` | 307 | 326 | `.stderr(predicate::str::contains("1 to install"))` | `.stdout(...)` |
| 5 | `diff_mcp_missing_command_counted_as_install` | 307 | 327 | `.stderr(predicate::str::contains("to configure").not())` | `.stdout(...)` |
| 6 | `diff_mcp_missing_command_and_missing_tool_install_count` | 332 | 354 | `.stderr(predicate::str::contains("2 to install"))` | `.stdout(...)` |
| 7 | `diff_secret_dedup_required_and_ref` | 358 | 381 | `.stderr(predicate::str::contains("1 secrets to resolve"))` | `.stdout(...)` |
| 7 | `diff_secret_dedup_required_and_ref` | 358 | 382 | `.stderr(predicate::str::contains("2 secrets").not())` | `.stdout(...)` |
| 8 | `diff_secret_ref_only_no_required_section` | 386 | 406 | `.stderr(predicate::str::contains("1 secrets to resolve"))` | `.stdout(...)` |

### Tests requiring NO changes (4 tests)

| # | Function | Fn line | Reason |
|---|----------|---------|--------|
| 1 | `diff_no_config_exits_nonzero` | 123 | Line 130: `.stderr(predicate::str::contains("great.toml"))` -- error path stays on stderr |
| 2 | `diff_disabled_mcp_skipped` | 182 | Lines 202-203: both `.stdout(...not())` and `.stderr(...not())` -- negative assertions on both channels are correct and remain |
| 3 | `diff_version_mismatch_shows_tilde` | 207 | Lines 226-227: both already `.stdout(...)` |
| 4 | `diff_unresolved_secret_shows_red_minus` | 283 | Lines 302-303: both already `.stdout(...)` -- this test was missing from the spec (Socrates caught it) |

---

## 5. Patterns to Follow

### Function naming
- Suffix `_stdout` appended to existing function name: `header` -> `header_stdout`, `info` -> `info_stdout`, `success` -> `success_stdout`
- No channel parameter, no trait abstraction -- purely additive

### Doc comment style (from output.rs)
```
/// Print a [description] to [channel] with a [color] [symbol] prefix.
```
Add a second paragraph explaining when to use this variant vs the stderr counterpart.

### Formatting macros
- stderr variants use `eprintln!` -- stdout variants use `println!`
- Symbol and color are identical between paired functions:
  - `header`: `msg.bold()` (no prefix symbol)
  - `info`: `"ℹ".blue()` prefix
  - `success`: `"✓".green()` prefix

### Colored crate
- Already imported via `use colored::Colorize;` at line 1 of `output.rs`
- No new import needed

---

## 6. Dependency Map

```
output.rs  (add 3 functions)
    |
    v
diff.rs    (7 call site changes)
    |
    v
cli_smoke.rs  (12 assertion changes across 8 test functions)
```

No other files touch `output::header`, `output::info`, or `output::success` in a way
that this change affects. The new `_stdout` functions are only called from `diff.rs`.

Blast radius verified: `output::header`, `output::info`, `output::success` are called
from `doctor.rs`, `vault.rs`, `apply.rs`, `status.rs`, `init.rs`, `template.rs`,
`loop_cmd.rs` -- none of those callers are touched.

---

## 7. Recommended Build Order

1. `src/cli/output.rs` -- insert 3 new functions after line 26
2. `src/cli/diff.rs` -- replace 7 call sites (can be done in any order; compiler enforces correctness)
3. `tests/cli_smoke.rs` -- update 12 assertion lines across 8 test functions
4. `cargo clippy` -- expect zero new warnings (pure additive change, all functions are `pub`)
5. `cargo test` -- all 12 diff tests must pass

Steps 1 and 2 can be done in either order. Step 3 must follow step 2 (tests fail
against the old binary until assertions are aligned with new behavior).

---

## 8. Risks and Gotchas

### Socrates advisory corrections (implement correctly)
- Spec said MCP Servers header is line 170 -- it is line **171**. Match on the string, not the line number.
- Spec said 11 tests / 9 changed -- actual counts are 12 tests / 8 changed. `diff_unresolved_secret_shows_red_minus` (line 283) needs no changes; its assertions are already `.stdout(...)`.

### `diff_missing_tool_shows_plus` has a mixed assertion (line 177 + 178)
Line 177 (`.stdout(...)`) is already correct. Only line 178 (`.stderr(...)`) changes.
Da Vinci must change only the stderr assertion, not both.

### `diff_mcp_missing_command_counted_as_install` has a `.not()` on stderr (line 327)
`.stderr(predicate::str::contains("to configure").not())` -- this asserts "to configure" does NOT appear on stderr. After the change it becomes `.stdout(predicate::str::contains("to configure").not())`. The `.not()` must be preserved.

### `diff_secret_dedup_required_and_ref` has a `.not()` on stderr (line 382)
`.stderr(predicate::str::contains("2 secrets").not())` -- same pattern. Preserve the `.not()` when migrating to `.stdout(...)`.

### No dead_code warnings expected
All three new functions are `pub`, so they will not trigger `dead_code` lint even before
being called from diff.rs (the `pub` visibility suppresses the lint).

### Unicode literals
The existing functions use literal `✓` and `ℹ` characters. Use the same literal form
in the new functions for consistency. The spec's `\u{2139}` and `\u{2713}` escapes
compile to identical bytes.
