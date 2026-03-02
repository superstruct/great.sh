# Humboldt Scout Report: 0007 Package Manager Abstraction Layer

**Date:** 2026-02-24
**Spec:** `.tasks/ready/0007-package-manager-spec.md`
**Socrates verdict:** REJECTED (one BLOCKING resolved below)

---

## Complete Call-Site Map for `available_managers()`

| File | Line | Current Call | Required Change |
|------|------|-------------|-----------------|
| `src/platform/package_manager.rs` | 348 | `pub fn available_managers() -> ...` | Change signature to accept `non_interactive: bool` |
| `src/platform/package_manager.rs` | 419 | `available_managers()` (in test) | `available_managers(false)` |
| `src/cli/apply.rs` | 542 | `package_manager::available_managers()` | `package_manager::available_managers(false)` |
| `src/cli/apply.rs` | 700 | `package_manager::available_managers()` | `package_manager::available_managers(false)` |
| `src/cli/apply.rs` | 773 | `package_manager::available_managers()` | `package_manager::available_managers(false)` |
| `src/cli/doctor.rs` | 108 | `package_manager::available_managers()` | `package_manager::available_managers(false)` — **BLOCKING: missed by spec, caught by Socrates** |

Total call sites: 5 external + 1 internal test = **6 sites** to update.

---

## Import Map

| File | Import Statement |
|------|-----------------|
| `src/cli/apply.rs` | `use crate::platform::package_manager::{self, PackageManager};` (line 11) |
| `src/cli/doctor.rs` | `use crate::platform::package_manager;` (line 6) — module-level only, no `PackageManager` trait import |
| `src/platform/mod.rs` | `pub mod package_manager;` (line 2) — declares module, no re-exports of individual items |

`package_manager` is NOT re-exported from `src/platform/mod.rs` with `pub use`. Consumers import it by full path. No re-export change is needed.

---

## Existing Struct Map (`src/platform/package_manager.rs`)

| Item | Lines | Type | Notes |
|------|-------|------|-------|
| `PackageManager` trait | 6–26 | trait | Object-safe. `Box<dyn PackageManager>` works. |
| `Homebrew` | 41 | unit struct | No fields. Add `is_available()` guard to `install` (line 76) and `update` (line 105). |
| `Homebrew::install` | 76–103 | fn | Add guard at line 77 (before idempotency check). |
| `Homebrew::update` | 105–114 | fn | Add guard at line 106. |
| `Apt` | 127 | unit struct `pub struct Apt;` | **Must become struct with `non_interactive: bool` field.** |
| `Apt::install` | 164–176 | fn | Add `is_available()` guard + `sudo -n` conditional. |
| `Apt::update` | 178–188 | fn | Add `is_available()` guard + `sudo -n` conditional. |
| `CargoInstaller` | 196 | unit struct | Add `is_available()` guard to `install` (line 232) and `update` (line 252). |
| `NpmInstaller` | 270 | unit struct | Add `is_available()` guard to `install` (line 307) and `update` (line 325). |
| `available_managers()` | 348–374 | pub fn | Change signature; change `let apt = Apt;` (line 369) to `let apt = Apt::new(non_interactive);` |

---

## Existing Tests (7 total in `package_manager.rs`)

| Name | Lines | Notes |
|------|-------|-------|
| `test_homebrew_is_available` | 385–389 | No change needed. |
| `test_apt_is_available` | 391–395 | `let apt = Apt;` must become `let apt = Apt::new(false);` |
| `test_cargo_is_available` | 397–402 | No change needed. |
| `test_cargo_is_installed_for_existing_binary` | 404–409 | No change needed. |
| `test_cargo_not_installed_for_fake_package` | 411–415 | No change needed. |
| `test_available_managers_returns_non_empty` | 417–422 | `available_managers()` → `available_managers(false)` |
| `test_trait_is_object_safe` | 424–428 | No change needed. |

**ADVISORY from Socrates:** Step 9 in the spec updates `test_apt_is_available` but does not explicitly mention updating `test_available_managers_returns_non_empty` (line 419). Both must change.

---

## Exact Lines to Modify

### `src/platform/package_manager.rs`

```
Line 127:  pub struct Apt;
           → pub struct Apt { non_interactive: bool }
           → Add impl Apt { pub fn new(non_interactive: bool) -> Self { ... } }

Line 164:  fn install(&self, package: &str, _version: Option<&str>) -> Result<()> {
           → Add is_available() guard at top
           → Add sudo -n conditional branch

Line 178:  fn update(&self, package: &str) -> Result<()> {
           → Add is_available() guard at top
           → Add sudo -n conditional branch

Line 76:   fn install(&self, package: &str, version: Option<&str>) -> Result<()> { (Homebrew)
           → Add is_available() guard before line 77

Line 105:  fn update(&self, package: &str) -> Result<()> { (Homebrew)
           → Add is_available() guard before line 106

Line 232:  fn install(&self, package: &str, version: Option<&str>) -> Result<()> { (CargoInstaller)
           → Add is_available() guard before line 233

Line 252:  fn update(&self, package: &str) -> Result<()> { (CargoInstaller)
           → Add is_available() guard before line 253

Line 307:  fn install(&self, package: &str, version: Option<&str>) -> Result<()> { (NpmInstaller)
           → Add is_available() guard before line 308

Line 325:  fn update(&self, package: &str) -> Result<()> { (NpmInstaller)
           → Add is_available() guard before line 326

Line 348:  pub fn available_managers() -> Vec<Box<dyn PackageManager>> {
           → pub fn available_managers(non_interactive: bool) -> Vec<Box<dyn PackageManager>> {

Line 369:  let apt = Apt;
           → let apt = Apt::new(non_interactive);

Line 393:  let apt = Apt;    (in test_apt_is_available)
           → let apt = Apt::new(false);

Line 419:  let managers = available_managers();    (in test_available_managers_returns_non_empty)
           → let managers = available_managers(false);
```

