# Spec 0008: Runtime Version Manager Integration (mise) -- REVISION 2

**Task:** `.tasks/backlog/0008-runtime-manager.md`
**Status:** ready (revised per Socrates review)
**Type:** bug fix + behavioral change + new tests (modification of existing file)
**Estimated Complexity:** S (targeted edits to one existing file, ~15 total tests)

---

## Summary

`/home/isaac/src/sh.great/src/platform/runtime.rs` already exists (253 lines, 6 public functions, 1 private helper, 7 tests). It compiles and is actively consumed by `src/cli/apply.rs` (lines 461-536).

This spec describes **targeted modifications** to the existing file. Each change is categorized as BUG FIX, BEHAVIORAL CHANGE, or NEW ADDITION. Code that is already correct is preserved as-is.

### Consumer

- **`src/cli/apply.rs`** (lines 461-536) -- imports `MiseManager` and `ProvisionAction`, calls `is_available()`, `ensure_installed()`, `installed_version(name)`, `version_matches(version, &cur)`, and `provision_from_config(tools)`. This file must NOT be modified.

`src/cli/diff.rs` is NOT a consumer of `MiseManager`. It uses `command_exists(name)` and `util::get_command_version(name)` independently. Any future integration belongs in a separate task.

### Files to Modify

| File | Description |
|------|-------------|
| `/home/isaac/src/sh.great/src/platform/runtime.rs` | Bug fixes, one behavioral change, expanded tests |

### Files NOT Modified

| File | Status |
|------|--------|
| `/home/isaac/src/sh.great/src/platform/mod.rs` | Already correct: `pub mod runtime;` (line 3), re-exports (line 12) |
| `/home/isaac/src/sh.great/src/cli/apply.rs` | Already correct: consumes MiseManager with the exact API we preserve |

---

## Existing Code Inventory

The table below lists every item in the existing `runtime.rs` and its disposition.

| Item | Lines | Disposition | Notes |
|------|-------|-------------|-------|
| `use anyhow::{bail, Context, Result};` | 1 | KEEP | Correct |
| `use super::detection::command_exists;` | 3 | KEEP | Correct |
| `ProvisionResult` struct | 6-11 | KEEP | Correct, matches apply.rs consumer |
| `ProvisionAction` enum | 13-23 | KEEP | Correct, matches apply.rs consumer |
| `MiseManager` struct | 26 | KEEP | Correct |
| `is_available()` | 30-32 | KEEP | Correct |
| `version()` | 35-54 | KEEP | Correct, `#[allow(dead_code)]` is acceptable |
| `ensure_installed()` | 57-78 | **MODIFY** | BEHAVIORAL CHANGE: add brew-first strategy |
| `install_runtime()` | 81-109 | KEEP | Correct |
| `installed_version()` | 112-131 | **MODIFY** | BUG FIX: handle both "No version" and "Not installed" |
| `version_matches()` | 133-143 | **MODIFY** | BUG FIX: prefix-boundary matching |
| `provision_from_config()` | 145-163 | KEEP | Correct |
| `provision_single()` | 165-203 | KEEP | Correct (private helper) |
| `mod tests` (7 tests) | 210-252 | **EXPAND** | NEW ADDITION: add 8 new tests, keep 5 existing |

---

## Change 1: BUG FIX -- `version_matches` prefix-boundary matching

### The Bug

Line 142 of the existing code:

```rust
installed.starts_with(declared)
```

This bare `starts_with` causes false positives when the declared version is a prefix of a longer numeric segment. For example, `version_matches("3.12", "3.120.0")` returns `true` because `"3.120.0".starts_with("3.12")` is true. Similarly, `version_matches("22", "220.0.0")` would return `true`.

This is a correctness bug that can cause `great apply` to skip installing the correct version of a runtime.

### Existing Code (lines 137-143)

```rust
pub fn version_matches(declared: &str, installed: &str) -> bool {
    if declared == "latest" || declared == "stable" {
        return true;
    }
    // Prefix match: "22" matches "22.x.y", "3.12" matches "3.12.z"
    installed.starts_with(declared)
}
```

### Replacement Code

