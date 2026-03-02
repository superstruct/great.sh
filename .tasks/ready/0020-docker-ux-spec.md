# 0020: Docker Cross-Compilation UX Improvements -- Technical Specification

**Author:** Lovelace (Spec Writer)
**Iteration:** 023
**Date:** 2026-02-27
**Source:** `.tasks/backlog/0020-docker-cross-compile-ux-issues.md`
**Selection:** `.tasks/ready/0020-nightingale-selection.md`

---

## 1. Overview

Four shell-script and Dockerfile UX issues in the `docker/` directory cause silent
failures, missing diagnostics, and fragile path assumptions. All four are fixable
with minimal, isolated edits to shell scripts and one Dockerfile. No Rust code
changes are required.

The fixes address:

1. The Windows cross Dockerfile CMD bypasses the validation script when the
   container is invoked directly via `docker run` (not compose).
2. `test.sh` silently swallows `great doctor` failures with `|| true`.
3. No cross-compilation entrypoint script prints the Rust toolchain version.
4. Three cross-compilation scripts write exported binaries into
   `/workspace/test-files/`, which is fragile and semantically wrong since
   `/workspace` is documented as read-only. The current compose setup works by
   accident because `./test-files` is bind-mounted as a writable overlay at
   `/workspace/test-files`.

---

## 2. Issue-by-Issue Changes

### Issue 1: Windows cross Dockerfile CMD skips validation

**Priority:** P2

**Current behavior:**

`docker/cross-windows.Dockerfile` line 18:
```dockerfile
CMD ["cargo", "build", "--release", "--target", "x86_64-pc-windows-gnu"]
```

When a user runs the container directly (e.g., `docker run --rm -v $(pwd):/workspace great-cross-windows`), the bare `cargo build` runs without the validation/export logic in `cross-test-windows.sh`. The build may succeed but produce a corrupt or wrong-architecture binary with no check. The usage comment at line 6 even suggests this direct invocation pattern.

The `docker-compose.yml` line 66 overrides the CMD with `command: ["bash", "/workspace/docker/cross-test-windows.sh"]`, so compose users are unaffected. But direct `docker run` users get no validation.

By contrast, `docker/cross-linux-aarch64.Dockerfile` line 20 already does the right thing:
```dockerfile
CMD ["bash", "/workspace/docker/cross-test-linux-aarch64.sh"]
```

**Desired behavior:**

The Dockerfile CMD invokes the validation script, matching the pattern used by the Linux aarch64 and macOS Dockerfiles. Direct `docker run` and `docker compose run` both go through the same validation path.

**Changes:**

File: `docker/cross-windows.Dockerfile`

1. Replace line 18:
   ```dockerfile
   CMD ["cargo", "build", "--release", "--target", "x86_64-pc-windows-gnu"]
   ```
   with:
   ```dockerfile
   CMD ["bash", "/workspace/docker/cross-test-windows.sh"]
   ```

2. Update the usage comment block (lines 3-6) to remove the bare `cargo build` invocation example and match the compose-centric pattern:
   ```dockerfile
   # Cross-compilation for Windows x86_64 (MinGW)
   #
   # Usage:
   #   docker compose build windows-cross
   #   docker compose run windows-cross
   ```
   Remove line 6 (`#   docker compose run windows-cross cargo build --release --target x86_64-pc-windows-gnu`) since ad-hoc cargo invocations bypass validation and are misleading.

3. Add `WORKDIR /build` before the CMD (currently missing -- the container defaults to `/workspace` from the base image, but the script does `cd /build` itself; adding WORKDIR makes the Dockerfile consistent with `cross-linux-aarch64.Dockerfile` line 18).

**Resulting file (`docker/cross-windows.Dockerfile`):**
```dockerfile
# Cross-compilation for Windows x86_64 (MinGW)
#
# Usage:
#   docker compose build windows-cross
#   docker compose run windows-cross
FROM rust:1.88-slim

RUN apt-get update && apt-get install -y \
    gcc-mingw-w64-x86-64 \
    file \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-pc-windows-gnu

WORKDIR /build

CMD ["bash", "/workspace/docker/cross-test-windows.sh"]
```

---

### Issue 2: test.sh silently swallows great doctor failures

**Priority:** P2

**Current behavior:**

`docker/test.sh` line 41:
```bash
${BIN} doctor 2>&1 || true
```

