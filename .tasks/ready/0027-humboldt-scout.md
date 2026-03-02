# 0027: Humboldt Scout Report — Wire `--non-interactive` Flag

**Scout:** Humboldt (Codebase Scout)
**Iteration:** 024
**Date:** 2026-02-27
**Spec:** `.tasks/ready/0027-non-interactive-spec.md`
**Review:** `.tasks/ready/0027-socrates-review.md` — APPROVED

---

## 1. File Map

All 4 files modified, 0 created. Exact verified line numbers against current codebase.

### `src/cli/sudo.rs`

| Lines | What | Change |
|-------|------|--------|
| 58–62 | Doc comment block + TODO comment | Replace TODO at lines 61–62 with doc line for new param |
| 63 | `pub fn ensure_sudo_cached(is_root: bool) -> SudoCacheResult {` | Add `non_interactive: bool` parameter |
| 69–72 | `if !std::io::stdin().is_terminal()` block | Change to `if non_interactive \|\| !std::io::stdin().is_terminal()` |
| 138 | `ensure_sudo_cached(true)` (inside test) | Change to `ensure_sudo_cached(true, false)` |
| 139 (after 140) | — | Add new test `non_interactive_flag_returns_non_interactive` after `keepalive_drop_signals_stop` |

Current function signature at **line 63**:
```rust
pub fn ensure_sudo_cached(is_root: bool) -> SudoCacheResult {
```

Current TODO at **lines 61–62** (to be removed):
```rust
// TODO: Also accept a `non_interactive` parameter from the --non-interactive
// global CLI flag once it is wired through to apply::run() / doctor::run().
```

Current terminal check at **lines 70–72**:
```rust
if !std::io::stdin().is_terminal() {
    return SudoCacheResult::NonInteractive;
}
```

Current unit test at **line 138**:
```rust
let result = ensure_sudo_cached(true);
```

---

### `src/cli/apply.rs`

| Lines | What | Change |
|-------|------|--------|
| 354–367 | `pub struct Args` (3 fields: config, dry_run, yes) | Add 4th field with `#[arg(skip)]` |
| 420–421 | `use crate::cli::sudo::{ensure_sudo_cached, SudoCacheResult};` + call | Pass `args.non_interactive` as 2nd arg |
| 572 | `let managers = package_manager::available_managers(false);` (CLI tools section) | Replace `false` with `args.non_interactive` |
| 730 | `let managers = package_manager::available_managers(false);` (bitwarden-cli install) | Replace `false` with `args.non_interactive` |
| 803 | `let managers = package_manager::available_managers(false);` (platform-specific tools) | Replace `false` with `args.non_interactive` |

Current `Args` struct at **lines 354–367**:
```rust
#[derive(ClapArgs)]
pub struct Args {
    /// Path to configuration file
    #[arg(long)]
    pub config: Option<String>,

    /// Preview changes without applying
    #[arg(long)]
    pub dry_run: bool,

    /// Skip confirmation prompts
    #[arg(long, short)]
    pub yes: bool,
}
```

Current `ensure_sudo_cached` call at **line 421**:
```rust
match ensure_sudo_cached(info.is_root) {
```

Context for the 3 `available_managers` calls:
- **Line 572**: inside `if let Some(cli_tools) = &tools.cli` block — `args` is in scope
- **Line 730**: inside `if let Some(secrets) = &cfg.secrets` / bitwarden branch — `args` is in scope
- **Line 803**: inside `if let Some(extra_tools) = override_tools` block — `args` is in scope

---

### `src/cli/doctor.rs`

| Lines | What | Change |
|-------|------|--------|
| 10–15 | `pub struct Args` (1 field: fix) | Add 2nd field with `#[arg(skip)]` |
| 97 | `let managers = package_manager::available_managers(false);` | Replace `false` with `args.non_interactive` |
| 111–112 | `use crate::cli::sudo::{ensure_sudo_cached, SudoCacheResult};` + call | Pass `args.non_interactive` as 2nd arg |

Current `Args` struct at **lines 10–15**:
```rust
#[derive(ClapArgs)]
pub struct Args {
    /// Attempt to fix issues automatically
    #[arg(long)]
    pub fix: bool,
}
```

Current `available_managers` call at **line 97**: inside the `if args.fix && !result.fixable.is_empty()` block.

Current `ensure_sudo_cached` call at **line 112**: inside `let _sudo_keepalive = if has_sudo_fix { ... }` block, which is itself inside the `if args.fix` block. The `args` binding is in scope for both. `args.non_interactive` is `Copy`, so no borrow conflict with `args.fix`.

---

### `src/main.rs`

| Lines | What | Change |
|-------|------|--------|
| 14 | `let cli = Cli::parse();` | Add `let non_interactive = cli.non_interactive;` on line 15 (new line) |
| 18 | `Command::Apply(args) => cli::apply::run(args),` | Change to `mut args` block |
| 23 | `Command::Doctor(args) => cli::doctor::run(args),` | Change to `mut args` block |

