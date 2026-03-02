# 0005: `great doctor` — Humboldt Scout Report

**Task:** 0005-doctor-command
**Scout:** Humboldt (Codebase Scout)
**Date:** 2026-02-24
**Spec:** `/home/isaac/src/sh.great/.tasks/ready/0005-doctor-command-spec.md`
**Socrates review:** `/home/isaac/src/sh.great/.tasks/ready/0005-socrates-review.md`

---

## 1. Primary File: `src/cli/doctor.rs` (739 lines)

**Path:** `/home/isaac/src/sh.great/src/cli/doctor.rs`

### Structure map

| Lines | Item | Change needed |
|-------|------|---------------|
| 1-7 | imports | Add `util` to `crate::cli::{bootstrap, output, tuning}` group |
| 9-60 | `DiagnosticResult`, `FixableIssue`, `FixAction` structs | No change |
| 62-234 | `pub fn run(args: Args) -> Result<()>` | Restructure: capture `check_config` return, add MCP check call, add `process::exit(1)` block |
| 236-249 | `fn pass/warn/fail` helpers | No change |
| 251-334 | `fn check_platform` | No change |
| 336-413 | `fn check_essential_tools` | Line 362: `get_command_version(cmd)` → `util::get_command_version(cmd)` |
| 415-454 | `fn check_ai_agents` | No change |
| 456-518 | `fn check_config(result) -> ()` | Signature changes to `-> Option<config::GreatConfig>`; line 462 `unwrap_or_default()` replaced with match |
| 518 | (insertion point) | New `fn check_mcp_servers()` inserted here |
| 520-543 | `fn check_shell` | No change |
| 545-635 | `fn check_system_prerequisites` | No change |
| 637-681 | `fn check_docker` | No change |
| 683-713 | `fn check_system_tuning` | No change |
| 715-738 | `fn get_command_version` (private) | **DELETE entire function** |

### Critical line: `unwrap_or_default()` at line 462
```rust
// Current (line 462):
match config::load(Some(path.to_str().unwrap_or_default())) {
// Replace with match on path.to_str() per spec Section 2c
```

### `run()` restructure: 3 changes
1. Line 91: `check_config(&mut result);` → `let loaded_config = check_config(&mut result);`
2. After that line: add new block:
   ```rust
   if let Some(ref cfg) = loaded_config {
       check_mcp_servers(&mut result, cfg);
   }
   ```
3. Before final `Ok(())`: add `process::exit(1)` block (same NOTE comment pattern as `status.rs`)

---

## 2. Duplicate to Extract: `src/cli/status.rs` (482 lines)

**Path:** `/home/isaac/src/sh.great/src/cli/status.rs`

### `get_tool_version` call sites (all 4 use identical pattern `get_tool_version(name)`)

| Line | Context |
|------|---------|
| 182 | `runtimes` loop, human-readable mode |
| 200 | `cli_tools` loop, human-readable mode |
| 320 | `runtimes` loop, JSON mode |
| 336 | `cli_tools` loop, JSON mode |

Note: spec said lines 182, 199, 320, 335 — Socrates caught the off-by-one; actual lines are 182, 200, 320, 336. Use `replace_all` to avoid line number fragility.

### Function to delete: lines 436-455
```rust
fn get_tool_version(tool: &str) -> Option<String> { ... }  // lines 436-455
```

### Import change: line 5-7
```rust
// Current:
use crate::cli::output;
// New:
use crate::cli::{output, util};
```

`command_exists` import at line 7 is unchanged: `use crate::platform::{self, command_exists};`

---

## 3. Module Registry: `src/cli/mod.rs` (77 lines)

**Path:** `/home/isaac/src/sh.great/src/cli/mod.rs`

**Change:** Insert `pub mod util;` after line 14 (`pub mod update;`), maintaining alphabetical order.

Current lines 14-15:
```rust
pub mod update;
pub mod vault;
```
New:
```rust
pub mod update;
pub mod util;
pub mod vault;
```

No other changes to `mod.rs`.

---

## 4. New File to Create: `src/cli/util.rs`

**Path:** `/home/isaac/src/sh.great/src/cli/util.rs` — does not yet exist.

Contains exactly one public function: `pub fn get_command_version(cmd: &str) -> Option<String>`.

