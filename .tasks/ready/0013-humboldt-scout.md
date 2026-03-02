# Scout Report 0013: Fix `statusLine` Schema in `great loop install`

**Scout:** Alexander von Humboldt
**Date:** 2026-02-24
**Spec:** `.tasks/ready/0013-statusline-fix-spec.md`
**Status:** READY TO BUILD

---

## Summary

Single-file bugfix. The broken `statusLine` JSON shape is written in exactly
two places inside `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`. No other
file in the codebase writes or reads this key. The spec and Socrates review
are accurate; line references in the spec have been verified against the live
source.

---

## File Map

### Primary change target

**`/home/isaac/src/sh.great/src/cli/loop_cmd.rs`**

| Lines | What | Change needed |
|-------|------|---------------|
| 141 | Closing `}` of `pub fn run` | Insert `statusline_value()` helper immediately after |
| 201-220 | `default_settings` `serde_json::json!` block (new-file path) | Replace lines 217-219 `"statusLine": { "command": "great statusline" }` with `"statusLine": statusline_value()` |
| 228-254 | `// Inject statusLine key into existing settings.json` block | Replace entire block with repair logic (detect missing `"type"` and rewrite) |
| 433-538 | `mod tests` block | Append 4 new unit tests before final closing `}` |

### No other files need changes

The search across all `.rs`, `.json`, `.toml`, and `.md` source files confirms
`statusLine` (the Claude Code settings key) appears only in:

- `/home/isaac/src/sh.great/src/cli/loop_cmd.rs` — the two broken write sites (lines 217-219 and 238)
- `/home/isaac/src/sh.great/src/cli/mod.rs` — only mentions `Statusline` CLI subcommand, unrelated
- `/home/isaac/src/sh.great/src/main.rs` — dispatch only, no JSON shape
- `/home/isaac/src/sh.great/src/cli/statusline.rs` — the rendering subcommand; reads no settings schema

---

## Broken State Confirmed

**`/home/isaac/.claude/settings.json` on disk right now:**

```json
{
  "statusLine": {
    "command": "great statusline"
  }
}
```

Missing `"type": "command"`. This is the live broken shape that triggers the
Claude Code "Settings Error — Files with errors are skipped entirely" dialog on
every launch.

The repair path (Change 3 in the spec) will fix this on the next `great loop install` run.

---

## Test Infrastructure

### Existing unit tests in `loop_cmd.rs` (lines 433-538)

All tests are in `mod tests` inside the same file. Pattern: plain `#[test]`
functions, no async, no tempdir, no filesystem access. Tests assert on
constants and `serde_json::Value` parsing only. The 4 new tests fit this
pattern exactly.

Run gate: `cargo test -- loop_cmd`

### Integration tests in `tests/cli_smoke.rs`

There are **zero** `great loop install` integration tests. The smoke test file
tests the `statusline` subcommand extensively (30+ test functions, lines
380-1130) but does not exercise `great loop install` at all. This is a gap
noted by Socrates but is out of scope for this bugfix.

Run gate: `cargo test` (full suite)

---

## The `great statusline` Subcommand

**`/home/isaac/src/sh.great/src/cli/statusline.rs`**

Registered in:
- `/home/isaac/src/sh.great/src/cli/mod.rs` line 75: `Statusline(statusline::Args)`
- `/home/isaac/src/sh.great/src/main.rs` line 28: dispatch to `cli::statusline::run`

The subcommand reads its own TOML config from
`~/.config/great/statusline.toml` (or env override). It does **not** read or
validate `~/.claude/settings.json`. No changes needed here.

---

## Dependency Map

```
loop_cmd.rs::run_install
  └── serde_json::json!  (Cargo.toml dep, already present)
  └── dirs::home_dir()   (Cargo.toml dep, already present)
  └── std::fs::read_to_string / write
  └── output::success / warning  (src/cli/output.rs, no changes needed)

statusline_value() [new helper]
  └── serde_json::json!  (same dep)
  └── no new imports required
```

