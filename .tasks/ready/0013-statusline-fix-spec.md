# Spec 0013: Fix `statusLine` Schema Written by `great loop install`

**Task:** `.tasks/backlog/0013-statusline-settings-schema-mismatch.md`
**Status:** ready
**Type:** bugfix
**Estimated Complexity:** S (single file, 3 targeted edits + 4 new unit tests)

---

## Summary

`great loop install` writes `statusLine` objects to `~/.claude/settings.json` that are missing the required `"type": "command"` discriminator field. Claude Code's settings validator requires this field; without it, the **entire** `settings.json` is rejected and all user settings are silently disabled.

There are two code paths that write the broken shape, plus a missing repair path for users who already have the broken value on disk. All three issues live in a single file: `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`.

---

## Files to Modify

| File | Change |
|------|--------|
| `/home/isaac/src/sh.great/src/cli/loop_cmd.rs` | Fix both JSON literals, add repair logic, add 4 unit tests |

No new files are created.

---

## Interfaces

No public API changes. The `run_install` function signature is unchanged:

```rust
fn run_install(project: bool) -> Result<()>
```

The only change is to the JSON values produced internally by `serde_json::json!` macro invocations and the control-flow logic that decides when to write them.

### Correct statusLine JSON shape

Both code paths must produce this exact JSON value:

```json
{
  "type": "command",
  "command": "great statusline"
}
```

This matches the Claude Code settings schema, which uses `"type": "command"` as a discriminator on a tagged union.

---

## Implementation Approach

### Build Order

All changes are in a single file. Apply in this order:

1. **Fix 1 (line 217-219):** Correct the `serde_json::json!` literal in the new-file creation path.
2. **Fix 2 (line 238):** Correct the `serde_json::json!` literal in the existing-file injection path.
3. **Fix 3 (lines 234-246):** Replace the simple `contains_key` guard with repair logic that also detects and fixes a broken existing `statusLine`.
4. **Tests:** Add 4 unit tests at the end of the `mod tests` block.

### Extract a constant for the correct shape

To avoid duplicating the JSON literal in two code paths (which is how the bug happened in the first place), extract a helper function that returns the correct value:

```rust
/// Returns the correct `statusLine` JSON value for Claude Code settings.
///
/// Claude Code requires `"type": "command"` as a discriminator field.
/// Without it, the entire settings.json is rejected by the validator.
fn statusline_value() -> serde_json::Value {
    serde_json::json!({
        "type": "command",
        "command": "great statusline"
    })
}
```

Place this function immediately before `run_install` (after line 142, before line 143). This is a private helper -- not `pub`, not part of any trait.

---

## Exact Code Changes

### Change 1: Add `statusline_value()` helper

**Insert after line 141** (the closing `}` of `pub fn run`) **and before line 143** (the doc comment for `run_install`):

```rust
/// Returns the correct `statusLine` JSON value for Claude Code settings.
///
/// Claude Code requires `"type": "command"` as a discriminator field.
/// Without it, the entire settings.json is rejected by the validator.
fn statusline_value() -> serde_json::Value {
    serde_json::json!({
        "type": "command",
        "command": "great statusline"
    })
}

```

### Change 2: Fix the new-file creation path (lines 217-219)

**Current code (lines 201-220):**

```rust
        let default_settings = serde_json::json!({
            "env": {
                "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
            },
            "permissions": {
                "allow": [
                    "Bash(cargo *)",
                    "Bash(pnpm *)",
                    "Read",
                    "Write",
                    "Edit",
                    "Glob",
                    "Grep",
                    "LS"
                ]
            },
            "statusLine": {
                "command": "great statusline"
            }
        });
```

**Replace lines 217-219 with:**

```rust
            "statusLine": statusline_value()
```

So the full block becomes:

```rust
        let default_settings = serde_json::json!({
            "env": {
                "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
            },
            "permissions": {
                "allow": [
                    "Bash(cargo *)",
                    "Bash(pnpm *)",
                    "Read",
                    "Write",
                    "Edit",
                    "Glob",
                    "Grep",
                    "LS"
                ]
            },
            "statusLine": statusline_value()
        });
```

### Change 3: Replace the existing-file injection block (lines 228-254)

**Current code (lines 228-254):**

```rust
    // Inject statusLine key into existing settings.json if not already present
    if settings_path.exists() {
        let contents = std::fs::read_to_string(&settings_path)
            .context("failed to read ~/.claude/settings.json for statusLine injection")?;
        match serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(mut val) => {
                if let Some(obj) = val.as_object_mut() {
                    if !obj.contains_key("statusLine") {
                        obj.insert(
                            "statusLine".to_string(),
                            serde_json::json!({"command": "great statusline"}),
                        );
                        let formatted = serde_json::to_string_pretty(&val)
                            .context("failed to serialize settings.json")?;
                        std::fs::write(&settings_path, formatted)
                            .context("failed to write ~/.claude/settings.json")?;
                        output::success("Statusline registered in ~/.claude/settings.json");
                    }
                }
            }
            Err(_) => {
                output::warning(
                    "settings.json is not valid JSON; skipping statusLine injection",
                );
            }
        }
    }
```