The implementation is byte-for-byte identical to both `doctor.rs:719-738` and `status.rs:436-455`. Copy from either.

---

## 5. Supporting Modules (Read-Only, No Changes)

### `src/cli/bootstrap.rs` (491 lines)
- `pub fn is_apt_distro(platform: &Platform) -> bool` — used by `doctor.rs`
- `pub fn is_linux_like(platform: &Platform) -> bool` — used by `doctor.rs`
- `pub fn ensure_curl/git/build_essential/unzip/docker/claude_code` — called in `run()` fix block
- No changes needed.

### `src/cli/tuning.rs` (109 lines)
- `pub fn apply_system_tuning(dry_run: bool, info: &PlatformInfo)` — called in `run()` fix block
- `pub fn check_inotify_watches() -> (Option<u64>, bool)` — called by `check_system_tuning()`
- No changes needed.

### `src/config/mod.rs` (120 lines)
- `pub fn discover_config() -> Result<PathBuf>` — used in `check_config()`
- `pub fn load(path: Option<&str>) -> Result<GreatConfig>` — used in `check_config()`
- **Advisory (Socrates #3):** `config::load()` already calls `cfg.validate()` and bails on `ConfigMessage::Error`. The `check_config` double-validation is dead code for errors — only warnings can appear in the second pass. Not a blocker. Add a comment.

### `src/config/schema.rs` (key types used)
- `GreatConfig.mcp: Option<HashMap<String, McpConfig>>` — used by `check_mcp_servers()`
- `McpConfig.command: String` — non-optional, safe to access directly
- `McpConfig.enabled: Option<bool>` — verified present at line 107
- `McpConfig.transport: Option<String>` — used in `check_mcp_servers()` pass message
- `config::schema::ConfigMessage::Warning(w)` / `::Error(e)` — used in `check_config()`
- `GreatConfig::validate()` returns `Vec<ConfigMessage>` — called in `check_config()`
- `GreatConfig::find_secret_refs()` — called in `check_config()`

### `src/platform/mod.rs`
- `command_exists` is re-exported from `detection.rs` via `pub use detection::{command_exists, ...}`
- Import in `doctor.rs` (line 7): `use crate::platform::{self, command_exists, Platform, PlatformInfo};`
- This import is unchanged — `command_exists` is already in scope for `check_mcp_servers()`

---

## 6. Integration Tests: `tests/cli_smoke.rs`

**Path:** `/home/isaac/src/sh.great/tests/cli_smoke.rs`

### Existing doctor tests — all locations

| Lines | Function | Currently asserts | Action needed |
|-------|----------|-------------------|---------------|
| 94-105 | `doctor_runs_diagnostics` | `.success()` | **Remove `.success()`** (spec 4e) |
| 107-117 | `doctor_fix_runs_without_crash` | `.success()` + `#[ignore]` | Leave as-is; `#[ignore]` protects CI; note for future unmask |
| 298-307 | `doctor_checks_system_prerequisites` | `.success()` | **Remove `.success()`** (Socrates advisory #1) |
| 309-318 | `doctor_checks_docker` | `.success()` | **Remove `.success()`** (Socrates advisory #1) |

All 3 active tests assert content via `.stderr(predicate::str::contains(...))` — those assertions are valid and unchanged.

### 4 new tests — append after line 318

```
doctor_with_mcp_config_checks_servers  — asserts "MCP Servers", "test-server"; no exit code assertion
doctor_mcp_missing_command_fails       — asserts "not found on PATH"; no exit code assertion
doctor_exits_nonzero_on_failure        — asserts `.failure()` + "Summary"
doctor_with_valid_config               — asserts "great.toml: found at", "great.toml: valid syntax"
```

Section delimiter to add: `// -----------------------------------------------------------------------`

---

## 7. Dependency Map

```
src/cli/util.rs (NEW)
    └── no imports from project

src/cli/mod.rs
    └── declares pub mod util (ADD)

src/cli/doctor.rs
    ├── crate::cli::{bootstrap, output, tuning, util}  (util ADDED)
    ├── crate::config  (unchanged)
    ├── crate::platform::package_manager  (unchanged)
    ├── crate::platform::{self, command_exists, Platform, PlatformInfo}  (unchanged)
    └── calls util::get_command_version  (replaces local fn)

src/cli/status.rs
    ├── crate::cli::{output, util}  (util ADDED)
    ├── crate::config  (unchanged)
    ├── crate::platform::{self, command_exists}  (unchanged)
    └── calls util::get_command_version  (replaces local fn)
```

No circular dependencies. `util.rs` has zero project imports — pure std.

---

## 8. Risks

### Risk 1: Three tests assert `.success()` and will break on CI (HIGH)
**Location:** `tests/cli_smoke.rs` lines 101, 305, 315.

The spec addresses line 101 (`doctor_runs_diagnostics`) but Socrates flagged that lines 305 and 315 also assert `.success()`. On any CI runner without Homebrew or Docker, `great doctor` will exit 1 after this change. All three `.success()` assertions must be removed.

### Risk 2: `check_config()` has only one call site (VERIFIED SAFE)
`fn check_config` is private (no `pub`). `grep` of the codebase confirms it is called only from `run()` at line 91. The return-type change from `()` to `Option<config::GreatConfig>` affects exactly one call site.

### Risk 3: Double validation in `check_config()` (LOW, pre-existing)
`config::load()` at `src/config/mod.rs:28-38` already calls `validate()` and bails on `ConfigMessage::Error`. The second `validate()` call in `check_config()` can only observe `Warning` messages. The `ConfigMessage::Error` arm in the doctor's second pass is dead code. Not a regression — existing behavior is identical. Add a comment per Socrates advisory #3.

### Risk 4: `process::exit(1)` skips Drop/flush (ESTABLISHED PATTERN)
`status.rs` uses the identical pattern at lines 283-288 with the same NOTE comment. All output via `output::*` helpers uses `eprintln!` which flushes on each call. Risk is documented and accepted.

### Risk 5: `doctor_fix_runs_without_crash` will fail if ever un-ignored
The `--fix` path exits 1 when `result.checks_failed > 0` (pre-fix counter). The test asserts `.success()`. Currently protected by `#[ignore]`. Leave the `#[ignore]` in place; note in a code comment that the `.success()` assertion needs updating before un-ignoring.

---

## 9. Recommended Build Order

Execute each step, verify `cargo build && cargo clippy -- -D warnings` before next step.

1. **Create** `/home/isaac/src/sh.great/src/cli/util.rs` — copy `get_command_version` body from doctor.rs
2. **Add** `pub mod util;` to `/home/isaac/src/sh.great/src/cli/mod.rs` (after `pub mod update;`)
3. **Update** `/home/isaac/src/sh.great/src/cli/status.rs`:
   - Change import line 5 to include `util`
   - `replace_all` `get_tool_version(name)` → `util::get_command_version(name)` (4 sites)
   - Delete `get_tool_version` function (lines 436-455)
4. **Update** `/home/isaac/src/sh.great/src/cli/doctor.rs` imports — add `util` (line 4)
5. **Update** `/home/isaac/src/sh.great/src/cli/doctor.rs` `check_essential_tools` — line 362 call site
6. **Delete** `get_command_version` from doctor.rs (lines 715-738)
7. **Rewrite** `check_config()` signature and body — fix `unwrap_or_default`, return `Option<GreatConfig>`
8. **Add** `check_mcp_servers()` function after `check_config()` in doctor.rs
9. **Update** `run()` — capture `loaded_config`, add MCP call, add `process::exit(1)` block
10. **Update tests** — remove `.success()` from 3 existing tests; add 4 new tests

Steps 1-2 may produce a `dead_code` warning (util.rs unused until step 3-4). This is a warning, not an error — clippy `-D warnings` will fail only if step 3 or 4 is not done in the same cargo invocation. To avoid, complete steps 1-4 before running clippy.

---

## 10. Technical Debt Flagged (Pre-Existing, Do Not Fix in This Task)

- **Duplicate regex** (`r"\$\{([A-Z_][A-Z0-9_]*)\}"`): `schema.rs`, `cli/apply.rs`, `mcp/mod.rs` — tracked in memory
- **`get_tool_version` / `get_command_version` duplication**: This task resolves it
- **Double validation** in `check_config()` (Socrates advisory #3): Pre-existing, add comment only
- **`process::exit(1)` in `status.rs`**: Pre-existing pattern; `doctor.rs` correctly replicates it
