# Scout Report 0016: Overwrite Safety for `great loop install`

**Scout:** Humboldt
**Date:** 2026-02-24
**Spec:** `/home/isaac/src/sh.great/.tasks/ready/0016-overwrite-safety-spec.md`
**Socrates review:** APPROVED (all concerns advisory)

---

## 1. Primary File: `src/cli/loop_cmd.rs`

**Path:** `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`
**Total lines:** 661

### Exact line numbers for required touchpoints

| Location | Lines | Notes |
|----------|-------|-------|
| `Install` variant (in `LoopCommand` enum) | 17-21 | Add `force: bool` field with `#[arg(long)]` |
| `pub fn run()` dispatch | 135-141 | Change `Install { project }` to `Install { project, force }` and update `run_install` call |
| `run_install` signature | 155 | Change `fn run_install(project: bool)` to `fn run_install(project: bool, force: bool)` |
| First `fs::write` call (agent files) | 176 | Overwrite check must be inserted before this line |
| Dir creation block ends | 171 | `create_dir_all(&teams_dir)` -- insert overwrite check immediately after |
| `statusline_value()` function ends | ~152 | New helpers `collect_existing_paths` and `confirm_overwrite` insert here |
| `mod tests` closing brace | 661 | 6 new unit tests append before this line |

### Stale comment (Socrates advisory #3)
Line 104: `/// All 4 slash-command files shipped with the great.sh Loop.` -- COMMANDS has 5 entries. Fix to "5" in the same pass.

