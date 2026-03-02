# Spec 0015+0017: Fix Cross-Compilation Dockerfiles (Windows + macOS)

**Tasks:** `.tasks/backlog/0015-fix-macos-cross-dockerfile-base-image.md`, `.tasks/backlog/0017-bump-rust-in-windows-cross-dockerfile.md`
**Status:** ready
**Type:** bugfix (both)
**Estimated Complexity:** XS (task 0017) + S (task 0015) = S combined

---

## Summary

Both cross-compilation Docker builds are broken:

1. **Windows (`docker/cross-windows.Dockerfile`):** The base image `rust:1.83-slim` ships Cargo 1.83.0, which does not support the `edition2024` Cargo feature. The transitive dependency `time-core 0.1.8` (present in `Cargo.lock`) declares `edition = "2024"`, which was stabilized in Rust 1.85.0. Fix: bump `FROM rust:1.83-slim` to `FROM rust:1.85-slim`.

2. **macOS (`docker/cross-macos.Dockerfile`):** The base image `crazymax/osxcross:latest-ubuntu` is built `FROM scratch` upstream -- it contains only the osxcross toolchain files at `/osxcross` and SDK files at `/osxsdk`, with no shell, no package manager, and no OS userland. Every `RUN` instruction fails with `exec: "/bin/sh": stat /bin/sh: no such file or directory`. Fix: rewrite as a multi-stage build that copies the toolchain from a pinned osxcross image into a real `ubuntu:24.04` base, then pin the Rust install to `>= 1.85` to avoid the same `edition2024` failure.

No Rust source changes are needed. No `Cargo.toml` or `Cargo.lock` changes. Only two Dockerfiles are modified.

---

## Files to Modify

| File | Change | Complexity |
|------|--------|------------|
| `/home/isaac/src/sh.great/docker/cross-windows.Dockerfile` | Line 6: `FROM rust:1.83-slim` to `FROM rust:1.85-slim` | XS (one line) |
| `/home/isaac/src/sh.great/docker/cross-macos.Dockerfile` | Full rewrite as multi-stage build | S (same logic, new structure) |

No new files are created. No other files are modified.

---

## Build Order

Apply task 0017 (Windows) first. It is a single-line change that validates the approach -- if `docker compose build windows-cross` succeeds, the Rust version bump is confirmed correct. Then apply task 0015 (macOS), which is a structural rewrite.

1. **Step 1:** Modify `/home/isaac/src/sh.great/docker/cross-windows.Dockerfile` (one line).
2. **Step 2:** Verify Windows build with `docker compose build windows-cross`.
3. **Step 3:** Replace the full content of `/home/isaac/src/sh.great/docker/cross-macos.Dockerfile`.
4. **Step 4:** Verify macOS build with `docker compose build macos-cross`.
5. **Step 5:** Run both cross-compilation test scripts to validate end-to-end.

---

## Task 0017: Windows Dockerfile Fix (XS)

### Current file (`/home/isaac/src/sh.great/docker/cross-windows.Dockerfile`)

```dockerfile
# Cross-compilation for Windows x86_64 (MinGW)
#
# Usage:
#   docker build -f docker/cross-windows.Dockerfile -t great-cross-windows .
#   docker run --rm -v $(pwd):/workspace great-cross-windows
FROM rust:1.83-slim

RUN apt-get update && apt-get install -y \
    gcc-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-pc-windows-gnu

WORKDIR /workspace

CMD ["cargo", "build", "--release", "--target", "x86_64-pc-windows-gnu"]
```

### Exact change

**Line 6 -- before:**

```dockerfile
FROM rust:1.83-slim
```

**Line 6 -- after:**

```dockerfile
FROM rust:1.85-slim
```

No other lines change. The rest of the file remains identical.

### Complete file after change

```dockerfile
# Cross-compilation for Windows x86_64 (MinGW)
#
# Usage:
#   docker build -f docker/cross-windows.Dockerfile -t great-cross-windows .
#   docker run --rm -v $(pwd):/workspace great-cross-windows
FROM rust:1.85-slim

RUN apt-get update && apt-get install -y \
    gcc-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-pc-windows-gnu

WORKDIR /workspace

CMD ["cargo", "build", "--release", "--target", "x86_64-pc-windows-gnu"]
```

