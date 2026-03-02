# 0027: Wire `--non-interactive` Flag -- Socrates Review

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Iteration:** 024
**Date:** 2026-02-27
**Spec reviewed:** `.tasks/ready/0027-non-interactive-spec.md`
**Backlog item:** `.tasks/backlog/0027-wire-non-interactive-flag.md`

---

## VERDICT: APPROVED

---

## Per-Change Analysis

### Change 1: `apply::Args` gains `#[arg(skip)] pub non_interactive: bool`

**Current state correctly identified?** Yes. The `Args` struct at `src/cli/apply.rs:354-367` matches the spec exactly -- three fields (`config`, `dry_run`, `yes`), no `non_interactive`.

**Does the proposed change fix the problem?** Yes. `#[arg(skip)]` is the correct clap 4 derive-mode attribute for fields that should not be parsed as CLI arguments. It defaults the field to `bool::default()` (i.e., `false`), which is the correct default when the global flag is absent.

**Alternative considered?** The backlog item mentions the alternative of passing `non_interactive` as a second parameter to `run()`. The spec chose the `#[arg(skip)]` struct field approach instead. This is the better choice: it keeps the `run()` signature uniform across subcommands and avoids the need to change the `Command::Apply(args) => run(args)` pattern to `run(args, flag)`. No concern here.

**Could this break something?** No. Adding a field with `#[arg(skip)]` to a clap derive struct does not change the CLI interface. Existing callers that construct `Args` directly (e.g., tests) would need to add the field, but there are no tests that construct `apply::Args` directly -- `cli_smoke.rs` tests invoke the binary via `assert_cmd`, not by calling `run()`.

### Change 2: `apply.rs` -- Pass `args.non_interactive` to `ensure_sudo_cached`

**Current state correctly identified?** Yes. Line 421 of `apply.rs` shows `ensure_sudo_cached(info.is_root)` with one argument.

**Does the proposed change fix the problem?** Yes. After Change 7 extends the signature, this call site must pass the second argument.

### Change 3: `apply.rs` -- Pass `args.non_interactive` to `available_managers` (3 sites)

**Current state correctly identified?** Yes. Verified all three call sites:
- Line 572: `package_manager::available_managers(false)` (CLI tools section)
- Line 730: `package_manager::available_managers(false)` (bitwarden-cli install)
- Line 803: `package_manager::available_managers(false)` (platform-specific tools)

All three currently pass hardcoded `false`.

**Does the proposed change fix the problem?** Yes. The `available_managers` function already accepts `non_interactive: bool` and passes it through to `Apt::new(non_interactive)`, which controls whether apt uses `sudo -n` (fail-fast) vs. `sudo` (interactive). Passing the actual flag value propagates the user's intent correctly.

### Change 4: `doctor::Args` gains `#[arg(skip)] pub non_interactive: bool`

Same analysis as Change 1. The `Args` struct at `src/cli/doctor.rs:10-15` matches the spec. Same pattern, same reasoning. No concerns.

### Change 5: `doctor.rs` -- Pass `args.non_interactive` to `available_managers`

**Current state correctly identified?** Yes. Line 97: `package_manager::available_managers(false)`.

**Does the proposed change fix the problem?** Yes.

**Concern about scope:** The `args` variable is available in the `if args.fix` block where this call lives (line 94). The borrow is fine since `args.non_interactive` is `Copy` (`bool`). No ownership issue.

### Change 6: `doctor.rs` -- Pass `args.non_interactive` to `ensure_sudo_cached`

**Current state correctly identified?** Yes. Line 112: `ensure_sudo_cached(info.is_root)`.

**Does the proposed change fix the problem?** Yes.

### Change 7: `sudo.rs` -- Extend `ensure_sudo_cached` signature, add `non_interactive ||` check

**Current state correctly identified?** Yes. Lines 58-71 match the spec's "current code" block exactly, including the TODO comment at lines 61-62.

**Does the proposed change fix the problem?** Yes. The new condition `if non_interactive || !std::io::stdin().is_terminal()` correctly short-circuits: when the flag is `true`, it returns `NonInteractive` without touching stdin or sudo. When the flag is `false`, it falls through to the existing terminal check.

**Is the ordering correct?** Yes. The `is_root` check (line 65) runs first, then `non_interactive || !is_terminal()`. This means:
- Root always returns `AlreadyRoot` (correct -- root does not need sudo regardless of flags).
- `non_interactive` is checked before `which::which("sudo")`, which is correct: if the user asked for non-interactive mode, we should not even check for sudo.

