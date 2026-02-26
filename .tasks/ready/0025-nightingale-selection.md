# Nightingale Selection — Iteration 025

**Date:** 2026-02-26
**Selected Task:** 0025 — Pre-cache sudo credentials before Homebrew install
**Selected By:** Florence Nightingale (Requirements Curator)

---

## Selection

**Task 0025** is selected for this loop iteration.

---

## Candidate Assessment

| Task | Priority | Feasibility | User Impact | Decision |
|------|----------|-------------|-------------|----------|
| 0009 apply command | P0 | Not applicable — source already fully implemented; backlog file is stale | n/a | Defer: move to done/, not a real open task |
| 0025 sudo pre-cache | P2 | High — zero deps, 2–3 files, blueprint fully written | High — prevents silent Homebrew install failure on macOS | SELECTED |
| 0021 diff output channel | P2 | High — small scope, no deps | Medium — CI quality-of-life | Defer to next iteration |
| 0020 docker cross-compile UX | P2 | Medium — vague acceptance criteria, multi-issue | Low — developer tooling only | Defer: needs acceptance criteria tightened first |
| 0014 backlog pruning | P3 | High | None (housekeeping) | Defer: lowest priority |

---

## Why 0025 Over 0021

Both are P2, unblocked, and small-scoped. The tiebreaker is user impact severity:

- **0025** prevents an outright installation failure. A new macOS user running `great apply` without cached sudo credentials sees `Failed to install Homebrew!` and the provisioning run aborts. This is a silent cliff edge — the user has admin rights but never gets the chance to use them.
- **0021** is a CI output routing nicety. Pipelines that discard stderr lose section headers; the fix is real but affects only advanced automated consumers, not first-run users.

One-iteration scope favors the higher user-impact fix.

---

## Note on 0009

The backlog file for 0009 (`apply` command) has `Status: pending` but the implementation is fully present in `src/cli/apply.rs` (241+ lines, full provisioning logic, Homebrew bootstrap, mise runtimes, CLI tools, MCP config, secrets, dry-run, --yes flag). All stated acceptance criteria are met in the current source. This file should be moved to `.tasks/done/` as part of the next 0014 pruning pass, not treated as open work.

---

## Scope for This Iteration

The 0025 task file is already well-scoped. No reduction needed. The full scope is:

1. Detect whether sudo will be needed before the install phase begins (Homebrew absent, or Linux/WSL platform).
2. If interactive (`stdin().is_terminal()`) and not already root, run `sudo -v` once to cache credentials and print a single explanatory message to the user.
3. Spawn a background keepalive thread (`sudo -vn` every 60 s) for applies that may exceed the sudo cache timeout.
4. Apply the same pre-cache pattern to `great doctor --fix`'s Homebrew path, extracting a shared `ensure_sudo_cached()` helper used by both `apply.rs` and `doctor.rs`.
5. Skip all of the above in non-interactive contexts (piped stdin, CI) — existing `NONINTERACTIVE=1` fail-fast behavior is correct there.
6. No new crate dependencies. Uses only `std::process::Command` and `std::io::stdin().is_terminal()` (stable since Rust 1.70).

---

## Acceptance Criteria (for Lovelace)

The following are the testable gates this iteration must clear:

- [ ] `great apply` on macOS, with Homebrew absent and uncached sudo, shows a single password prompt before installation begins — not a failure message.
- [ ] After the `sudo -v` prompt, the Homebrew install with `NONINTERACTIVE=1` completes successfully using cached credentials.
- [ ] `great doctor --fix` also pre-caches sudo before attempting Homebrew install (shared helper, not duplicated inline).
- [ ] In non-interactive contexts (piped stdin, CI flags), no `sudo -v` is attempted — existing fail-fast behavior is unchanged.
- [ ] The keepalive background thread produces no visible output and does not outlive the process.

---

## Key Files for Lovelace

| File | Lines of Interest | Change |
|------|-------------------|--------|
| `src/cli/apply.rs` | 406–438 (Homebrew install block) | Add `ensure_sudo_cached()` call before the install block |
| `src/cli/doctor.rs` | 122–124 (Homebrew fix block) | Add `ensure_sudo_cached()` call before the install block |
| `src/cli/apply.rs` or new `src/cli/sudo_cache.rs` | new | `ensure_sudo_cached(needs_sudo: bool, is_root: bool)` helper + keepalive thread |

---

## Out of Scope for This Iteration

- Changes to `bootstrap.rs` apt-get calls (already interactive, already correct)
- Consolidated error messaging for tools skipped due to Homebrew failure (separate concern, separate task)
- Any changes to 0021 (diff output channel) or 0020 (docker UX)