No new dependencies. No new files. No public API changes.

---

## Exact Locations of Broken Code

**Bug site 1** — `/home/isaac/src/sh.great/src/cli/loop_cmd.rs` lines 217-219:

```rust
            "statusLine": {
                "command": "great statusline"
            }
```

**Bug site 2** — `/home/isaac/src/sh.great/src/cli/loop_cmd.rs` line 238:

```rust
                            serde_json::json!({"command": "great statusline"}),
```

Both produce `{"command": "great statusline"}` — missing the required
`"type": "command"` discriminator.

---

## Existing Tests That Must Continue to Pass

```
test_agents_count               (line 438)
test_commands_count             (line 443)
test_agent_names_unique         (line 448)
test_command_names_unique       (line 455)
test_teams_config_valid_json    (line 464)
test_teams_config_has_loop_name (line 470)
test_no_architecton_in_agents   (line 476)
test_no_architecton_in_commands (line 486)
test_no_architecton_in_teams_config (line 496)
test_observer_template_not_empty (line 505)
test_all_expected_agents_present (line 511)
```

None of these touch `statusLine` — they are unaffected by the fix.

---

## Risks

| Risk | Severity | Notes |
|------|----------|-------|
| Double-write on fresh install | LOW | Edge case: new file created (lines 200-226), then injection block entered (lines 228-254). After new file is written, `statusLine` is already present with correct `"type"` field, so `needs_write` will be `false`. No double-write. Confirmed safe by spec edge case table. |
| Idempotency regression | LOW | If `"type"` is already present, repair branch is not taken. File not rewritten. Idempotent. |
| `serde_json::json!` macro with function call in value position | NONE | `serde_json::json!` explicitly supports function calls as values. `"statusLine": statusline_value()` compiles correctly. Confirmed in Socrates review. |
| No integration test for `great loop install` | ADVISORY | Existing gap in test coverage. Out of scope for this bugfix. Manual verification steps cover the integration path. |
| `run_status` does not check `statusLine` shape | ADVISORY | Users who ran old `great loop install` won't see a warning from `great loop status`. Fix is: re-run `great loop install`. Follow-up task, not blocking. |

---

## Recommended Build Order

Apply all changes to `/home/isaac/src/sh.great/src/cli/loop_cmd.rs` in this
order (use content matching, not line numbers, since inserting the helper shifts
subsequent lines):

1. Insert `statusline_value()` helper after `pub fn run` closing `}` (before `run_install` doc comment)
2. Replace the broken `"statusLine"` literal in the `default_settings` block
3. Replace the existing-file injection block with the repair logic
4. Append 4 new unit tests inside `mod tests`

Build gates (both must pass):

```bash
cargo clippy -- -D warnings
cargo test -- loop_cmd
```

---

## Prior Art in This Codebase

The `serde_json::json!` macro is already used throughout `loop_cmd.rs` for the
`default_settings` block. The `output::success` / `output::warning` pattern for
user feedback is consistent with the rest of the function. No new patterns need
to be introduced.

---

## Files Referenced

| Path | Role |
|------|------|
| `/home/isaac/src/sh.great/src/cli/loop_cmd.rs` | **Only file to modify** |
| `/home/isaac/src/sh.great/src/cli/statusline.rs` | `great statusline` renderer — no changes |
| `/home/isaac/src/sh.great/src/cli/mod.rs` | CLI dispatch — no changes |
| `/home/isaac/src/sh.great/src/main.rs` | Entry point — no changes |
| `/home/isaac/src/sh.great/tests/cli_smoke.rs` | Integration tests — no changes |
| `/home/isaac/.claude/settings.json` | Live broken file — repaired by running the fixed binary |
| `/home/isaac/src/sh.great/.tasks/ready/0013-statusline-fix-spec.md` | Approved spec with exact code |
| `/home/isaac/src/sh.great/.tasks/ready/0013-socrates-review.md` | Socrates review — APPROVED |
