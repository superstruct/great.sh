# Scout Report 0006: `great diff` Gap Completion

**Author:** Alexander von Humboldt (Codebase Scout)
**Date:** 2026-02-25
**Spec:** `.tasks/ready/0006-diff-gaps-spec.md`
**Status:** Ready for Da Vinci

---

## 1. File Map

### Primary target: `src/cli/diff.rs`

- **Path:** `/home/isaac/src/sh.great/src/cli/diff.rs`
- **Line count:** 199 (lines 1-198, trailing newline on 199)
- **Key function:** `pub fn run(args: Args) -> Result<()>` — lines 29-198

**Structural map of `run()`:**

| Lines | Content |
|-------|---------|
| 1-7   | Imports: `anyhow::Result`, `clap::Args as ClapArgs`, `colored::Colorize`, `crate::cli::output`, `crate::config`, `crate::platform::command_exists` |
| 9-19  | `Args` struct with `config: Option<String>` |
| 21-28 | Doc comment for `run()` — needs `-` (red) marker added |
| 29-43 | Config discovery / load block |
| 31-40 | `match &args.config` — the `Err` arm on lines 35-38 returns `Ok(())` (GAP 2 target) |
| 44-50 | `output::header` + `output::info` + `println!()` |
| 52    | `let mut has_diff = false;` — counters go AFTER this line (GAP 4) |
| 55-99 | Tools diff block (`if let Some(tools)`) |
| 59-74 | Runtimes loop — GAP 1 target (replace entire loop body) |
| 77-89 | CLI tools loop — GAP 1 target (replace entire loop body) |
| 91-98 | Section flush: `output::header("Tools — need to install:")` — change to `"Tools"` |
| 101-140 | MCP servers diff block |
| 105   | `for (name, mcp) in mcps {` — GAP 3: add `enabled == Some(false)` guard here |
| 107-115 | Missing command push site — add `configure_count += 1` |
| 118-129 | Needs .mcp.json push site — add `configure_count += 1` |
| 143-167 | Secrets diff block (`secrets.required`) |
| 148-154 | Per-key push — GAP 5: change `"+".green()` to `"-".red()`, add `secrets_count += 1` |
| 170-189 | Secret refs block (`find_secret_refs()`) |
| 181-187 | Per-ref println — GAP 5: change `"+".green()` to `"-".red()`, add `secrets_count += 1` |
| 191-195 | Summary block — GAP 4: replace entirely with counter-based summary |
| 197   | `Ok(())` |

### Secondary target: `tests/cli_smoke.rs`

- **Path:** `/home/isaac/src/sh.great/tests/cli_smoke.rs`
- **Line count:** approximately 820+ lines (file is large, paged below 800)
- **Existing diff section:** lines 119-131 (section header + single test `diff_no_config_shows_error`)
- **Insertion point for new tests:** after line 131, before the `// Template` section header at line 133

**Existing diff test that must be REPLACED (not appended to):**
```
lines 122-131: fn diff_no_config_shows_error — currently asserts .success()
```
This test must be renamed `diff_no_config_exits_nonzero` and changed to `.failure()`.

### Reference files (read-only for patterns):

| File | Path | Purpose |
|------|------|---------|
| `util.rs` | `/home/isaac/src/sh.great/src/cli/util.rs` | `get_command_version` (28 lines) |
| `output.rs` | `/home/isaac/src/sh.great/src/cli/output.rs` | All output helpers (43 lines) |
| `doctor.rs` | `/home/isaac/src/sh.great/src/cli/doctor.rs` | `enabled` guard + `process::exit` pattern |
| `status.rs` | `/home/isaac/src/sh.great/src/cli/status.rs` | `process::exit` pattern, `util::get_command_version` usage |
| `schema.rs` | `/home/isaac/src/sh.great/src/config/schema.rs` | `McpConfig.enabled`, `find_secret_refs()` |
| `mod.rs` | `/home/isaac/src/sh.great/src/cli/mod.rs` | Module structure (78 lines) |
| `Cargo.toml` | `/home/isaac/src/sh.great/Cargo.toml` | Dependency versions |

---

## 2. Import Inventory

### Current imports in `diff.rs` (lines 1-7):
```rust
use anyhow::Result;
use clap::Args as ClapArgs;
use colored::Colorize;

use crate::cli::output;
use crate::config;
use crate::platform::command_exists;
```

### Import that must be added (Gap 1):
```rust
use crate::cli::util;
```
Insert at line 6, after `use crate::cli::output;`. The `util` module is already declared as `pub mod util;` in `src/cli/mod.rs` (line 15) and exports `get_command_version` as a public function.