Current dispatch block at **lines 16–29** (full match):
```rust
match cli.command {
    Command::Init(args) => cli::init::run(args),
    Command::Apply(args) => cli::apply::run(args),
    Command::Status(args) => cli::status::run(args),
    Command::Sync(args) => cli::sync::run(args),
    Command::Vault(args) => cli::vault::run(args),
    Command::Mcp(args) => cli::mcp::run(args),
    Command::Doctor(args) => cli::doctor::run(args),
    Command::Update(args) => cli::update::run(args),
    Command::Diff(args) => cli::diff::run(args),
    Command::Template(args) => cli::template::run(args),
    Command::Loop(args) => cli::loop_cmd::run(args),
    Command::Statusline(args) => cli::statusline::run(args),
}
```

---

### `src/cli/mod.rs` — NO CHANGES

The `non_interactive` field at **lines 33–35** is already correct:
```rust
/// Disable interactive prompts (for CI/automation)
#[arg(long, global = true)]
pub non_interactive: bool,
```

### `src/platform/package_manager.rs` — NO CHANGES

`available_managers(non_interactive: bool)` at **line 438** already accepts the flag and passes it to `Apt::new(non_interactive)` at **line 458**. The signature is correct; only the call sites in `apply.rs` and `doctor.rs` need fixing.

---

## 2. Patterns

### Args struct pattern (confirmed across `init`, `status`, `apply`, `doctor`)

Every subcommand uses:
```rust
#[derive(ClapArgs)]
pub struct Args {
    #[arg(...)]
    pub field: Type,
}
pub fn run(args: Args) -> Result<()> { ... }
```

The `#[arg(skip)]` attribute is **not currently used anywhere** in the codebase (grep returned zero matches). This task introduces it for the first time. It is a standard clap 4 derive attribute that:
- Excludes the field from CLI argument parsing
- Uses `Default::default()` for initialization (bool defaults to `false`)
- Does NOT appear in `--help` output
- Does NOT create a `--non-interactive` argument at the subcommand level (no conflict with the global flag)

### Move semantics in main.rs

`Cli::parse()` returns an owned `Cli` struct. The `match cli.command` expression moves the `command` field out of `cli`, making `cli` partially moved. After that statement, `cli` cannot be referenced.

The fix: extract `cli.non_interactive` (a `Copy` type) into a local `let` binding **before** the match. This is idiomatic Rust and the correct pattern.

### `use` imports are local in apply.rs and doctor.rs

Both files use block-local imports inside the `if needs_sudo` / `if has_sudo_fix` guards:
```rust
use crate::cli::sudo::{ensure_sudo_cached, SudoCacheResult};
```
This import is already present at the call sites. Da Vinci does not need to add a top-level import.

---

## 3. Call Graph

### `ensure_sudo_cached` callers (2 total, confirmed by grep)

```
src/cli/apply.rs:421  -- inside `if needs_sudo { ... }` guard
src/cli/doctor.rs:112 -- inside `if has_sudo_fix { ... }` guard
```

No other callers. No external crate calls this function (it is `pub` but not re-exported from the crate root).

### `available_managers` callers (3 in apply.rs + 1 in doctor.rs + tests)

```
src/cli/apply.rs:572   -- CLI tools section
src/cli/apply.rs:730   -- bitwarden-cli install section
src/cli/apply.rs:803   -- platform-specific tools section
src/cli/doctor.rs:97   -- auto-fix section (inside `if args.fix` block)
src/platform/package_manager.rs:509,553,554,563  -- test-only calls (pass false; no change needed)
```

All 4 production call sites in `apply.rs` and `doctor.rs` currently hardcode `false`. All 4 must be updated.

### Who calls `sudo.rs` functions from outside `cli/`

Nothing outside `cli/` calls `ensure_sudo_cached`. The `SudoCacheResult` and `SudoKeepalive` types are also only used within `cli/apply.rs` and `cli/doctor.rs`.

---

## 4. Gotchas

### G1: `#[arg(skip)]` is new to this codebase

No existing example to copy from. Da Vinci must add `#[arg(skip)]` exactly — not `#[arg(hidden)]` (which still parses the argument, just hides it from help), not a plain field without attributes (which would make clap try to parse it as a positional arg).

The correct form:
```rust
/// Set by main.rs from the global --non-interactive flag.
/// Not a CLI argument -- hidden from clap.
#[arg(skip)]
pub non_interactive: bool,
```

### G2: `args` is consumed by `run()` — extract `non_interactive` first

`run(args: Args)` takes `Args` by value. If `args.non_interactive` needs to be passed to multiple call sites inside `run()`, it must be copied before any consumption. Since `bool` is `Copy`, reading `args.non_interactive` multiple times is fine — no move occurs.

### G3: The `args` variable is in scope at all 4 `available_managers` call sites

Verified by reading the full function bodies:
- apply.rs line 572: inside `if let Some(cli_tools) = &tools.cli` — `args` is the function parameter, always in scope
- apply.rs line 730: inside `if let Some(secrets) = &cfg.secrets` — same
- apply.rs line 803: inside `if let Some(extra_tools) = override_tools` — same
- doctor.rs line 97: inside `if args.fix && !result.fixable.is_empty()` — `args` referenced by the `if` condition itself, so it is definitely in scope

