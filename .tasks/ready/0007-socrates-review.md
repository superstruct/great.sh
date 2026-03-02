# Socrates Review: 0007 Package Manager Abstraction Layer

**Spec:** `/home/isaac/src/sh.great/.tasks/ready/0007-package-manager-spec.md`
**Task:** `/home/isaac/src/sh.great/.tasks/backlog/0007-package-manager.md`
**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-24

---

## VERDICT: REJECTED

One BLOCKING concern must be resolved before build. The spec will cause a compilation failure because it misses a call site.

---

## Concerns

### 1. BLOCKING -- Missing call site in `doctor.rs`

```
{
  "gap": "The spec claims changes are needed only in `package_manager.rs` and `apply.rs`, listing three call sites at apply.rs lines 542, 700, 773. But `available_managers()` is also called at `/home/isaac/src/sh.great/src/cli/doctor.rs` line 108.",
  "question": "What happens to `doctor.rs:108` when `available_managers()` changes signature to `available_managers(non_interactive: bool)`? It will fail to compile.",
  "severity": "BLOCKING",
  "recommendation": "Add `doctor.rs` to the Files to Modify table. Update doctor.rs line 108 from `package_manager::available_managers()` to `package_manager::available_managers(false)`. Update the verification gate to allow changes in `doctor.rs` as well."
}
```

Evidence -- the grep result:

```
src/cli/doctor.rs:108:        let managers = package_manager::available_managers();
```

The spec's verification gate says: "git diff shows changes only in `src/platform/package_manager.rs` and `src/cli/apply.rs`". This is wrong. `doctor.rs` must also change or the build breaks.

---

### 2. ADVISORY -- Existing `test_available_managers_returns_non_empty` not listed in step-by-step

```
{
  "gap": "Step 9 updates `test_apt_is_available` to use `Apt::new(false)`, but `test_available_managers_returns_non_empty` at line 419 also calls `available_managers()` without arguments. After the signature change, this test will fail to compile.",
  "question": "Is the builder expected to infer this, or should it be explicitly listed as a step?",
  "severity": "ADVISORY",
  "recommendation": "Add a step (e.g. Step 9b) to update line 419 from `available_managers()` to `available_managers(false)`. Or note it alongside Step 9 since the pattern is identical."
}
```

Evidence -- current code at line 419:
```rust
let managers = available_managers();
```

The new tests do use `available_managers(false)` which is correct, but the existing test is not called out for update. A careful builder will catch this, but explicit is better.

---

### 3. ADVISORY -- Spec describes `is_available()` as using `which::which()` but code uses `command_exists()`

```
{
  "gap": "The Interface specification tables say is_available() uses `which::which(\"brew\")`, `which::which(\"apt-get\")`, etc. The actual implementation uses `command_exists(\"brew\")` from `super::detection`, which wraps `which::which`. The spec's method tables are inconsistent with the existing code.",
  "question": "Will the builder implement new `which::which` calls directly, breaking the pattern of using the codebase's `command_exists()` wrapper?",
  "severity": "ADVISORY",
  "recommendation": "Clarify in the Interface tables that `is_available()` uses `command_exists()` (the existing wrapper), not `which::which` directly. The current code at package_manager.rs line 49 reads `command_exists(\"brew\")`, not `which::which(\"brew\")`."
}
```

---

### 4. ADVISORY -- Task requirement for `dnf` (Fedora/RHEL) is silently dropped

```
{
  "gap": "The original task at 0007-package-manager.md mentions 'dnf on Fedora/RHEL' in the Context section. The spec implements only Homebrew, Apt, Cargo, and Npm -- no Dnf struct. The spec does not acknowledge or justify this omission.",
  "question": "Is Fedora/RHEL support intentionally deferred? If so, should the spec state this explicitly so future readers know it was a deliberate scoping decision rather than an oversight?",
  "severity": "ADVISORY",
  "recommendation": "Add a sentence to the Summary or a 'Not in scope' section stating that Dnf support is deferred, matching the existing code's scope."
}
```

---

### 5. ADVISORY -- Non-interactive error message is misleading when sudo fails for non-password reasons

```
{
  "gap": "When `sudo -n` fails (non-zero exit code), the spec always blames a password prompt: 'sudo requires a password'. But `sudo -n` can also fail because the user is not in sudoers, sudo is not installed, or the sudoers policy denies the command.",
  "question": "Should the error message be more generic (e.g., 'sudo -n failed') or should it attempt to distinguish password-required from other sudo failures?",
  "severity": "ADVISORY",
  "recommendation": "Consider softening the message to 'apt-get install <package> failed -- sudo -n returned non-zero (password may be required or sudo access denied). Run interactively or use: sudo apt-get install -y <package>'. Alternatively, accept the current wording as 'good enough' since the actionable command is still correct."
}
```

---

### 6. ADVISORY -- `is_available()` guard is redundant when called through `available_managers()`

```
{
  "gap": "The factory function `available_managers()` already checks `is_available()` before adding a manager to the list. The spec adds `is_available()` guards inside each `install()`/`update()` method too. This means the check runs twice on the normal path (factory -> install).",
  "question": "Is the double-check intentional (defensive programming for direct construction like `Homebrew.install(...)`) or is it unnecessary overhead?",
  "severity": "ADVISORY",
  "recommendation": "The spec already justifies this in Gap #2 ('If the underlying tool is not on PATH, std::process::Command::new returns a confusing error'). This is fine as defensive programming. No change needed, but the builder should understand the guard exists for direct-construction callers, not for the factory path."
}
```

---

### 7. ADVISORY -- `CargoInstaller::installed_version` returns the full `--version` output, not a semver string

```
{
  "gap": "The spec says `installed_version` 'returns first line trimmed'. For cargo-installed tools, `<tool> --version` often outputs something like 'ripgrep 14.1.0' or 'bat 0.24.0 (fc95468)'. The method returns the entire first line, not just the version number.",
  "question": "Is this intentional? The task requirement says 'get the installed version'. Returning 'ripgrep 14.1.0' instead of '14.1.0' may confuse version comparison logic downstream.",
  "severity": "ADVISORY",
  "recommendation": "This is existing behavior (the spec is documenting what already exists), so it is not a regression. Flag for a future task if version comparison is needed."
}
```

---

### 8. ADVISORY -- No test verifies the `sudo -n` path actually triggers

```
{
  "gap": "The spec adds 5 new tests but none exercise the non-interactive sudo path. The `test_apt_non_interactive_struct` test only checks that `Apt::new(true)` and `Apt::new(false)` have the same name and availability -- it does not test that `install()` actually passes `-n` to sudo.",
  "question": "Is the non-interactive sudo behavior testable in CI without root access? If not, is the manual verification step (Section: Manual verification, item 3) sufficient?",
  "severity": "ADVISORY",
  "recommendation": "Acknowledge in the spec that the `sudo -n` behavior is verified by code review + manual testing, not automated tests. The manual verification section covers this but the Testing Strategy section implies 12 tests cover all changes -- they do not cover the sudo -n branch."
}
```

---

## Summary

The spec is thorough and well-structured, with accurate line-number references, exhaustive edge cases, and correct code snippets. However, it has one BLOCKING defect: the signature change to `available_managers()` will break `/home/isaac/src/sh.great/src/cli/doctor.rs` line 108, which the spec does not account for. Once that call site is added to the Files to Modify table and the verification gate is updated, this spec is ready to build.

Resolve the BLOCKING concern (add `doctor.rs` to the change set) and this becomes APPROVED.
