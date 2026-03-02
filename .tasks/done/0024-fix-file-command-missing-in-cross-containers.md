# 0024 — Fix `file` command missing in cross-compilation containers

**Status:** DONE
**Commit:** `239b827`
**Iteration:** 018
**Date:** 2026-02-26

## Summary

Added `file` package to `apt-get install` in two Dockerfiles:
- `docker/cross-windows.Dockerfile`
- `docker/cross-linux-aarch64.Dockerfile`

This fixes `file: command not found` at binary validation step [3/4] in cross-compilation test scripts. The `rust:1.88-slim` base images (Debian slim) strip non-essential packages; `file` was missing but required by the test scripts.
