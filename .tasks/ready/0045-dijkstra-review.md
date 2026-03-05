# Dijkstra Review — Task 0045: `--only` / `--skip` flags for `great apply`

**Reviewer:** Edsger W. Dijkstra
**Date:** 2026-03-05
**Files reviewed:**
- `src/cli/apply.rs` (ApplyCategory enum, Args fields, should_apply, section gates, unit tests)
- `tests/cli_smoke.rs` (integration tests: lines 859–962)

---

## VERDICT: REJECTED

---

## Findings

### 1. [BLOCKING] `src/cli/apply.rs:394,399` — Asymmetric `conflicts_with` declaration

`--only` declares `#[arg(conflicts_with = "skip")]`, but `--skip` does **not** declare `conflicts_with = "only"`. In clap 4, `conflicts_with` is one-directional: if the user supplies `--skip` before `--only` on the command line, the conflict is evaluated from `--skip`'s perspective and is **not** caught. The integration test at line 941 only exercises `--only <value> --skip <value>` (only-first ordering), which passes through the declared conflict on `--only` and gives a false sense of correctness.

The fix is either:
- Add `conflicts_with = "only"` to the `--skip` arg, or
- Replace both with `conflicts_with_all` / `ArgGroup` to make the constraint symmetric.

Until this is fixed, a user invoking `great apply --skip mcp --only tools` will silently run with both flags set. The `should_apply` function will then only consult `only` (because `only.is_empty()` is false), making `--skip` a silent no-op — a confusing, uncaught misbehavior.

### 2. [BLOCKING] `tests/cli_smoke.rs:931-945` — Conflict test covers only one argument ordering

The integration test `apply_only_and_skip_conflict` only tests `--only tools --skip mcp`. There is no test for the reverse order `--skip mcp --only tools`. As a consequence, finding 1 above would go undetected by the test suite even if the asymmetric declaration were the intended behavior.

A correct mutual-exclusion test must exercise both orderings and assert failure for each.

### 3. [ADVISORY] `src/cli/apply.rs:956-959` — Agents category emits output unconditionally

The Agents gate always prints `"  agents: no provisioning configured (reserved for future use)"` when the category runs, even under `--only agents --dry-run` where no configuration exists. The MCP and Secrets categories correctly suppress their output when there is nothing to do (`has_mcp_config`, `if let Some(secrets)`). The Agents category is inconsistent: it emits a message for a non-functional placeholder, which pollutes normal runs that do not use `--only agents`. This is a minor UX inconsistency today, but will require a gate condition once the category is implemented.

### 4. [ADVISORY] `tests/cli_smoke.rs:895-911` — `apply_only_agents_dry_run` asserts absence, not presence

The test verifies that "CLI Tools" and "MCP Servers" do **not** appear in stderr, but makes no positive assertion that the agents code path was reached (e.g., `stderr(predicate::str::contains("agents"))`). If `should_apply` were broken and returned false for Agents, this test would still pass. The negative-only assertion does not distinguish between "agents ran and excluded others" and "nothing ran at all."

### 5. [ADVISORY] `src/cli/apply.rs:365-374` — `From<&ApplyCategory>` for display is roundabout

The `From<&ApplyCategory> for &'static str` impl is used only for formatting filter info messages. This trait is functional, but a simple `fn name(self) -> &'static str` method on the enum would be more idiomatic for an internal display helper and would avoid the implicit conversion via `.into()` that requires the reader to find the impl. The current approach is not wrong, but it is unnecessarily indirect for a private formatting concern.

### 6. [ADVISORY] `src/cli/apply.rs:409` — `should_apply` parameter names shadow the field names

The function signature `fn should_apply(category: ApplyCategory, only: &[ApplyCategory], skip: &[ApplyCategory])` is clear in isolation. However, the call sites pass `&args.only` and `&args.skip` — if the struct field names ever change, the call sites and the function signature will diverge silently. This is a minor coupling risk, not a defect.

---

## Changes required before approval

1. **Finding 1 (BLOCKING):** Add `conflicts_with = "only"` to the `--skip` arg declaration so the mutual exclusion is enforced regardless of argument order.

2. **Finding 2 (BLOCKING):** Add an integration test `apply_skip_first_only_conflict` that exercises `--skip <cat> --only <cat>` and asserts `.failure()` with `stderr(contains("cannot be used with"))`.

---

## Summary

The `should_apply` logic is correct and minimal; the enum, naming, and section gates are all well-structured. The implementation is rejected on one defect: the mutual-exclusion constraint between `--only` and `--skip` is declared on only one of the two flags, making it bypassable by reversing argument order, and the test suite does not cover that path.
