# 0018: Bump Rust Version in Linux aarch64 Cross-Compilation Dockerfile

**Priority:** P2
**Type:** bugfix
**Module:** `docker/cross-linux-aarch64.Dockerfile`
**Status:** done (superseded by task 0019, which bumps all three cross-compilation Dockerfiles to Rust 1.88)
**Estimated Complexity:** XS

## Context

`docker build -f docker/cross-linux-aarch64.Dockerfile` and subsequent `docker run` will fail during `cargo build` because the pinned base image `rust:1.83-slim` ships Cargo 1.83.0, which does not support the `edition2024` Cargo feature. `time-core 0.1.8` (a transitive dependency pulled in via the `time` crate) declares `edition = "2024"` in its manifest, which requires this feature.

Error observed (same as in task 0017):

```
error: failed to parse manifest at time-core-0.1.8/Cargo.toml

Caused by:
  feature `edition2024` is required
  The package requires the Cargo feature called `edition2024`, but that feature is
  not stabilized in this version of Cargo (1.83.0)
```

### Root cause

`edition2024` was stabilized in Rust 1.85.0 (released 2025-02-20). Pinning to `rust:1.83-slim` on line 6 of `docker/cross-linux-aarch64.Dockerfile` means the compiler and Cargo pre-date that stabilization.

### Cargo.toml note

The project's own `Cargo.toml` declares `edition = "2021"` and specifies no explicit MSRV field. The `time`/`time-core` dependency is transitive — it is not listed directly but is pulled in by one or more direct dependencies (e.g., `reqwest`, `tokio`). Upgrading the Rust toolchain in the Dockerfile is the correct fix; no changes to `Cargo.toml` or `Cargo.lock` are required.

### Relationship to task 0017

Task 0017 identified and fixed the same issue in `docker/cross-windows.Dockerfile`. This task applies the same fix to the Linux aarch64 cross-compilation Dockerfile.

### Candidate fix

Change line 6 of `docker/cross-linux-aarch64.Dockerfile`:

```dockerfile
# Before
FROM rust:1.83-slim

# After
FROM rust:1.85-slim
```

`rust:1.85-slim` is the minimum version that satisfies the `edition2024` requirement. Pinning to `1.85` rather than `latest` or `stable` keeps the build reproducible while eliminating the breakage.

## Acceptance Criteria

- [ ] `docker build -f docker/cross-linux-aarch64.Dockerfile -t great-cross-aarch64 .` completes without error on a Linux host with Docker installed.
- [ ] `docker run --rm -v $(pwd):/workspace great-cross-aarch64` completes without error; Cargo resolves and compiles all crates including `time-core 0.1.8`.
- [ ] The produced artifact at `target/aarch64-unknown-linux-gnu/release/great` is a valid ELF binary (`file` reports `ELF 64-bit LSB executable, ARM aarch64`).
- [ ] `cargo clippy` and `cargo test` pass unchanged — the Dockerfile change introduces no modifications to Rust source or `Cargo.toml`.
- [ ] The `FROM` line in `docker/cross-linux-aarch64.Dockerfile` pins to a specific Rust version (`>= 1.85`) rather than a floating `latest` or `stable` tag.

## Files That Need to Change

- `docker/cross-linux-aarch64.Dockerfile` — line 6: change `FROM rust:1.83-slim` to `FROM rust:1.85-slim` (or a later pinned version).

## Dependencies

None. Standalone single-line Dockerfile change; no Rust source changes required. Can be implemented independently and in parallel with task 0017.

## Out of Scope

- Fixing other cross-compilation Dockerfiles (e.g., task 0015 macOS) — separate issues.
- Adding Dependabot or Renovate automation to keep Docker base images current — that is a follow-on hardening task.
- Pinning `time-core` to an older edition in `Cargo.lock` — not a viable fix as the dependency is transitive and controlled upstream.
