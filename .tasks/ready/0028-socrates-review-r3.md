# 0028: Statusline Hooks and Non-Destructive Install -- Socrates Review Round 3

**Spec:** `.tasks/ready/0028-statusline-hooks-spec.md`
**Task:** `.tasks/in-progress/0028-statusline-stuck-and-install.md`
**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-27
**Round:** 3 (FINAL)

---

## VERDICT: APPROVED

All BLOCKING concerns from rounds 1 and 2 are fully resolved. No new blocking
issues were introduced. The flock guard fix is correct, internally consistent,
and documented in all four places where flock is referenced across the spec.

---

## BLOCK 3 (Round 2) -- flock macOS unavailability -- RESOLVED

The spec implements option (a) from round 2's recommendation:

```bash
# flock is Linux-only (util-linux). On macOS it is absent unless installed
# via Homebrew; we degrade gracefully to the racy-but-mostly-correct path.
# -w 5 bounds the wait; || true prevents set -e from aborting on timeout.
if command -v flock >/dev/null 2>&1; then
  exec 9>"${STATE_DIR}/.lock"
  flock -w 5 9 || true
fi
```

**Verification checklist:**

1. **`command -v flock` guard** -- On macOS, where `flock` is absent from the
   default BSD userland, the `if` block is skipped entirely. The script
   continues past the `fi` with no error, no fd opened, no lock attempted.
   The `flock: command not found` failure under `set -e` is fully prevented.
   PASS.

2. **`exec 9>` placement inside the `if` block** -- The `exec 9>...` redirect
   is inside the `if`, not before it. On macOS, fd 9 is never opened. On
   Linux, fd 9 is opened for the lock file and held until the script exits,
   releasing the lock automatically. The canonical bash flock pattern is
   correctly applied. PASS.

3. **`flock -w 5 9 || true`** -- Addresses Advisory A (unbounded block) from
   round 2. The 5-second timeout converts a potentially infinite wait into a
   bounded one. The `|| true` prevents `set -e` from aborting on timeout,
   allowing the script to continue on the racy-but-mostly-correct path (same
   degradation as macOS). PASS.

4. **Critical section scope** -- The lock is acquired just before the `jq`
   invocation and released when the shell exits. The read (`"$STATE_FILE"`
   arg), transform (jq body), and atomic write (`> "$TMPFILE" && mv`) are all
   inside the critical section. The minimum necessary scope is protected.
   PASS.

5. **Consistency across all five flock references:**

   | Location | Content | Consistent? |
   |----------|---------|-------------|
   | Script line 188 (comment) | "serialized with flock on Linux" | Yes |
   | Script lines 192-198 (implementation) | `command -v` guard + `-w 5 || true` | Yes |
   | Section 1.7 edge cases | Describes Linux flock + macOS graceful degradation | Yes |
   | Part 8 macOS section | "detects availability via `command -v flock` at runtime" | Yes |
   | Part 11 error handling table | "flock -w 5 ... || true fallback ... macOS: no locking" | Yes |

   All five locations agree. No contradictions. PASS.

---

## Advisory A (Round 2) -- `flock` timeout -- RESOLVED

Addressed by the same fix: `flock -w 5 9 || true` bounds the maximum wait to
5 seconds.

## Advisory B (Round 2) -- fallback path documentation -- RESOLVED

Section 3.2 `run_inner()` now includes a detailed inline comment (lines
625-635) documenting the known limitation:

> "Known limitation: when session_id is absent (e.g., running `great
> statusline` manually outside Claude Code), all concurrent invocations fall
> back to `config.state_file` -- a single shared path. Multiple concurrent
> Claude Code sessions without session_id injection would read/write the same
> file, causing cross-session contamination. This is a known limitation of the
> non-session-scoped fallback and only affects manual invocation; normal
> Claude Code usage always provides session_id."

PASS.

---

## Round 1 Advisories -- Final Status

| Advisory | Round 1 Status | Final Status |
|----------|---------------|--------------|
| Advisory 2 (conditional `modified`) | Resolved in R2 | PASS |
| Advisory 3 (heading "Replace lines 292-299" vs "290-362") | Resolved in R2 | PASS |
| Advisory 4 (cleanup on every tick ~300ms) | Accepted as-is | ADVISORY (no change required) |
| Advisory 5 (session_id validation in hook) | Resolved in R2 | PASS |
| Advisory 6 (jq test explicit failure) | Resolved in R2 | PASS |

Advisory 4 (cleanup running on every statusline tick) remains an acknowledged
advisory. The function short-circuits on a missing `/tmp/great-loop/` directory
with a single failed `read_dir` call. The overhead is negligible for the
expected case. The spec accepts this as-is. No action required.

---

## No New Issues Introduced

The R3 revision is minimal: only the flock guard and its documentation were
changed. The following were verified to be unchanged and still correct:

- **BLOCK 1+2 resolution** (`.name` as lookup key, `$name` variable removed
  from jq invocation): still correct in R3 script.
- **Session ID validation** (`^[a-zA-Z0-9_-]+$` regex at lines 131-133):
  unchanged and positioned correctly before any path operations.
- **SessionEnd cleanup** (`rm -rf "$STATE_DIR"` before `mkdir -p`): unchanged
  and correct.
- **`TMPFILE` PID-scoped naming**: unchanged and correct. Defined before the
  flock block, which is correct (the name itself is safe; the lock protects
  the read-transform-write of the content).
- **hooks_value() / is_great_loop_hook()**: unchanged. Still correct.
- **Settings merge idempotency (snapshot comparison)**: unchanged. Still
  correct.
- **Integration test jq explicit failure**: unchanged. Still correct.
- **Part 8 and Part 11 updated**: flock behavior documented correctly on both
  macOS and Linux. Consistent with the implementation.

---

## Remaining Non-Blocking Observations

These are implementation notes for Da Vinci, not blockers:

1. **`statusline_value()` visibility**: Section 2.4.2 calls `statusline_value()`
   inside `run_install()`. This function must exist in `loop_cmd.rs` scope
   (either defined there or imported). The spec does not show it explicitly in
   R3 but it appears to pre-exist (referenced as "existing" in prior context).
   Da Vinci should verify this is already in scope before adding the new code.
   ADVISORY.

2. **`default_settings` uses `.unwrap()` at line 503**: The fresh-file creation
   path calls `default_settings.as_object_mut().unwrap()`. This is in
   application code, not test code, which nominally violates the "no `.unwrap()`
   in production" convention. However, the value is a `serde_json::json!(...)`
   literal which is always an object -- the unwrap cannot fail. Da Vinci should
   use `if let Some(obj) = ...` pattern to satisfy clippy and the convention.
   ADVISORY.

3. **`child.stdin.take().unwrap()` in integration test**: The integration test
   uses `.unwrap()` on stdin, which is acceptable in `#[cfg(test)]` context
   per project convention. PASS.

---

## Summary

This is a clean spec that correctly addresses all blocking concerns from two
rounds of adversarial review. The flock guard fix is technically sound,
implemented with the standard bash pattern, and consistently documented in all
five locations where concurrent-write behavior is described. The spec is
implementable without further clarification.
