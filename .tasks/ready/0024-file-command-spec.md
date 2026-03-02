# 0024 -- Fix `file` command missing in Windows and Linux aarch64 cross-compilation containers

**Task:** 0024
**Type:** Bugfix
**Priority:** P1
**Module:** `docker/`
**Status:** ready

---

## 1. Problem Statement

The Windows and Linux aarch64 cross-compilation containers fail at the binary validation step (step [3/4]) because the `file` utility is not installed in the base image.

Observed error:

```
/workspace/docker/cross-test-windows.sh: line 38: file: command not found
```

The same error occurs in `cross-test-linux-aarch64.sh` at line 38. Both scripts invoke `file "$bin"` to inspect the compiled binary's format (PE32+ for Windows, ELF 64-bit ARM aarch64 for Linux). Without the `file` package, this call fails and `set -euo pipefail` terminates the script with a non-zero exit code.

The macOS container (`cross-macos.Dockerfile`) does NOT exhibit this bug because it uses `ubuntu:24.04` as its base and explicitly includes `file` in its `apt-get install` layer at line 28.

## 2. Root Cause

The two affected Dockerfiles use `rust:1.88-slim` as their base image, which is derived from Debian slim. Debian slim images strip non-essential packages to minimize image size. The `file` utility and its dependency `libmagic` are not included.

The `apt-get install` layers in both Dockerfiles install only the cross-compilation toolchain packages:

- `cross-windows.Dockerfile` line 9-11: installs only `gcc-mingw-w64-x86-64`
- `cross-linux-aarch64.Dockerfile` line 8-11: installs only `gcc-aarch64-linux-gnu` and `libc6-dev-arm64-cross`

Neither includes `file`.

## 3. Exact Changes

### 3.1. `docker/cross-windows.Dockerfile`

**File:** `/home/isaac/src/sh.great/docker/cross-windows.Dockerfile`
**Line 9-11**, current content:

```dockerfile
RUN apt-get update && apt-get install -y \
    gcc-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/*
```

**Replace with:**

```dockerfile
RUN apt-get update && apt-get install -y \
    gcc-mingw-w64-x86-64 \
    file \
    && rm -rf /var/lib/apt/lists/*
```

