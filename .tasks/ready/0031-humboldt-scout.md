# 0031 — Humboldt Scout Report: Loop and MCP Bridge Smoke Tests

## Single file to modify

`/home/isaac/src/sh.great/tests/cli_smoke.rs` (1893 lines total)

---

## Exact insertion points

### Loop tests — insert after line 1548

Line 1548 is the closing `}` of `loop_install_force_overwrites_existing`.
The next content (line 1550) is `statusline_with_state_file_renders_agents` — an
unrelated statusline test. The spec's "after line 1548" instruction is correct.
Insert a new section block immediately after line 1548.

### MCP Bridge test — insert after line 1892

Line 1892 is the closing `}` of `mcp_bridge_unknown_preset_fails`.
Line 1893 is EOF. Insert immediately before EOF.

---

## Existing patterns to reuse

All imports already present at the top of the file (lines 1-3):

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
```

Helper (lines 6-8):

```rust
fn great() -> Command {
    Command::cargo_bin("great").expect("binary exists")
}
```

Section comment style (lines 1469-1471):

```rust
// -----------------------------------------------------------------------
// Loop install -- overwrite safety
// -----------------------------------------------------------------------
```

HOME override pattern (lines 1483-1494):

```rust
let dir = TempDir::new().unwrap();
great()
    .args(["loop", "install", "--force"])
    .env("HOME", dir.path())
    .assert()
    .success();
```

Filesystem assertion pattern (lines 1491-1493):

```rust
assert!(dir.path().join(".claude/agents/nightingale.md").exists());
```

---

## Exact strings to assert

### Test 1 — `loop --help` subcommand listing

Clap derives lowercase variant names. `LoopCommand` has variants `Install`,
`Status`, `Uninstall` (loop_cmd.rs lines 21, 31, 33). Clap renders them as
`install`, `status`, `uninstall` in help text. Assert on:

```
"install"
"status"
```

### Test 2 — `loop status` on fresh HOME: "not installed"

`run_status()` (loop_cmd.rs line 569):
```rust
output::error("Agent personas: not installed");
```
`output::error` calls `eprintln!` (output.rs line 15), so this lands on **stderr**.
Assert: `stderr` contains `"not installed"`.

Exit code: 0. `run_status()` always returns `Ok(())` (line 654).

### Test 3 — `loop uninstall` on fresh HOME: graceful no-op

`run_uninstall()` (loop_cmd.rs lines 658-712) iterates AGENTS and COMMANDS,
only deletes files that `path.exists()`. On empty HOME no deletions occur.
Always returns `Ok(())` (line 711). Assert: `.success()` only.

### Test 4 — `mcp-bridge --preset invalid_preset_xyz` stderr message

`mcp_bridge.rs` line 105-108:
```rust
let preset = Preset::from_str(&preset_str).context(format!(
    "invalid preset '{}' — use: minimal, agent, research, full",
    preset_str
))?;
```

The `?` propagates `Err` to `main() -> Result<()>`. Rust prints anyhow errors
to stderr as `Error: invalid preset 'invalid_preset_xyz' — use: minimal, agent,
research, full`.

Assert: `stderr` contains `"invalid preset"`.

Note: the em-dash in the source is a Unicode `\u{2014}` (`—`), not ASCII `--`.
The spec uses `"invalid preset"` as the substring — safe, avoids the em-dash.

### Test 5 — hook script path

Written at (loop_cmd.rs lines 351-355):
```rust
let hooks_dir = claude_dir.join("hooks").join("great-loop");
let hook_script_path = hooks_dir.join("update-state.sh");
std::fs::write(&hook_script_path, HOOK_UPDATE_STATE)...
```

Assert path: `dir.path().join(".claude/hooks/great-loop/update-state.sh")`

### Test 6 — settings.json path and contents

Written at (loop_cmd.rs lines 371, 489-495) when no settings.json exists:
```rust
let settings_path = claude_dir.join("settings.json");
// ...creates JSON with "env", "permissions", "statusLine", "hooks"
std::fs::write(&settings_path, formatted)...
```

Assert path: `dir.path().join(".claude/settings.json")`
Assert `content.contains("hooks")` — present as top-level key in written JSON.
Assert `content.contains("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS")` — present
inside the `"env"` object (loop_cmd.rs line 469).

---

## Dependency map

No production code changes required. All six tests exercise code paths that
already exist:

| Test | Production function | File | Lines |
|------|---------------------|------|-------|
| 1 | clap help generation | loop_cmd.rs | 18-34 |
| 2 | `run_status()` | loop_cmd.rs | 557-655 |
| 3 | `run_uninstall()` | loop_cmd.rs | 658-712 |
| 4 | `run()` preset validation | mcp_bridge.rs | 105-108 |
| 5 | `run_install()` hook write | loop_cmd.rs | 351-367 |
| 6 | `run_install()` settings write | loop_cmd.rs | 466-496 |

---

## Risks

1. **Test 2 stderr colour codes**: `output::error` uses `colored` crate. In a
   subprocess spawned by assert_cmd, `colored` detects no TTY and disables ANSI
   by default, so the raw string `"not installed"` will match cleanly. Low risk.

2. **Test 4 em-dash encoding**: The error string contains `\u{2014}` (em-dash).
   Using `"invalid preset"` as the substring avoids any encoding fragility.

3. **Test 3 uninstall exit code**: `run_uninstall()` calls
   `dirs::home_dir().context(...)` at line 659. With `HOME` set to the temp dir,
   `dirs` reads `$HOME` on Linux/macOS and returns `Some(path)`, so no error.
   The function reliably returns `Ok(())`. No risk.

4. **Test 6 JSON key ordering**: `serde_json::to_string_pretty` on a
   `serde_json::json!({})` literal preserves insertion order. The key `"hooks"`
   is inserted last (line 487) but `content.contains("hooks")` substring search
   is order-independent. No risk.

---

## Build order

Spec's recommended order is correct:

1. Add tests 1-3 (loop --help, status, uninstall) after line 1548
2. Add test 4 (mcp-bridge stderr) after line 1892
3. Add tests 5-6 (hook script, settings.json) in the same loop section as 1-3
4. `cargo test --test cli_smoke -- loop_ mcp_bridge_ --nocapture`
5. `cargo clippy`
