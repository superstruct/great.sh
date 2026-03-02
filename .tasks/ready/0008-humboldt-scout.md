# Scout Report 0008: Runtime Version Manager Integration (mise)

**Scout:** Alexander von Humboldt
**Date:** 2026-02-25
**Spec:** `.tasks/ready/0008-runtime-manager-spec.md` (APPROVED, Round 2)
**Target:** `/home/isaac/src/sh.great/src/platform/runtime.rs` -- targeted modification only

---

## Verdict

The approved spec is precise. The file already exists and compiles. Three surgical edits
plus test expansions. No new files. No import changes. No consumer breakage.

---

## Primary File: `/home/isaac/src/sh.great/src/platform/runtime.rs`

253 lines. Complete symbol inventory:

| Lines | Symbol | Kind | Disposition |
|-------|--------|------|-------------|
| 1 | `use anyhow::{bail, Context, Result};` | import | KEEP |
| 3 | `use super::detection::command_exists;` | import | KEEP -- `command_exists` already imported; Change 2 uses it directly |
| 6-11 | `ProvisionResult` | pub struct | KEEP -- fields: `name: String`, `declared_version: String`, `action: ProvisionAction` |
| 13-23 | `ProvisionAction` | pub enum | KEEP -- variants: `AlreadyCorrect`, `Installed`, `Updated`, `Failed(String)` |
| 26 | `MiseManager` | pub struct (unit) | KEEP |
| 30-32 | `MiseManager::is_available()` | `pub fn() -> bool` | KEEP -- wraps `command_exists("mise")` |
| 35-54 | `MiseManager::version()` | `pub fn() -> Option<String>` | KEEP -- runs `mise --version`, parses first line |
| 57-78 | `MiseManager::ensure_installed()` | `pub fn() -> Result<()>` | **MODIFY** (Change 2) |
| 81-109 | `MiseManager::install_runtime()` | `pub fn(name, version) -> Result<()>` | KEEP -- runs `mise install <spec>` then `mise use --global <spec>` |
| 112-131 | `MiseManager::installed_version()` | `pub fn(name) -> Option<String>` | **MODIFY** (Change 3) -- lines 120-130 only |
| 133-143 | `MiseManager::version_matches()` | `pub fn(declared, installed) -> bool` | **MODIFY** (Change 1) -- body replacement |
| 145-163 | `MiseManager::provision_from_config()` | `pub fn(tools: &ToolsConfig) -> Vec<ProvisionResult>` | KEEP -- skips "cli" key, calls `provision_single` per entry |
| 165-203 | `provision_single()` (private) | `fn(name, declared_version) -> ProvisionResult` | KEEP -- calls `installed_version`, `version_matches`, `install_runtime` |
| 210-252 | `mod tests` (6 tests) | test module | **EXPAND** -- add 8 new tests, modify 1 existing |

### Fully-qualified path usage (no top-level `use std::process`)

`std::process::Command` and `std::process::Stdio` appear as inline fully-qualified paths
on lines 37, 63, 89, 99, 113. This is the existing project pattern in this file. The new
`ensure_installed` code follows the same style.

---

## Change Map (exact line numbers)

### Change 1 -- BUG FIX: `version_matches` (lines 137-143)

Replace the body of `version_matches`. The entire function (lines 137-143) is replaced:

```
BEFORE (line 142):
    installed.starts_with(declared)

AFTER:
    if installed == declared { return true; }
    if installed.starts_with(declared) {
        let rest = &installed[declared.len()..];
        return rest.starts_with('.');
    }
    false
```

Bug: `"3.120.0".starts_with("3.12")` is `true`. Fix enforces dot-boundary after prefix.

### Change 2 -- BEHAVIORAL CHANGE: `ensure_installed` (lines 57-78)

Replace the entire function body. When `command_exists("brew")` is true, use
`brew install mise` before falling back to the curl installer. The function
signature `pub fn ensure_installed() -> Result<()>` is unchanged.

Key: `command_exists` is already imported (line 3). No new imports needed.

### Change 3 -- BUG FIX: `installed_version` (lines 120-130)

Replace the inner `if` condition only:

```
BEFORE (line 123):
    if version.is_empty() || version.contains("No version") {

AFTER:
    let lower = version.to_lowercase();
    if version.is_empty()
        || lower.contains("no version")
        || lower.contains("not installed")
    {
```

New mise (2025.x+) outputs `"Not installed"`. Case-insensitive check via `to_lowercase()`
covers all observed variants.

### Test Modifications (lines 238-241)

`test_version_no_match` gains two additional `assert!` calls:

