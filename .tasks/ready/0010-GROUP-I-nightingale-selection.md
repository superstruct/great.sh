# Nightingale Selection — Iteration 016
# Task: 0010 GROUP I — Dead Code and Safety Cleanup

**Selected:** 2026-02-25
**Priority:** P1
**Type:** refactor
**Size:** S (no dependencies, pure cleanup)
**Blocked by:** Nothing

---

## Evidence of Selection

### Groups Verified as Already Done (not in 0010 task file yet)

- **GROUP C (MCP add, P1, S):** `src/cli/mcp.rs` lines 109-164 contain a complete
  `run_add()` implementation using `toml_edit` for format-preserving writes,
  duplicate detection, and success messaging. `toml_edit = "0.22"` is present in
  `Cargo.toml`. This group is done.

- **GROUP G (Sync pull --apply, P1, S):** `src/cli/sync.rs` lines 62-131 contain
  a complete `run_pull(apply: bool)` implementation -- the `--apply` flag backs up
  `great.toml` to `great.toml.bak`, validates the TOML blob before writing, writes
  the file, and the preview mode shows content without modifying anything. This group
  is done.

### Groups Remaining (genuinely open as of 2026-02-25)

| Group | Name                  | Priority | Size | Deps       |
|-------|-----------------------|----------|------|------------|
| B     | Starship config       | P1       | M    | A          |
| D     | Doctor --fix          | P1       | L    | A          |
| E     | Update command        | P1       | M    | None       |
| F     | Vault completion      | P1       | L    | None       |
| **I** | **Dead code cleanup** | **P1**   | **S**| **None**   |
| K     | Docker test rigs      | P1       | L    | J          |
| H     | Template update       | P2       | M    | E          |

### Selection Rationale

GROUP I is the only remaining Size S task with no dependencies. It is a P1 item
that provides measurable quality improvement (zero-warning `cargo clippy`, zero
`.unwrap()` in production paths) without introducing new features or incurring
scope risk. Completing it also improves the signal-to-noise ratio for all future
work -- clean clippy output makes new regressions immediately visible.

---

## Task: 0010 GROUP I — Dead Code and Safety Cleanup

**Priority:** P1
**Type:** refactor
**Module:** Multiple (see file list below)
**Status:** ready

### Context

`cargo clippy` currently produces dead code warnings across multiple modules.
Additionally, several `.unwrap_or("")` calls in production CLI code use a pattern
that is safe but inconsistent with project convention (`?` or `.unwrap_or_default()`).
The project CLAUDE.md explicitly forbids `.unwrap()` in production code. Cleaning
these up now means every subsequent iteration starts from a zero-warning baseline,
making new regressions immediately detectable.

### Files in Scope

- `src/config/mod.rs` -- dead code items
- `src/error.rs` -- unused `GreatError` variants (Network, possibly others)
- `src/mcp/mod.rs` -- dead code items
- `src/platform/package_manager.rs` -- dead code items
- `src/platform/runtime.rs` -- dead code items
- `src/sync/mod.rs` -- dead code items
- `src/vault/mod.rs` -- dead code items
- `src/cli/status.rs` -- `.unwrap_or("")` on line 191
- `src/cli/doctor.rs` -- `.unwrap_or("")` on line 377

### Requirements

1. Run `cargo clippy -- -W dead_code` and resolve every warning by one of:
   (a) removing unused code, (b) adding `#[allow(dead_code)]` with a comment
   explaining why it is retained (e.g., "used by GROUP E update command"), or
   (c) using the item somewhere it logically belongs.
2. Replace all `.unwrap()` calls in non-test production code with `?`,
   `.unwrap_or()`, `.unwrap_or_default()`, or `.ok()` as appropriate.
3. For `tokio` and `reqwest` (not yet called from any code path until GROUP E
   lands), add `#[allow(unused_imports)]` with the comment "used by GROUP E".
4. Confirm `cargo clippy` exits with zero warnings after changes.
5. Confirm `cargo test` still passes after changes.

### Acceptance Criteria

- [ ] `cargo clippy` produces zero warnings with default lint level.
- [ ] Zero `.unwrap()` calls exist in `src/cli/*.rs`, `src/config/*.rs`,
  `src/platform/*.rs`, `src/mcp/*.rs`, `src/sync/*.rs`, `src/vault/*.rs`
  outside of `#[cfg(test)]` blocks.
- [ ] All dead code items are resolved: removed, used, or annotated with a
  comment explaining the retention reason.
- [ ] `cargo test` passes with no regressions.
- [ ] Diff is pure refactor: no behavior changes, no new features.

### Dependencies

None. This group is independently implementable and does not block any other group.

### Notes

- Keep changes minimal. If a dead-code item is genuinely needed by an upcoming
  group (e.g., a GROUP E network helper), annotate rather than remove.
- The `.unwrap_or("")` pattern in `status.rs` and `doctor.rs` is technically
  safe but violates the project convention. Replace with `.unwrap_or_default()`
  which communicates intent more clearly for `&str`/`String` returns.
- Do not refactor logic, rename functions, or reorganize modules -- clippy and
  unwrap cleanup only.
