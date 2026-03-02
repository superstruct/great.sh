# Nightingale Selection — 0022: Diff Counter/Marker Consistency

**Selected task:** 0022 — Align `great diff` counter buckets with visual markers
**Priority:** P2
**Type:** bugfix/enhancement
**Date:** 2026-02-26

---

## Why This Task

### First: Task 0009 is done and should be closed

Before selecting 0022, I read `src/cli/apply.rs` in full. It is 973 lines of complete
implementation covering all five acceptance criteria from the 0009 backlog entry:
- Config loading with `--config` and auto-discovery
- Dry-run mode with full plan preview
- `--yes` flag for non-interactive execution
- Full provisioning pipeline: Homebrew bootstrap, mise runtimes, CLI tools,
  MCP server config, secrets check, platform overrides, Docker, Claude Code, tuning
- Error isolation (individual step failures do not abort the run)

The 0009 backlog file should be moved to `.tasks/done/`.

Similarly, most 0010 sub-groups are fully implemented:
- Group A (tool mapping): `tool_install_spec()` in `apply.rs` with 8 tool entries
- Group B (Starship): `configure_starship()` fully implemented in `apply.rs`
- Group E (Update): `update.rs` is 241 lines with real GitHub API + binary swap
- Group F (Vault): `vault.rs` has all 4 subcommands implemented
- Group H (Template update): `template.rs` `run_update()` fetches from GitHub
- Group J (Integration tests): `tests/cli_smoke.rs` has 80+ tests covering all major subcommands

### Why 0022 over 0025

Both are P2 with no dependencies. 0022 wins because:

1. **Size S vs. M**: 0022 touches one file (`src/cli/diff.rs`), 0025 touches three files
   (`apply.rs`, `doctor.rs`, a new `ensure_sudo_cached()` helper) plus a background thread.

2. **Correctness bug, not UX enhancement**: 0022 fixes a data integrity issue — CI
   consumers parsing `great diff` output get miscounted summary numbers. This is observable
   wrong behavior, not a missing convenience.

3. **Lower risk**: Thread management in 0025 (sudo keepalive) introduces concurrency
   that needs careful testing. 0022 is pure logic in a single function.

4. **Immediately verifiable**: The fix in 0022 can be validated with the existing
   integration tests in `tests/cli_smoke.rs` (specifically `diff_summary_shows_counts`).

---

## Scoped Requirements

### Problem Statement

In `src/cli/diff.rs`, two bugs exist:

**Bug 1: MCP bucket mismatch**
MCP servers with missing commands display a `+` (green "install") marker but their
count goes into `configure_count`. The summary line prints "N to install, M to configure"
but the visual markers contradict the bucket assignment. Any CI script checking exit
codes or parsing summary counts gets wrong data.

**Bug 2: Duplicate secrets count**
Secrets appearing in both `secrets.required` and `find_secret_refs()` are counted
twice in `secrets_count`, inflating the number reported in the summary.

### Fix Specification

**1. Classify MCP actions correctly**

Current logic (needs reading to confirm exact lines, but the pattern is):
- MCP server missing command → shows `+` marker → added to `configure_count`

Required logic:
- `+` marker (new item to install/add) → `install_count`
- `~` marker (existing item to reconfigure) → `configure_count`
- `-` marker (item to remove) → its own count or `configure_count` (document the rule)

Define the invariant: **every diff output line's marker must match its summary bucket**.

**2. Deduplicate secrets**

Collect secrets into a `HashSet<String>` before counting to eliminate duplicates
between `secrets.required` and the refs extracted by `find_secret_refs()`.

**3. Section header consistency (stretch)**

If all section headers use action-verb style ("Tools to install", "MCP to configure")
or noun style ("Tools", "MCP Servers"), pick one and apply it uniformly.
If this requires broader output changes, scope it as a separate follow-up.

---

## Files Involved

| File | Role |
|------|------|
| `src/cli/diff.rs` | Primary change — counter logic and marker assignment |
| `tests/cli_smoke.rs` | Update `diff_summary_shows_counts` and add targeted secret dedup test |

The diff subcommand integration tests that already exist and must continue passing:
- `diff_no_config_exits_nonzero`
- `diff_satisfied_config_exits_zero`
- `diff_missing_tool_shows_plus`
- `diff_disabled_mcp_skipped`
- `diff_version_mismatch_shows_tilde`
- `diff_with_custom_config_path`
- `diff_summary_shows_counts`
- `diff_unresolved_secret_shows_red_minus`

---

## What "Done" Looks Like

- [ ] Every line with a `+` marker increments `install_count` in the summary
- [ ] Every line with a `~` marker increments `configure_count` in the summary
- [ ] A secret appearing in both `secrets.required` and `find_secret_refs()` is
      counted exactly once in `secrets_count`
- [ ] All existing diff integration tests pass without modification to assertions
- [ ] `diff_summary_shows_counts` test validates the corrected bucket math
- [ ] `cargo clippy` produces zero new warnings
- [ ] No new crate dependencies added

---

## Context for the Spec Writer (Lovelace)

The full diff implementation is in `src/cli/diff.rs`. Read that file first to
identify the exact counter increment sites and the MCP section logic. The
integration test harness is in `tests/cli_smoke.rs` — the `diff_summary_shows_counts`
test is the key one to extend. The config schema is in `src/config/schema.rs`
where `find_secret_refs()` is defined.

Key questions to answer in the spec:
1. What is the exact line in `diff.rs` where MCP items are bucketed wrongly?
2. What data structure holds secrets before counting — slice or iterator?
3. Should section header normalization be in scope or deferred?
