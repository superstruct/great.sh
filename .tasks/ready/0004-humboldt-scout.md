# 0004: Humboldt Scout Report -- `great status` Command

**Scout:** Alexander von Humboldt
**Spec:** `/home/isaac/src/sh.great/.tasks/ready/0004-status-command-spec.md`
**Socrates verdict:** APPROVED (8 advisory concerns, 0 blockers)
**Date:** 2026-02-24

---

## Territory Map

### Files to Modify (Exact)

| File | Lines | Role |
|------|-------|------|
| `/home/isaac/src/sh.great/src/cli/status.rs` | 216 | Primary: full rewrite of `run()`, `run_json()`, `print_tool_status()` |
| `/home/isaac/src/sh.great/tests/cli_smoke.rs` | 1211 | Append 12 new tests in `// Status -- expanded (task 0004)` section |

No new files. No `Cargo.toml` changes.

### Files Referenced But Not Modified

| File | Lines | Why relevant |
|------|-------|--------------|
| `/home/isaac/src/sh.great/src/cli/output.rs` | 43 | `output::{header, info, success, warning, error}` -- all five used in status.rs |
| `/home/isaac/src/sh.great/src/cli/mod.rs` | 77 | `Command::Status(status::Args)` dispatch at line 48 |
| `/home/isaac/src/sh.great/src/main.rs` | 30 | `Command::Status(args) => cli::status::run(args)` at line 19 |
| `/home/isaac/src/sh.great/src/config/mod.rs` | 119 | `discover_config()` (line 44) and `load()` (line 18) -- both already used |
| `/home/isaac/src/sh.great/src/config/schema.rs` | 803 | `GreatConfig` struct and all sub-types |
| `/home/isaac/src/sh.great/src/platform/mod.rs` | 65 | Re-exports `PlatformInfo`, `command_exists`, `detect_platform_info` |
| `/home/isaac/src/sh.great/src/platform/detection.rs` | 592 | `PlatformInfo`, `PlatformCapabilities`, `command_exists` impl |
| `/home/isaac/src/sh.great/src/cli/doctor.rs` | 738 | Pattern reference; contains duplicate `get_command_version` |

---

## Dispatch Trace

```
src/main.rs:19       Command::Status(args) => cli::status::run(args)
src/cli/mod.rs:48    Command::Status(status::Args)
src/cli/status.rs:26 pub fn run(args: Args) -> Result<()>
```

`status::Args` is a flat struct with two fields: `verbose: bool` (long `--verbose`, short `-v`) and `json: bool` (long `--json`). No subcommands.

---

## Key Functions in `src/cli/status.rs` (Current, 216 lines)

| Function | Lines | What changes |
|----------|-------|--------------|
| `pub fn run(args: Args)` | 26-161 | Full restructure: hoist config discovery above `--json` branch, add `has_critical_issues` tracker, call `process::exit(1)` at end when needed |
| `fn run_json(info)` | 164-172 | Replace: add `config_path: Option<&str>` and `config: Option<&GreatConfig>` params; serialize full `StatusReport` struct via `serde_json::to_string_pretty` |
| `fn get_tool_version(tool)` | 178-197 | NO CHANGE (task 0005 boundary). Stays in place. |
| `fn print_tool_status(...)` | 203-215 | Add `verbose: bool` param; verbose=true shows full version string, verbose=false shows last whitespace token |

---

## Duplicate Function (Technical Debt)

The function `get_tool_version` in `src/cli/status.rs` (lines 178-197) is a near-exact duplicate of `get_command_version` in `src/cli/doctor.rs` (lines **719-738**).

Both have identical logic:
```rust
fn get_command_version(cmd: &str) -> Option<String> {  // doctor.rs:719
fn get_tool_version(tool: &str) -> Option<String> {    // status.rs:178
```
Both: spawn `cmd --version`, capture stdout, return first non-empty line trimmed.

The spec explicitly forbids extraction in task 0004 -- that is task 0005. Do not touch the duplicate.

---

## Existing Patterns to Follow

### Import conventions in `status.rs` (current lines 1-6)

```rust
use anyhow::Result;
use clap::Args as ClapArgs;
use crate::cli::output;
use crate::config;
use crate::platform::{self, command_exists};
```

New additions needed:
```rust
use serde::Serialize;
```
No `use serde_json;` needed in production code -- it is called as `serde_json::to_string_pretty(...)` with full path (matches pattern in `src/cli/statusline.rs` which does the same).

### `serde_json` usage in the codebase

