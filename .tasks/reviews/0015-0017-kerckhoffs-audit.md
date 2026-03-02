# Security Audit: 0015+0017 Cross-Build Dockerfile Fixes

**Auditor:** Auguste Kerckhoffs
**Date:** 2026-02-24
**Scope:** `docker/cross-windows.Dockerfile`, `docker/cross-macos.Dockerfile`, `docker-compose.yml` (cross-build services)
**Verdict:** PASS -- No CRITICAL or HIGH findings. Commit unblocked.

---

## Checklist

### 1. Supply Chain -- Image Tag Pinning

| Image | Tag | Pinned? | Digest-pinned? |
|-------|-----|---------|-----------------|
| `rust` | `1.85-slim` | YES | No (out of scope per spec) |
| `crazymax/osxcross` | `26.1-r0-ubuntu` | YES | No (out of scope per spec) |
| `ubuntu` | `24.04` | YES | No (out of scope per spec) |

**Result:** PASS. No floating tags (`latest`, `stable`). All three images use semver-pinned tags. SHA256 digest pinning is explicitly deferred per spec -- acceptable for dev/test Dockerfiles.

### 2. Supply Chain -- Base Image Trust

| Image | Source | Trusted? |
|-------|--------|----------|
| `rust:1.85-slim` | Docker Official Image (Rust project) | YES |
| `crazymax/osxcross:26.1-r0-ubuntu` | CrazyMax (well-known Docker community maintainer, also maintains `docker/buildx`) | YES |
| `ubuntu:24.04` | Docker Official Image (Canonical) | YES |

**Result:** PASS.

### 3. Supply Chain -- Rustup Download

Line 40-44 of `cross-macos.Dockerfile`:
```dockerfile
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    -y \
    --default-toolchain 1.85.0 \
    --profile minimal \
    --no-modify-path
```

- `--proto '=https'` enforces HTTPS-only (no downgrade to HTTP).
- `--tlsv1.2` enforces TLS 1.2 minimum.
- Toolchain pinned to `1.85.0` (not `stable`).

**Result:** PASS.

### 4. Privilege Escalation

- Neither Dockerfile uses `USER` -- builds run as root inside the container. This is standard for build containers that are ephemeral and not exposed to networks.
- `docker-compose.yml` cross-build services (`macos-cross`, `windows-cross`) have NO `privileged`, `cap_add`, or `devices` directives.
- Workspace is mounted read-only (`:ro`).
- The `test-files` mount (`./test-files:/workspace/test-files`) is read-write, which is necessary for binary export. This is scoped to a single directory.

**Result:** PASS.

### 5. Secrets

- No `ARG` or `ENV` instructions contain credentials, API keys, or tokens.
- No `COPY` of `.env`, credential files, or vault data.
- `COPY Cargo.toml Cargo.lock* ./` copies only build manifests.
- No `--secret` or `--ssh` BuildKit mounts.
- Cross-test scripts (`cross-test-macos.sh`, `cross-test-windows.sh`) handle no secrets -- they only copy source, build, validate binaries, and export.

**Result:** PASS.

### 6. Build Context Leakage

The `docker-compose.yml` sets `context: .` (project root) for both cross-build services. There is **no `.dockerignore` file** at the project root.

However, the Dockerfiles only `COPY Cargo.toml Cargo.lock* ./` -- no broad `COPY . .` that would pull the entire context into a layer. The full context is *sent* to the Docker daemon but never materialized in an image layer.

**Result:** PASS (no secrets leak into image layers). See L1 below for the efficiency concern.

### 7. Multi-Stage Build Security

The macOS Dockerfile correctly uses a two-stage pattern:
- Stage 1 (`osxcross`): FROM scratch image, used only as a COPY source.
- Stage 2 (`ubuntu:24.04`): Real build environment.
- `COPY --from=osxcross /osxcross /osxcross` brings only the toolchain directory.
- No secrets, build args, or environment variables leak between stages.

**Result:** PASS.

### 8. Network Exposure

- No `EXPOSE` directives in either Dockerfile.
- No `ports` mapping in docker-compose for cross-build services.
- Containers are ephemeral build environments with no network services.

**Result:** PASS.

---

## Findings

### L1 (LOW): Missing `.dockerignore` at project root

**File:** (missing) `/home/isaac/src/sh.great/.dockerignore`
**Impact:** LOW -- performance only, no security impact.
**Detail:** The build context (`context: .`) sends the entire project tree to the Docker daemon, including `.git/`, `node_modules/` (if not gitignored at Docker level), `target/`, etc. While no `COPY . .` exists in the Dockerfiles (so nothing leaks into image layers), the context transfer is unnecessarily large and slow.
**Recommendation:** Add a `.dockerignore` file to exclude `.git/`, `node_modules/`, `target/`, `site/`, `infra/`, `macos-storage/`, `windows-storage/`, `ubuntu-vm-storage/`. This is a P3 improvement.

### L2 (LOW, pre-existing, out of scope): `cross-linux-aarch64.Dockerfile` still uses `rust:1.83-slim`

**File:** `/home/isaac/src/sh.great/docker/cross-linux-aarch64.Dockerfile:6`
**Impact:** LOW -- this Dockerfile was not in scope for 0015+0017, but it has the same `edition2024` breakage as the Windows Dockerfile had before this fix.
**Recommendation:** Bump to `rust:1.85-slim` in a follow-on task.

### L3 (LOW, pre-existing, out of scope): `ubuntu.Dockerfile` and `fedora.Dockerfile` use unpinned Rust toolchain

**File:** `/home/isaac/src/sh.great/docker/ubuntu.Dockerfile:18`, `/home/isaac/src/sh.great/docker/fedora.Dockerfile:16`
**Impact:** LOW -- `rustup` with `-y` and no `--default-toolchain` installs `stable`, which is a floating channel. Builds are not reproducible across time.
**Recommendation:** Pin `--default-toolchain 1.85.0` in a follow-on task for consistency.

---

## Summary

| Severity | Count | Details |
|----------|-------|---------|
| CRITICAL | 0 | -- |
| HIGH | 0 | -- |
| MEDIUM | 0 | -- |
| LOW | 3 | L1: missing .dockerignore, L2: aarch64 Rust unpinned (pre-existing), L3: ubuntu/fedora Rust unpinned (pre-existing) |

**Verdict: PASS.** No CRITICAL or HIGH findings. The cross-build Dockerfile changes are secure and ready for commit. All LOW findings are either pre-existing or performance-only improvements for follow-on tasks.