```rust
pub fn version_matches(declared: &str, installed: &str) -> bool {
    if declared == "latest" || declared == "stable" {
        return true;
    }
    if installed == declared {
        return true;
    }
    // Prefix match with dot-boundary: "22" matches "22.11.0" but NOT "220.0.0".
    // "3.12" matches "3.12.5" but NOT "3.120.0".
    if installed.starts_with(declared) {
        let rest = &installed[declared.len()..];
        return rest.starts_with('.');
    }
    false
}
```

### Why This Fix Is Safe

The parameter name stays `installed` (not renamed to `actual`) to minimize diff. The function signature is unchanged. All existing callers (`apply.rs` line 474: `MiseManager::version_matches(version, &cur)`) continue to work without modification. The early `installed == declared` check preserves exact-match behavior that was previously covered by the `starts_with` path. The `rest.is_empty()` case from the exact match is now handled by the explicit equality check above, so the dot-boundary branch only fires when there IS remaining text.

### Regression Tests

Two new tests specifically target this bug:

```rust
#[test]
fn test_version_matches_no_false_prefix() {
    // Regression: "22" must NOT match "2.2.0" (existing test already covers this
    // via the no-common-prefix path, but adding "220.0.0" is the real boundary test)
    assert!(!MiseManager::version_matches("22", "2.2.0"));
    assert!(!MiseManager::version_matches("22", "220.0.0"));
}

#[test]
fn test_version_matches_no_false_longer_prefix() {
    // Regression: "3.12" must NOT match "3.120.0"
    // This is the specific bug that existed in the original code.
    assert!(!MiseManager::version_matches("3.12", "3.120.0"));
}
```

### Truth Table

| declared | installed | existing code | fixed code | correct |
|----------|-----------|---------------|------------|---------|
| `"22"` | `"22.11.0"` | true | true | true |
| `"22"` | `"22"` | true | true | true |
| `"22"` | `"220.0.0"` | **true** | false | false |
| `"3.12"` | `"3.12.5"` | true | true | true |
| `"3.12"` | `"3.12"` | true | true | true |
| `"3.12"` | `"3.120.0"` | **true** | false | false |
| `"latest"` | `"22.11.0"` | true | true | true |
| `"stable"` | `"1.78.0"` | true | true | true |
| `"22"` | `"2.2.0"` | false | false | false |

---

## Change 2: BEHAVIORAL CHANGE -- `ensure_installed` brew-first strategy

### Current Behavior (lines 57-78)

The existing code always uses `curl -fsSL https://mise.jdx.dev/install.sh | sh` regardless of platform:

```rust
pub fn ensure_installed() -> Result<()> {
    if Self::is_available() {
        return Ok(());
    }

    // Use the official installer
    let status = std::process::Command::new("sh")
        .args(["-c", "curl -fsSL https://mise.jdx.dev/install.sh | sh"])
        .status()
        .context("failed to run mise installer")?;

    if !status.success() {
        bail!("mise installation failed — install manually: https://mise.jdx.dev");
    }

    // Verify it worked
    if !Self::is_available() {
        bail!("mise was installed but not found on PATH — you may need to restart your shell");
    }

    Ok(())
}
```

### New Behavior

When `brew` is on PATH, use `brew install mise` first. This is the preferred installation method on macOS, where Homebrew manages PATH entries and updates automatically. If brew is not available, fall back to the existing curl installer.

### Rationale

- `great apply` installs Homebrew before calling `ensure_installed()` on macOS, so brew is guaranteed to be on PATH by the time this runs.
- Homebrew-managed mise gets automatic updates via `brew upgrade`, reducing maintenance burden.
- The curl installer places mise in `~/.local/bin` which may not be on PATH, causing the post-install verification to fail on fresh systems.
- On Linux without Homebrew (the common case), the curl installer remains the correct strategy.

### Replacement Code

```rust
pub fn ensure_installed() -> Result<()> {
    if Self::is_available() {
        return Ok(());
    }

    if command_exists("brew") {
        // Prefer Homebrew when available (macOS, Linuxbrew)
        let status = std::process::Command::new("brew")
            .args(["install", "mise"])
            .status()
            .context("failed to run brew install mise")?;

        if !status.success() {
            bail!(
                "brew install mise failed (exit code {:?}). Install manually: https://mise.jdx.dev",
                status.code()
            );
        }
    } else {
        // Fall back to the official curl installer
        let status = std::process::Command::new("sh")
            .args(["-c", "curl -fsSL https://mise.jdx.dev/install.sh | sh"])
            .status()
            .context("failed to run mise installer")?;

        if !status.success() {
            bail!(
                "mise install script failed (exit code {:?}). Install manually: https://mise.jdx.dev",
                status.code()
            );
        }
    }

    // Verify it worked
    if !Self::is_available() {
        bail!(
            "mise was installed but not found on PATH — you may need to restart your shell or add ~/.local/bin to PATH"
        );
    }

    Ok(())
}
```

