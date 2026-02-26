# 0028: Statusline Hooks and Non-Destructive Install -- Socrates Review Round 2

**Spec:** `.tasks/ready/0028-statusline-hooks-spec.md`
**Task:** `.tasks/in-progress/0028-statusline-stuck-and-install.md`
**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-27
**Round:** 2

---

## VERDICT: REJECTED

One new BLOCKING concern was introduced by the Advisory 1 fix (flock). All
Round 1 BLOCKING concerns are resolved. Three advisories remain open (one new,
two carry-forward).

---

## Round 1 BLOCKING Concerns -- Disposition

### BLOCK 1 + 2 -- RESOLVED

The `$name` / `$key` ambiguity is fully resolved. The revised spec:

- Removes `--arg name "$AGENT_NAME"` from the jq invocation entirely. The jq
  body now uses only `--arg key "$AGENT_KEY"`, `--arg status`, and `--argjson now`.
- States explicitly in section 1.4: "The `.name` field in the state JSON IS the
  lookup key -- it is not a human-readable display name."
- Updates the schema example in section 1.2 to show `"name": "agent-abc123"`
  (an agent_id, not a display string like `"davinci"`).
- Provides the invariant: for each event pair, the same source field is used for
  both insert and lookup.
- Updates the 1.4 table to show the lookup key and source field explicitly for
  all five events.

The jq upsert logic is now internally consistent: `"name": $key` on insert,
`index($key)` on lookup. No contradiction remains. PASS.

---

## New BLOCKING Concern Introduced by Advisory 1 Fix

### BLOCK 3 -- `flock` is not available on macOS

**Gap:** Advisory 1 was addressed by adding `flock` serialization:

```bash
exec 9>"${STATE_DIR}/.lock"
flock 9
```

`flock` is a Linux-specific utility from the `util-linux` package. It is NOT
part of macOS's default BSD userland. On macOS, `flock` does not exist unless
explicitly installed (`brew install util-linux`, which is uncommon and conflicts
with Homebrew's own locking). The spec's Part 8 explicitly lists macOS (ARM64
and x86_64) as a supported platform.

On macOS, the hook script will fail with:
```
update-state.sh: line N: flock: command not found
```
Because `set -euo pipefail` is active, this causes the script to exit with a
non-zero status before any state is written. The entire state-writing mechanism
is silently broken on macOS.

**Question:** Has the spec author verified that `flock` is available in the
target macOS execution environment? If macOS users install `util-linux` via
Homebrew, does `flock` land in PATH where Claude Code's hook invocation can
find it?

**Severity:** BLOCKING

**Recommendation:** Either:

(a) **Use a `flock`-with-fallback pattern** -- detect availability at runtime:
```bash
if command -v flock >/dev/null 2>&1; then
  exec 9>"${STATE_DIR}/.lock"
  flock 9
fi
# ... jq upsert ...
```
This silently downgrades to the racy-but-mostly-correct behavior on macOS,
which the spec previously accepted as a known limitation.

(b) **Use Python as an alternative lock mechanism** (available on both
platforms: macOS ships Python 3). Heavier dependency.

(c) **Document that macOS requires `brew install util-linux`** and have
`great loop status` check for `flock` alongside `jq`. Add to Part 8 macOS
section explicitly.

(d) **Accept the race on macOS** -- revert to no serialization but explicitly
document the known limitation in section 1.7. The pre-existing behavior before
Advisory 1 was to have no locking; the race condition only matters for
simultaneous events (SubagentStart firing for two agents within the same
jq-execution window), which is rare.

Option (a) is the lowest-risk fix and preserves the Linux guarantee while
gracefully degrading on macOS.

---

## Round 1 Advisory Concerns -- Disposition

### Advisory 1 (flock serialization) -- INTRODUCED NEW BLOCKING (see above)

The fix was correct for Linux but breaks macOS. See BLOCK 3.

### Advisory 2 (conditional `modified = true`) -- RESOLVED

The revised spec adds a snapshot-comparison pattern:

```rust
let hooks_before = serde_json::to_string(hooks_map).unwrap_or_default();
// ... merge logic ...
let hooks_after = serde_json::to_string(hooks_map).unwrap_or_default();
if hooks_before != hooks_after {
    modified = true;
}
```

This correctly sets `modified` only when the hooks object actually changed.
The `unwrap_or_default()` calls are safe (`to_string` on a `Map` object cannot
fail). PASS.

### Advisory 3 (heading "Replace lines 292-299" vs "Replace lines 290-362") -- RESOLVED

Section 2.4.2 now reads "Replace lines 290-362 (the entire settings.json
handling block)" with the full range stated in both the heading and the body.
PASS.