`serde_json` is in `[dependencies]` (not just `[dev-dependencies]`), so it is available everywhere. It is heavily used:
- `src/cli/loop_cmd.rs` -- `serde_json::json!`, `to_string_pretty`, `from_str`
- `src/cli/apply.rs` -- `serde_json::Value`, `json!`, `to_string_pretty`
- `src/cli/statusline.rs` -- `serde_json::from_slice`, `from_str` (with `#[derive(Deserialize)]`)
- `src/cli/update.rs` -- `serde_json::Value`
- `src/mcp/mod.rs` -- `to_string_pretty`, `from_str`
- `src/platform/detection.rs` -- only in `#[cfg(test)]`

Pattern: `serde_json` is called with full path, never imported with `use`. Follow this.

### `serde` on structs

All serializable structs use `#[derive(Serialize)]`. Optional fields that may be absent use `#[serde(skip_serializing_if = "Option::is_none")]`. This matches the pattern in `src/config/schema.rs`.

### `output::*` functions

All write to **stderr** (`eprintln!`). The `run_json` path writes to **stdout** (`println!`). This split is correct and must be maintained -- tests currently check stderr for human-readable output and stdout for JSON.

### `process::exit(1)`

This is a new pattern in production code -- currently zero uses across `src/`. Every other subcommand uses `anyhow::Result` propagation. The spec documents the rationale: `status` must print its full report before exiting non-zero, which is incompatible with mid-stream `bail!()`. Add a `// NOTE` comment in the code per Socrates advisory #2.

### Config loading pattern

Used identically in `src/cli/doctor.rs` (lines 459-499) and current `status.rs` (lines 67-82):
```rust
match config::discover_config() {
    Ok(path) => { ... config::load(Some(path_str)) ... }
    Err(_) => { /* warn: no config */ }
}
```
The spec restructures this to hoist it before the `--json` branch and capture `(Option<String>, Option<GreatConfig>)` as a tuple.

---

## Config Schema Available in `GreatConfig`

Fields the new `run_json()` will iterate:
- `cfg.tools` → `Option<ToolsConfig>` with `runtimes: HashMap<String, String>` (flattened) and `cli: Option<HashMap<String, String>>`
- `cfg.agents` → `Option<HashMap<String, AgentConfig>>` with `.provider`, `.model`
- `cfg.mcp` → `Option<HashMap<String, McpConfig>>` with `.command`, `.args`, `.transport`
- `cfg.secrets` → `Option<SecretsConfig>` with `.required: Option<Vec<String>>`

All types re-exported from `config::schema` via `pub use` in `config/mod.rs`. The `run_json` function signature uses `config::GreatConfig` directly (not `config::schema::GreatConfig`) -- this works because of the re-export.

---

## Test Patterns in `tests/cli_smoke.rs`

**Current structure (1211 lines):**
- Helper `fn great() -> Command` at line 6: `Command::cargo_bin("great")`
- Sections delimited by `// -----------------------------------------------------------------------` comments
- Existing status tests at lines 57-88 (3 tests)

**Existing status tests to preserve:**
- `status_shows_platform` (line 58): checks stderr contains `"Platform:"`
- `status_warns_no_config` (line 69): checks stderr contains `"No great.toml found"` -- IMPORTANT: the spec's restructured `run()` still prints this warning (at the `else` branch in the config section, line ~946 in spec's Section 6). Verify this survives the refactor.
- `status_json_outputs_json` (line 80): checks stdout contains `"platform"` -- will still pass since new JSON contains `"platform"` key

**Pattern for new tests:**
```rust
fn test_name() {
    let dir = TempDir::new().unwrap();
    // optionally: std::fs::write(dir.path().join("great.toml"), "...").unwrap();
    great()
        .current_dir(dir.path())
        .args(["status", ...])
        .assert()
        .success() / .failure()
        .stderr(predicate::str::contains("..."))
        .stdout(predicate::str::contains("..."));
}
```

**Tests using `serde_json`:** The 4 JSON-parsing tests call `serde_json::from_str::<serde_json::Value>(&stdout)`. No `use serde_json;` import needed in the test file -- use full path `serde_json::from_str(...)`. However, adding `use serde_json;` at the top of the test file is cleaner and matches how `assert_cmd` and `predicates` are imported. Socrates advisory #8 flags this -- builder should add the import.

---

## Dependency Map

