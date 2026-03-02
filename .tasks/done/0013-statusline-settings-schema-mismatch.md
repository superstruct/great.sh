# 0013: Fix `statusLine` Schema Written by `great loop install`

**Priority:** P1
**Type:** bugfix
**Module:** `src/cli/loop_cmd.rs`
**Status:** pending
**Estimated Complexity:** S

## Context

`great loop install` injects a `statusLine` key into `~/.claude/settings.json`
in this shape:

```json
"statusLine": {
  "command": "great statusline"
}
```

Claude Code's settings validator rejects this with:

```
Settings Error

 /home/isaac/.claude/settings.json
  └ statusLine
    └ type: Invalid value. Expected one of: "command"

 Files with errors are skipped entirely, not just the invalid settings.
```

The validator requires an explicit `"type": "command"` discriminator field in
the `statusLine` object. Without it the entire `settings.json` is silently
skipped by Claude Code, disabling **all** user settings (permissions,
`effortLevel`, `alwaysThinkingEnabled`, `env`, etc.) — not just the statusline.

This is a two-part problem:

1. **great.sh writes the wrong shape.** The injected JSON is missing the
   required `"type": "command"` discriminator that Claude Code's schema
   demands.

2. **Claude Code's error recovery is harsh.** The whole file is skipped rather
   than ignoring just the invalid key. This is an upstream issue in Claude Code
   and is out of scope for great.sh to fix, but it should be documented so
   users understand why losing the statusLine config causes all their other
   settings to disappear too.

### Correct shape (required by Claude Code)

```json
"statusLine": {
  "type": "command",
  "command": "great statusline"
}
```

### Affected code paths

Both injection sites in `src/cli/loop_cmd.rs` write the wrong shape:

- **Line 217 (new file path):** `serde_json::json!` literal in the
  `default_settings` block (when `settings.json` does not yet exist).
- **Line 238 (existing file path):** `serde_json::json!` literal injected into
  an existing parsed `settings.json` when `statusLine` key is absent.

### Current state of `/home/isaac/.claude/settings.json`

The real file on disk already contains the broken shape and triggers the error
on every Claude Code launch. The fix to `loop_cmd.rs` must be accompanied by
guidance (or an automated repair path) for users who already have the bad value
written.

## Reproduction Steps

1. Run `great loop install` on a machine where `~/.claude/settings.json` does
   not exist (or does not contain `statusLine`).
2. Open Claude Code — observe the Settings Error dialog.
3. Inspect `~/.claude/settings.json` — `statusLine` lacks `"type": "command"`.

## Expected vs Actual Behavior

| | Expected | Actual |
|---|---|---|
| Written JSON | `{"type": "command", "command": "great statusline"}` | `{"command": "great statusline"}` |
| Claude Code startup | No settings error | Settings Error dialog; entire file skipped |
| User impact | Statusline renders; all other settings apply | All settings silently disabled |

## Acceptance Criteria

- [ ] After the fix, `great loop install` writes `"statusLine": { "type": "command", "command": "great statusline" }` (with the `type` discriminator) in both the new-file and existing-file injection paths.
- [ ] `great loop install` on a machine with the old broken shape detects `statusLine` missing `type` and rewrites it to the correct shape (repair path).
- [ ] Running `great loop install` twice on a machine with a correct `settings.json` does not overwrite or duplicate the `statusLine` key.
- [ ] A unit test in `src/cli/loop_cmd.rs` asserts that the `statusLine` value in both `serde_json::json!` literals contains `"type": "command"`.
- [ ] `cargo clippy` produces zero new warnings after the change.

## Files That Need to Change

- `src/cli/loop_cmd.rs` — both `serde_json::json!` literals for `statusLine`
  (lines ~217 and ~238); add repair logic in the existing-file injection block.

## References

- Claude Code statusline docs: https://code.claude.com/docs/en/statusline
- Schema requires `"type": "command"` — see "Manually configure a status line" section

## Dependencies

None. Standalone bugfix.

## Out of Scope

- Fixing Claude Code's "skip entire file on any error" behavior — that is an
  upstream issue to report to Anthropic separately.
- Migrating users' existing broken `settings.json` automatically at install
  time beyond the repair path described in criterion 2 (e.g., no automatic
  schema migration on `great doctor`).