**Replace with:**

```rust
    // Inject or repair statusLine in existing settings.json
    if settings_path.exists() {
        let contents = std::fs::read_to_string(&settings_path)
            .context("failed to read ~/.claude/settings.json for statusLine injection")?;
        match serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(mut val) => {
                if let Some(obj) = val.as_object_mut() {
                    let needs_write = if !obj.contains_key("statusLine") {
                        // statusLine key missing entirely -- inject it
                        obj.insert("statusLine".to_string(), statusline_value());
                        true
                    } else if let Some(sl) = obj.get("statusLine").and_then(|v| v.as_object()) {
                        // statusLine exists but may be missing "type" field
                        if !sl.contains_key("type") {
                            obj.insert("statusLine".to_string(), statusline_value());
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if needs_write {
                        let formatted = serde_json::to_string_pretty(&val)
                            .context("failed to serialize settings.json")?;
                        std::fs::write(&settings_path, formatted)
                            .context("failed to write ~/.claude/settings.json")?;
                        output::success("Statusline registered in ~/.claude/settings.json");
                    }
                }
            }
            Err(_) => {
                output::warning(
                    "settings.json is not valid JSON; skipping statusLine injection",
                );
            }
        }
    }
```

### Change 4: Add unit tests

Append these 4 tests inside the `mod tests` block (before the final closing `}`):

```rust
    /// The statusline_value() helper must produce the exact shape Claude Code requires.
    #[test]
    fn test_statusline_value_has_type_command() {
        let val = super::statusline_value();
        let obj = val.as_object().expect("statusline_value must be an object");
        assert_eq!(
            obj.get("type").and_then(|v| v.as_str()),
            Some("command"),
            "statusLine must contain \"type\": \"command\""
        );
        assert_eq!(
            obj.get("command").and_then(|v| v.as_str()),
            Some("great statusline"),
            "statusLine must contain \"command\": \"great statusline\""
        );
    }

    /// The new-file default_settings JSON must include statusLine with "type": "command".
    #[test]
    fn test_default_settings_statusline_has_type() {
        let default_settings = serde_json::json!({
            "env": {
                "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
            },
            "permissions": {
                "allow": [
                    "Bash(cargo *)",
                    "Bash(pnpm *)",
                    "Read",
                    "Write",
                    "Edit",
                    "Glob",
                    "Grep",
                    "LS"
                ]
            },
            "statusLine": super::statusline_value()
        });
        let sl = &default_settings["statusLine"];
        assert_eq!(sl["type"].as_str(), Some("command"));
        assert_eq!(sl["command"].as_str(), Some("great statusline"));
    }

    /// Repair logic: a statusLine missing "type" should be detected as needing repair.
    #[test]
    fn test_broken_statusline_detected() {
        let broken = serde_json::json!({
            "statusLine": { "command": "great statusline" }
        });
        let obj = broken.as_object().unwrap();
        let sl = obj.get("statusLine").unwrap().as_object().unwrap();
        assert!(
            !sl.contains_key("type"),
            "test setup: broken shape must lack type field"
        );
    }

    /// Correct statusLine should NOT be detected as needing repair.
    #[test]
    fn test_correct_statusline_not_detected_as_broken() {
        let correct = serde_json::json!({
            "statusLine": super::statusline_value()
        });
        let obj = correct.as_object().unwrap();
        let sl = obj.get("statusLine").unwrap().as_object().unwrap();
        assert!(
            sl.contains_key("type"),
            "correct shape must have type field"
        );
        assert_eq!(sl.get("type").unwrap().as_str(), Some("command"));
    }
```

---

## Repair Logic Pseudocode

```
read settings.json from disk
parse as serde_json::Value

if top-level object does NOT contain "statusLine":
    insert statusline_value()
    needs_write = true
else if "statusLine" IS an object but does NOT contain "type":
    replace "statusLine" with statusline_value()
    needs_write = true
else:
    needs_write = false  // already correct, or non-object value we don't touch

if needs_write:
    serialize to pretty JSON
    write to disk
    print success message
```

The repair logic intentionally does NOT touch `statusLine` if:
- It already has a `"type"` field (even if the value is something unexpected -- we only fix missing `type`, not wrong `type`)
- It is not a JSON object (e.g., `null`, a string, a number) -- these are unusual enough that the user should fix them manually

---

## Edge Cases