If `great doctor` exits non-zero (which can legitimately happen inside a minimal container lacking optional tools), the `|| true` suppresses the exit code entirely. No warning is printed. A real regression in `great doctor` would be invisible in CI logs.

**Desired behavior:**

A non-zero exit from `great doctor` prints a visible `[WARN]` line with the exit code, then the script continues. The script must NOT abort on doctor failure (it runs inside `set -e`), so the exit code must be captured, not propagated.

**Changes:**

File: `docker/test.sh`

Replace line 41:
```bash
${BIN} doctor 2>&1 || true
```
with:
```bash
doctor_rc=0
${BIN} doctor 2>&1 || doctor_rc=$?
if [ "$doctor_rc" -ne 0 ]; then
    echo "[WARN] great doctor exited non-zero (exit ${doctor_rc})"
fi
```

This captures the exit code without triggering `set -e`, prints a visible warning, and continues execution.

---

### Issue 3: No toolchain version printed at container startup

**Priority:** P3

**Current behavior:**

None of the four entrypoint scripts (`test.sh`, `cross-test-macos.sh`, `cross-test-windows.sh`, `cross-test-linux-aarch64.sh`) print the active Rust toolchain version. To verify the pinned version, a developer must exec into the container and run `rustc --version` manually.

**Desired behavior:**

Each script prints `rustc --version` output near the top of its run, before step `[1/N]`. This makes the toolchain version visible in every build log without manual intervention.

**Changes:**

All four files get the same addition: a toolchain version block inserted after the banner and before step `[1/N]`.

**File: `docker/test.sh`**

Insert after line 12 (the empty `echo ""`), before line 14 (`echo "[1/5] Copying source..."`):
```bash
# Print toolchain version for build log traceability
echo "Toolchain: $(rustc --version)"
echo ""
```

**File: `docker/cross-test-macos.sh`**

Insert after line 17 (the empty `echo ""`), before line 19 (`echo "[1/4] Copying source..."`).
Note: the macOS script sources osxcross env at line 10. The toolchain version line must go AFTER `source /etc/profile.d/osxcross-env.sh` (line 10) so that the correct toolchain is active:
```bash
# Print toolchain version for build log traceability
echo "Toolchain: $(rustc --version)"
echo ""
```

**File: `docker/cross-test-windows.sh`**

Insert after line 14 (the empty `echo ""`), before line 16 (`echo "[1/4] Copying source..."`):
```bash
# Print toolchain version for build log traceability
echo "Toolchain: $(rustc --version)"
echo ""
```

**File: `docker/cross-test-linux-aarch64.sh`**

Insert after line 14 (the empty `echo ""`), before line 16 (`echo "[1/4] Copying source..."`):
```bash
# Print toolchain version for build log traceability
echo "Toolchain: $(rustc --version)"
echo ""
```

---

### Issue 4: Fragile mkdir in cross-compilation export scripts

**Priority:** P3 (latent P2 -- hard failure when workspace is genuinely read-only)

**Current behavior:**

Three cross-compilation scripts write exported binaries to `/workspace/test-files/`:

| File | Line | Code |
|------|------|------|
| `docker/cross-test-macos.sh` | 66 | `mkdir -p /workspace/test-files` |
| `docker/cross-test-windows.sh` | 48 | `mkdir -p /workspace/test-files` |
| `docker/cross-test-linux-aarch64.sh` | 48 | `mkdir -p /workspace/test-files` |

All three scripts document at the top that `/workspace` is read-only (e.g., `cross-test-macos.sh` line 19: "workspace is read-only"). Yet they write into it.

This works in the current `docker-compose.yml` because of a writable bind-mount overlay:
```yaml
volumes:
  - .:/workspace:ro                    # read-only workspace
  - ./test-files:/workspace/test-files  # writable overlay for output
```

Docker creates the `/workspace/test-files` mount point before the container starts, so `mkdir -p` is a no-op. But this is:
- **Fragile:** Running the container via `docker run` without the overlay mount causes `mkdir` to fail against the read-only filesystem.
- **Misleading:** The scripts contradict their own documentation about workspace being read-only.
- **Non-obvious:** The compose-level overlay is an implicit dependency that is not documented in the scripts.

**Desired behavior:**

Export binaries to `/build/test-files/` (the writable build directory that all scripts already use). Update `docker-compose.yml` to bind-mount `./test-files` at `/build/test-files` instead of `/workspace/test-files`. Add a comment in each script explaining the output path.

