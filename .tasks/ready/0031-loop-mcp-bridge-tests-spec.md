# 0031 -- Loop and MCP Bridge Smoke Tests: Specification

| Field      | Value                                      |
|------------|--------------------------------------------|
| Priority   | P2                                         |
| Type       | feature (test coverage)                    |
| Module     | `tests/cli_smoke.rs`                       |
| Status     | ready                                      |
| Complexity | S                                          |

## Summary

Add missing smoke tests for the `loop` and `mcp-bridge` subcommands to
`tests/cli_smoke.rs`. Several tests already exist (added during iterations
016/028/029) but gaps remain. This spec identifies the exact gaps and specifies
each new test function.

## Current Coverage (already in cli_smoke.rs)

| Test function                            | What it covers                       |
|------------------------------------------|--------------------------------------|
| `loop_install_force_flag_accepted`       | `loop install --help` shows `--force`|
| `loop_install_force_fresh_succeeds`      | Fresh install writes agent files     |
| `loop_install_non_tty_existing_files_aborts` | Non-TTY re-install aborts        |
| `loop_install_force_overwrites_existing` | `--force` replaces modified files    |
| `mcp_bridge_help_shows_description`      | `mcp-bridge --help` exits 0         |
| `mcp_bridge_unknown_preset_fails`        | `mcp-bridge --preset invalid` fails  |

## Gaps to Fill

| # | Missing test                               | Acceptance criterion                                         |
|---|--------------------------------------------|--------------------------------------------------------------|
| 1 | `loop --help` top-level                    | stdout contains "install" and "status" (subcommand listing)  |
| 2 | `loop status` on fresh HOME                | exit 0, stderr contains "not installed"                      |
| 3 | `loop uninstall` on fresh HOME             | exit 0, graceful no-op                                       |
| 4 | `mcp-bridge --preset invalid` stderr msg   | stderr contains "invalid preset" (existing test only checks exit code) |
| 5 | `loop install --force` writes hook script  | `$HOME/.claude/hooks/great-loop/update-state.sh` exists      |
| 6 | `loop install --force` writes settings.json| `$HOME/.claude/settings.json` exists and contains "hooks"    |

---

## Interfaces (test function signatures)

All tests are `#[test] fn name()` with no parameters, no return type. They
follow the existing pattern in `cli_smoke.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn great() -> Command {
    Command::cargo_bin("great").expect("binary exists")
}
```

### Test 1: `loop_help_shows_subcommands`

```rust
#[test]
fn loop_help_shows_subcommands() {
    great()
        .args(["loop", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("install").and(predicate::str::contains("status")));
}
```

**Rationale:** Verifies the `LoopCommand` enum variants surface in help text.
Clap generates lowercase subcommand names from the variant names, so
"install" and "status" are stable strings.

### Test 2: `loop_status_fresh_home_reports_not_installed`

```rust
#[test]
fn loop_status_fresh_home_reports_not_installed() {
    let dir = TempDir::new().unwrap();
    great()
        .args(["loop", "status"])
        .env("HOME", dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("not installed"));
}
```

**Rationale:** `run_status()` calls `output::error("Agent personas: not installed")`
which writes to stderr via `eprintln!`. The test uses a fresh `TempDir` as HOME
so no `~/.claude/agents/nightingale.md` exists. Exit code is 0 because status
reporting is informational, not a failure.

### Test 3: `loop_uninstall_fresh_home_is_noop`

```rust
#[test]
fn loop_uninstall_fresh_home_is_noop() {
    let dir = TempDir::new().unwrap();
    great()
        .args(["loop", "uninstall"])
        .env("HOME", dir.path())
        .assert()
        .success();
}
```

**Rationale:** `run_uninstall()` iterates managed file paths and only deletes
those that exist. On a fresh HOME, none exist, so the loop body is a no-op.
The function must exit 0 without panic. If it currently panics or returns
`Err`, that is a bug to fix alongside this test.

### Test 4: `mcp_bridge_unknown_preset_shows_error_message`

```rust
#[test]
fn mcp_bridge_unknown_preset_shows_error_message() {
    great()
        .args(["mcp-bridge", "--preset", "invalid_preset_xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid preset"));
}
```

**Rationale:** The existing `mcp_bridge_unknown_preset_fails` only asserts
`.failure()`. This new test uses a different (longer) preset string and
asserts the stderr message. The error comes from `anyhow::Context` wrapping
`Preset::from_str` returning `None`, producing:
`"invalid preset 'invalid_preset_xyz' -- use: minimal, agent, research, full"`.

Note: The builder may choose to strengthen the existing test instead of adding
a new function. Either approach satisfies this criterion.

### Test 5: `loop_install_force_writes_hook_script`

