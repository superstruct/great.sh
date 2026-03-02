# 0019: Bump Rust Toolchain in Cross-Compilation Dockerfiles

**Priority:** P0
**Created:** 2026-02-25
**Completed:** 2026-02-25
**Status:** done
**Commit:** d44175d
**Component:** docker / cross-compilation

## Summary

Bumped Rust from 1.83/1.85 to 1.88.0 in three cross-compilation Dockerfiles
to fix `time@0.3.47` MSRV breakage. Added `linux-aarch64-cross` compose service
and test script. Supersedes task 0018.

## Files Changed

- docker/cross-macos.Dockerfile — `--default-toolchain 1.88.0`
- docker/cross-windows.Dockerfile — `FROM rust:1.88-slim`
- docker/cross-linux-aarch64.Dockerfile — `FROM rust:1.88-slim`
- docker-compose.yml — added linux-aarch64-cross service, fixed usage comments
- docker/cross-test-linux-aarch64.sh — new test script
