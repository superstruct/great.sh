# Release Notes: Task 0026 — `great diff` Output Channel Redesign

**Date:** 2026-02-27
**Scope:** `src/cli/output.rs`, `src/cli/diff.rs`, `tests/cli_smoke.rs`

---

## What Changed

`great diff` now writes all output — section headers, diff lines, and the final summary — to stdout. Previously, section headers ("Tools", "MCP Servers", "Secrets"), the config path info line, and the summary ("nothing to do" / "N to install, ...") were written to stderr, while the actual `+`, `~`, and `-` marker lines went to stdout. Fatal errors (missing `great.toml`) remain on stderr and continue to exit with code 1.

---

## Why It Matters

The split-channel behavior made `great diff` awkward to use in pipelines and CI scripts. Piping output — `great diff | grep "+"` — silently dropped all section headers and the summary, leaving orphaned marker lines with no context. Redirecting stderr — `great diff 2>/dev/null` — produced the same incomplete result in reverse. CI pipelines that captured stdout received only marker lines, with no indication of what was being compared or what action was recommended. `great diff` is a read-only, pipeline-oriented command; its entire output is data, and Unix convention places data on stdout. All output now arrives on a single channel, so redirection, piping, and log capture work without surprises.

---

## No Action Required

This change is transparent. No `great.toml` changes, flags, or configuration updates are needed. Scripts that already captured the marker lines from stdout are unaffected. Scripts that captured headers or the summary from stderr should redirect from stdout instead, but no existing `great diff` usage silently breaks — the output is simply now available on the correct channel.
