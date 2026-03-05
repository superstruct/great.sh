# 0044 — Docker Ubuntu 22.04 image GLIBC_2.39 incompatibility

| Field | Value |
|---|---|
| Priority | P1 |
| Type | bug |
| Module | `docker/ubuntu.Dockerfile`, `docker/test.sh` |
| Status | done |
| Estimated Complexity | S |

## Problem

The Ubuntu 22.04 Docker image ships GLIBC 2.35, but the `cargo build --release` binary links against GLIBC_2.39 (likely pulled by a dependency). This causes the binary to fail at runtime partway through the comprehensive test suite (`test-in-docker.sh`):

```
/usr/local/bin/great: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by /usr/local/bin/great)
```

The binary works for initial tests (init, status, diff, apply, vault, mcp, template, sync, update) but fails for later ones (loop, statusline, mcp-bridge, edge cases). This is non-deterministic — likely a lazy-loading dynamic symbol that only resolves when certain code paths execute.

## Impact

- **42 of 192 comprehensive CLI tests fail** due to this single root cause
- Ubuntu 22.04 is a common deployment target — real users on 22.04 would hit this
- CI tests pass because `cargo test` builds/runs in the same toolchain, but the installed binary fails

## Proposed Fix

Either:
1. **Upgrade Docker image to Ubuntu 24.04** (`ubuntu:24.04`, ships GLIBC 2.39) — simplest fix
2. **Use `musl` target** (`x86_64-unknown-linux-musl`) for static linking — eliminates GLIBC dependency entirely
3. **Pin Rust toolchain** in Dockerfile to a version that doesn't require GLIBC 2.39

Option 2 is the most robust for distribution.

## Evidence

Docker test run 2026-03-04. 42 failures all showing identical GLIBC error. First 80+ tests pass (early subcommands), then binary starts failing for `loop`, `statusline`, `mcp-bridge` subcommands.