**TODO removal:** The spec removes the TODO at lines 61-62 and replaces it with a proper doc comment for the new parameter. Correct.

### Change 8: `sudo.rs` -- Update existing unit test

**Current state correctly identified?** Yes. Line 138: `ensure_sudo_cached(true)` with one argument.

**Does the proposed change fix the problem?** Yes. The test passes `(true, false)` meaning "is root, not non-interactive" -- this tests the `AlreadyRoot` path, which is unchanged in behavior.

### Change 9: `sudo.rs` -- Add new unit test for `non_interactive`

**Does this test what it claims?** Yes. `ensure_sudo_cached(false, true)` means "not root, but non-interactive" -- this should return `NonInteractive` via the new `non_interactive ||` check, without ever reaching the `stdin().is_terminal()` call or the `sudo -v` invocation. The `assert!(matches!(result, SudoCacheResult::NonInteractive))` verifies exactly this.

**Is this test safe to run in CI?** Yes. When `non_interactive` is `true`, the function returns before any system calls. No sudo prompt, no stdin check.

### Change 10: `main.rs` -- Extract `non_interactive` and forward to `Apply` and `Doctor`

**Current state correctly identified?** Yes. Lines 13-29 match exactly.

**Does the proposed change fix the problem?** Yes. The `let non_interactive = cli.non_interactive;` binding extracts the value before the `match cli.command` consumes the `command` field via move. Then `mut args` + field assignment in the `Apply` and `Doctor` arms sets it correctly.

**Is the move semantics analysis correct?** Yes. `match cli.command` moves `command` out of `cli`. After that, `cli` is partially moved and cannot be accessed. By extracting `non_interactive` (a `Copy` type, `bool`) beforehand, the value is preserved. This is the standard Rust pattern for this situation.

---

## Gaps Found

### Concern 1: `bootstrap.rs` calls `sudo` directly without respecting `--non-interactive`

```
{
  "gap": "bootstrap::run_sudo_apt_install() and bootstrap::ensure_docker() call Command::new('sudo') directly at 15+ call sites. These calls inherit stdin and will prompt for a password even when --non-interactive is passed. The spec only wires the flag through ensure_sudo_cached and available_managers, but the bootstrap module bypasses both.",
  "question": "When a user runs `great apply --non-interactive` on Ubuntu and the sudo cache has NOT been primed, ensure_sudo_cached returns NonInteractive (no prompt) -- but then bootstrap::ensure_prerequisites() at apply.rs:430 immediately calls run_sudo_apt_install() which runs `Command::new('sudo').args(['apt-get', 'install', '-y', ...])`. This sudo invocation inherits stdin and will still prompt for a password. Is the spec's claim that 'exits without any sudo prompt' actually achievable with these changes alone?",
  "severity": "ADVISORY",
  "recommendation": "Document this as a known limitation in the spec. The ensure_sudo_cached path is fixed, and available_managers will use sudo -n via Apt::new(true), but bootstrap's direct sudo calls remain. This is acceptable because (a) ensure_sudo_cached's NonInteractive return means the keepalive is not started, so subsequent sudo calls will fail fast if no cached credentials exist, and (b) fixing bootstrap.rs to accept non_interactive is a separate, larger refactor. However, the acceptance criterion 'exits without any sudo prompt' is misleading -- it should say 'ensure_sudo_cached does not prompt' rather than claiming the entire apply run is prompt-free."
}
```

**Severity rationale:** ADVISORY, not BLOCKING. The spec's changes are correct and valuable as a first step. The bootstrap.rs gap existed before this task and is explicitly out of scope per the backlog item's statement that the task covers "ensure_sudo_cached or available_managers" call sites. The sudo calls in bootstrap.rs are a pre-existing condition. However, the acceptance criteria should acknowledge this limitation rather than implying full coverage.

### Concern 2: `tuning.rs` also calls `sudo` directly

```
{
  "gap": "tuning::apply_system_tuning() calls Command::new('sudo').args(['sysctl', ...]) at tuning.rs:65 and uses 'sudo tee' at tuning.rs:92. This is called from both apply.rs:839 and doctor.rs:216 (FixAction::FixInotifyWatches). Same issue as bootstrap.rs -- direct sudo calls bypass the non_interactive flag.",
  "question": "Is the tuning.rs sudo usage covered by the ensure_sudo_cached pre-caching (since it runs after the cache attempt), or will it still prompt when --non-interactive is set?",
  "severity": "ADVISORY",
  "recommendation": "Same as Concern 1 -- document as known limitation. When --non-interactive is set, ensure_sudo_cached returns NonInteractive (no keepalive), so subsequent sudo calls in tuning.rs will prompt or fail depending on whether the user has cached credentials by other means."
}
```