```
status.rs
  -> anyhow (Result, anyhow!)
  -> clap (Args derive)
  -> serde (Serialize derive) [NEW]
  -> serde_json (to_string_pretty) [NEW call -- crate already in deps]
  -> crate::cli::output (header, info, success, warning, error)
  -> crate::config (discover_config, load, GreatConfig)
     -> crate::config::schema (GreatConfig, ToolsConfig, AgentConfig, McpConfig, SecretsConfig)
  -> crate::platform (detect_platform_info, command_exists, PlatformInfo)
     -> crate::platform::detection (PlatformInfo, PlatformCapabilities, Platform, Architecture)
  -> std::process (Command for get_tool_version, exit for exit code)
  -> std::env (var for secret checks)
```

No new crate dependencies. No tokio (status is fully synchronous).

---

## Risks and Advisories

### Risk 1: `status_warns_no_config` test compatibility (Socrates #4)
The existing test (line 69) asserts `stderr contains "No great.toml found"`. In the refactored `run()`, this warning moves from inside the config discovery block to a separate `else` branch after config discovery. Verify the message still fires on the non-json path with no config. The spec's Section 6 reference implementation shows the warning at the `else` branch -- it is preserved.

### Risk 2: `process::exit(1)` is a new production pattern (Socrates #2)
First use of `process::exit` in production code. Add a `// NOTE` comment explaining why `bail!()` cannot be used here (must print full report first). The pattern is technically sound: `eprintln!` is line-buffered and flushes on each call, so no buffering concern (Socrates #1).

### Risk 3: Version display heuristic `split_whitespace().last()` (Socrates #6)
For tools like `rustc` which output `rustc 1.77.0 (aedd173a2 2024-03-17)`, last token is `2024-03-17)` (a date). The spec chose this heuristic; the builder should implement it as specified. Task 0005 can improve it.

### Risk 4: `run_json` borrows `issues` in closures
The `run_json` function builds `issues: Vec<String>` mutably while calling `config.and_then(...)` closures that push to it. The spec uses `issues.push(...)` inside closures that borrow `issues` mutably. This is a **borrow-checker issue** -- the closures would be FnMut borrowing `issues` while `config` (also borrowed from a local) is in scope. The spec's Section 6 reference implementation handles this via direct `issues.push()` inside closures that capture `&mut issues`. In Rust, this requires the closures to be sequential (not concurrent), which they are. However, the borrow checker may still flag simultaneous mutable borrows if the closures are expressed as `and_then` chains. The builder should verify this compiles and may need to restructure into explicit `if let` blocks instead of chained `and_then`.

**Recommended mitigation:** Use `if let Some(tools) = config.and_then(|c| c.tools.as_ref())` patterns with explicit `for` loops where `issues.push()` is called, rather than closures. This avoids the borrow-checker friction.

### Risk 5: HashMap import in Section 3 is spurious (Socrates #3)
Section 3 of the spec includes `use std::collections::HashMap;` but the new structs do not use `HashMap`. Section 6 (the authoritative full rewrite) correctly omits it. The builder should follow Section 6.

---

## Recommended Build Order

1. Add `use serde::Serialize;` import and five new `#[derive(Serialize)]` structs above `Args` struct. Run `cargo build`.
2. Hoist config discovery to top of `run()`, returning `(Option<String>, Option<GreatConfig>)` tuple. Fix `unwrap_or_default()` with `?` propagation. Run `cargo build`.
3. Route `--json` branch to call `run_json(&info, config_path_str.as_deref(), config.as_ref())`. Run `cargo build`.
4. Implement expanded `run_json()`. Run `cargo build`. Run `cargo test --test cli_smoke status_json` to verify existing JSON test still passes.
5. Add `verbose: bool` param to `print_tool_status()`. Update all call sites (2 in runtime loop, 2 in cli loop). Add verbose MCP display. Run `cargo build`.
6. Add `has_critical_issues` tracker. Set to true in tool-not-installed and secret-missing branches. Add `if has_critical_issues { std::process::exit(1); }` before `Ok(())`. Run `cargo build`.
7. Run `cargo test --test cli_smoke` to verify all 3 existing status tests still pass. Fix any regressions before proceeding.
8. Append 12 new tests to `tests/cli_smoke.rs`. Add `use serde_json;` import at top of file. Run `cargo test --test cli_smoke` -- all tests should pass.
9. Run `cargo clippy` -- clean.

---

## Line Count Summary

| File | Before | Expected After |
|------|--------|----------------|
| `src/cli/status.rs` | 216 | ~280 (structs add ~60 lines, refactored functions comparable) |
| `tests/cli_smoke.rs` | 1211 | ~1470 (12 new tests ~260 lines) |
