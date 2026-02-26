# Scout Report 0025 — Pre-cache sudo credentials before Homebrew install

**Scout:** Alexander von Humboldt
**Spec:** `.tasks/ready/0025-homebrew-sudo-spec.md`
**Review:** `.tasks/ready/0025-socrates-review.md` — APPROVED
**Date:** 2026-02-26

---

## Verified Insertion Points

### `src/cli/apply.rs`

| Line | What is there | Action |
|------|--------------|--------|
| 390 | `let info = platform::detect_platform_info();` | `info` binding created here — `info.is_root` available from this point |
| 394–397 | `if args.dry_run { ... println!() }` — dry-run warning block | New `needs_sudo` + `_sudo_keepalive` block goes AFTER this block ends |
| 399 | `// 2b. System prerequisites — before Homebrew...` | This comment is the upper bound; insertion is between lines 397 and 399 |
| 400 | `bootstrap::ensure_prerequisites(args.dry_run, &info);` | First `sudo apt-get` call — must be AFTER `ensure_sudo_cached()` |
| 406–415 | `let needs_homebrew = match &info.platform { ... }` | This block computes the same logic as the spec's `needs_sudo` inner check — **Socrates Advisory 4**: duplication is justified by ordering (sudo must precede prerequisites at line 400); add a comment explaining this |
| 417 | `if needs_homebrew && !info.capabilities.has_homebrew {` | Homebrew install gate |
| 433–438 | `std::process::Command::new("bash").args(["-c", "NONINTERACTIVE=1 /bin/bash..."])` | The actual Homebrew install — benefits from pre-cached sudo; no change to this code |

**Exact insertion site in apply.rs:** After line 397 (`println!();` at end of dry-run block), before line 399 (the `// 2b.` comment).

### `src/cli/doctor.rs`

| Line | What is there | Action |
|------|--------------|--------|
| 62 | `let info = platform::detect_platform_info();` | `info` is already bound in outer scope — do NOT call `detect_platform_info()` again in the fix block (Socrates Advisory 3 — use existing `info`) |
| 94 | `if args.fix && !result.fixable.is_empty() {` | Outer fix block begins here |
| 97 | `let managers = package_manager::available_managers(false);` | New `has_sudo_fix` + `_sudo_keepalive` code goes after this line |
| 98 | `let mut fixed = 0;` | New code goes before this line |
| 100 | `for issue in &result.fixable {` | Fix iteration loop — must come after `ensure_sudo_cached()` |
| 121–124 | `FixAction::InstallHomebrew` match arm calls `Command::new("bash").args(["-c", "NONINTERACTIVE=1..."])` | Homebrew install in doctor --fix; benefits from pre-cached sudo; no change |

**Exact insertion site in doctor.rs:** Lines 97–98 boundary — after `let managers = ...;`, before `let mut fixed = 0;`.

**Note on Socrates Advisory 3:** The spec at Step 4 (line 236) shows `let info = platform::detect_platform_info();` inside the fix block. This is a mistake in the spec — `info` is already in scope from line 62. Da Vinci must use the outer `info` directly:

```rust
match ensure_sudo_cached(info.is_root) {
```

not:

```rust
let info = platform::detect_platform_info();  // DO NOT add this
match ensure_sudo_cached(info.is_root) {
```

### `src/cli/mod.rs`

| Line | What is there | Action |
|------|--------------|--------|
| 9 | `pub mod status;` | New line `pub mod sudo;` goes after this |
| 10 | `pub mod statusline;` | New module sits between `status` and `statusline` alphabetically |

### New file: `src/cli/sudo.rs`

Does not exist. Must be created. Full implementation given verbatim in spec.

---

## Existing Patterns to Follow

### Module declaration style
`src/cli/mod.rs` lines 1–16: all module declarations are bare `pub mod name;` lines, no `use` re-exports at this level.

### `output::` usage pattern (from `output.rs`)
```rust
output::info("message");     // blue ℹ — informational
output::warning("message");  // yellow ⚠ — non-fatal concern
output::success("message");  // green ✓ — confirmation
output::error("message");    // red ✗ — failure (does not bail)
```
All write to stderr. The new `ensure_sudo_cached()` uses `output::info(...)` and `output::warning(...)` — correct pattern.

### Error handling pattern
No `anyhow::bail!()` in `ensure_sudo_cached()` — the function returns an enum, not `Result`. All callers do `match` and assign `Some(keepalive)` or `None`. This matches the project's pattern of non-fatal warnings for optional operations (e.g., font install in `apply.rs:246–256`).

### `which` crate usage
Already used in `src/platform/detection.rs`. Pattern in sudo.rs: `which::which("sudo").is_err()` — consistent with how the crate is used elsewhere.

### `IsTerminal` usage
Confirmed at `src/cli/loop_cmd.rs:195`:
```rust
use std::io::IsTerminal;
```
Standard library (Rust 1.70+). Identical import needed in `sudo.rs`.