```rust
assert!(!MiseManager::version_matches("22", "220.0.0"));
assert!(!MiseManager::version_matches("3.12", "3.120.0"));
```

### New Tests (appended after line 251, before closing `}` of `mod tests`)

8 new functions in this order:

| # | Name | Type |
|---|------|------|
| 7 | `test_version_matches_exact_only_declared` | pure logic |
| 8 | `test_version_matches_stable_keyword` | pure logic |
| 9 | `test_version_matches_no_false_longer_prefix` | pure logic (regression) |
| 10 | `test_version_matches_partial_major` | pure logic |
| 11 | `test_version_does_not_panic` | no-panic |
| 12 | `test_installed_version_nonexistent_runtime` | no-panic |
| 13 | `test_installed_version_does_not_panic` | no-panic |
| 14 | `test_provision_skips_cli_key` | pure logic |
| 15 | `test_provision_empty_runtimes` | pure logic |

Tests 14 and 15 require inside `mod tests`:

```rust
use std::collections::HashMap;
use crate::config::schema::ToolsConfig;
```

These are local `use` statements inside the specific test functions per spec (not
hoisted to `mod tests` top). Both match the `ToolsConfig` struct exactly as defined
in `src/config/schema.rs` lines 65-74.

---

## Module Host: `/home/isaac/src/sh.great/src/platform/mod.rs`

66 lines. Already correct. No modifications needed.

| Lines | Declaration |
|-------|-------------|
| 3 | `pub mod runtime;` |
| 12 | `pub use runtime::{MiseManager, ProvisionAction, ProvisionResult};` |

All three public types are re-exported. `provision_single` (private) is not re-exported
(correct -- it is a private helper).

---

## Consumer: `/home/isaac/src/sh.great/src/cli/apply.rs`

Lines 461-536 consume `MiseManager`. All call sites use the unchanged public API:

| Line | Call | Signature used |
|------|------|----------------|
| 12 | `use crate::platform::runtime::{MiseManager, ProvisionAction};` | import |
| 472 | `MiseManager::installed_version(name)` | `fn(name: &str) -> Option<String>` |
| 474 | `MiseManager::version_matches(version, &cur)` | `fn(declared: &str, installed: &str) -> bool` |
| 493 | `MiseManager::is_available()` | `fn() -> bool` |
| 495 | `MiseManager::ensure_installed()` | `fn() -> Result<()>` |
| 504 | `MiseManager::provision_from_config(tools)` | `fn(&ToolsConfig) -> Vec<ProvisionResult>` |

`apply.rs` must NOT be modified. The signature of every function it calls is preserved.

---

## Config Type: `/home/isaac/src/sh.great/src/config/schema.rs`

`ToolsConfig` struct (lines 65-74):

```rust
pub struct ToolsConfig {
    #[serde(flatten)]
    pub runtimes: HashMap<String, String>,   // top-level keys in [tools] table
    pub cli: Option<HashMap<String, String>>, // [tools.cli] sub-table
}
```

Tests 14 and 15 construct `ToolsConfig` directly. The `runtimes` and `cli` fields
are both `pub`. Direct construction is valid -- no builder or `Default` required
(though `Default` is derived, line 65).

---

## Package Manager: `/home/isaac/src/sh.great/src/platform/package_manager.rs`

`PackageManager` trait (lines 6-26):

```rust
pub trait PackageManager {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn is_installed(&self, package: &str) -> bool;
    fn installed_version(&self, package: &str) -> Option<String>;
    fn install(&self, package: &str, version: Option<&str>) -> Result<()>;
    fn update(&self, package: &str) -> Result<()>;
}
```

`Homebrew` (lines 41-121): `fn install()` runs `brew install <package>` (line 83).

`command_exists` is in `super::detection::command_exists` (detection.rs line 135):
uses the `which` crate (`which::which(cmd).is_ok()`), no shell spawning.

Note: The backlog (0008) proposed `ensure_installed(pkg_manager: &dyn PackageManager)`.
The approved spec overrides this -- the actual consumer (`apply.rs` line 495) calls
with zero arguments. The `PackageManager` trait is NOT used in `runtime.rs`. Do not
introduce a trait dependency.

---

## Dependency Map

```
runtime.rs
  imports:
    anyhow::{bail, Context, Result}       (Cargo.toml: anyhow = "1.0")
    super::detection::command_exists      (detection.rs line 135, uses which crate)
  uses (fully-qualified):
    std::process::Command
    std::process::Stdio
  function signature reference:
    crate::config::schema::ToolsConfig    (schema.rs lines 65-74)
  consumed by:
    src/cli/apply.rs (lines 461-536)
  test module imports:
    std::collections::HashMap             (std)
    crate::config::schema::ToolsConfig    (schema.rs)
```