### Concern 3: `bootstrap::ensure_prerequisites` is called BEFORE the sudo cache check can take effect

```
{
  "gap": "In apply.rs, the execution order is: (1) ensure_sudo_cached at line 419-427, (2) ensure_prerequisites at line 430. The spec correctly wires non_interactive into the ensure_sudo_cached call. But bootstrap::ensure_prerequisites itself runs sudo commands directly. When --non-interactive is true, ensure_sudo_cached returns NonInteractive (no keepalive started), so bootstrap's sudo commands will run without cached credentials.",
  "question": "Is this the desired behavior? The spec says 'apt commands using sudo -n will fail fast with actionable error' -- but bootstrap::run_sudo_apt_install does NOT use sudo -n; it uses bare 'sudo apt-get install -y'. It will hang waiting for a password, not fail fast.",
  "severity": "ADVISORY",
  "recommendation": "The behavioral table in Section 4 should clarify that the 'fail fast' behavior only applies to commands routed through package_manager::Apt::new(true) (which uses sudo -n). Direct sudo calls in bootstrap.rs do NOT use sudo -n and will hang or prompt. This is a pre-existing issue, but the spec should not imply it is solved."
}
```

### Concern 4: No integration test for flag acceptance

```
{
  "gap": "The spec states 'No new integration tests required' and suggests manual verification. However, a simple smoke test verifying that the flag is accepted without error would be trivial and valuable.",
  "question": "Would it not be prudent to add at least one integration test like `great apply --non-interactive --dry-run` or `great doctor --non-interactive --help` to cli_smoke.rs, verifying the flag does not produce an 'unrecognized argument' error?",
  "severity": "ADVISORY",
  "recommendation": "Consider adding a minimal smoke test. The --dry-run variant is safe (no system modifications) and would catch regressions if the #[arg(skip)] pattern is accidentally removed."
}
```

---

## Risk Assessment

**Overall risk: LOW**

This is a straightforward plumbing task. The changes are mechanical: add a field, pass it through, extend a signature. No new error paths, no new dependencies, no new files.

**What could go wrong:**
1. **`#[arg(skip)]` misuse** -- If `skip` did not default the field to `false`, constructing `Args` via clap would fail. But clap 4's `#[arg(skip)]` uses `Default::default()` for the field type, and `bool::default()` is `false`. Verified correct.
2. **Duplicate flag error** -- If someone accidentally removes `#[arg(skip)]` and adds `#[arg(long)]`, the global `--non-interactive` and subcommand-level `--non-interactive` would conflict. The spec documents this risk in the Notes section. Low probability.
3. **Incomplete coverage** -- As noted in Concerns 1-3, direct `sudo` calls in `bootstrap.rs` and `tuning.rs` are not covered. This is a pre-existing condition, not a regression introduced by this spec.

**What the spec gets right:**
- Correct identification of all `ensure_sudo_cached` and `available_managers(false)` call sites (verified by grep).
- Correct use of `#[arg(skip)]` for injecting global state into subcommand args.
- Correct move semantics analysis for `main.rs` (extracting `non_interactive` before the match).
- Correct short-circuit ordering in the `non_interactive || !is_terminal()` condition.
- Edge cases are thoroughly analyzed (dry-run, already-root, piped stdin, no sudo binary).
- Build order is correct and the dependency chain is clearly explained.
- Line numbers are accurate against the current codebase.

---

## Backlog Cross-Reference

The spec addresses all 5 requirements from the backlog item:

| Backlog Req | Spec Coverage | Verified |
|-------------|--------------|----------|
| 1. main.rs must extract and forward non_interactive | Change 10 | Yes |
| 2. apply::Args must gain non_interactive field | Change 1 | Yes |
| 3. doctor::Args must gain the same treatment | Change 4 | Yes |
| 4. ensure_sudo_cached signature must be extended | Change 7 | Yes |
| 5. available_managers(false) must pass actual flag | Changes 3, 5 | Yes (all 4 call sites) |

All 5 acceptance criteria from the backlog are addressed by the spec's 9 acceptance criteria.

---

## Summary

A clean, well-scoped spec for a mechanical plumbing task. All line numbers verified, all call sites accounted for, correct Rust patterns used. The four advisory concerns relate to pre-existing limitations in `bootstrap.rs` and `tuning.rs` that are explicitly out of scope but should be acknowledged in the behavioral documentation rather than implied as solved. None are blocking.