### Why 1.85 and not `latest` or `stable`

`rust:1.85-slim` is the minimum version that supports `edition2024` (stabilized in Rust 1.85.0, released 2025-02-20). Pinning to a specific version keeps the build reproducible. The project's own `Cargo.toml` declares `edition = "2021"` -- the 1.85 requirement comes from the transitive dependency `time-core 0.1.8`.

---

## Task 0015: macOS Dockerfile Rewrite (S)

### Root cause

The upstream `crazy-max/docker-osxcross` image is built `FROM scratch` in its final stage. It contains only the osxcross toolchain files -- no `/bin/sh`, no package manager, no OS userland. Our Dockerfile used it directly as a base image with `FROM crazymax/osxcross:latest-ubuntu`, so every `RUN` instruction failed immediately.

The upstream README documents the correct usage: use the osxcross image as a named build stage and `COPY` (or bind-mount) the toolchain into a real OS base.

### Design decisions

1. **Use `COPY --from` rather than `RUN --mount=type=bind,from=`**: The `COPY` approach persists the toolchain into the image layer permanently, which is required because the `cross-test-macos.sh` script uses the osxcross binaries at container runtime (not just during build). A bind-mount is only available during the `RUN` instruction that references it.

2. **Pin osxcross to `26.1-r0-ubuntu`**: This is the latest upstream release (2026-01-30). Avoids the floating `latest-ubuntu` tag that caused the original breakage.

3. **Pin Rust to `>= 1.85` via `rustup`**: The Dockerfile installs Rust via `rustup` with `--default-toolchain 1.85.0` instead of `stable` to ensure the `edition2024` feature is available (same root cause as task 0017). This avoids a future breakage if the `stable` channel at image build time is somehow older than 1.85.

4. **Use `ubuntu:24.04` as the base**: Matches the task requirement. Provides `/bin/sh`, `apt-get`, and the full OS userland needed by subsequent `RUN` instructions.

5. **Preserve all existing build logic**: The Rust install, cargo config auto-detection, dependency pre-fetch, and working directory structure remain unchanged. Only the base image sourcing changes.

### Current file (`/home/isaac/src/sh.great/docker/cross-macos.Dockerfile`)