No new Cargo dependencies. All types are already in scope.

---

## Error Handling Pattern

The codebase uses:
- `anyhow::bail!()` for early-exit errors with formatted messages (see lines 69, 74, 83, 95, 105)
- `.context("message")?` for wrapping command spawn errors (lines 66, 92, 102)
- `.ok()?` for `Option` short-circuit on command output (lines 41, 117)
- No `.unwrap()` in production code (enforced by acceptance criteria)
- `#[allow(dead_code)]` on `version()` (line 35) -- keep this attribute

The Change 2 error messages follow the existing bail pattern with exit codes:
`bail!("brew install mise failed (exit code {:?}). ...", status.code())`

---

## Test Patterns (from existing tests and `tests/cli_smoke.rs`)

Unit tests in `src/platform/runtime.rs` follow the inline `#[cfg(test)] mod tests` pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;        // already present, brings MiseManager etc. into scope

    #[test]
    fn test_name() {
        // pure logic: just assertions, no TempDir, no Command spawning
        assert!(MiseManager::version_matches("22", "22.11.0"));
    }
}
```

No-panic tests (smoke tests for external commands):

```rust
#[test]
fn test_version_does_not_panic() {
    let _ = MiseManager::version();  // ignores result, just verifies no panic
}
```

Integration tests in `tests/cli_smoke.rs` use `assert_cmd::Command` and `TempDir`.
The new tests for 0008 are all unit tests (no `assert_cmd`).

The test module already has `use super::*;` at line 212. No additional module-level
`use` is needed for the 13 pure-logic and no-panic tests. Tests 14 and 15
(`test_provision_skips_cli_key` and `test_provision_empty_runtimes`) need
local `use` statements inside the test function body per spec pattern.

---

## Risks and Gotchas

1. **Off-by-one in spec headings.** The spec says "7 tests existing" and "8 tests new"
   in prose but the actual counts are 6 existing and 9 new (total 15). The enumerated
   table at lines 488-505 of the spec is correct and is the authoritative source.
   Socrates review concern #9 confirms this. Follow the table, not the prose headings.

2. **`test_version_no_match` is modified, not added.** The spec modifies the existing
   test at lines 238-241 (adds two boundary assertions). Do not create a duplicate
   function with a new name.

3. **`ensure_installed` calls `command_exists("brew")`, not `Homebrew::is_available()`.**
   The `Homebrew` struct is in `package_manager.rs` and is NOT imported in `runtime.rs`.
   Use `command_exists("brew")` directly -- it is already imported.

4. **`provision_from_config` signature uses fully-qualified type in fn signature.**
   Line 148: `tools: &crate::config::schema::ToolsConfig`. This is the existing pattern.
   Do not change it to a `use` import at module level. The test functions import
   `ToolsConfig` locally inside the function body.

5. **`rest.starts_with('.')` uses a `char` literal, not a `&str` slice.**
   `'.'` (char) is the correct form. `"."` (str) would also compile but `char` is
   idiomatic for single-character checks in Rust.

6. **No `status.code()` unwrap in bail messages.** `status.code()` returns
   `Option<i32>`. The `{:?}` formatter handles `None` as `None`. Do not call
   `.unwrap()` or `.expect()` on it.

7. **The curl path error message changes.** The existing message `"mise installation
   failed"` becomes `"mise install script failed (exit code {:?})..."`. Any downstream
   string matching on the error message (there is none -- `apply.rs` uses `output::error`
   which just formats the `anyhow::Error`) is not a concern.

8. **`lower` variable in `installed_version`.** The `lower` binding is introduced
   before the `if` that uses it. Clippy may warn about intermediate binding if
   the variable is used in only one place. To avoid any warning, inline:
   `version.to_lowercase().contains("no version")` -- or keep the binding since
   it is used twice (once for "no version", once for "not installed"). The binding
   is the cleaner form and avoids double allocation.

---

## Recommended Build Order

Per spec (Section "Build Order", lines 530-538):

1. Apply Change 1: Replace `version_matches` body (lines 137-143).
2. Apply Change 2: Replace `ensure_installed` body (lines 57-78).
3. Apply Change 3: Replace `installed_version` inner condition (lines 120-130).
4. Modify `test_version_no_match` (lines 238-241): add two boundary assertions.
5. Append 8 new test functions to `mod tests` block (after line 251, before `}`).
6. `cargo build` -- zero errors expected.
7. `cargo clippy` -- zero warnings expected.
8. `cargo test platform::runtime` -- all 15 tests must pass.

No changes to any other file. Total diff is approximately 40-50 lines changed.
