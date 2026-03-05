# Dijkstra Review (R2) — Task 0045: `--only` / `--skip` flags for `great apply`

**Reviewer:** Edsger W. Dijkstra
**Cycle:** 2 of 2
**Date:** 2026-03-05
**Files reviewed:**
- `src/cli/apply.rs` (lines 388–406, 955–963)
- `tests/cli_smoke.rs` (lines 895–962)
**Prior verdict:** REJECTED (2 BLOCKING findings)

---

## VERDICT: APPROVED

---

## Resolution of BLOCKING findings

### Finding 1 — Asymmetric `conflicts_with` declaration — RESOLVED

`src/cli/apply.rs:399` now carries `conflicts_with = "only"` on the `--skip` field, mirroring the existing `conflicts_with = "skip"` on `--only` at line 394. The mutual exclusion is now symmetric and will be enforced regardless of argument order.

### Finding 2 — Conflict test covers only one argument ordering — RESOLVED

`tests/cli_smoke.rs:948` adds `apply_skip_and_only_conflict_reverse_order`, which exercises `--skip mcp --only tools` (skip-first ordering) and asserts `.failure()` with `stderr(contains("cannot be used with"))`. Both orderings are now tested.

---

## Status of ADVISORY findings

### Finding 3 — Agents category emits output unconditionally — ADDRESSED

`src/cli/apply.rs:959` now wraps the placeholder message in a guard:

```
if !args.only.is_empty() || !args.skip.is_empty() {
    output::info("  agents: no provisioning configured (reserved for future use)");
}
```

The message is suppressed on unfiltered runs. The guard condition is precisely the right criterion: the message is visible when and only when the user explicitly filtered categories, which is when the feedback is meaningful.

### Finding 4 — `apply_only_agents_dry_run` asserts absence, not presence — OPEN (advisory)

`tests/cli_smoke.rs:909-910` remains unchanged: the test still makes no positive assertion that the agents code path was entered. This does not block approval — it was advisory in the first review and remains so. The weakness is noted for future authors: a positive assertion such as `stderr(contains("agents"))` would close the gap.

### Findings 5 and 6 — Unchanged, advisory only

The `From<&ApplyCategory>` impl and the `should_apply` parameter naming carry no defects. Both remain open as advisory observations for future maintainers; neither warrants action now.

---

## Note on `cargo test`

This reviewer is read-only and cannot execute test runs. The structural fixes have been verified by source inspection. The responsible engineer must confirm `cargo test` passes before merging.

---

## Summary

Both BLOCKING findings from cycle 1 are correctly resolved: `conflicts_with` is now symmetric at the clap declaration level, and the test suite covers both argument orderings. The agents-gate advisory was also addressed. The implementation is approved for merge.