```rust
#[test]
fn loop_install_force_writes_hook_script() {
    let dir = TempDir::new().unwrap();
    great()
        .args(["loop", "install", "--force"])
        .env("HOME", dir.path())
        .assert()
        .success();

    let hook = dir.path().join(".claude/hooks/great-loop/update-state.sh");
    assert!(hook.exists(), "hook script must be written");
}
```

**Rationale:** The existing `loop_install_force_fresh_succeeds` checks agents,
commands, and teams config, but does not check the hook script path added in
iteration 028. This confirms the hook write path executes.

### Test 6: `loop_install_force_writes_settings_json`

```rust
#[test]
fn loop_install_force_writes_settings_json() {
    let dir = TempDir::new().unwrap();
    great()
        .args(["loop", "install", "--force"])
        .env("HOME", dir.path())
        .assert()
        .success();

    let settings = dir.path().join(".claude/settings.json");
    assert!(settings.exists(), "settings.json must be created");
    let content = std::fs::read_to_string(&settings).unwrap();
    assert!(
        content.contains("hooks"),
        "settings.json must contain hooks configuration"
    );
    assert!(
        content.contains("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS"),
        "settings.json must contain agent teams env"
    );
}
```

**Rationale:** `run_install()` creates `settings.json` when absent (with hooks,
env, permissions, statusLine). This test verifies the settings write path which
is not covered by any existing test.

---

## Test Isolation Strategy

All `loop` tests that exercise install/status/uninstall MUST use:

```rust
let dir = TempDir::new().unwrap();
// ...
.env("HOME", dir.path())
```

This overrides `dirs::home_dir()` which reads `$HOME` on all platforms.
`TempDir` auto-cleans on drop. No network, no keychain, no real `~/.claude/`
mutation.

The `mcp-bridge` tests do NOT need HOME override because they fail before
touching the filesystem (preset validation happens before `discover_backends()`).

**Platform note:** On macOS, `dirs::home_dir()` reads `$HOME` from the
environment (not `dscl`), so the env override works identically across
macOS ARM64/x86_64, Ubuntu, and WSL2.

---

## Implementation Approach

### Build Order

1. Add tests 1-3 (loop --help, status, uninstall) -- no code changes needed
2. Add test 4 (mcp-bridge stderr message) -- no code changes needed
3. Add tests 5-6 (hook script, settings.json) -- no code changes needed
4. Run `cargo test` to confirm all pass
5. Run `cargo clippy` to confirm no warnings

### File to Modify

Single file: `/home/isaac/src/sh.great/tests/cli_smoke.rs`

Insert the new loop tests after the existing "Loop install -- overwrite safety"
section (after line 1548), and the new mcp-bridge test after the existing
"MCP Bridge" section (after line 1892).

### Placement Convention

Follow the existing section-comment style:

```rust
// -----------------------------------------------------------------------
// Loop -- help and status
// -----------------------------------------------------------------------
```

If the builder prefers to group all loop tests under one section, that is
also acceptable. The key constraint is that new tests must not be interspersed
randomly between unrelated sections.

---

## Edge Cases

| Case | Expected behavior | Covered by |
|------|-------------------|------------|
| `loop status` with no HOME set | `dirs::home_dir()` returns None, anyhow error | Not tested (extreme edge, requires `env_remove("HOME")` which may break the process) |
| `loop uninstall` on empty HOME | Graceful no-op, exit 0 | Test 3 |
| `mcp-bridge` with no flags (blocks on stdin) | Out of scope for smoke tests | N/A |
| `mcp-bridge --preset` with valid preset but no backends | Exits 1 with "no AI CLI backends found" | Not tested here (would need PATH isolation; low value for smoke test) |
| `loop install --force` twice (idempotent) | Already covered by `loop_install_force_overwrites_existing` | Existing |

---

## Error Handling

No new error handling is needed in production code. All tested code paths
already use `anyhow::Result` and `output::error()`. The tests verify that
errors surface as expected exit codes and messages.

If `loop uninstall` on a fresh HOME currently panics (it should not, based
on code review of `run_uninstall()` lines 658-711), the builder should file
a separate bug. The test is the signal.

---

## Security Considerations

None. These are read-only smoke tests that write to temporary directories
only. No secrets, no network access, no privilege escalation.

---

## Testing Strategy

All six tests run as part of `cargo test`. No `#[ignore]` attributes.
No feature flags required. Expected total runtime increase: < 2 seconds
(each test spawns the binary which is already compiled).

Verification command:

```bash
cargo test --test cli_smoke -- loop_ mcp_bridge_ --nocapture
```

This runs only the loop and mcp-bridge tests with output visible.

### CI Matrix

Tests must pass on:
- `ubuntu-latest` (GitHub Actions)
- `macos-latest` (GitHub Actions)

No Windows target (project does not support Windows natively; WSL2 is Linux).