### No other imports needed. Confirmed present:
- `colored::Colorize` — already imported; `.red()` is available on `&str` (Gap 5 uses `.red()` which is in the same trait as `.green()` and `.yellow()` already used)
- `std::process::exit` — used as `std::process::exit(1)` without import (same pattern as `doctor.rs` line 246 and `status.rs` line 287)
- `crate::config::GreatConfig::find_secret_refs` — accessed via `cfg.find_secret_refs()` already present at line 170

---

## 3. Pattern Catalog

### Pattern A: `process::exit(1)` for diagnostic commands

From `src/cli/doctor.rs` lines 242-247:
```rust
// NOTE: Intentional use of process::exit — the doctor command must print
// its full report before exiting non-zero. Using bail!() would abort
// mid-report, which is wrong for a diagnostic command.
if result.checks_failed > 0 {
    std::process::exit(1);
}
```

From `src/cli/status.rs` lines 283-288:
```rust
// NOTE: Intentional use of process::exit — the status command must print
// its full report before exiting non-zero. Using bail!() would abort
// mid-report, which is wrong for a diagnostic command.
if has_critical_issues {
    std::process::exit(1);
}
```

**For Gap 2:** use `std::process::exit(1)` directly after `output::error(...)`. Same pattern but at the top of `run()` before any report is printed:
```rust
Err(_) => {
    output::error("No great.toml found. Run `great init` to create one.");
    std::process::exit(1);
}
```

### Pattern B: MCP `enabled == Some(false)` guard

From `src/cli/doctor.rs` lines 566-570:
```rust
for (name, mcp) in mcps {
    // Skip disabled servers
    if mcp.enabled == Some(false) {
        pass(result, &format!("{}: disabled (skipped)", name));
        continue;
    }
```

**For Gap 3:** identical guard but WITHOUT the `pass(...)` line (diff is silent for disabled servers):
```rust
for (name, mcp) in mcps {
    // Skip disabled servers
    if mcp.enabled == Some(false) {
        continue;
    }
```

### Pattern C: `util::get_command_version` usage

From `src/cli/status.rs` lines 181-193:
```rust
let installed = command_exists(name);
let actual_version = if installed {
    util::get_command_version(name)
} else {
    has_critical_issues = true;
    None
};
```

**For Gap 1:** inline usage within `if/else if` chain:
```rust
let installed = command_exists(name);
if !installed {
    install_count += 1;
    tool_diffs.push(...);
} else if declared_version != "latest" && declared_version != "stable" {
    if let Some(actual) = util::get_command_version(name) {
        if !actual.contains(declared_version.as_str()) {
            configure_count += 1;
            tool_diffs.push(...);
        }
    }
}
```

Note: `declared_version` is a `&String` from the HashMap iteration. Use `.as_str()` or let auto-deref handle the `contains` call. The spec uses `declared_version` directly as `contains(declared_version)` — this will compile because `String: Pattern` in the `str::contains` trait. Both approaches are valid.

### Pattern D: Output routing — stderr vs stdout

**Critical note for the builder and test author.**

All `output::*` functions write to **stderr** via `eprintln!`. The diff lines
themselves (tool_diffs, mcp_diffs, secret_diffs) are flushed to **stdout** via `println!`.
This is the routing already in the file and the spec's test section (Test 3, line 586-592 of spec) explicitly confirms this:

- `output::header`, `output::info`, `output::success`, `output::error`, `output::warning` → **stderr**
- `println!("{}", diff)` within the flush loops → **stdout**

The unresolved secret refs block (lines 181-187) also uses `println!` directly (not `output::*`), so it goes to **stdout** as well.

Integration tests must assert:
- `.stderr(predicate::str::contains(...))` for headers and summary lines
- `.stdout(predicate::str::contains(...))` for `+`, `~`, `-` marker lines

---

## 4. Test Infrastructure

### Confirmed imports in `tests/cli_smoke.rs` (lines 1-3):
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
```

All three are already present. No new test imports needed.

### Helper function (line 6-8):
```rust
fn great() -> Command {
    Command::cargo_bin("great").expect("binary exists")
}
```

### TempDir pattern (established across all existing tests):
```rust
let dir = TempDir::new().unwrap();
std::fs::write(
    dir.path().join("great.toml"),
    r#"[project]\nname = "test"\n..."#,
).unwrap();
great()
    .current_dir(dir.path())
    .arg("diff")
    .assert()
    .success()
    .stderr(predicate::str::contains("..."));
