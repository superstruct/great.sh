# Release Notes: Task 0027 — `--non-interactive` flag now functional

**Date:** 2026-02-27
**Scope:** `src/cli/sudo.rs`, `src/cli/apply.rs`, `src/cli/doctor.rs`, `src/main.rs`

---

## What Changed

`great apply --non-interactive` and `great doctor --fix --non-interactive` now
suppress sudo password prompts as documented. Previously, the `--non-interactive`
global flag was parsed by clap but never forwarded to any function that acted on
it. The `ensure_sudo_cached` call in both subcommands received a hard-coded
`false`, so the flag had no effect — interactive password prompts appeared even
when the caller had explicitly opted out of them.

The fix threads the flag end to end:

- `ensure_sudo_cached(is_root, non_interactive)` now accepts the flag as a
  second parameter. When `true`, it returns `SudoCacheResult::NonInteractive`
  immediately — the same result returned when stdin is not a terminal.
- `ApplyArgs` and `DoctorArgs` each gain a `non_interactive: bool` field
  annotated `#[arg(skip)]` so clap ignores it; `main.rs` writes the flag value
  into the struct before dispatch.
- A dead TODO comment in `sudo.rs` that noted this wiring was missing has been
  removed.

The removed TODO was added in task 0025 as a placeholder for exactly this work.

---

## Why It Matters

CI pipelines and automation scripts pass `--non-interactive` precisely to avoid
blocking on a password prompt. The gap between the documented behavior and the
actual behavior made `great apply` unsafe for use in unattended environments
even when the caller took the documented precaution. Any pipeline that ran
`great apply --non-interactive` could still hang waiting for a sudo password.
The flag now does what it says.

---

## Migration

No `great.toml` changes are needed. The only visible change is behavioral: a
`sudo -v` prompt that previously appeared despite `--non-interactive` will no
longer appear. Scripts that relied on the flag and were already working around
the bug (e.g., by pre-seeding sudo credentials externally) continue to work
without modification.
