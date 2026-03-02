# 0020: Docker Cross-Compilation UX Improvements

**Priority:** P2 (overall; individual items rated below)
**Type:** bugfix / UX
**Module:** `docker/`
**Status:** complete
**Completed:** 2026-02-27
**Iteration:** 023
**Commit:** `2099d40`
**Estimated Complexity:** S
**Created:** 2026-02-25
**Source:** Nielsen UX review during Loop iteration 019

## Issues

### 1. [P2] Windows cross Dockerfile CMD skips validation
`docker/cross-windows.Dockerfile` CMD runs bare `cargo build` — skips validation/export if invoked directly via raw Docker (misleading relative to the Dockerfile usage comment which suggests running via compose with the test script).

### 2. [P2] test.sh silently swallows great doctor failures
`docker/test.sh` line 40: `|| true` silently swallows `great doctor` failures. A failing doctor check should at minimum produce a warning.

### 3. [P3] No toolchain version printed at container startup
Cross-compilation containers don't print the active Rust toolchain version at startup. Cannot confirm pinned Rust version is active without manual `rustc --version`.

### 4. [P3] Fragile mkdir in macOS cross container
`mkdir -p /workspace/test-files` inside the macOS cross container runs against a partially read-only workspace mount. This is undocumented and fragile.

## Acceptance Criteria
- [x] Each issue resolved or explicitly deferred with rationale
