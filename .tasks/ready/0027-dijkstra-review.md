# Dijkstra Code Review — Task 0027: Wire --non-interactive Flag Through CLI

**Iteration:** 024
**Reviewer:** Dijkstra (Code Reviewer)
**Date:** 2026-02-27
**Prior results:** Turing PASS (1 fix cycle), Kerckhoffs CLEAN, Nielsen NO BLOCK, Wirth PASS

---

## VERDICT: APPROVED

---

## Issues

None blocking. One advisory note below.

- [WARN] `src/cli/apply.rs:916-918` — `configure_starship` reads `std::env::var("SHELL")` with
  `.unwrap_or_default()`, producing an empty string that silently defaults to the bash branch.
  This is pre-existing and unrelated to this task; noted only because it is in the same file.
  Not introduced by this change.

---

## Criterion-by-criterion assessment

### 1. Correctness — changes match spec exactly

All ten spec changes are present and correct:

- **Change 7** (`sudo.rs:62`): Signature extended to `ensure_sudo_cached(is_root: bool,
  non_interactive: bool)`. TODO comment removed. The condition at line 69 is
  `if non_interactive || !std::io::stdin().is_terminal()` — flag short-circuits first,
  matching spec section 5a. The doc-comment at line 61 names both parameters.

- **Change 8** (`sudo.rs:137`): Existing test updated to `ensure_sudo_cached(true, false)`.
  Correct.

- **Change 9** (`sudo.rs:142-145`): New test `non_interactive_flag_returns_non_interactive`
  calls `ensure_sudo_cached(false, true)` and asserts `NonInteractive`. Correct.

- **Changes 1/3** (`apply.rs:354-372`): `#[arg(skip)] pub non_interactive: bool` added as
  the last field of `Args`, with the specified doc comment. All four call sites updated:
  - Line 426: `ensure_sudo_cached(info.is_root, args.non_interactive)` — correct.
  - Line 577: `available_managers(args.non_interactive)` — correct (CLI tools section).
  - Line 735: `available_managers(args.non_interactive)` — correct (bitwarden-cli section).
  - Line 808: `available_managers(args.non_interactive)` — correct (platform-specific tools).

- **Changes 4/5/6** (`doctor.rs:9-20`): `#[arg(skip)] pub non_interactive: bool` added.
  Both call sites updated:
  - Line 102: `available_managers(args.non_interactive)` — correct.
  - Line 117: `ensure_sudo_cached(info.is_root, args.non_interactive)` — correct.

- **Change 10** (`main.rs:14-36`): `non_interactive` extracted at line 15 before the match.
  `Apply` and `Doctor` arms use `mut args` and set `args.non_interactive = non_interactive`
  before calling `run()`. All other arms unchanged.

The spec counts three `available_managers` call sites in `apply.rs` (lines 572, 730, 803 in
the pre-change file). The implementation has them at lines 577, 735, 808 in the post-change
file — all three are updated. Count confirmed.

### 2. Consistency — #[arg(skip)] pattern

`#[arg(skip)]` is the correct clap mechanism for a field that must be excluded from CLI
parsing but initialized at runtime. The pattern is used identically in both `apply::Args`
and `doctor::Args`. The doc comment "Set by main.rs from the global --non-interactive flag.
Not a CLI argument -- hidden from clap." is clear and consistent across both structs. The
field defaults to `false` (clap's zero-value for bool), which is the correct default for
"interactive mode unless told otherwise."

### 3. Move semantics in main.rs

`let non_interactive = cli.non_interactive;` at line 15 copies the `bool` before
`match cli.command` moves `cli` at line 17. Because `bool` is `Copy`, the binding is
unconditional and correct. There is no risk of the field being inaccessible after the move.
The block arms for `Apply` and `Doctor` do not alter control flow relative to the
single-expression arms for the other commands — `cli::apply::run(args)` is still the last
expression and its `Result<()>` is still returned correctly.

### 4. Test quality

Three tests in `sudo.rs`:

- `already_root_returns_immediately` — verifies the root early-exit with updated arity.
- `non_interactive_flag_returns_non_interactive` — verifies the new `non_interactive`
  parameter returns `NonInteractive` when `is_root` is `false`. This is the critical path
  for the bug fix and is correctly isolated: it does not touch stdin, does not call sudo,
  and cannot flake.
- `keepalive_drop_signals_stop` — unchanged; still tests `SudoKeepalive` drop semantics.

The test for the new path uses `is_root = false` (not `true`), which is the only interesting
case: had `is_root` been `true`, the function would return `AlreadyRoot` before ever
reaching the `non_interactive` check. The test correctly exercises the new branch.

The spec explicitly declines to add integration tests for sudo prompts (section 8b), which
is appropriate — those require a real system with configured sudo and cannot be automated
reliably.

### 5. No accidental changes

The four modified files contain only spec-required changes. No other files were touched.
`src/cli/mod.rs` is unchanged. The `Cli` struct's `non_interactive` field remains at line 35
with `global = true`, unchanged. No refactoring, no formatting changes, no scope creep
observed.

### 6. Error handling

No new error paths introduced. The `NonInteractive` variant of `SudoCacheResult` was already
handled by the `_ => None` arm in both callers before this change, and remains so. The
`available_managers` function already accepted the `bool` parameter; wiring the actual flag
value rather than `false` is a behavioral fix, not a structural change. No errors are
swallowed.

---

## Summary

All ten spec-prescribed changes are implemented exactly, the `#[arg(skip)]` pattern is
applied consistently across both structs, move semantics in main.rs are sound, and the new
unit test isolates the added branch correctly — no issues block approval.