### `src/cli/apply.rs`

```
Line 542:  let managers = package_manager::available_managers();
           → let managers = package_manager::available_managers(false);

Line 700:  let managers = package_manager::available_managers();
           → let managers = package_manager::available_managers(false);

Line 773:  let managers = package_manager::available_managers();
           → let managers = package_manager::available_managers(false);
```

### `src/cli/doctor.rs` — BLOCKING FIX

```
Line 108:  let managers = package_manager::available_managers();
           → let managers = package_manager::available_managers(false);
```

---

## New Tests to Add (5)

Append to the `mod tests` block in `src/platform/package_manager.rs` after line 428:

1. `test_homebrew_is_installed_nonexistent` — `Homebrew.is_installed("nonexistent_package_xyz_12345")` returns `false` without panic.
2. `test_npm_is_installed_nonexistent` — `NpmInstaller.is_installed(...)` returns `false` without panic.
3. `test_apt_non_interactive_struct` — `Apt::new(true)` and `Apt::new(false)` both return `"apt"` for `name()` and have identical `is_available()` results.
4. `test_available_managers_with_non_interactive_flag` — Passing `true` vs `false` yields the same set of manager names.
5. `test_all_managers_name_non_empty` — Every manager from `available_managers(false)` has a non-empty name.

All test code is in the spec at lines 458–503.

---

## Dependency Map

```
src/platform/package_manager.rs
    ↑ imports
    super::detection::command_exists         (no change)
    anyhow::{bail, Context, Result}          (no change)

    ↓ consumed by
    src/cli/apply.rs     line 11   (import)
                         line 542  (call site 1)
                         line 700  (call site 2)
                         line 773  (call site 3)
    src/cli/doctor.rs    line 6    (import)
                         line 108  (call site 4 — BLOCKING)

    ↓ declared in
    src/platform/mod.rs  line 2    (pub mod package_manager — no re-export)
```

The `which` crate is already in `Cargo.toml` line 26 (`which = "7"`). The spec describes `is_available()` as using `which::which()` directly but the actual code uses `command_exists()` from `super::detection`. Keep using `command_exists()` — do NOT introduce direct `which::which` calls (Socrates Advisory #3).

The global `--non-interactive` flag exists on `Cli` in `src/cli/mod.rs` line 34 (`pub non_interactive: bool`) but is NOT threaded to subcommand `Args` structs. The spec correctly passes `false` at all call sites for now, deferring that wiring to a future task.

---

## Risks and Surprises

1. **BLOCKING resolved:** `doctor.rs` line 108 was missed by the spec. It is a compile-time failure — the build will not pass until it is fixed. No ambiguity: must be changed to `available_managers(false)`.

2. **Two tests in package_manager.rs need signature fixes** — not just one (spec Step 9 only mentions `test_apt_is_available`; `test_available_managers_returns_non_empty` also calls the old signature).

3. **`Apt` is a unit struct today.** Changing `pub struct Apt;` to a struct with fields is a breaking change at every `Apt` construction site. The only construction sites are:
   - `available_managers()` at line 369 (same file) — will be updated
   - `test_apt_is_available` at line 393 (same file) — will be updated
   No external construction sites found anywhere else in the codebase.

4. **`apply.rs` calls `available_managers()` three times** in separate code paths:
   - Line 542: CLI tools install block
   - Line 700: Bitwarden CLI install block (step 5b)
   - Line 773: Platform-specific tools block (step 7)
   All three are local `let managers = ...` bindings. They are not shared. All three pass `false`.

5. **`CargoInstaller::is_installed` uses `command_exists(package)`** — it checks if the binary is on PATH, not if cargo knows about it. Adding an `is_available()` guard before `install` is correct; no behavior change on the happy path.

6. **No integration tests in `tests/cli_smoke.rs`** exercise `package_manager` directly or call `great apply` with package manager behavior. The smoke tests only check CLI surface. No changes needed in `tests/cli_smoke.rs`.

---

## Recommended Build Order

1. **Edit `src/platform/package_manager.rs`** — all changes in one file:
   a. Change `Apt` from unit struct to field struct; add `Apt::new()`
   b. Add `is_available()` guards to `Homebrew::install` and `Homebrew::update`
   c. Update `Apt::install` and `Apt::update` with guard + `sudo -n` branch
   d. Add `is_available()` guards to `CargoInstaller::install` and `CargoInstaller::update`
   e. Add `is_available()` guards to `NpmInstaller::install` and `NpmInstaller::update`
   f. Update `available_managers` signature and `Apt` construction
   g. Fix `test_apt_is_available` (line 393)
   h. Fix `test_available_managers_returns_non_empty` (line 419)
   i. Append 5 new tests

2. **Edit `src/cli/apply.rs`** — lines 542, 700, 773: add `false` argument.

3. **Edit `src/cli/doctor.rs`** — line 108: add `false` argument.

4. **Build gate:**
   ```bash
   cargo build 2>&1       # catches all compile errors
   cargo clippy -- -D warnings
   cargo test -- package_manager
   ```

Step 4 must exit 0 with 12 passing tests and zero warnings before declaring done.