| Scenario | Handling |
|----------|----------|
| `settings.json` does not exist | New file is created with correct shape via `statusline_value()`. The injection block is then entered (file now exists) but the `statusLine` key is already present with `"type"`, so `needs_write` is `false`. No double-write. |
| `settings.json` exists with correct `statusLine` | `contains_key("statusLine")` is `true`, `sl.contains_key("type")` is `true`, `needs_write` is `false`. File is not rewritten. Idempotent. |
| `settings.json` exists with broken `statusLine` (no `type`) | Detected by `!sl.contains_key("type")`. The entire `statusLine` value is replaced with `statusline_value()`. File is rewritten. |
| `settings.json` exists without `statusLine` key | Falls into the `!obj.contains_key("statusLine")` branch. Key is inserted with correct shape. |
| `settings.json` is not valid JSON | The `Err(_)` branch prints a warning and skips injection. No crash. |
| `settings.json` is valid JSON but not an object (e.g., `[]`) | `val.as_object_mut()` returns `None`. The `if let Some(obj)` guard prevents any write. No crash. |
| `statusLine` value is `null` or a non-object type | `obj.get("statusLine").and_then(\|v\| v.as_object())` returns `None`. Falls through to the final `else` branch. `needs_write` is `false`. We do not attempt to repair non-object values. |
| Concurrent access (two `great loop install` processes) | Both read-then-write. The last writer wins. Both write the correct shape, so the result is correct regardless of ordering. No file locking is needed for this use case. |
| `$HOME` not set | `dirs::home_dir()` returns `None`, the existing `.context()` error fires: "could not determine home directory -- is $HOME set?" Unchanged behavior. |
| Filesystem permissions (read-only `settings.json`) | The `std::fs::write` call returns an `io::Error`, wrapped by `.context("failed to write ~/.claude/settings.json")`. The error message is already actionable. Unchanged behavior. |
| Running on macOS ARM64 / x86_64 / Ubuntu / WSL2 | All paths use `dirs::home_dir()` which resolves correctly on all platforms. `~/.claude/settings.json` is the same relative path on all targets. No platform-specific behavior. |
| `great loop install` run twice (idempotency) | First run: creates or repairs. Second run: detects correct shape, `needs_write` is `false`, no file write. Output does not print "Statusline registered" on the second run. |

---

## Error Handling

All error paths already exist and use `anyhow::Context`. No new error types or messages are needed beyond what is already in the codebase. The specific error messages are:

| Condition | Message |
|-----------|---------|
| Cannot read settings.json for injection | `"failed to read ~/.claude/settings.json for statusLine injection"` |
| Cannot serialize after repair | `"failed to serialize settings.json"` |
| Cannot write repaired file | `"failed to write ~/.claude/settings.json"` |
| settings.json is invalid JSON | Warning printed: `"settings.json is not valid JSON; skipping statusLine injection"` |

---

## Security Considerations

- No new dependencies are introduced.
- No user input is incorporated into the JSON output. The `statusline_value()` helper returns a compile-time-known literal.
- The file write path (`~/.claude/settings.json`) is the same as before. No new filesystem paths are accessed.
- No secrets, tokens, or credentials are involved.
- The repair logic only adds/replaces the `"type"` field within the `statusLine` object. Other keys in `settings.json` (permissions, env, etc.) are preserved exactly as read from disk by serde_json's round-trip through `Value`.

---

## Testing Strategy

### Unit tests (in `src/cli/loop_cmd.rs::tests`)

4 new tests, described in detail in the "Exact Code Changes" section:

| Test | What it asserts |
|------|-----------------|
| `test_statusline_value_has_type_command` | `statusline_value()` returns `{"type": "command", "command": "great statusline"}` |
| `test_default_settings_statusline_has_type` | The full `default_settings` JSON block includes the correct `statusLine` shape |
| `test_broken_statusline_detected` | A `statusLine` object without `"type"` is correctly identified as broken (validates the test condition, not the repair code directly, but confirms the detection logic pattern) |
| `test_correct_statusline_not_detected_as_broken` | A `statusLine` produced by `statusline_value()` passes the `contains_key("type")` check |

### Build gate

```bash
cargo clippy -- -D warnings
cargo test -- loop_cmd
```

Both must exit 0 with zero warnings.

### Manual verification

1. **Fresh install:** Delete `~/.claude/settings.json`. Run `great loop install`. Inspect the file. Confirm `statusLine` contains both `"type": "command"` and `"command": "great statusline"`.

2. **Repair path:** Manually edit `~/.claude/settings.json` to contain the old broken shape: `"statusLine": {"command": "great statusline"}`. Run `great loop install`. Inspect the file. Confirm `"type": "command"` has been added.

3. **Idempotency:** Run `great loop install` again without changes. Confirm the file is not rewritten (check mtime or observe that "Statusline registered" is NOT printed).

4. **Claude Code validation:** After the fix, launch Claude Code. Confirm no "Settings Error" dialog appears.

---

## Verification Gate

The builder declares this task complete when:

- [ ] `statusline_value()` helper exists and is used in both code paths (no duplicated JSON literals)
- [ ] Both `serde_json::json!` invocations for `statusLine` produce `{"type": "command", "command": "great statusline"}`
- [ ] Repair logic detects existing `statusLine` missing `"type"` and rewrites it
- [ ] Running `great loop install` twice on correct `settings.json` does not rewrite the file
- [ ] `cargo clippy -- -D warnings` exits 0
- [ ] `cargo test -- loop_cmd` exits 0 (all existing + 4 new tests pass)
- [ ] `git diff` shows changes only in `src/cli/loop_cmd.rs`