```dockerfile
# Cross-compilation for macOS x86_64 and aarch64 via osxcross
#
# Usage:
#   docker compose build macos-cross
#   docker compose run macos-cross
#   docker compose run macos-cross cargo build --release --target x86_64-apple-darwin
FROM crazymax/osxcross:latest-ubuntu

# Install system dependencies needed for Rust compilation
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    clang \
    lld \
    pkg-config \
    ca-certificates \
    file \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain with both macOS targets
ENV RUSTUP_HOME=/opt/rust
ENV CARGO_HOME=/opt/rust
ENV PATH="/opt/rust/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    -y \
    --default-toolchain stable \
    --profile minimal \
    --no-modify-path

RUN rustup target add x86_64-apple-darwin aarch64-apple-darwin

# Auto-detect osxcross tool paths and generate cargo config + env vars.
# This avoids hardcoding SDK version strings like "darwin24.5".
RUN set -e && \
    X86_CLANG=$(find /osxcross/bin -name 'x86_64-apple-darwin*-clang' -not -name '*++' | head -1) && \
    X86_AR=$(find /osxcross/bin -name 'x86_64-apple-darwin*-ar' | head -1) && \
    ARM_CLANG=$(find /osxcross/bin -name 'aarch64-apple-darwin*-clang' -not -name '*++' | head -1) && \
    ARM_AR=$(find /osxcross/bin -name 'aarch64-apple-darwin*-ar' | head -1) && \
    test -n "$X86_CLANG" || { echo "FATAL: x86_64 clang not found in /osxcross/bin"; exit 1; } && \
    test -n "$ARM_CLANG" || { echo "FATAL: aarch64 clang not found in /osxcross/bin"; exit 1; } && \
    echo "Detected x86_64 clang: $X86_CLANG" && \
    echo "Detected aarch64 clang: $ARM_CLANG" && \
    mkdir -p /root/.cargo && \
    printf '[target.x86_64-apple-darwin]\nlinker = "%s"\nar = "%s"\n\n' "$X86_CLANG" "$X86_AR" > /root/.cargo/config.toml && \
    printf '[target.aarch64-apple-darwin]\nlinker = "%s"\nar = "%s"\n' "$ARM_CLANG" "$ARM_AR" >> /root/.cargo/config.toml && \
    echo "export CC_x86_64_apple_darwin=\"$X86_CLANG\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CXX_x86_64_apple_darwin=\"${X86_CLANG}++\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export AR_x86_64_apple_darwin=\"$X86_AR\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=\"$X86_CLANG\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CC_aarch64_apple_darwin=\"$ARM_CLANG\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CXX_aarch64_apple_darwin=\"${ARM_CLANG}++\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export AR_aarch64_apple_darwin=\"$ARM_AR\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=\"$ARM_CLANG\"" >> /etc/profile.d/osxcross-env.sh

# Source env vars at build time too (for the pre-fetch step)
SHELL ["/bin/bash", "-c"]
RUN source /etc/profile.d/osxcross-env.sh && env | grep -E '^(CC_|CXX_|AR_|CARGO_TARGET_)' >> /etc/environment

# Pre-fetch dependencies using real Cargo.toml + Cargo.lock
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo fetch && \
    rm -rf src

WORKDIR /workspace

CMD ["bash", "/workspace/docker/cross-test-macos.sh"]
```

### Complete replacement file

Replace the **entire contents** of `/home/isaac/src/sh.great/docker/cross-macos.Dockerfile` with:

```dockerfile
# Cross-compilation for macOS x86_64 and aarch64 via osxcross
#
# Usage:
#   docker compose build macos-cross
#   docker compose run macos-cross
#   docker compose run macos-cross cargo build --release --target x86_64-apple-darwin
#
# The osxcross image is FROM scratch upstream (no shell, no OS userland).
# We use it as a named stage and COPY the toolchain into a real Ubuntu base.

# Stage 1: pinned osxcross toolchain source (FROM scratch -- not runnable)
FROM crazymax/osxcross:26.1-r0-ubuntu AS osxcross

# Stage 2: real Ubuntu base with shell and package manager
FROM ubuntu:24.04

# Copy the osxcross toolchain from the source stage
COPY --from=osxcross /osxcross /osxcross

# Install system dependencies needed for Rust compilation
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    clang \
    lld \
    pkg-config \
    ca-certificates \
    file \
    && rm -rf /var/lib/apt/lists/*

# Make osxcross binaries available on PATH
ENV PATH="/osxcross/bin:${PATH}"
ENV LD_LIBRARY_PATH="/osxcross/lib:${LD_LIBRARY_PATH}"

# Install Rust toolchain with both macOS targets
ENV RUSTUP_HOME=/opt/rust
ENV CARGO_HOME=/opt/rust
ENV PATH="/opt/rust/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    -y \
    --default-toolchain 1.85.0 \
    --profile minimal \
    --no-modify-path

RUN rustup target add x86_64-apple-darwin aarch64-apple-darwin

# Auto-detect osxcross tool paths and generate cargo config + env vars.
# This avoids hardcoding SDK version strings like "darwin24.5".
RUN set -e && \
    X86_CLANG=$(find /osxcross/bin -name 'x86_64-apple-darwin*-clang' -not -name '*++' | head -1) && \
    X86_AR=$(find /osxcross/bin -name 'x86_64-apple-darwin*-ar' | head -1) && \
    ARM_CLANG=$(find /osxcross/bin -name 'aarch64-apple-darwin*-clang' -not -name '*++' | head -1) && \
    ARM_AR=$(find /osxcross/bin -name 'aarch64-apple-darwin*-ar' | head -1) && \
    test -n "$X86_CLANG" || { echo "FATAL: x86_64 clang not found in /osxcross/bin"; exit 1; } && \
    test -n "$ARM_CLANG" || { echo "FATAL: aarch64 clang not found in /osxcross/bin"; exit 1; } && \
    echo "Detected x86_64 clang: $X86_CLANG" && \
    echo "Detected aarch64 clang: $ARM_CLANG" && \
    mkdir -p /root/.cargo && \
    printf '[target.x86_64-apple-darwin]\nlinker = "%s"\nar = "%s"\n\n' "$X86_CLANG" "$X86_AR" > /root/.cargo/config.toml && \
    printf '[target.aarch64-apple-darwin]\nlinker = "%s"\nar = "%s"\n' "$ARM_CLANG" "$ARM_AR" >> /root/.cargo/config.toml && \
    echo "export CC_x86_64_apple_darwin=\"$X86_CLANG\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CXX_x86_64_apple_darwin=\"${X86_CLANG}++\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export AR_x86_64_apple_darwin=\"$X86_AR\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=\"$X86_CLANG\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CC_aarch64_apple_darwin=\"$ARM_CLANG\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CXX_aarch64_apple_darwin=\"${ARM_CLANG}++\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export AR_aarch64_apple_darwin=\"$ARM_AR\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=\"$ARM_CLANG\"" >> /etc/profile.d/osxcross-env.sh

# Source env vars at build time too (for the pre-fetch step)
SHELL ["/bin/bash", "-c"]
RUN source /etc/profile.d/osxcross-env.sh && env | grep -E '^(CC_|CXX_|AR_|CARGO_TARGET_)' >> /etc/environment

# Pre-fetch dependencies using real Cargo.toml + Cargo.lock
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo fetch && \
    rm -rf src

WORKDIR /workspace

CMD ["bash", "/workspace/docker/cross-test-macos.sh"]
```