### Advisory 5 (session_id validation in hook script) -- RESOLVED

The regex guard is present at lines 131-133 of the script:

```bash
if [[ ! "$SESSION_ID" =~ ^[a-zA-Z0-9_-]+$ ]]; then
  exit 0
fi
```

This appears after the `SESSION_ID` extraction and before any path operations.
PASS.

### Advisory 6 (integration test jq skip -> explicit failure) -- RESOLVED

The integration test now uses:

```rust
let jq_output = std::process::Command::new("jq").arg("--version").output()
    .expect("jq is required for this test and must be installed");
assert!(jq_output.status.success(), "jq --version exited with non-zero status");
```

This will cause an explicit test failure (not a silent skip) when jq is absent.
PASS.

---

## New Advisory Concerns

### Advisory A -- `flock 9` blocks indefinitely; no timeout specified

**Gap:** The flock invocation `flock 9` (without `-w SECS`) blocks until the
lock is acquired. If a prior hook invocation is hung inside a jq execution
(e.g., jq crashes mid-write, leaving the lock held), subsequent hooks will
block indefinitely. Claude Code's async hook mechanism has no enforced timeout
per hook process.

**Question:** Is there a Claude Code-enforced timeout on async hook processes
that would unblock the waiting hook?

**Severity:** ADVISORY (jq hangs are extremely rare; the flock is released
automatically when the owning process exits, so a crashed jq would release the
lock. The risk is low.)

**Recommendation:** Change `flock 9` to `flock -w 5 9 || exit 0` -- wait at
most 5 seconds, then bail silently rather than blocking indefinitely. This
converts a potentially unbounded block into a bounded one.

### Advisory B -- New `run_inner()` renumbers steps but `config.state_file` is still used in fallback

**Gap:** The new `run_inner()` in section 3.2 uses `config.state_file` as the
fallback path when `session_id` is absent:

```rust
_ => config.state_file.clone(),
```

The `StatuslineConfig.state_file` default value is not shown in this spec. If
its default is `/tmp/great-loop/state.json` (a non-session-scoped path), then
the fallback path could conflict between multiple concurrent Claude Code
sessions on the same machine -- all would read the same file if they have no
`session_id`.

**Question:** What is the default value of `StatuslineConfig.state_file`? Is
the fallback to a non-scoped path intentional for backward compatibility, and
if so, is the cross-session contamination documented as a known limitation?

**Severity:** ADVISORY (this is pre-existing behavior; the spec does not change
it. The session-scoped path is the new happy path; the fallback is a legacy
path for installations without Claude Code session_id injection.)

**Recommendation:** Add a sentence to section 3.2 documenting the fallback
semantics: "When `session_id` is absent (e.g., running `great statusline`
directly without Claude Code), the renderer uses the configured fallback path.
Multiple concurrent sessions sharing this path is a known limitation of the
non-session-scoped fallback."

---

## Carry-forward Advisories from Round 1 (unchanged)

### Advisory 4 (cleanup on every tick) -- No Change, Still ADVISORY

The concern about `cleanup_stale_sessions()` running on every ~300ms tick was
flagged ADVISORY in Round 1 and remains ADVISORY. The spec did not address it
and does not need to for approval. The function short-circuits on a missing
directory with one `readdir` call.

---

## Requirement Coverage -- No Regressions

All 6 requirements remain covered as in Round 1. The BLOCK 3 concern does not
remove any coverage -- R2 (hook handler scripts) is implemented, it just fails
at runtime on macOS due to the missing `flock` command.

---

## Required Changes for Approval

1. **BLOCK 3:** Resolve the `flock` unavailability on macOS. The recommended
   fix is option (a): wrap the `exec 9>`/`flock 9` block in a `command -v
   flock` availability check, falling back to the racy-but-acceptable
   behavior. Alternatively, accept option (c) and document the macOS `flock`
   dependency in Part 8, add `flock` availability to `great loop status`
   checks.

2. **ADVISORY A (recommended):** Change `flock 9` to `flock -w 5 9 || exit 0`
   to bound the maximum wait time.

---

## Summary

Both Round 1 BLOCKING concerns are correctly and completely resolved. The spec
is cleaner and internally consistent. However, the Advisory 1 fix (flock
serialization) introduced a new BLOCKING regression: `flock` does not exist on
macOS, which is an explicitly supported platform. The hook script will fail on
macOS at the `flock` line under `set -e`, silently breaking all state writing
on that platform.