### G4: `cli.non_interactive` extraction order in main.rs

The `let non_interactive = cli.non_interactive;` line MUST come before `match cli.command`. If placed after or inside the match, the partial move of `cli` will cause a compile error. The spec's desired code is correct.

### G5: Socrates advisory concerns — bootstrap.rs and tuning.rs

Socrates (ADVISORY, non-blocking) identified that `bootstrap.rs` and `tuning.rs` contain direct `Command::new("sudo")` calls that are NOT routed through `ensure_sudo_cached` or `available_managers`. These are pre-existing and explicitly out of scope for this task. Da Vinci must not fix them in this task — they belong to a separate future task.

### G6: No new integration tests in `tests/cli_smoke.rs`

The spec says no new integration tests are required. However, Socrates recommends (ADVISORY) adding a minimal smoke test for `great apply --non-interactive --dry-run` to catch regressions. Da Vinci may add it; it is optional not required.

The existing test `apply_dry_run_no_sudo_prompt` (line 742) already covers a related path via piped stdin. The new flag would add an explicit flag path on top.

### G7: Clippy warning will resolve automatically

The current unused `cli.non_interactive` field in `main.rs` likely generates a dead_code or unused-read warning. After extracting it into a `let` binding and using it, the warning disappears. No `#[allow(...)]` annotation needed.

### G8: `#[arg(skip)]` field ordering — add at end of struct

Adding the skip field at the END of the struct is the cleanest diff and avoids disrupting the visual grouping of real CLI arguments. Both `apply::Args` and `doctor::Args` should have `non_interactive` as their last field.

---

## 5. Verification Commands

Run these in order after implementation:

```bash
# 1. Compile check — no errors expected
cargo build 2>&1

# 2. Clippy — zero warnings related to non_interactive expected
cargo clippy 2>&1

# 3. Unit tests — all must pass including new test in sudo.rs
cargo test 2>&1

# 4. Verify flag accepted at top level (global)
cargo run -- --non-interactive apply --dry-run 2>&1

# 5. Verify flag accepted as subcommand prefix (global flag position)
cargo run -- apply --non-interactive --dry-run 2>&1

# 6. Verify flag does NOT appear in apply --help output (not a subcommand arg)
cargo run -- apply --help 2>&1 | grep -c "non-interactive"
# Expected: 0

# 7. Verify flag does NOT appear in doctor --help output
cargo run -- doctor --help 2>&1 | grep -c "non-interactive"
# Expected: 0

# 8. Verify global --help shows the flag once
cargo run -- --help 2>&1 | grep "non-interactive"
# Expected: one line showing the global flag

# 9. Run integration tests
cargo test --test cli_smoke 2>&1
```

### For confirming the TODO is removed

```bash
grep -n "TODO.*non_interactive" /home/isaac/src/sh.great/src/cli/sudo.rs
# Expected: no output
```

### For confirming all hardcoded false values are replaced

```bash
grep -n "available_managers(false)" /home/isaac/src/sh.great/src/cli/apply.rs
grep -n "available_managers(false)" /home/isaac/src/sh.great/src/cli/doctor.rs
# Expected: no output from either (all 4 sites replaced)
```

---

## 6. Recommended Build Order

The spec's dependency order is correct. For a single-commit build, implement in this sequence to keep the compiler as your guide:

1. **`src/cli/sudo.rs`** — Extend `ensure_sudo_cached` signature to `(is_root: bool, non_interactive: bool)`, add `non_interactive ||` to the condition, remove TODO, update existing test, add new test. After this, `cargo build` will produce errors in `apply.rs` and `doctor.rs` at the call sites (missing 2nd argument).

2. **`src/cli/apply.rs`** — Add `#[arg(skip)] pub non_interactive: bool` to `Args`. Update `ensure_sudo_cached(info.is_root)` → `ensure_sudo_cached(info.is_root, args.non_interactive)`. Update all 3 `available_managers(false)` → `available_managers(args.non_interactive)`. After this, `cargo build` still fails in `main.rs` only if there's a mismatch (there won't be, since `#[arg(skip)]` gives a default).

3. **`src/cli/doctor.rs`** — Same pattern: add field, update `ensure_sudo_cached` call, update `available_managers` call.

4. **`src/main.rs`** — Extract `cli.non_interactive` before the match; add `mut args` + field assignment for `Apply` and `Doctor` arms. After this, `cargo build` succeeds.

Total: 4 files, ~30 lines changed, 2 tests added/updated.

---

## 7. Summary Risk Assessment

**Risk: VERY LOW.** This is pure mechanical plumbing. All types involved are `Copy` (`bool`). No new error paths. No new files. No new dependencies. The only novel pattern (`#[arg(skip)]`) is well-documented in clap 4 and has a clear implementation.

The Socrates review was APPROVED with 4 ADVISORY concerns, none blocking. The advisories concern pre-existing limitations in `bootstrap.rs` and `tuning.rs` that are explicitly out of scope.
