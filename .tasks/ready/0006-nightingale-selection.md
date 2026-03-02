# Nightingale Selection: 0006 `great diff`

**Selected:** 2026-02-25
**Curator:** Florence Nightingale (Requirements Curator)

---

## Task ID and Title

**0006 -- `great diff`: Declared vs Actual State**

Module: `src/cli/diff.rs`
Priority: P1
Type: feature

---

## Confirmed Dependencies (all landed)

| Task | What it provides | Location confirmed |
|------|------------------|--------------------|
| 0001 | `platform::command_exists(cmd: &str) -> bool` | `src/platform/detection.rs:135` |
| 0002 | `config::discover_config()`, `config::load()`, `GreatConfig::find_secret_refs()` | `src/config/mod.rs`, `src/config/schema.rs:246` |
| 0003 | `output::success/warning/error/info/header`, `cli::util` module | `src/cli/output.rs`, `src/cli/util.rs` |
| 0004 | `util::get_command_version(cmd: &str) -> Option<String>` extracted to shared util | `src/cli/util.rs:9` |
| 0005 | `McpConfig.enabled: Option<bool>` field confirmed; skip-disabled pattern confirmed | `src/cli/doctor.rs:567` |

---

## Current State of `src/cli/diff.rs`

The file is NOT a stub. It is a substantial working implementation (~198 lines) with:

- Config discovery and loading (with `--config` flag support)
- Tools diff loop (runtimes + CLI tools), missing tools shown with `+` (green)
- MCP servers diff loop, missing commands shown with `+`, command-available-but-no-.mcp.json shown with `~`
- Secrets diff: iterates `secrets.required` and `find_secret_refs()` results
- Summary line ("nothing needed" vs "run `great apply`")

The task file previously described this as "a stub" -- that description was stale and has been corrected.

---

## Gaps Identified (what remains to implement)

### Gap 1: Version comparison missing from tools diff

**Current behavior:** Tools that ARE installed are silently omitted from output. No version comparison occurs.

**Required behavior:** Call `util::get_command_version(name)` for installed tools. Compare the result against the declared version string. Show a `~` (yellow) line for version mismatches. Show a dim/grey "ok" line (or simply omit with a count) for matching tools.

**Import needed:** `use crate::cli::util;` -- not currently present in `diff.rs`.

### Gap 2: Exit code 0 instead of 1 on missing config

**Current behavior** (`diff.rs` lines 35-38):
```rust
Err(_) => {
    output::error("No great.toml found. Run `great init` to create one.");
    return Ok(());   // exits 0
}
```

**Required behavior:** Exit with code 1. Pattern from `status.rs`/`doctor.rs`:
```rust
output::error("No great.toml found. Run `great init` to create one.");
std::process::exit(1);
```

### Gap 3: Disabled MCP servers not skipped

**Current behavior:** All entries in the `mcp` map are processed regardless of `enabled` field.

**Required behavior:** Skip servers where `mcp.enabled == Some(false)`, consistent with `doctor.rs`.

---

## Updated File References

The task file previously referenced `get_tool_version()` as living in `status.rs`. This is now
incorrect. The correct reference is:

- **Old:** `get_tool_version()` in `src/cli/status.rs`
- **Correct:** `util::get_command_version()` in `src/cli/util.rs`

All other references in the task file were accurate and have been preserved.

---

## Scope Boundary

### In scope for this task

- Add `use crate::cli::util;` import to `diff.rs`
- Call `util::get_command_version()` for installed tools and show `~` on version mismatch
- Change no-config exit path from `return Ok(())` to `std::process::exit(1)`
- Skip disabled MCP servers (`enabled == Some(false)`)
- Numeric summary ("N items to install, M items to configure, K secrets to resolve")
- Integration tests covering the three key scenarios (missing tool, satisfied config, no config)

### Out of scope for this task

- Defining `DiffResult` struct for use by `apply` (architectural suggestion, deferred to 0009)
- Writing or modifying `.mcp.json` (read-only check only)
- `great apply` implementation (task 0009, unblocked once this lands)
- Any changes to `status.rs`, `doctor.rs`, `util.rs`, or config modules

---

## Risk Assessment

Low. The implementation is mostly written. The two functional gaps are small, well-understood
changes. No new dependencies are required -- `util` is already a peer module. The exit-code fix
is a one-line change. The version comparison adds roughly 10-15 lines following patterns already
in `status.rs`.

The integration test for "diff without config exits with code 1" requires an `assert_cmd` test
that checks `.failure()` and exit code. The existing test suite in `tests/` already uses
`assert_cmd` patterns from tasks 0004 and 0005.
