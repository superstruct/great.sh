# Release Notes: Task 0025 — Pre-cache sudo credentials before Homebrew install

**Date:** 2026-02-26
**Scope:** `src/cli/sudo.rs` (new), `src/cli/apply.rs`, `src/cli/doctor.rs`

---

## What Changed

`great apply` and `great doctor --fix` now prompt for your administrator password once, up front, before any installation begins. Previously, Homebrew's non-interactive installer ran with the password prompt suppressed — which caused it to fail immediately with "Need sudo access on macOS" even on machines where the user has admin rights. The new `ensure_sudo_cached` helper runs `sudo -v` at the start of the apply or fix flow (while the terminal is free and visible), caches the credentials, and keeps them alive in a background thread for the duration of the command. All subsequent operations that require root — Homebrew installation, `apt-get` system prerequisites, Docker setup — then complete silently using the cached session.

---

## Why It Matters

Running `great apply` on a fresh macOS machine with Homebrew absent would produce a hard failure before any tools were installed. The root cause was `NONINTERACTIVE=1`, which is required to prevent the Homebrew installer from hanging in CI — but which also suppresses the `sudo` password prompt that the installer needs on interactive machines. The result was a confusing error that gave no indication of how to proceed. This change eliminates that failure path: users see one clear password prompt with a short explanation, and the rest of the install proceeds unattended.

---

## No Action Required

This change is automatic. No configuration, flag, or `great.toml` change is needed. The behavior is transparent in CI and other non-interactive environments: when stdin is not a terminal, `ensure_sudo_cached` returns immediately without prompting, and the existing fail-fast behavior for passwordless-sudo requirements is preserved.
