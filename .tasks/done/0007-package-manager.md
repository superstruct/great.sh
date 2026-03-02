# 0007: Package Manager Abstraction Layer

**Priority:** P1
**Type:** feature
**Module:** `src/platform/package_manager.rs`
**Status:** done (iteration 009, commit ceac8e6)

## What Was Done

The existing `package_manager.rs` implementation (PackageManager trait, Homebrew, Apt, CargoInstaller, NpmInstaller structs, factory function, 7 tests) was completed by closing 3 gaps:

1. **Non-interactive sudo handling for Apt**: Added `non_interactive: bool` field and `Apt::new()` constructor. When `non_interactive` is true, `sudo -n` is used (fails immediately if password needed) with a clear error message including the manual command to run.

2. **is_available() pre-checks**: All 4 implementations now have `is_available()` methods using `command_exists()`. Guards added to `install()` and `update()` methods to bail with clear error messages instead of spawning commands that would produce confusing OS-level errors.

3. **Additional unit tests**: 5 new tests added (total 12): homebrew nonexistent package, cargo is_installed, homebrew/npm install fails gracefully when manager absent, apt non_interactive struct.

All call sites updated for `available_managers(non_interactive: bool)` signature change: 3 in apply.rs, 1 in doctor.rs (Socrates blocking fix), 2 in existing tests.

## Acceptance Criteria

- [x] `cargo build` succeeds and `cargo clippy` produces zero warnings
- [x] The `PackageManager` trait compiles and is object-safe (`Box<dyn PackageManager>`)
- [x] Unit tests verify is_installed behavior and graceful Err returns
- [x] No `.unwrap()` or `.expect()` calls in production code
- [x] Module re-exported from `src/platform/mod.rs`

## Files Changed

| File | Change |
|------|--------|
| `src/platform/package_manager.rs` | Apt non_interactive, is_available() guards, 5 new tests |
| `src/cli/apply.rs` | 3 call sites: `available_managers()` → `available_managers(false)` |
| `src/cli/doctor.rs` | 1 call site: `available_managers()` → `available_managers(false)` |
