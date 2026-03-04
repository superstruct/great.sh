# Iteration 035 — Observer Report

**Observer:** W. Edwards Deming
**Date:** 2026-03-04
**Task:** 0038 — CLI: Handle SIGPIPE / EPIPE Gracefully Instead of Panicking
**Priority:** P2 | **Type:** bugfix | **Complexity:** S

## Task Completed

All `great` subcommands now handle SIGPIPE gracefully. Piped commands like
`great status --json | head` exit cleanly (exit 141 via kernel SIGPIPE) instead
of panicking with exit 101 and a "BrokenPipe" error on stderr.

## Changes

| File | Lines Changed | Description |
|---|---|---|
| `src/main.rs` | +6 | `libc::signal(SIGPIPE, SIG_DFL)` at start of main, gated by `#[cfg(unix)]` |
| `tests/cli_smoke.rs` | +57 | Regression test `sigpipe_no_broken_pipe_on_stderr` (added by Turing) |

## Commits

- `ed136c4` fix(cli): restore default SIGPIPE handling to prevent BrokenPipe panics

## Agent Performance

| Agent | Retries | Notes |
|---|---|---|
| Nightingale | 0 | Clean selection |
| Lovelace | 0 | Short spec, one page as requested |
| Socrates | 0 | Approved immediately, zero concerns |
| Humboldt | 0 | Minimal scout for minimal change |
| Turing | 0 | Thorough: strace verification, regression test added |
| Kerckhoffs | 0 | Zero findings, unsafe block verified sound |
| Dijkstra | 0 | Approved, one advisory (comment wording fixed) |
| Wirth | 0 | Zero cost confirmed |
| Hopper | 0 | Clean commit |
| Knuth | 0 | Release notes written, no docs needed |

## Process Observations

**Streamlined for simplicity.** This was a one-line fix (plus test). Full 4-agent
team was replaced with direct implementation + parallel review agents. This saved
~5 minutes of team setup and coordination overhead while maintaining all quality
gates. The Architecton Loop correctly scaled down for a trivial fix.

**Option A (`unix_sigpipe`) was unavailable.** The backlog and spec both recommended
Option A (`#[unix_sigpipe = "sig_dfl"]` attribute), but this attribute is still
unstable in Rust 1.90. Switched to Option B (`libc::signal()`) at build time.
The spec should have verified attribute availability before recommending it —
Lovelace and Socrates both missed this. Not a blocker (Option B works identically)
but a process gap.

**Turing proactively added a regression test.** The spec said "manual verification
only" for SIGPIPE, but Turing found a way to write an automated regression test
by spawning the process, dropping the stdout reader, and checking stderr. Good
initiative — the test catches future regressions.

## Bottleneck

**Spec accuracy on Rust feature availability.** Both Lovelace and Socrates accepted
Option A without verifying `unix_sigpipe` was stable. The build failure was caught
immediately (cargo test), but the wasted compile cycle could have been avoided.

## Metrics

- **Tests:** 356 passed, 0 failed, 1 ignored (up from 355 — new regression test)
- **Clippy:** 0 warnings
- **Binary size delta:** -50 KB (-0.56%) — within measurement noise
- **Runtime overhead:** 1 syscall (~1us) at startup — negligible

## Config Change

**None.** The spec accuracy gap is a human-process issue (verify compiler feature
availability before recommending it), not a tooling configuration issue.
