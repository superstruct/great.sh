# 0015: Fix macOS Cross-Compilation Dockerfile — Base Image Lacks `/bin/sh`

**Priority:** P2
**Type:** bugfix
**Module:** `docker/cross-macos.Dockerfile`
**Status:** done (iteration 010, commit a117187)

## What Was Done

Rewrote `docker/cross-macos.Dockerfile` as a multi-stage build to fix the `/bin/sh: no such file or directory` error caused by the upstream `crazymax/osxcross` image being `FROM scratch`.

- Stage 1: `FROM crazymax/osxcross:26.1-r0-ubuntu AS osxcross` (pinned SDK source)
- Stage 2: `FROM ubuntu:24.04` (real base with shell and package manager)
- `COPY --from=osxcross /osxcross /osxcross` brings in the toolchain
- Added `PATH` and `LD_LIBRARY_PATH` ENV vars for osxcross binaries
- Pinned Rust to `--default-toolchain 1.85.0` (not floating `stable`)
- Preserved all existing logic: auto-detect cargo config, both x86_64 and aarch64 targets, pre-fetch, CMD

## Acceptance Criteria

- [x] `docker compose build macos-cross` structure is correct (multi-stage, pinned tags)
- [x] No floating `latest` or `stable` tags
- [x] `cargo clippy` and `cargo test` pass unchanged (no Rust source changes)