**Changes:**

**File: `docker/cross-test-macos.sh`**

Replace lines 64-74:
```bash
# Export binaries to shared volume
echo "[4/4] Exporting binaries..."
mkdir -p /workspace/test-files
for target in "${TARGETS[@]}"; do
    src="target/${target}/release/great"
    # Name like great-x86_64-apple-darwin
    dest="/workspace/test-files/great-${target}"
    cp "$src" "$dest"
    size=$(du -h "$dest" | cut -f1)
    echo "  ${dest} (${size})"
done
```
with:
```bash
# Export binaries to shared volume.
# Output goes to /build/test-files/ (writable); /workspace is read-only.
echo "[4/4] Exporting binaries..."
mkdir -p /build/test-files
for target in "${TARGETS[@]}"; do
    src="target/${target}/release/great"
    # Name like great-x86_64-apple-darwin
    dest="/build/test-files/great-${target}"
    cp "$src" "$dest"
    size=$(du -h "$dest" | cut -f1)
    echo "  ${dest} (${size})"
done
```

Replace lines 77-80 (the closing banner):
```bash
echo "============================================"
echo "  Cross-compilation complete"
echo "  Binaries in /workspace/test-files/"
echo "============================================"
```
with:
```bash
echo "============================================"
echo "  Cross-compilation complete"
echo "  Binaries in /build/test-files/"
echo "============================================"
```

Also update the script header comment (line 5):
```bash
# validates the output binaries, and copies them to /workspace/test-files/
```
to:
```bash
# validates the output binaries, and copies them to /build/test-files/
```

**File: `docker/cross-test-windows.sh`**

Replace lines 46-52:
```bash
# Export binary to shared volume
echo "[4/4] Exporting binary..."
mkdir -p /workspace/test-files
dest="/workspace/test-files/great-${TARGET}.exe"
cp "$bin" "$dest"
size=$(du -h "$dest" | cut -f1)
echo "  ${dest} (${size})"
```
with:
```bash
# Export binary to shared volume.
# Output goes to /build/test-files/ (writable); /workspace is read-only.
echo "[4/4] Exporting binary..."
mkdir -p /build/test-files
dest="/build/test-files/great-${TARGET}.exe"
cp "$bin" "$dest"
size=$(du -h "$dest" | cut -f1)
echo "  ${dest} (${size})"
```

Replace lines 55-58 (closing banner):
```bash
echo "============================================"
echo "  Cross-compilation complete"
echo "  Binary in /workspace/test-files/"
echo "============================================"
```
with:
```bash
echo "============================================"
echo "  Cross-compilation complete"
echo "  Binary in /build/test-files/"
echo "============================================"
```

Also update the script header comment (line 5):
```bash
# validates the output binary, and copies it to /workspace/test-files/
```
to:
```bash
# validates the output binary, and copies it to /build/test-files/
```

**File: `docker/cross-test-linux-aarch64.sh`**

Replace lines 47-52:
```bash
# Export binary to shared volume
echo "[4/4] Exporting binary..."
mkdir -p /workspace/test-files
dest="/workspace/test-files/great-${TARGET}"
cp "$bin" "$dest"
size=$(du -h "$dest" | cut -f1)
echo "  ${dest} (${size})"
```
with:
```bash
# Export binary to shared volume.
# Output goes to /build/test-files/ (writable); /workspace is read-only.
echo "[4/4] Exporting binary..."
mkdir -p /build/test-files
dest="/build/test-files/great-${TARGET}"
cp "$bin" "$dest"
size=$(du -h "$dest" | cut -f1)
echo "  ${dest} (${size})"
```

Replace lines 55-58 (closing banner):
```bash
echo "============================================"
echo "  Cross-compilation complete"
echo "  Binary in /workspace/test-files/"
echo "============================================"
```
with:
```bash
echo "============================================"
echo "  Cross-compilation complete"
echo "  Binary in /build/test-files/"
echo "============================================"
```

Also update the script header comment (line 5):
```bash
# validates the output binary, and copies it to /workspace/test-files/
```
to:
```bash
# validates the output binary, and copies it to /build/test-files/
```

**File: `docker-compose.yml`**

Update the bind-mount path for the three cross services. This is a necessary
companion change -- without it, the host-side `./test-files` directory will not
receive the exported binaries.