### Important Notes

- **The function signature is unchanged.** `apply.rs` line 495 calls `MiseManager::ensure_installed()` with zero arguments. The backlog suggested `ensure_installed(pkg_manager: &dyn PackageManager)` but the actual consumer does not pass a package manager. This spec matches the consumer.
- **`command_exists` is already imported** via `use super::detection::command_exists;` on line 3. No new import needed.
- **Error messages are more specific** than the existing code. The existing `"mise installation failed"` becomes either `"brew install mise failed (exit code ...)"` or `"mise install script failed (exit code ...)"` depending on the path taken.
- **The post-install verification message is enhanced** to mention `~/.local/bin` which is the curl installer's default location.

### Platform Impact

| Platform | Existing behavior | New behavior |
|----------|------------------|--------------|
| macOS (brew present) | curl installer | `brew install mise` |
| macOS (no brew, unlikely) | curl installer | curl installer (unchanged) |
| Ubuntu bare metal | curl installer | curl installer (unchanged) |
| Ubuntu with Linuxbrew | curl installer | `brew install mise` |
| WSL2 (Ubuntu) | curl installer | curl installer (unchanged) |
| WSL2 with Linuxbrew | curl installer | `brew install mise` |

The Linuxbrew case is the only surprise. On systems where Linuxbrew is installed but slow, `brew install mise` may add latency compared to the direct curl installer. This is an acceptable trade-off because: (a) Linuxbrew systems are uncommon in dev environments, (b) Homebrew-managed packages are easier to maintain, and (c) the curl path remains available as fallback if brew is uninstalled.

---

## Change 3: BUG FIX -- `installed_version` should handle both "No version" and "Not installed"

### The Bug

Line 123 of the existing code:

```rust
if version.is_empty() || version.contains("No version") {
```

The `mise current <tool>` command may output different strings depending on the mise version:

- Older mise versions: `"No version set"` or `"No version"`
- Newer mise versions (2025.x): `"Not installed"` (case varies)

The existing code only checks for `"No version"`. If a user has a newer mise that outputs `"Not installed"`, the function would return `Some("Not installed")` instead of `None`, which would then be compared via `version_matches` and fail.

### Existing Code (lines 120-130)

```rust
if output.status.success() {
    let text = String::from_utf8_lossy(&output.stdout);
    let version = text.trim();
    if version.is_empty() || version.contains("No version") {
        None
    } else {
        Some(version.to_string())
    }
} else {
    None
}
```

### Replacement Code

```rust
if output.status.success() {
    let text = String::from_utf8_lossy(&output.stdout);
    let version = text.trim();
    let lower = version.to_lowercase();
    if version.is_empty()
        || lower.contains("no version")
        || lower.contains("not installed")
    {
        None
    } else {
        Some(version.to_string())
    }
} else {
    None
}
```

### Why Both Strings

The `mise current` command's output format has changed across versions. Checking both strings (case-insensitively) ensures compatibility with mise 2024.x through 2026.x. The case-insensitive check via `to_lowercase()` guards against future casing changes.

---

## Change 4: NEW ADDITION -- Import `ToolsConfig` for expanded tests

### Current Import Block (lines 1-3)

```rust
use anyhow::{bail, Context, Result};

use super::detection::command_exists;
```

The existing code does not import `ToolsConfig` at the module level. The `provision_from_config` function at line 147 uses the fully-qualified path `crate::config::schema::ToolsConfig`. This works but the new tests need `ToolsConfig` in the test module.

### No Change to Import Block

The existing import block is fine. `provision_from_config` already uses the fully-qualified path in its function signature. The test module will import `ToolsConfig` locally via `use crate::config::schema::ToolsConfig;` inside `mod tests`.