```

### `.not()` predicate pattern (used in `diff_disabled_mcp_skipped`):
```rust
.stdout(predicate::str::contains("disabled-server").not())
```
This pattern exists in the test suite (confirmed used elsewhere in the statusline tests implicitly, and the `.not()` combinator is valid on `predicates` 3.0).

### Existing diff test location: lines 119-131

```
119: // -----------------------------------------------------------------------
120: // Diff
121: // -----------------------------------------------------------------------
122:
123: #[test]
124: fn diff_no_config_shows_error() {
125:     let dir = TempDir::new().unwrap();
126:     great()
127:         .current_dir(dir.path())
128:         .arg("diff")
129:         .assert()
130:         .success()              // <-- MUST become .failure()
131:         .stderr(predicate::str::contains("great.toml"));
132: }
```

The next section after line 132 is:
```
133: // -----------------------------------------------------------------------
134: // Template
135: // -----------------------------------------------------------------------
```

All 7 new tests (Tests 2-8 from spec) are inserted between line 132 and line 133.

---

## 5. Dependency Confirmation

From `/home/isaac/src/sh.great/Cargo.toml`:

| Crate | Version | Status |
|-------|---------|--------|
| `colored` | `3.0` | CONFIRMED present in `[dependencies]` |
| `assert_cmd` | `2.0` | CONFIRMED present in `[dev-dependencies]` |
| `predicates` | `3.0` | CONFIRMED present in `[dev-dependencies]` |
| `tempfile` | `3.0` | CONFIRMED present in `[dev-dependencies]` |
| `anyhow` | `1.0` | CONFIRMED present |
| `clap` | `4.5` | CONFIRMED present |

No new dependencies required. All needed crates are already in `Cargo.toml`.

**`colored` 3.0 trait methods confirmed available:**
- `.green()` — already used in diff.rs lines 69, 82, 110 (green `+`)
- `.yellow()` — already used in diff.rs line 124 (yellow `~`)
- `.red()` — Gap 5 introduces this; it is in the same `Colorize` trait (imported on line 3) as `.green()` and `.yellow()`. No additional import needed.
- `.bold()` — already used in diff.rs lines 70, 83, 112, 126 etc.
- `.dimmed()` — already used throughout

---

## 6. Risk Flags

### Risk 1: Line numbers shift during sequential edits — HIGH

The spec states (spec line 33): "When applying changes, the builder should use the BEFORE code snippets as anchors for finding the replacement targets, not the line numbers." This is correct. The spec's stated line numbers are approximate. Apply changes using the BEFORE text as the anchor string, not line numbers. Edit tool matches on exact string content.

**Recommended edit order** (from spec build order, validated against actual file):
1. Add `use crate::cli::util;` import — anchored to `use crate::cli::output;`
2. Add 3 counter declarations — anchored to `let mut has_diff = false;`
3. Gap 3: MCP disabled guard — anchored to `for (name, mcp) in mcps {`
4. Gap 2: exit code — anchored to `output::error("No great.toml found..."); return Ok(());`
5. Gap 5: red `-` for `secrets.required` — anchored to `"+".green(),` in the secrets for-loop
6. Gap 5: red `-` for secret refs — anchored to `"+".green(),` in the `for name in &unresolved_refs` block
7. Gap 1: runtimes block — anchored to `for (name, declared_version) in &tools.runtimes {`
8. Gap 1: CLI tools block — anchored to `if let Some(cli_tools) = &tools.cli {`
9. Tools section header — anchored to `output::header("Tools — need to install:");`
10. Gap 4: summary block — anchored to `if !has_diff {`
11. Docstring update — anchored to `/// - \`~\` (yellow) — partially configured, needs attention`

### Risk 2: Two distinct `"+".green()` occurrences in secrets — MEDIUM

There are two separate locations where secrets use `"+".green()`:
- Lines 148-154: inside `for key in required {` (the `secrets.required` block)
- Lines 181-187: inside `for name in &unresolved_refs {` (the MCP secret refs block)

Both must be changed to `"-".red()`. The Edit tool's `replace_all` would erroneously also replace the tool diff push sites (lines 69, 82) which legitimately use `"+".green()`. Use targeted replacements with sufficient surrounding context to distinguish the two secrets sites from the tools sites.

The distinguishing context:
- Tools site: `name.bold(),` followed by `format!("(need {})", declared_version).dimmed()`
- Secrets required site: `key.bold(),` followed by `"(not set in environment)".dimmed()`
- Secrets refs site: `name.bold(),` followed by `"(referenced in MCP env, not set)".dimmed()`

### Risk 3: `declared_version` type is `&String` in HashMap iteration — LOW

In `for (name, declared_version) in &tools.runtimes`, both `name` and `declared_version` are `&String`. The expression `declared_version != "latest"` compiles via `PartialEq<str>` for `String`. The expression `actual.contains(declared_version)` where `actual: String` and `declared_version: &String` compiles via `str::contains<&str>` with auto-deref. No explicit `.as_str()` needed, but adding it (`.contains(declared_version.as_str())`) is also correct and explicit.

### Risk 4: `has_diff` vs counter tracking — LOW

The existing `has_diff` flag is separate from the counters. `has_diff = true` must remain in the existing push locations (the `if !tool_diffs.is_empty()` / `if !mcp_diffs.is_empty()` / `if !secret_diffs.is_empty()` blocks at lines 91, 132, 158). The counters are incremented at the individual push sites inside the loops, before or alongside each `tool_diffs.push` / `mcp_diffs.push` / `secret_diffs.push` call. Do NOT replace `has_diff` with the counters — both serve different roles (`has_diff` gates section printing; counters gate the summary format).

The `unresolved_refs` section (lines 177-189) sets `has_diff = true` and has inline `println!` calls rather than a deferred flush buffer. `secrets_count += 1` is added inside the `for name in &unresolved_refs` loop, immediately before each `println!`.

### Risk 5: Test 1 is a replacement, not an addition — MEDIUM

The spec replaces `diff_no_config_shows_error` (lines 122-131) rather than appending a new test. If the builder appends instead of replaces, two tests with contradictory assertions will exist (`success()` vs `failure()`) and one will fail. The builder must identify and replace lines 122-131 entirely.

### Risk 6: `predicate::str::contains("...").not()` syntax — LOW

The `.not()` method is a combinator on the `Predicate` trait in `predicates` 3.0. The import `use predicates::prelude::*;` brings `PredicateStrExt` into scope, which includes `.not()` on string predicates. This is confirmed working in the test file for boolean logic. No issue expected.

### Risk 7: Counter variable naming collision — LOW

The variable `configure_count` is introduced by this task. The name does not conflict with any existing variable in `diff.rs`. However, `install_count` is a common English word — verify no shadowing issue with any loop variable. There is none: the loops use `name`, `declared_version`, `mcp`, `key`, `ref_name` as their iteration variables.

---

## 7. Recommended Build Order

Per spec build order (validated against actual file structure):

1. **Import**: add `use crate::cli::util;` after `use crate::cli::output;` in diff.rs
2. **Counter variables**: add 3 `let mut` declarations after `let mut has_diff = false;`
3. **Gap 3** (simplest, 3 lines): MCP disabled guard
4. **Gap 2** (1 line change): `return Ok(())` → `std::process::exit(1)`
5. **Gap 5 part 1**: `secrets.required` block — `"+".green()` → `"-".red()` + `secrets_count += 1`
6. **Gap 5 part 2**: secret refs block — `"+".green()` → `"-".red()` + `secrets_count += 1`
7. **Gap 1 part 1**: runtimes loop — replace entire loop body
8. **Gap 1 part 2**: CLI tools loop — replace entire `if let Some(cli_tools)` block
9. **Section header**: `"Tools — need to install:"` → `"Tools"`
10. **MCP counter increments**: add `configure_count += 1` at both MCP push sites
11. **Gap 4**: replace summary block
12. **Docstring**: add `-` (red) line to doc comment
13. **Test replacement**: replace `diff_no_config_shows_error` with `diff_no_config_exits_nonzero`
14. **New tests 2-8**: append in the diff section after the replaced test
15. `cargo clippy` — expect zero warnings
16. `cargo test` — all 8 diff tests plus existing suite must pass

---

## 8. Dependency Map

```
diff.rs change depends on:
  crate::cli::util          (already exists, add import only)
  crate::cli::output        (already imported, no change)
  crate::config             (already imported, no change)
  crate::platform::command_exists (already imported, no change)
  colored::Colorize         (already imported, .red() in same trait)
  std::process::exit        (no import needed, called as qualified path)

tests/cli_smoke.rs changes depend on:
  assert_cmd::Command       (already imported)
  predicates::prelude::*    (already imported, includes .not())
  tempfile::TempDir         (already imported)
  the great binary           (built by cargo test)
```

No new modules, no new files, no cross-crate dependencies. All changes are self-contained within two files.