### `std::process::Command` pattern
Bootstrap.rs uses `Command::new("sudo").args([...]).status()` throughout. The new `ensure_sudo_cached()` uses `.stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit())` to pass through the password prompt — this is correct and intentional (vs bootstrap's `Stdio::null()`).

### Keepalive/background thread pattern
No existing precedent in this codebase. The pattern matches `std::sync::atomic::AtomicBool` + `thread::spawn` + `Drop`. This is idiomatic Rust — no external crate needed.

---

## Dependency Map

```
src/cli/sudo.rs (NEW)
  imports:
    std::io::IsTerminal       (stdlib, stable since 1.70)
    std::process::{Command, Stdio}
    std::sync::atomic::{AtomicBool, Ordering}
    std::sync::Arc
    std::thread
    std::time::Duration
    which (Cargo.toml line 25 — which = "7" — already present)
    crate::cli::output        (output.rs, already exists)

src/cli/mod.rs
  adds: pub mod sudo;

src/cli/apply.rs
  adds import: crate::cli::sudo::{ensure_sudo_cached, SudoCacheResult}
    (inline use inside the if-block, not a top-level use)
  reads: info.is_root (PlatformInfo.is_root, detection.rs:73)
  reads: info.capabilities.has_homebrew (PlatformCapabilities.has_homebrew, detection.rs:61)
  calls: bootstrap::is_apt_distro(&info.platform) (bootstrap.rs:7 — pub fn)

src/cli/doctor.rs
  adds import: crate::cli::sudo::{ensure_sudo_cached, SudoCacheResult}
    (inline use inside the fix block)
  reads: info.is_root (outer scope, line 62)
  reads: result.fixable (DiagnosticResult.fixable — Vec<FixableIssue>)
  matches: FixAction variants (InstallHomebrew, InstallSystemPrerequisite, InstallDocker, FixInotifyWatches)
```

No new crate entries in `Cargo.toml` required.

---

## Risks and Gotchas

### Risk 1 — Socrates Advisory 3 (HIGH IMPORTANCE)
The spec's Step 4 code block shows `let info = platform::detect_platform_info();` inside the fix block. **This is wrong.** `info` is already bound at `doctor.rs:62` and is in scope for the entire `run()` function. Da Vinci must skip this line and use the outer `info` directly. Adding a second `detect_platform_info()` call wastes time reading `/etc/os-release` and other system files.

### Risk 2 — Keepalive thread join latency (LOW)
The keepalive thread sleeps 60 seconds before checking the stop flag. Drop will block up to 60s if called while the thread is sleeping. Socrates marked this ADVISORY (acceptable). Da Vinci may optionally break the sleep into 1-second increments to make `Drop` nearly instant — but this is not required for correctness.

### Risk 3 — `--non-interactive` global flag not wired
`src/cli/mod.rs:34` declares `pub non_interactive: bool` as a global flag, but it is not passed into `apply::run()` or `doctor::run()`. `ensure_sudo_cached()` therefore cannot check it. Socrates confirmed this is a **pre-existing gap**, out of scope for 0025. The doc comment in the spec mentions `--non-interactive flag` as a skip condition — Da Vinci should either remove that mention or add a TODO comment per Socrates Advisory 1.

### Risk 4 — `needs_sudo` logic duplication in apply.rs
The spec's `needs_sudo` block (lines 187–200 of the spec) reimplements the `needs_homebrew` platform match from `apply.rs:406–415`. This duplication is intentional — the sudo prompt must precede `ensure_prerequisites()` at line 400, which is before the `needs_homebrew` variable is computed. A comment in the code explaining this ordering constraint will prevent future refactoring mistakes.

### Risk 5 — Test assertion precision
The spec's integration test (`apply_dry_run_no_sudo_prompt`) only asserts `.success()`, not that the "administrator access" info line is absent. With piped stdin, `NonInteractive` path is taken so no message is printed anyway. Socrates Advisory 6: optionally add `.stderr(predicate::str::contains("administrator access").not())` for precision.

---

## Test File Location and Patterns

### Integration tests
**File:** `/home/isaac/src/sh.great/tests/cli_smoke.rs`

Pattern for a new integration test:
```rust
#[test]
fn apply_dry_run_no_sudo_prompt() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        "[tools.runtimes]\nnode = \"22\"\n",
    ).unwrap();

    great()
        .current_dir(dir.path())
        .args(["apply", "--dry-run"])
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success();
}
```

Tests that require real package installs are tagged `#[ignore]` — see `cli_smoke.rs:107`. Follow this pattern for any test that would invoke actual sudo or Homebrew.

### Unit tests
Go inside `src/cli/sudo.rs` under `#[cfg(test)] mod tests { ... }`. See `src/cli/bootstrap.rs:410–490` for the existing unit test pattern in this codebase. The spec provides two specific unit tests: `already_root_returns_immediately` and `keepalive_drop_signals_stop` — both are safe to run in CI.

---

## Recommended Build Order

1. **Create** `/home/isaac/src/sh.great/src/cli/sudo.rs` — full module from spec (includes unit tests)
2. **Edit** `/home/isaac/src/sh.great/src/cli/mod.rs` — add `pub mod sudo;` at line 10 (between `status` and `statusline`)
3. **Edit** `/home/isaac/src/sh.great/src/cli/apply.rs` — insert `needs_sudo` + `_sudo_keepalive` block after line 397, before line 399
4. **Edit** `/home/isaac/src/sh.great/src/cli/doctor.rs` — insert `has_sudo_fix` + `_sudo_keepalive` block after line 97, before line 98 (using outer `info`, NOT a new `detect_platform_info()` call)
5. **Edit** `/home/isaac/src/sh.great/tests/cli_smoke.rs` — add `apply_dry_run_no_sudo_prompt` integration test
6. `cargo check` — verify no compile errors
7. `cargo clippy` — verify no new warnings
8. `cargo test` — run full suite

Steps 1–2 must be done together (module must be registered before it is used). Steps 3–4 are independent of each other and could be done in parallel.