Similarly, `std::process::Command` and `std::process::Stdio` are used via fully-qualified paths throughout the file (e.g., `std::process::Command::new("mise")` at lines 37, 63, 89, 99, 113). This is the existing pattern and will not be changed.

---

## Change 5: NEW ADDITION -- Expanded Test Module

### Existing Tests to KEEP (5 of 7)

The existing test module has 7 tests. Five are correct and will be kept as-is:

| Test | Lines | Status |
|------|-------|--------|
| `test_mise_is_available` | 215-218 | KEEP |
| `test_version_matches_exact` | 221-223 | KEEP |
| `test_version_matches_prefix` | 226-229 | KEEP |
| `test_version_matches_latest` | 232-235 | KEEP |
| `test_provision_action_eq` | 244-251 | KEEP |

### Existing Tests to MODIFY (2 of 7)

| Test | Lines | Status | Reason |
|------|-------|--------|--------|
| `test_version_no_match` | 238-241 | MODIFY | Add boundary cases `"220.0.0"` and `"3.120.0"` |

The existing `test_version_no_match` test:

```rust
#[test]
fn test_version_no_match() {
    assert!(!MiseManager::version_matches("22", "20.11.0"));
    assert!(!MiseManager::version_matches("3.12", "3.11.5"));
}
```

Replace with:

```rust
#[test]
fn test_version_no_match() {
    assert!(!MiseManager::version_matches("22", "20.11.0"));
    assert!(!MiseManager::version_matches("3.12", "3.11.5"));
    // Boundary bug regression: prefix must end at a dot, not mid-number
    assert!(!MiseManager::version_matches("22", "220.0.0"));
    assert!(!MiseManager::version_matches("3.12", "3.120.0"));
}
```

### New Tests to ADD (8 tests)

Add these tests to the existing `mod tests` block, after the existing tests:

```rust
// ---------------------------------------------------------------
// version_matches: additional coverage for boundary fix
// ---------------------------------------------------------------

#[test]
fn test_version_matches_exact_only_declared() {
    // Declared version equals installed version exactly (no dots after)
    assert!(MiseManager::version_matches("22", "22"));
    assert!(MiseManager::version_matches("3.12.5", "3.12.5"));
}

#[test]
fn test_version_matches_stable_keyword() {
    assert!(MiseManager::version_matches("stable", "1.78.0"));
    assert!(MiseManager::version_matches("stable", "2.0.0-beta"));
}

#[test]
fn test_version_matches_no_false_longer_prefix() {
    // Regression test for the specific prefix-boundary bug.
    // "3.12" must NOT match "3.120.0" because "120" != "12".
    assert!(!MiseManager::version_matches("3.12", "3.120.0"));
    // "1.7" must NOT match "1.78.0" because "78" != "7".
    assert!(!MiseManager::version_matches("1.7", "1.78.0"));
}

#[test]
fn test_version_matches_partial_major() {
    // "3" matches "3.12.5" (dot after "3")
    assert!(MiseManager::version_matches("3", "3.12.5"));
    // "3" does NOT match "30.0.0"
    assert!(!MiseManager::version_matches("3", "30.0.0"));
}

// ---------------------------------------------------------------
// version (no-panic test)
// ---------------------------------------------------------------

#[test]
fn test_version_does_not_panic() {
    let _ = MiseManager::version();
}

// ---------------------------------------------------------------
// installed_version (no-panic + nonexistent runtime)
// ---------------------------------------------------------------

#[test]
fn test_installed_version_nonexistent_runtime() {
    // Should return None for a runtime that can't exist
    let result = MiseManager::installed_version("nonexistent_runtime_xyz_12345");
    assert!(result.is_none());
}

#[test]
fn test_installed_version_does_not_panic() {
    let _ = MiseManager::installed_version("node");
}

// ---------------------------------------------------------------
// provision_from_config (exercises skip-cli logic)
// ---------------------------------------------------------------

#[test]
fn test_provision_skips_cli_key() {
    use std::collections::HashMap;
    use crate::config::schema::ToolsConfig;
    let tools = ToolsConfig {
        runtimes: {
            let mut m = HashMap::new();
            m.insert("cli".to_string(), "ignored".to_string());
            m
        },
        cli: None,
    };
    let results = MiseManager::provision_from_config(&tools);
    // The "cli" key must be skipped entirely
    assert!(results.is_empty(), "cli key should be skipped");
}

#[test]
fn test_provision_empty_runtimes() {
    use std::collections::HashMap;
    use crate::config::schema::ToolsConfig;
    let tools = ToolsConfig {
        runtimes: HashMap::new(),
        cli: None,
    };
    let results = MiseManager::provision_from_config(&tools);
    assert!(results.is_empty());
}
```