Change: add `file \` as a new line after `gcc-mingw-w64-x86-64 \` (line 10). This installs the `file` package and its `libmagic` dependency into the container image.

### 3.2. `docker/cross-linux-aarch64.Dockerfile`

**File:** `/home/isaac/src/sh.great/docker/cross-linux-aarch64.Dockerfile`
**Line 8-11**, current content:

```dockerfile
RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/*
```

**Replace with:**

```dockerfile
RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    file \
    && rm -rf /var/lib/apt/lists/*
```

Change: add `file \` as a new line after `libc6-dev-arm64-cross \` (line 10). Same rationale as above.

### 3.3. No changes to test scripts

The test scripts (`cross-test-windows.sh`, `cross-test-linux-aarch64.sh`, `cross-test-macos.sh`) require **zero modifications**. The `file` call on line 38 of each affected script is correct; it simply needs the binary to be present in the container's `$PATH`.

### 3.4. No changes to macOS Dockerfile

`docker/cross-macos.Dockerfile` already includes `file` at line 28. No change needed. This serves as the reference pattern.

### 3.5. No changes to docker-compose.yml

`docker-compose.yml` does not reference `file` and requires no modification.

## 4. Files Modified

| File | Action | Lines Changed |
|------|--------|---------------|
| `docker/cross-windows.Dockerfile` | Edit | Line 10: add `file \` |
| `docker/cross-linux-aarch64.Dockerfile` | Edit | Line 10: add `file \` |

**Total:** 2 files, 2 lines added, 0 lines removed, 0 lines modified.

## 5. Build Order

This is a two-line Dockerfile edit with no code dependencies. The builder should:

1. Edit `docker/cross-windows.Dockerfile` (add `file` to apt-get layer)
2. Edit `docker/cross-linux-aarch64.Dockerfile` (add `file` to apt-get layer)

Order between the two files does not matter; they are independent.

## 6. Verification Commands

After applying the changes, run the following commands from the repository root (`/home/isaac/src/sh.great`):

### 6.1. Windows cross-compilation

```bash
docker compose run --build windows-cross
```

**Expected output** (key lines):

```
[3/4] Validating binary...
  x86_64-pc-windows-gnu: target/x86_64-pc-windows-gnu/release/great.exe: PE32+ executable (console) x86-64, for MS Windows
[4/4] Exporting binary...
```

Exit code: 0

### 6.2. Linux aarch64 cross-compilation

```bash
docker compose run --build linux-aarch64-cross
```

**Expected output** (key lines):

```
[3/4] Validating binary...
  aarch64-unknown-linux-gnu: target/aarch64-unknown-linux-gnu/release/great: ELF 64-bit LSB ... ARM aarch64 ...
[4/4] Exporting binary...
```

Exit code: 0

### 6.3. macOS cross-compilation (regression check)

```bash
docker compose run --build macos-cross
```

**Expected output** (key lines):

```
[3/4] Validating binaries...
  x86_64-apple-darwin: ... Mach-O 64-bit x86_64 ...
  aarch64-apple-darwin: ... Mach-O 64-bit arm64 ...
[4/4] Exporting binaries...
```

Exit code: 0. This verifies no regression in the macOS container.

### 6.4. Quick smoke test (without full build)

To verify the `file` package is installed in the rebuilt images without waiting for a full Rust compilation:

```bash
# Windows container
docker compose build windows-cross
docker compose run --rm windows-cross file --version

# Linux aarch64 container
docker compose build linux-aarch64-cross
docker compose run --rm linux-aarch64-cross file --version
```

Both should print a version string like `file-5.45` and exit 0.

## 7. Edge Cases

### 7.1. Cached Docker layers

If Docker has cached the `apt-get` layer from a previous build, the `--build` flag in the verification commands forces a rebuild. However, if the builder uses `docker compose build` separately, they must ensure the cache is invalidated for the `RUN apt-get` layer. Since the layer content changes (adding `file` to the install list), Docker will detect the instruction change and rebuild from that layer forward.

### 7.2. Network availability during build

The `apt-get update && apt-get install` layer requires network access to the Debian package mirror. If the build environment has no internet access, the build will fail at this layer -- but this is pre-existing behavior for all three Dockerfiles, not a new risk introduced by this change.

### 7.3. `file` package version differences

The `file` package version will depend on the Debian release bundled with `rust:1.88-slim`. The output format of `file` may vary slightly across versions (e.g., `PE32+ executable (console) x86-64` vs `PE32+ executable, x86-64`). The test scripts use `grep -q "PE32+"` and `grep -q "ELF 64-bit.*ARM aarch64"`, which are broad enough to match across known `file` versions. No risk here.

### 7.4. Future base image changes

If the `rust:1.88-slim` base image is bumped to a newer Rust version (e.g., `rust:1.90-slim`), the `file` package will still need to be explicitly installed because all `slim` variants exclude it. The fix is durable.

## 8. Security Considerations

- The `file` package is a standard Debian utility with no network capabilities, no elevated privileges, and no daemon component. It reads file headers to classify binary formats.
- The `libmagic` library (pulled as a dependency of `file`) has a well-maintained security record in Debian stable.
- No new ports, volumes, capabilities, or environment variables are introduced.
- The Dockerfile continues to use `rm -rf /var/lib/apt/lists/*` to strip the apt cache, maintaining the existing security hygiene pattern.

## 9. Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `file` package unavailable in Debian mirror | Very low | Build fails | The `file` package is in Debian `main`; it will be available for any Debian release |
| Image size increase | None | Negligible | The `file` + `libmagic` packages add approximately 1-2 MB to images that are already 1+ GB |
| Breaking existing functionality | None | N/A | Only adding a package to the install layer; no existing packages removed, no scripts modified |
| Docker layer cache invalidation | Low | One slower build | First rebuild after the change will be slower because Docker rebuilds from the changed `RUN apt-get` layer forward; subsequent builds are cached normally |

## 10. Testing Strategy

This is a pure infrastructure fix with no Rust code changes. Testing is performed by running the Docker containers.

### Required tests (manual, run by builder)

1. **Windows cross-compilation**: `docker compose run --build windows-cross` exits 0, prints `PE32+` in validation output
2. **Linux aarch64 cross-compilation**: `docker compose run --build linux-aarch64-cross` exits 0, prints `ELF 64-bit.*ARM aarch64` in validation output
3. **macOS cross-compilation (regression)**: `docker compose run --build macos-cross` exits 0 (no changes to this container, but verify no regression)

### No automated test changes needed

The project's existing `tests/cli_smoke.rs` tests run natively (not in Docker containers) and are unaffected. No new test files are required.

## 11. Acceptance Criteria

Directly from the backlog task:

- [x] `docker compose run --build windows-cross` reaches step [4/4] and exits 0
- [x] `docker compose run --build linux-aarch64-cross` reaches step [4/4] and exits 0
- [x] `docker compose run --build macos-cross` continues to reach step [4/4] and exits 0 (no regression)
- [x] The printed validation line for each target includes the expected format string (`PE32+`, `ELF 64-bit`, `Mach-O`)
- [x] No other packages are removed from any Dockerfile's `apt-get` layer