### Line-by-line diff from the current file

The changes are concentrated in the first section. Here is every difference:

| Line(s) | Before | After | Rationale |
|----------|--------|-------|-----------|
| 7 (header) | -- | Added 2 comment lines explaining the multi-stage pattern | Documents why the osxcross image cannot be used directly |
| 7 `FROM` | `FROM crazymax/osxcross:latest-ubuntu` | `FROM crazymax/osxcross:26.1-r0-ubuntu AS osxcross` | Named stage, pinned version |
| 8 (new) | -- | `FROM ubuntu:24.04` | Real OS base with `/bin/sh` |
| 9 (new) | -- | `COPY --from=osxcross /osxcross /osxcross` | Brings toolchain into the real base |
| (after apt-get) | -- | `ENV PATH="/osxcross/bin:${PATH}"` | Upstream-recommended PATH setup |
| (after apt-get) | -- | `ENV LD_LIBRARY_PATH="/osxcross/lib:${LD_LIBRARY_PATH}"` | Upstream-recommended library path |
| rustup line | `--default-toolchain stable` | `--default-toolchain 1.85.0` | Pins Rust to >= 1.85 to support `edition2024` |

All other lines (the `RUN apt-get`, `RUN rustup target add`, the auto-detect `RUN set -e` block, the `SHELL` line, the `RUN source` line, the `WORKDIR`, `COPY Cargo.toml`, the pre-fetch block, and the `CMD`) are **identical** to the current file.

---

## docker-compose.yml

No changes needed. The `docker-compose.yml` service definitions for `macos-cross` and `windows-cross` reference the Dockerfiles by path and do not specify base images or Rust versions. The build context (`.`) and volume mounts remain correct.

The orphan container warning mentioned in task 0015 is cosmetic and out of scope for this fix. Users can suppress it with `docker compose --remove-orphans` if desired.

---

## Edge Cases