### Removed Test: `test_provision_result_fields`

The previous spec included `test_provision_result_fields` which constructs a `ToolsConfig` with a nonexistent runtime and asserts `ProvisionAction::Failed`. This test has environment-dependent behavior:

- If mise is NOT installed: `installed_version` returns `None`, then `install_runtime` is called which bails because mise is not available. Result: `Failed`. Test passes.
- If mise IS installed: `installed_version("nonexistent_xyz_12345")` returns `None`, then `mise install nonexistent_xyz_12345@99.99` is called which hits the network and eventually fails. Result: `Failed`. Test passes, but is slow and flaky.

This test is removed from the spec. The provision logic is adequately covered by `test_provision_skips_cli_key` and `test_provision_empty_runtimes` (pure logic, no external commands).

### Final Test Count: 15 tests

| # | Test name | Type | Origin |
|---|-----------|------|--------|
| 1 | `test_mise_is_available` | no-panic | EXISTING (kept) |
| 2 | `test_version_matches_exact` | pure logic | EXISTING (kept) |
| 3 | `test_version_matches_prefix` | pure logic | EXISTING (kept) |
| 4 | `test_version_matches_latest` | pure logic | EXISTING (kept) |
| 5 | `test_version_no_match` | pure logic | EXISTING (modified: +2 assertions) |
| 6 | `test_provision_action_eq` | pure logic | EXISTING (kept) |
| 7 | `test_version_matches_exact_only_declared` | pure logic | NEW |
| 8 | `test_version_matches_stable_keyword` | pure logic | NEW |
| 9 | `test_version_matches_no_false_longer_prefix` | pure logic | NEW |
| 10 | `test_version_matches_partial_major` | pure logic | NEW |
| 11 | `test_version_does_not_panic` | no-panic | NEW |
| 12 | `test_installed_version_nonexistent_runtime` | no-panic | NEW |
| 13 | `test_installed_version_does_not_panic` | no-panic | NEW |
| 14 | `test_provision_skips_cli_key` | pure logic | NEW |
| 15 | `test_provision_empty_runtimes` | pure logic | NEW |

---

## Public API (Unchanged)

The public API of `MiseManager` is unchanged. All six public functions retain their existing signatures:

```rust
pub fn is_available() -> bool
#[allow(dead_code)]
pub fn version() -> Option<String>
pub fn ensure_installed() -> Result<()>
pub fn install_runtime(name: &str, version: &str) -> Result<()>
pub fn installed_version(name: &str) -> Option<String>
pub fn version_matches(declared: &str, installed: &str) -> bool
pub fn provision_from_config(tools: &crate::config::schema::ToolsConfig) -> Vec<ProvisionResult>
```

The `ProvisionResult` and `ProvisionAction` types are unchanged. The private helper `provision_single` is unchanged.

---

## Build Order

1. Apply the three code changes to `/home/isaac/src/sh.great/src/platform/runtime.rs`:
   - Change 1: Replace `version_matches` body (lines 137-143)
   - Change 2: Replace `ensure_installed` body (lines 57-78)
   - Change 3: Replace the if-condition in `installed_version` (lines 120-130)
2. Modify the existing `test_version_no_match` test (lines 238-241) to add boundary assertions.
3. Add 8 new tests to the bottom of the existing `mod tests` block (before the closing `}`).
4. Run `cargo build` -- must succeed.
5. Run `cargo clippy` -- must produce zero warnings.
6. Run `cargo test platform::runtime` -- all 15 tests must pass.

---

## Error Handling

All error messages use `anyhow::bail!` with actionable instructions.