### Current imports (line 1-4)
```rust
use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Subcommand};

use crate::cli::output;
```
Change line 1 to: `use anyhow::{bail, Context, Result};`
`use std::io::IsTerminal;` goes inside `confirm_overwrite` as a scoped import (per Socrates advisory #7 -- authoritative version is Change 5 code block).

---

## 2. Integration Test File: `tests/cli_smoke.rs`

**Path:** `/home/isaac/src/sh.great/tests/cli_smoke.rs`
**Total lines:** 1132

### Existing imports (lines 1-3)
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
```
All imports required for new tests are already present. No new `use` statements needed.

### Existing helper (lines 6-8)
```rust
fn great() -> Command {
    Command::cargo_bin("great").expect("binary exists")
}
```
All new tests use `great()`. Pattern is identical to existing tests.

### TempDir usage
`TempDir` is used throughout (lines 59, 70, 81, 98, 110, 128, 163, 179, 189, etc.). The integration tests for loop install follow the exact same pattern:
- `TempDir::new().unwrap()`
- `.env("HOME", dir.path())` to isolate `~/.claude/`
- `dir.path().join(".claude/agents/nightingale.md").exists()` for assertions

### Append location
New tests go at the end of the file, after line 1132, in a new section:
```rust
// -----------------------------------------------------------------------
// Loop install -- overwrite safety
// -----------------------------------------------------------------------
```

---

## 3. `Cargo.toml`: Dev Dependencies

**Path:** `/home/isaac/src/sh.great/Cargo.toml`

```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.0"    # <-- already present
```

`tempfile` is already in `dev-dependencies`. No `Cargo.toml` changes needed. Unit tests use `tempfile::TempDir::new()` via the crate already in scope.

---

## 4. Callers of `run_install`

`run_install` is called from exactly one site:

**`/home/isaac/src/sh.great/src/cli/loop_cmd.rs` line 137:**
```rust
LoopCommand::Install { project } => run_install(project),
```

No other callers. The function is `fn` (not `pub fn`), so it cannot be called from outside the module. Signature change from `(project: bool)` to `(project: bool, force: bool)` has exactly one construction site to update.

---

## 5. Dependency Map

```
LoopCommand::Install { project, force }   [enum variant -- loop_cmd.rs:17]
  |
  v
run() match arm                           [loop_cmd.rs:137]
  |
  v
run_install(project, force)               [loop_cmd.rs:155]
  |
  +-- create_dir_all (agents, commands, teams/loop)   [lines 167-171]
  |
  +-- collect_existing_paths(&claude_dir)             [NEW, insert after 171]
  |     iterates: AGENTS (15), COMMANDS (5), teams config (1) = 21 paths
  |
  +-- if !existing.is_empty() && !force               [NEW guard]
  |     confirm_overwrite(&existing)                  [NEW]
  |       - lists paths to stderr
  |       - checks std::io::stdin().is_terminal()
  |       - reads one line from stdin
  |       - returns Ok(bool)
  |     bail! if !confirmed
  |
  +-- if force && !existing.is_empty()               [NEW info message]
  |
  +-- write agent files (15)                         [lines 174-182, unchanged]
  +-- write command files (5)                        [lines 184-192, unchanged]
  +-- write teams config (1)                         [lines 195-199, unchanged]
  +-- handle settings.json                           [lines 201-274, unchanged]
  +-- handle --project                               [lines 277-316, unchanged]
```

No circular dependencies. `collect_existing_paths` and `confirm_overwrite` are pure module-private helpers with no side effects beyond `Path::exists()` and `eprintln!` / `stdin().read_line()` respectively.

---

## 6. Build Order

Per spec (7 steps, all in one file):

1. Change `use anyhow::{Context, Result}` → `use anyhow::{bail, Context, Result}` (line 1)
2. Add `force: bool` field to `Install` variant (lines 17-21)
3. Update `run()` dispatch: destructure `force`, pass to `run_install` (lines 137, 155)
4. Insert `collect_existing_paths` helper after `statusline_value()` (~line 152)
5. Insert `confirm_overwrite` helper after `collect_existing_paths`
6. Insert overwrite check block in `run_install` after line 171
7. Append 6 unit tests before closing `}` of `mod tests` (line 661)
8. Append 4 integration tests to `tests/cli_smoke.rs` after line 1132
9. Fix stale comment on line 104: "4" → "5"

---

## 7. Risks

| Risk | Severity | Note |
|------|----------|------|
| `bail!` not imported | Blocking if missed | Change 1 is mandatory; code won't compile without it |
| `IsTerminal` scoped vs module import | Low | Spec is ambiguous (Socrates #7); use scoped import inside `confirm_overwrite` as shown in Change 5 code block |
| stderr flush before `read_line` | Low | Socrates advisory #1: add `std::io::stderr().flush()` after `eprint!("Overwrite? [y/N] ")` and before `read_line` to be defensive |
| `create_dir_all` before abort | Accepted | Creates empty dirs even if user declines; idempotent and harmless (Socrates #2) |
| stale comment line 104 | Cosmetic | Fix "4" → "5" while in the area |
| Integration tests touch real `~/.claude/` | Blocking if missed | Must set `.env("HOME", dir.path())` on every loop install integration test |

---

## 8. Existing Patterns to Follow

- **`tempfile::TempDir` in unit tests:** Pattern already used in `loop_cmd.rs` tests (e.g., `test_collect_existing_paths_*`) and throughout `tests/cli_smoke.rs`
- **`bail!` macro:** Used throughout `src/cli/apply.rs` and other modules -- consistent with project style
- **`output::warning` / `output::error` / `output::info`:** Already used in `run_install` for all user-visible messages; `confirm_overwrite` follows same pattern
- **`dirs::home_dir()` isolation via `HOME` env var:** Already proven in Socrates Q5 -- `dirs::home_dir()` reads `HOME` on Linux, so `.env("HOME", dir.path())` in integration tests reliably isolates the child process
- **Clap `#[arg(long)]` bool flag:** Pattern matches existing `project: bool` field on the same `Install` variant

---

## Summary

**Single-file change** (`loop_cmd.rs`) plus integration test additions (`tests/cli_smoke.rs`). No new dependencies, no new files, no `Cargo.toml` changes. The 7-step build order in the spec is correct and complete. The three advisory items from Socrates worth actioning: add `bail` import (blocking), flush stderr before read (defensive), fix stale "4" comment (cosmetic).
