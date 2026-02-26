# Review: Spec 0025 -- Pre-cache sudo credentials before Homebrew install

**Reviewer:** Socrates
**Spec:** `.tasks/ready/0025-homebrew-sudo-spec.md`
**Backlog:** `.tasks/backlog/0025-homebrew-non-admin-failure-handling.md`
**Date:** 2026-02-26

---

## VERDICT: APPROVED (with ADVISORY notes)

The spec is well-structured, correctly identifies the root cause, and proposes a clean solution with appropriate edge case coverage. No BLOCKING concerns remain after analysis. The approach aligns with standard patterns (Homebrew's own `sudo -v`, Ansible `become`, macOS installer scripts).

---

## Concerns

### 1. `--non-interactive` global flag is ignored

```
{
  "gap": "The Cli struct at src/cli/mod.rs:34 declares `pub non_interactive: bool` as a global flag (`--non-interactive`), but ensure_sudo_cached() only checks `stdin().is_terminal()`. A user could pass `great --non-interactive apply` on an interactive terminal and still get a sudo prompt.",
  "question": "Should ensure_sudo_cached() also accept a non_interactive parameter that reflects the --non-interactive global CLI flag?",
  "severity": "ADVISORY",
  "recommendation": "This is a pre-existing architectural gap -- the --non-interactive flag is declared but not wired through to apply::run() or doctor::run() (main.rs does not pass it). Fixing it is out of scope for 0025. However, the spec's doc comment at line 68 mentions `--non-interactive flag` as a skip condition but the code does not check it. Either remove the mention from the doc comment, or add a TODO comment acknowledging this gap. The stdin().is_terminal() check is sufficient for the actual failure scenario (NONINTERACTIVE=1 in CI with piped stdin)."
}
```

### 2. Keepalive thread sleeps before checking stop flag -- up to 60s delay on Drop

```
{
  "gap": "The keepalive thread at line 112 calls `thread::sleep(Duration::from_secs(60))` BEFORE checking the stop flag. When SudoKeepalive is dropped, the join in Drop will block for up to 60 seconds waiting for the sleep to complete. The spec acknowledges this at line 294 ('The thread sleeps for at most 60 seconds, so the join blocks for at most 60 seconds') but dismisses it as acceptable.",
  "question": "Is a potential 60-second hang on process exit acceptable for a CLI tool? Could the sleep be broken into smaller intervals (e.g., 12 iterations of 5 seconds each, checking the stop flag between each)?",
  "severity": "ADVISORY",
  "recommendation": "Breaking the 60-second sleep into smaller chunks (e.g., 1-second sleeps in a loop checking the stop flag) would make shutdown nearly instant. This is a minor ergonomic improvement. The current design is functionally correct -- most apply runs take longer than 60 seconds anyway, and the thread will have already completed its first sleep by the time Drop is called. If the implementer wants to optimize this, they can; it does not block approval."
}
```

### 3. Redundant `detect_platform_info()` call in doctor.rs

```
{
  "gap": "The spec's doctor.rs insertion (Step 4, line 236) calls `let info = platform::detect_platform_info();` inside the fix block, but doctor.rs already has `let info = platform::detect_platform_info();` at line 62. The existing `info` binding is in scope.",
  "question": "Why call detect_platform_info() a second time when the result is already available as `info` from line 62?",
  "severity": "ADVISORY",
  "recommendation": "Use the existing `info` variable. Change `let info = platform::detect_platform_info();` in the fix block to just reference `info` from the outer scope. This is a minor inefficiency (detect_platform_info reads /etc/os-release etc.) but not a correctness issue."
}
```

### 4. Spec's `needs_sudo` logic in apply.rs duplicates Homebrew check logic

```
{
  "gap": "The spec's needs_sudo check at Step 3 (lines 187-200) reimplements the same platform-matching logic that already exists at apply.rs:406-415 (the `needs_homebrew` variable). The apply.rs code already computes `needs_homebrew` and checks `!info.capabilities.has_homebrew` at line 417.",
  "question": "Could the insertion point be moved to after the existing `needs_homebrew` calculation (after line 415) to reuse that variable instead of duplicating the match?",
  "severity": "ADVISORY",
  "recommendation": "The spec's insertion point is 'after line 397, before line 399' -- which is BEFORE the existing needs_homebrew calculation at line 406. The spec duplicates the logic because it needs to run before `ensure_prerequisites()`. However, `ensure_prerequisites()` at line 400 runs `sudo apt-get` calls (via bootstrap.rs:run_sudo_apt_install), so the sudo cache must be primed BEFORE that call. The duplication is therefore justified by ordering constraints. No change needed, but a code comment explaining why would help maintainability."
}
```

### 5. Spec claims 'no new crate dependencies' -- verify `which` usage

```
{
  "gap": "The spec uses `which::which('sudo')` and claims no new dependencies. Need to verify `which` is already in Cargo.toml.",
  "question": "Is the `which` crate already a dependency?",
  "severity": "RESOLVED",
  "recommendation": "Verified: `which = '7'` is at Cargo.toml line 25. No issue."
}
```

### 6. Integration test asserts `success()` but apply may exit non-zero without great.toml runtimes

```
{
  "gap": "The integration test at line 371-388 creates a great.toml with `[tools.runtimes]\\nnode = '22'` and runs `apply --dry-run`. The test asserts `.success()`. In dry-run mode this should work, but the test does not assert that no sudo-related output appears -- it only checks exit code.",
  "question": "Should the test also assert on stderr to confirm no sudo prompt was attempted (e.g., no 'administrator access' info message)?",
  "severity": "ADVISORY",
  "recommendation": "Adding `.stderr(predicates::str::contains('administrator access').not())` would make the test more precise. However, the test already runs with piped stdin (assert_cmd default), so NonInteractive path is taken and no info message is printed regardless. The test as written is sufficient as a regression guard for the --dry-run path."
}
```

### 7. Security: `sudo -v` extends timestamp for ALL sudo calls, not just great's

```
{
  "gap": "The spec correctly notes that sudo -v caches credentials using the system's standard mechanism. However, the keepalive thread extends this cache indefinitely (every 60 seconds) for the duration of the apply run. This means ANY process running under the same user can sudo without a prompt during this window.",
  "question": "Is the indefinite sudo cache extension an acceptable security posture for a developer tool?",
  "severity": "ADVISORY",
  "recommendation": "This is the same behavior as Homebrew's own installer, Ansible's become mechanism, and macOS system installer scripts. The attack window is limited to the duration of `great apply` (typically 1-5 minutes). The user explicitly authenticated. This is standard practice and acceptable for a developer-facing CLI tool. The spec's security section at lines 324-328 covers this adequately."
}
```

---

## Verification Summary

| Check | Result |
|-------|--------|
| Backlog alignment | PASS -- spec addresses the exact root cause described in backlog |
| Acceptance criteria match | PASS -- spec's AC is a superset of backlog's AC (adds dry-run, clippy, test criteria) |
| Line number accuracy | PASS -- verified apply.rs:390 (detect_platform_info), :397 (end of dry-run block), :399 (prerequisites comment), :433-438 (NONINTERACTIVE=1 brew install); doctor.rs:94 (fix block), :97 (managers), :122-124 (NONINTERACTIVE=1 brew) |
| Module structure | PASS -- `pub mod sudo;` between `status` (line 9) and `statusline` (line 10) is correct alphabetical insertion |
| `which` crate available | PASS -- Cargo.toml line 25 |
| `IsTerminal` precedent | PASS -- used in `src/cli/loop_cmd.rs:195` |
| `PlatformInfo.is_root` exists | PASS -- `src/platform/detection.rs:73` |
| `bootstrap::is_apt_distro` is pub | PASS -- `src/cli/bootstrap.rs:7` |
| No `.unwrap()` in production code | PASS -- spec code has zero unwrap calls |
| Error propagation | PASS -- all fallible operations return enum variants or use match, no panics |
| Edge cases covered | PASS -- non-interactive, root, no-sudo-binary, prompt-cancelled, timeout, dry-run, concurrent exit, full platform matrix |

---

## Summary

A focused, well-specified S-complexity enhancement that correctly solves the Homebrew `NONINTERACTIVE=1` failure with a standard `sudo -v` pre-cache pattern. The advisory notes are minor ergonomic improvements (sleep granularity, variable reuse, doc comment accuracy) that the implementer can address at their discretion without changing the spec. Approved for implementation.
