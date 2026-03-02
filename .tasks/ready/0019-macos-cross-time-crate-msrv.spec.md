# Spec: 0019 -- Bump Rust Toolchain in Cross-Compilation Dockerfiles

**Task:** .tasks/backlog/0019-macos-cross-time-crate-msrv.md
**Author:** Lovelace
**Date:** 2026-02-25
**Option:** A (bump Rust toolchain; Dockerfile-only fix)

## Summary

The `time` crate v0.3.47 -- a transitive dependency pulled in through `zip` 2.4.2 -- raised its MSRV to rustc 1.88.0. The macOS cross-compilation Dockerfile (`docker/cross-macos.Dockerfile`) pins Rust at 1.85.0 via `--default-toolchain 1.85.0`, and the Windows cross-compilation Dockerfile (`docker/cross-windows.Dockerfile`) uses `FROM rust:1.85-slim`. Both must be bumped to Rust 1.88.0 to satisfy the transitive MSRV constraint. The Linux aarch64 cross-compilation Dockerfile (`docker/cross-linux-aarch64.Dockerfile`) is pinned even lower at 1.83.0 and must also be bumped. No Rust source code or Cargo.toml changes are required; this is a Dockerfile-only fix.

## Dependency Chain

```
Cargo.toml
  zip = "2"
    -> zip 2.4.2 (Cargo.lock line 2555)
         -> time 0.3.47 (Cargo.lock line 1720)
              MSRV: rustc 1.88.0 (confirmed via crates.io API)
              -> time-core 0.1.8 (Cargo.lock line 1733)
                   MSRV: rustc 1.88.0
```

## Rust Version Verification

| Artifact                   | Status    | Evidence                                             |
|----------------------------|-----------|------------------------------------------------------|
| `rustup` toolchain 1.88.0 | Available | `rust-1.88.0-x86_64-unknown-linux-gnu.tar.gz` exists on `static.rust-lang.org` (released 2025-06-26) |
| `rust:1.88-slim` Docker tag| Available | Docker Hub tag confirmed present with amd64, arm64, i386, ppc64le, s390x, armv7 architectures |
| Current stable Rust        | 1.93.1    | `channel-rust-stable.toml` dated 2026-02-12          |

Rust 1.88.0 is eight releases behind current stable (1.93.1), making it a conservative and well-tested choice. It is the minimum version required by the dependency chain -- not a bleeding-edge selection.

## Changes

### 1. `docker/cross-macos.Dockerfile`

**Line 42:** Change `--default-toolchain 1.85.0` to `--default-toolchain 1.88.0`

```dockerfile
# Before (line 40-44):
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    -y \
    --default-toolchain 1.85.0 \
    --profile minimal \
    --no-modify-path

# After:
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    -y \
    --default-toolchain 1.88.0 \
    --profile minimal \
    --no-modify-path
```

**Rationale:** The rustup installer in this Dockerfile fetches the toolchain specified by `--default-toolchain`. Bumping from 1.85.0 to 1.88.0 satisfies the `time 0.3.47` MSRV. The osxcross toolchain, Ubuntu base image, and all `find`-based SDK detection logic remain unchanged.

### 2. `docker/cross-windows.Dockerfile`

**Line 7:** Change `FROM rust:1.85-slim` to `FROM rust:1.88-slim`

```dockerfile
# Before (line 7):
FROM rust:1.85-slim

# After:
FROM rust:1.88-slim
```

**Rationale:** The Windows Dockerfile uses the official `rust:` Docker image rather than rustup. The `1.88-slim` tag is confirmed available on Docker Hub. The `gcc-mingw-w64-x86-64` system package and the `x86_64-pc-windows-gnu` target remain compatible across Rust 1.85 to 1.88.

### 3. `docker/cross-linux-aarch64.Dockerfile`

**Line 6:** Change `FROM rust:1.83-slim` to `FROM rust:1.88-slim`

```dockerfile
# Before (line 6):
FROM rust:1.83-slim

# After:
FROM rust:1.88-slim
```

**Rationale:** This Dockerfile is pinned to Rust 1.83.0, which is even older than the macOS/Windows Dockerfiles. It will encounter the identical `time 0.3.47` MSRV failure. Bumping it to 1.88.0 brings all three cross-compilation Dockerfiles to the same Rust version, preventing the same breakage and simplifying future maintenance.

## Files to Modify

| File | Change | Lines Affected |
|------|--------|----------------|
| `docker/cross-macos.Dockerfile` | `1.85.0` -> `1.88.0` in `--default-toolchain` | Line 42 |
| `docker/cross-windows.Dockerfile` | `rust:1.85-slim` -> `rust:1.88-slim` in `FROM` | Line 7 |
| `docker/cross-linux-aarch64.Dockerfile` | `rust:1.83-slim` -> `rust:1.88-slim` in `FROM` | Line 6 |

No other files are modified. In particular: no changes to `Cargo.toml`, `Cargo.lock`, any `.rs` source file, `docker-compose.yml`, CI workflows, or shell scripts.