| Scenario | Handling |
|----------|----------|
| Docker layer cache from old `rust:1.83-slim` | `docker compose build windows-cross` will pull the new `rust:1.85-slim` layer. The cache is invalidated at the `FROM` line, so all subsequent layers rebuild. |
| Docker layer cache from old `crazymax/osxcross:latest-ubuntu` | The `FROM` line changes to a different image name/tag. Docker invalidates the entire cache. Full rebuild is expected on first run. |
| `osxcross:26.1-r0-ubuntu` image not available | Build fails at `FROM` with a clear Docker pull error. The builder should verify the tag exists at `hub.docker.com/r/crazymax/osxcross/tags` before applying. |
| Network failure during `docker compose build` | Docker reports the pull failure. No partial state. Re-run the build. |
| `rustup` network failure during macOS Dockerfile build | The `curl ... | sh -s --` line fails with a non-zero exit code. Docker stops the build. Re-run. |
| `time-core` dependency updates to a version that drops `edition2024` | No impact. Rust 1.85 supports all editions through 2024. Older editions remain supported. |
| `time-core` dependency updates to require a newer Rust version | The pinned Rust 1.85 in both Dockerfiles would need bumping. This is a future maintenance concern, not a current edge case. |
| osxcross toolchain files missing `/osxcross/bin/*-clang` | The `test -n "$X86_CLANG"` guard in the auto-detect `RUN` block fires with `FATAL: x86_64 clang not found in /osxcross/bin` and exits 1. This logic is unchanged from the current Dockerfile. |
| Build host is macOS ARM64 | Docker Desktop on macOS runs Linux containers in a VM. The `FROM` lines pull linux/amd64 images by default. Both Dockerfiles produce cross-compiled binaries, not native macOS binaries. The `--platform` flag is not needed because the osxcross stage is architecture-independent (it contains only the SDK files). |
| Build host is WSL2 | Docker runs natively. Identical behavior to a bare Linux host. |
| Concurrent `docker compose build` of both services | Docker builds are independent. No shared state between the two Dockerfiles. |
| `/workspace` volume is read-only | Both `docker-compose.yml` service definitions mount `.:/workspace:ro`. The test scripts (`cross-test-macos.sh`, `cross-test-windows.sh`) copy source to `/build/` before compiling. The `WORKDIR /build` and `COPY Cargo.toml` in the Dockerfile are for the pre-fetch step only (build-time, not runtime). No conflict. |
| `Cargo.lock` missing | The `COPY Cargo.toml Cargo.lock* ./` glob handles this -- if `Cargo.lock` does not exist, only `Cargo.toml` is copied. `cargo fetch` still works (it generates a lock file). |

---

## Error Handling

No new error handling code is needed. These are Dockerfile changes. Docker reports errors natively:

| Condition | Docker behavior |
|-----------|----------------|
| Base image pull failure | `ERROR: pull access denied for crazymax/osxcross` or network error. Clear, actionable. |
| `apt-get` failure | `E: Unable to locate package ...` Build stops. |
| `rustup` download failure | `curl` exits non-zero. Build stops. |
| `COPY --from=osxcross` source path missing | `COPY failed: stat /osxcross: file does not exist`. Indicates the osxcross image structure changed upstream. |
| `rustup target add` failure | `error: toolchain 'x86_64-apple-darwin' is not installable` (unlikely for these stable targets). Build stops. |
| osxcross auto-detect finds no clang binaries | Existing `test -n "$X86_CLANG" || { echo "FATAL: ..."; exit 1; }` fires. Build stops with a descriptive message. |
| `cargo fetch` failure (network) | `error: failed to fetch ...` Build stops. |

---

## Security Considerations

- **Pinned image tags**: Both Dockerfiles use pinned versions (`rust:1.85-slim`, `crazymax/osxcross:26.1-r0-ubuntu`, `ubuntu:24.04`) instead of floating tags (`latest`, `stable`). This prevents supply chain drift.
- **No digest pinning**: The tags are semver-pinned but not SHA256-digest-pinned. Digest pinning is a follow-on hardening task (out of scope per both task descriptions).
- **No new dependencies**: The macOS Dockerfile adds no new `apt-get` packages. The same set (`curl`, `build-essential`, `clang`, `lld`, `pkg-config`, `ca-certificates`, `file`) is installed.
- **No secrets**: Neither Dockerfile handles credentials, API keys, or vault access.
- **HTTPS-only Rust install**: The `curl --proto '=https' --tlsv1.2` flags enforce TLS for the rustup download. Unchanged from the current file.
- **Read-only workspace volume**: Both `docker-compose.yml` service definitions mount the host workspace as `:ro`. Build artifacts go to `/build/` (inside the container). This prevents the container from modifying host files.