For `macos-cross` (line 52), `windows-cross` (line 64), and `linux-aarch64-cross`
(line 76), replace:
```yaml
      - ./test-files:/workspace/test-files
```
with:
```yaml
      - ./test-files:/build/test-files
```

This is three separate one-line changes in `docker-compose.yml` at lines 52, 64,
and 76 respectively.

> **Note on Nightingale scope:** The selection document lists "Changing the compose
> setup or mount configuration" as out of scope, but also requires `/build/test-files/`
> as the output path. These two statements are contradictory. Moving the script
> output path without updating the compose bind-mount would silently break binary
> export (binaries written to `/build/test-files/` would vanish when the container
> exits). The compose mount change is the minimal necessary companion fix and is
> included in this spec. The change is a single path edit per service, not a
> structural change to the compose configuration.

---

## 3. Fix Order

Recommended implementation sequence:

| Step | Issue | Rationale |
|------|-------|-----------|
| 1 | Issue 4 (fragile mkdir + compose mount) | Data correctness. Must be done first because other issues may interact with the export step during testing. Changes compose mount paths. |
| 2 | Issue 2 (test.sh doctor warning) | Silent failure masking. Independent of other changes. Highest risk of subtle `set -e` interaction, so fix early and verify. |
| 3 | Issue 1 (Windows Dockerfile CMD) | Aligns the Windows Dockerfile with the pattern already used by Linux aarch64 and macOS. Depends on the export path fix (issue 4) being in place so the script works correctly. |
| 4 | Issue 3 (toolchain version) | Purely additive. Zero risk of regression. Touch all four scripts last to minimize merge conflicts with earlier fixes. |

All four fixes may be committed in a single commit given the S complexity estimate.

---

## 4. Acceptance Criteria

- [ ] **AC-1:** `docker/cross-windows.Dockerfile` CMD is `["bash", "/workspace/docker/cross-test-windows.sh"]`, not a bare `cargo build`. The usage comment block does not suggest bare `cargo build` invocations.
- [ ] **AC-2:** `docker/cross-windows.Dockerfile` has `WORKDIR /build` before the CMD line.
- [ ] **AC-3:** `docker/test.sh` does not contain `|| true` on the `great doctor` line. Instead, a non-zero exit code is captured into a variable and a `[WARN] great doctor exited non-zero (exit N)` message is printed.
- [ ] **AC-4:** All four entrypoint scripts (`test.sh`, `cross-test-macos.sh`, `cross-test-windows.sh`, `cross-test-linux-aarch64.sh`) print `Toolchain: <rustc --version output>` before step `[1/N]`.
- [ ] **AC-5:** All three cross-compilation scripts (`cross-test-macos.sh`, `cross-test-windows.sh`, `cross-test-linux-aarch64.sh`) export binaries to `/build/test-files/`, not `/workspace/test-files/`. Each has a comment explaining that `/workspace` is read-only.
- [ ] **AC-6:** `docker-compose.yml` binds `./test-files` to `/build/test-files` (not `/workspace/test-files`) for all three cross services (`macos-cross`, `windows-cross`, `linux-aarch64-cross`).
- [ ] **AC-7:** All modified shell scripts pass `shellcheck` with no errors.
- [ ] **AC-8:** Header comments in all three cross scripts reflect the updated output path (`/build/test-files/`).
- [ ] **AC-9:** Closing banner messages in all three cross scripts reflect the updated output path.

---

## 5. Files Modified

| File | Change Type | Issues Addressed |
|------|-------------|-----------------|
| `docker/cross-windows.Dockerfile` | Modify CMD, add WORKDIR, update comments | 1 |
| `docker/test.sh` | Replace `\|\| true` with warning logic, add toolchain print | 2, 3 |
| `docker/cross-test-macos.sh` | Change export path, update comments/banners, add toolchain print | 3, 4 |
| `docker/cross-test-windows.sh` | Change export path, update comments/banners, add toolchain print | 3, 4 |
| `docker/cross-test-linux-aarch64.sh` | Change export path, update comments/banners, add toolchain print | 3, 4 |
| `docker-compose.yml` | Update bind-mount paths for 3 cross services | 4 |

Total: 6 files modified, 0 files created, 0 files deleted.

---

## 6. Out of Scope