## Risks

### New Compiler Lints (Low Risk)

Rust 1.88.0 introduces two new warn-by-default lints:

- **`dangerous_implicit_autorefs`** -- Warns on implicit autoref of raw pointer dereferences. The great.sh codebase contains zero `unsafe` blocks and zero raw pointer operations. Not applicable.
- **`invalid_null_arguments`** -- Warns on passing null pointers to functions that expect non-null. Not applicable (no FFI, no raw pointer code).

### `#[bench]` Hard Error (No Risk)

Rust 1.88.0 makes `#[bench]` without `#![feature(custom_test_frameworks)]` a hard error. The project does not use `#[bench]` anywhere.

### `i686-pc-windows-gnu` Demotion (No Risk)

Rust 1.88.0 demotes `i686-pc-windows-gnu` to Tier 2. The project targets `x86_64-pc-windows-gnu` only.

### Docker Layer Cache Invalidation (Expected)

Changing the base image tag (`FROM rust:1.88-slim`) or the rustup toolchain version invalidates all subsequent Docker layers. The first build after this change will be a full rebuild including dependency fetching. This is expected and unavoidable.

### Transitive Dependency Compatibility (Low Risk)

Other transitive dependencies that previously compiled under 1.85.0 will also compile under 1.88.0 (Rust maintains backward compatibility). No regression is expected.

## Edge Cases

### Docker Image Pull Failure

If a builder's Docker daemon cannot reach Docker Hub or `sh.rustup.rs` (air-gapped environments, network partitions), the build will fail at the `FROM` or `RUN curl` step respectively. This is the existing failure mode and is unchanged by this spec.

### Cargo.lock Drift

If `Cargo.lock` is regenerated between now and when this change lands, the `time` version might change. However, any version of `time >= 0.3.47` will require `>= 1.88.0`, and any version of `time < 0.3.47` already works with 1.85.0. Bumping to 1.88.0 handles both cases.

### Platform Architectures

- The `rust:1.88-slim` image supports `linux/amd64` and `linux/arm64` (verified on Docker Hub), covering both x86_64 and ARM64 Docker hosts.
- The rustup installer in the macOS Dockerfile fetches the specified version for whatever host architecture the container runs on (always `x86_64-unknown-linux-gnu` since it runs on an Ubuntu x86_64 base), then adds the macOS cross-compilation targets. No architecture issue.

## Verification

The builder must verify all three Dockerfiles after applying the changes:

### 1. macOS Cross-Compilation

```bash
cd /home/isaac/src/sh.great
docker compose build macos-cross
docker compose run macos-cross
```

**Expected:** Builds complete for both `x86_64-apple-darwin` and `aarch64-apple-darwin`. Validation script (`docker/cross-test-macos.sh`) confirms Mach-O binaries. Exit code 0.

### 2. Windows Cross-Compilation

```bash
docker compose build windows-cross
docker compose run windows-cross
```

**Expected:** Build completes for `x86_64-pc-windows-gnu`. Validation script (`docker/cross-test-windows.sh`) confirms PE32+ binary. Exit code 0.

### 3. Linux aarch64 Cross-Compilation

```bash
docker build -f docker/cross-linux-aarch64.Dockerfile -t great-cross-aarch64 .
docker run --rm -v $(pwd):/workspace great-cross-aarch64
```

**Expected:** Build completes for `aarch64-unknown-linux-gnu`. Exit code 0.

### 4. Native Toolchain Smoke Tests

```bash
cargo clippy -- -D warnings
cargo test
```

**Expected:** No new warnings, all tests pass. These commands use the host's Rust toolchain (not the Docker containers), but confirm the codebase itself is sound.

### 5. Verify Rust Version Inside Containers

After building, spot-check that the correct toolchain is installed:

```bash
docker compose run macos-cross rustc --version
# Expected: rustc 1.88.0 (...)

docker compose run windows-cross rustc --version
# Expected: rustc 1.88.0 (...)
```

## Out of Scope

- **Bumping to latest stable Rust (1.93.1):** The spec targets the minimum version required (1.88.0) for reproducibility and conservatism. A future task may bump all Dockerfiles to a newer version.
- **Pinning `time` to an older version (Option B):** Rejected as fragile; the next `cargo update` would re-break the build.
- **Replacing the `zip` crate (Option C):** Highest effort, changes application code, not justified when a Dockerfile version bump resolves the issue.
- **Ubuntu and Fedora test Dockerfiles:** `docker/ubuntu.Dockerfile` and `docker/fedora.Dockerfile` use `rustup` without `--default-toolchain`, which installs the current stable (1.93.1). They are not affected.
- **CI/CD workflow changes:** The release workflow (`.github/workflows/release.yml`) uses `dtolnay/rust-toolchain@stable`, which resolves to current stable. It is not affected.
- **Rust source code changes:** No `.rs` files or `Cargo.toml` are modified.
- **`docker-compose.yml` changes:** The compose file references Dockerfiles by path; no changes needed.