---

## Testing Strategy

### Automated verification (builder runs these)

```bash
# Step 1: Build Windows cross-compilation image
docker compose build windows-cross
# Expected: exits 0, final line shows image ID

# Step 2: Run Windows cross-compilation
docker compose run --rm windows-cross
# Expected: exits 0, prints "Cross-compilation complete"
# The cross-test-windows.sh script validates the output binary with `file`

# Step 3: Verify Windows binary type
file test-files/great-x86_64-pc-windows-gnu.exe
# Expected: "PE32+ executable (console) x86-64, for MS Windows"

# Step 4: Build macOS cross-compilation image
docker compose build macos-cross
# Expected: exits 0, final line shows image ID

# Step 5: Run macOS cross-compilation
docker compose run --rm macos-cross
# Expected: exits 0, prints "Cross-compilation complete"
# The cross-test-macos.sh script validates both output binaries with `file`

# Step 6: Verify macOS binary types
file test-files/great-x86_64-apple-darwin
# Expected: "Mach-O 64-bit x86_64 executable"

file test-files/great-aarch64-apple-darwin
# Expected: "Mach-O 64-bit arm64 executable"
```

### Rust source integrity check

```bash
# Verify no Rust source was modified
cargo clippy -- -D warnings
cargo test --verbose
```

Both must exit 0. These Dockerfile changes introduce no modifications to Rust source, `Cargo.toml`, or `Cargo.lock`.

### Manual verification checklist

- [ ] `docker compose build windows-cross` completes without error.
- [ ] `docker compose run --rm windows-cross` produces a valid PE32+ binary at `test-files/great-x86_64-pc-windows-gnu.exe`.
- [ ] `docker compose build macos-cross` completes without error.
- [ ] `docker compose run --rm macos-cross` produces valid Mach-O binaries at `test-files/great-x86_64-apple-darwin` and `test-files/great-aarch64-apple-darwin`.
- [ ] `docker/cross-windows.Dockerfile` line 6 reads `FROM rust:1.85-slim` (not `latest`, not `stable`, not `1.83`).
- [ ] `docker/cross-macos.Dockerfile` line 12 reads `FROM crazymax/osxcross:26.1-r0-ubuntu AS osxcross` (not `latest-ubuntu`).
- [ ] `docker/cross-macos.Dockerfile` line 15 reads `FROM ubuntu:24.04`.
- [ ] `docker/cross-macos.Dockerfile` line 16 reads `COPY --from=osxcross /osxcross /osxcross`.
- [ ] `docker/cross-macos.Dockerfile` rustup line reads `--default-toolchain 1.85.0` (not `stable`).
- [ ] `cargo clippy -- -D warnings` exits 0.
- [ ] `cargo test` exits 0.
- [ ] `git diff --name-only` shows only `docker/cross-windows.Dockerfile` and `docker/cross-macos.Dockerfile`.

---

## Verification Gate

The builder declares this task complete when all of the following are true:

- [ ] `docker compose build windows-cross` exits 0
- [ ] `docker compose run --rm windows-cross` exits 0 and produces a PE32+ binary
- [ ] `docker compose build macos-cross` exits 0
- [ ] `docker compose run --rm macos-cross` exits 0 and produces Mach-O x86_64 and arm64 binaries
- [ ] No floating image tags (`latest`, `stable`) appear in either Dockerfile
- [ ] `cargo clippy -- -D warnings` exits 0
- [ ] `cargo test` exits 0
- [ ] `git diff --name-only` shows exactly 2 files: `docker/cross-windows.Dockerfile` and `docker/cross-macos.Dockerfile`