| Scenario | Error message | Recovery |
|----------|--------------|----------|
| `brew install mise` fails | `"brew install mise failed (exit code {code:?}). Install manually: https://mise.jdx.dev"` | User installs manually |
| curl installer fails | `"mise install script failed (exit code {code:?}). Install manually: https://mise.jdx.dev"` | User checks network, installs manually |
| mise installed but not on PATH | `"mise was installed but not found on PATH -- you may need to restart your shell or add ~/.local/bin to PATH"` | User fixes PATH |
| mise not available for `install_runtime` | `"mise is not installed -- run \`great doctor\` for installation instructions"` | User runs `great apply` first |
| `mise install <spec>` fails | `"mise install {spec} failed"` | User checks runtime name/version |
| `mise use --global <spec>` fails | `"mise use --global {spec} failed"` | User runs manually |

---

## Platform-Specific Behavior

### macOS (ARM64 and x86_64)

- `ensure_installed()`: Uses `brew install mise` (BEHAVIORAL CHANGE from curl-only).
- All other functions: Identical across platforms.

### Ubuntu / Debian (bare metal)

- `ensure_installed()`: Uses `curl -fsSL https://mise.jdx.dev/install.sh | sh` (unchanged).
- The user must have `~/.local/bin` in PATH. If not, the post-install verification catches it.

### WSL2 (Ubuntu under WSL)

- Identical to Ubuntu bare metal. The mise installer detects the platform correctly.
- No Windows-side integration needed -- mise runs entirely within WSL2.

---

## Security Considerations

1. **curl pipe to sh:** The `ensure_installed()` function pipes a remote script to `sh`. This is the official mise install method. The URL `https://mise.jdx.dev/install.sh` uses HTTPS. In environments where this is unacceptable, users should install mise via their package manager before running `great apply`.

2. **No credential exposure:** `MiseManager` does not handle any secrets or credentials. Runtime version strings are not sensitive.

3. **Command injection:** Runtime names and version strings come from `great.toml` which is a project file under the developer's control. They are passed as separate arguments to `Command::new("mise").args(["install", &spec])`, not interpolated into a shell string. This prevents shell injection.

---

## Edge Cases

| Edge case | Behavior |
|-----------|----------|
| Empty runtime name `""` | `mise install @<version>` will fail; `Failed` result with mise's error |
| Empty version string `""` | `mise install <name>@` will fail; `Failed` result |
| Version string `"latest"` | Passed to mise as `mise install node@latest`; mise handles this natively |
| Version string `"stable"` | Used for Rust; `mise install rust@stable` delegates to rustup |
| Rust declared while rustup exists | `mise install rust@<version>` wraps rustup; no conflict |
| mise available but offline | `mise install` will fail with a network error; `Failed` result |
| Concurrent `great apply` runs | `mise install` acquires its own file locks; safe |
| `tools.runtimes` contains only `"cli"` key | Loop body skips it; `provision_from_config` returns empty `Vec` |
| mise outputs `"Not installed"` for a runtime | `installed_version` returns `None` (BUG FIX, Change 3) |
| mise outputs `"No version set"` for a runtime | `installed_version` returns `None` (existing behavior, preserved) |
| mise version output has extra metadata | `version()` returns the full first line; callers display it as-is |
| `version_matches("3.12", "3.120.0")` | Returns `false` (BUG FIX, Change 1) |
| brew on PATH but brew install fails | Clear error with exit code and manual install URL |
| brew not on PATH, curl not on PATH | `sh -c "curl ..."` fails; error message with exit code |

---

## Acceptance Criteria

- [ ] All three code changes applied to `/home/isaac/src/sh.great/src/platform/runtime.rs`.
- [ ] `cargo build` succeeds with zero errors.
- [ ] `cargo clippy` produces zero warnings for `src/platform/runtime.rs`.
- [ ] `cargo test platform::runtime` passes all 15 tests.
- [ ] `version_matches("3.12", "3.120.0")` returns `false` (regression test for prefix-boundary bug).
- [ ] `version_matches("22", "22.11.0")` returns `true` (preserved behavior).
- [ ] `version_matches("22", "2.2.0")` returns `false` (preserved behavior).
- [ ] The existing `apply.rs` call sites (lines 472, 474, 493, 495, 504) compile without changes.
- [ ] No `.unwrap()` or `.expect()` calls exist outside `#[cfg(test)]`.
- [ ] `installed_version` returns `None` for both `"No version"` and `"Not installed"` output strings.