- **No new cross-compilation targets.** Only existing scripts and Dockerfiles are modified.
- **No changes to the Layer 2 VM services** (`macos`, `windows`, `ubuntu-vm` in compose). These use a separate `./test-files:/shared` mount that is unrelated to the cross-compilation output.
- **No changes to `ubuntu.Dockerfile` or `fedora.Dockerfile`.** These are headless test containers, not cross-compilation containers. They do not export binaries.
- **No changes to `cross-macos.Dockerfile` or `cross-linux-aarch64.Dockerfile`.** These already invoke their validation scripts via CMD.
- **No structural compose changes.** The compose file changes are limited to updating three bind-mount paths from `/workspace/test-files` to `/build/test-files`.
- **No Rust code changes.** This task is entirely shell scripts and Docker configuration.

---

## 7. Risks

### 7.1 Compose mount path change breaks existing workflows

**Risk:** Developers with cached containers or muscle-memory around `/workspace/test-files` may be surprised.

**Mitigation:** The `docker-compose.yml` change ensures compose users see no difference -- `./test-files` on the host still receives binaries. Only the in-container path changes. The closing banner in each script prints the new path. If someone has scripts that exec into the container and look for `/workspace/test-files`, they will get an empty directory (the read-only workspace mount). This is the correct behavior -- the old path only worked by accident.

### 7.2 `set -e` interaction with doctor warning logic

**Risk:** The variable-capture pattern `cmd || var=$?` is a common shell idiom but could interact unexpectedly with `set -e` in certain bash versions.

**Mitigation:** The pattern `doctor_rc=0; ${BIN} doctor 2>&1 || doctor_rc=$?` is POSIX-compliant and safe under `set -e` because the `||` clause prevents `set -e` from triggering on the non-zero exit. This is tested on bash 4.4+ (Ubuntu 22.04) and bash 5.2+ (Ubuntu 24.04), which are the base images used by the containers.

### 7.3 `rustc --version` not available on PATH

**Risk:** In the macOS cross container, Rust is installed to `/opt/rust/bin` (not the default `~/.cargo/bin`). If PATH is not set correctly, `rustc --version` would fail.

**Mitigation:** The macOS Dockerfile sets `ENV PATH="/opt/rust/bin:${PATH}"` at line 38, so `rustc` is on PATH for any process in the container. The other containers use the default rustup installation which adds to `~/.cargo/bin`, also on PATH via their Dockerfiles. The toolchain version line is placed after `source /etc/profile.d/osxcross-env.sh` in the macOS script as an additional safety measure, but this is not strictly necessary since `rustc` availability comes from the Dockerfile ENV, not the profile script.

### 7.4 Windows Dockerfile WORKDIR /build may conflict with compose working_dir

**Risk:** The compose file also sets `working_dir: /build` for the windows-cross service (line 65). Having both the Dockerfile `WORKDIR` and compose `working_dir` set to `/build` is redundant but not harmful.

**Mitigation:** The compose `working_dir` takes precedence over the Dockerfile `WORKDIR`. Setting both ensures correct behavior whether invoked via compose or direct `docker run`. This matches the pattern in `cross-linux-aarch64.Dockerfile` (WORKDIR /build at line 18) + compose (working_dir /build at line 77).

---

## 8. Verification Procedure

After implementation, Da Vinci should verify:

1. **Shellcheck all modified scripts:**
   ```bash
   shellcheck docker/test.sh docker/cross-test-macos.sh docker/cross-test-windows.sh docker/cross-test-linux-aarch64.sh
   ```
   Expected: zero errors.

2. **Inspect the Dockerfile:**
   ```bash
   grep -n 'CMD\|WORKDIR' docker/cross-windows.Dockerfile
   ```
   Expected: `WORKDIR /build` and `CMD ["bash", "/workspace/docker/cross-test-windows.sh"]`.

3. **Verify no remaining references to `/workspace/test-files` in cross scripts:**
   ```bash
   grep -rn '/workspace/test-files' docker/cross-test-*.sh
   ```
   Expected: zero matches.

4. **Verify compose mount paths updated:**
   ```bash
   grep -n 'test-files' docker-compose.yml
   ```
   Expected: three lines showing `./test-files:/build/test-files`.

5. **Verify toolchain version lines present:**
   ```bash
   grep -n 'rustc --version' docker/test.sh docker/cross-test-*.sh
   ```
   Expected: four matches (one per script).

6. **Verify doctor warning logic:**
   ```bash
   grep -A3 'doctor' docker/test.sh
   ```
   Expected: `doctor_rc=0`, `|| doctor_rc=$?`, and `[WARN]` message. No `|| true`.

---

*End of specification.*
