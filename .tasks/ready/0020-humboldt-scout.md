# 0020: Docker Cross-Compilation UX Improvements — Humboldt Scout Report

**Scout:** Alexander von Humboldt
**Iteration:** 023
**Date:** 2026-02-27
**Spec:** `.tasks/ready/0020-docker-ux-spec.md`
**Socrates:** `.tasks/ready/0020-socrates-review.md`

---

## 1. File Map

### `/home/isaac/src/sh.great/docker/cross-windows.Dockerfile` (18 lines)

**Purpose:** Builds the MinGW cross-compilation image for x86_64-pc-windows-gnu.

| Line | Content | Issue |
|------|---------|-------|
| 1-6  | Comment block — line 6 is the misleading bare-cargo invocation example | Issue 1 |
| 16   | `WORKDIR /workspace` — should be `/build` | Issue 1 |
| 18   | `CMD ["cargo", "build", "--release", "--target", "x86_64-pc-windows-gnu"]` — bypasses validation | Issue 1 |

**All changes confined to this file for Issue 1.**

Target state (full file, 9 lines):
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

### `/home/isaac/src/sh.great/docker/test.sh` (47 lines)

**Purpose:** Main test entrypoint for ubuntu/fedora headless containers. Invoked by
both `ubuntu` and `fedora` compose services. Not a cross-compilation script.

| Line | Content | Issue |
|------|---------|-------|
| 6    | `set -euo pipefail` | Context for Issue 2 — set -e is active |
| 12   | `echo ""` | **Issue 3 insertion point** — toolchain block goes AFTER this line |
| 13   | `(blank)` | — |
| 14   | `# Copy source (workspace is read-only mounted)` | Context comment |
| 15   | `echo "[1/5] Copying source..."` | **Issue 3** — toolchain block must be BEFORE this line |
| 41   | `${BIN} doctor 2>&1 \|\| true` | **Issue 2** — silent failure to replace |
| 44   | `echo ""` | Second blank echo (closing banner) |

**Spec line number note:** The spec says "Insert after line 12 (the empty echo), before line 14
(echo [1/5])". Actual file: line 12 is `echo ""`, line 13 is blank, line 14 is a comment,
line 15 is `echo "[1/5]"`. Insert after line 12, before line 13 (the blank line before the
comment). The insertion context is unambiguous from the surrounding code.

**Issue 2 replacement:**
- Remove line 41: `${BIN} doctor 2>&1 || true`
- Insert in its place:
  ```bash
  doctor_rc=0
  ${BIN} doctor 2>&1 || doctor_rc=$?
  if [ "$doctor_rc" -ne 0 ]; then
      echo "[WARN] great doctor exited non-zero (exit ${doctor_rc})"
  fi
  ```

**Issue 3 insertion** (after line 12, before the blank/comment block):
```bash
# Print toolchain version for build log traceability
echo "Toolchain: $(rustc --version)"
echo ""
```

---

### `/home/isaac/src/sh.great/docker/cross-test-macos.sh` (80 lines)

**Purpose:** Entrypoint for the macOS cross-compilation container. Builds two targets
(x86_64-apple-darwin, aarch64-apple-darwin), validates ELF type, exports to shared volume.

| Line | Content | Issue |
|------|---------|-------|
| 5    | `# validates the output binaries, and copies them to /workspace/test-files/` | **Issue 4** — update path in comment |
| 10   | `source /etc/profile.d/osxcross-env.sh` | Context — toolchain block must go AFTER this line |
| 17   | `echo ""` | **Issue 3 insertion point** — toolchain block goes AFTER this line |
| 18   | `(blank)` | — |
| 19   | `# Copy source to writable build dir (workspace is read-only)` | Context comment |
| 20   | `echo "[1/4] Copying source..."` | **Issue 3** — toolchain block before this |
| 64-74 | Export block — `mkdir -p /workspace/test-files`, `dest="/workspace/test-files/..."` | **Issue 4** — change to `/build/test-files` |
| 66   | `mkdir -p /workspace/test-files` | **Issue 4** — change to `mkdir -p /build/test-files` |
| 70   | `dest="/workspace/test-files/great-${target}"` | **Issue 4** — change to `/build/test-files/...` |
| 76-80 | Closing banner — `echo "  Binaries in /workspace/test-files/"` | **Issue 4** — update path |
| 79   | `echo "  Binaries in /workspace/test-files/"` | **Issue 4** — change to `/build/test-files/` |

**Spec line number note (Socrates Concern 1):** The spec says "Insert after line 17 (the empty
echo), before line 19 (echo [1/4])". Actual file: line 17 is `echo ""`, line 18 is blank,
line 19 is a comment, line 20 is `echo "[1/4]"`. Insertion point is after line 17.

**Issue 4 export block (lines 64-74 actual):**
- Line 64: `# Export binaries to shared volume`
- Line 65: `echo "[4/4] Exporting binaries..."`
- Line 66: `mkdir -p /workspace/test-files`
- Lines 67-74: loop with `dest="/workspace/test-files/great-${target}"`

Replace lines 64-74 with the `/build/test-files` equivalent (spec Section 2, Issue 4).

**Issue 4 closing banner (lines 76-80 actual):**
- Line 76: `echo ""`
- Line 77: `echo "============================================"`
- Line 78: `echo "  Cross-compilation complete"`
- Line 79: `echo "  Binaries in /workspace/test-files/"`
- Line 80: `echo "============================================"`

Replace only line 79.

---

### `/home/isaac/src/sh.great/docker/cross-test-windows.sh` (58 lines)

**Purpose:** Entrypoint for the Windows cross-compilation container. Builds
x86_64-pc-windows-gnu, validates PE32+ format, exports to shared volume.

| Line | Content | Issue |
|------|---------|-------|
| 5    | `# validates the output binary, and copies it to /workspace/test-files/` | **Issue 4** — update path in comment |
| 7    | `set -euo pipefail` | Context |
| 14   | `echo ""` | **Issue 3 insertion point** — toolchain block goes AFTER this line |
| 15   | `(blank)` | — |
| 16   | `# Copy source to writable build dir (workspace is read-only)` | Context comment |
| 17   | `echo "[1/4] Copying source..."` | **Issue 3** — toolchain block before this |
| 46-52 | Export block | **Issue 4** |
| 46   | `# Export binary to shared volume` | **Issue 4** — first line of block |
| 47   | `echo "[4/4] Exporting binary..."` | — |
| 48   | `mkdir -p /workspace/test-files` | **Issue 4** — change to `/build/test-files` |
| 49   | `dest="/workspace/test-files/great-${TARGET}.exe"` | **Issue 4** |
| 50   | `cp "$bin" "$dest"` | — |
| 51   | `size=$(du -h "$dest" \| cut -f1)` | — |
| 52   | `echo "  ${dest} (${size})"` | — |
| 54   | `echo ""` | Closing blank |
| 55   | `echo "============================================"` | — |
| 56   | `echo "  Cross-compilation complete"` | — |
| 57   | `echo "  Binary in /workspace/test-files/"` | **Issue 4** — change to `/build/test-files/` |
| 58   | `echo "============================================"` | — |

**Spec line number note (Socrates Concern 1):** The spec says "Insert after line 14 (the empty
echo), before line 16 (echo [1/4])". Actual file: line 14 is `echo ""`, line 15 is blank,
line 16 is a comment, line 17 is `echo "[1/4]"`. Insertion point is after line 14.

---

### `/home/isaac/src/sh.great/docker/cross-test-linux-aarch64.sh` (58 lines)

**Purpose:** Entrypoint for the Linux aarch64 cross-compilation container. Builds
aarch64-unknown-linux-gnu, validates ELF ARM aarch64 format, exports to shared volume.

| Line | Content | Issue |
|------|---------|-------|
| 5    | `# validates the output binary, and copies it to /workspace/test-files/` | **Issue 4** — update path in comment |
| 7    | `set -euo pipefail` | Context |
| 14   | `echo ""` | **Issue 3 insertion point** — toolchain block goes AFTER this line |
| 15   | `(blank)` | — |
| 16   | `# Copy source to writable build dir (workspace is read-only)` | Context comment |
| 17   | `echo "[1/4] Copying source..."` | **Issue 3** — toolchain block before this |
| 46-52 | Export block | **Issue 4** |
| 46   | `# Export binary to shared volume` | **Issue 4** — first line of block |
| 47   | `echo "[4/4] Exporting binary..."` | — |
| 48   | `mkdir -p /workspace/test-files` | **Issue 4** — change to `/build/test-files` |
| 49   | `dest="/workspace/test-files/great-${TARGET}"` | **Issue 4** |
| 50   | `cp "$bin" "$dest"` | — |
| 51   | `size=$(du -h "$dest" \| cut -f1)` | — |
| 52   | `echo "  ${dest} (${size})"` | — |
| 54   | `echo ""` | Closing blank |
| 55   | `echo "============================================"` | — |
| 56   | `echo "  Cross-compilation complete"` | — |
| 57   | `echo "  Binary in /workspace/test-files/"` | **Issue 4** — change to `/build/test-files/` |
| 58   | `echo "============================================"` | — |

**Note:** `cross-test-windows.sh` and `cross-test-linux-aarch64.sh` are structurally
identical in their export blocks (lines 46-58). The only differences are the binary name
(`.exe` suffix vs none), format validation string (`PE32+` vs `ELF 64-bit.*ARM aarch64`),
and `dest` filename. Both files require the same set of changes.

---

### `/home/isaac/src/sh.great/docker-compose.yml` (152 lines)

**Purpose:** Orchestrates all 8 services (2 headless test, 3 cross-compilation, 3 VMs).

| Line | Content | Issue |
|------|---------|-------|
| 52   | `- ./test-files:/workspace/test-files` (macos-cross volumes) | **Issue 4** — change to `/build/test-files` |
| 53   | `working_dir: /build` | Context — /build is writable in macos-cross |
| 64   | `- ./test-files:/workspace/test-files` (windows-cross volumes) | **Issue 4** — change to `/build/test-files` |
| 65   | `working_dir: /build` | Context — confirms /build is writable |
| 66   | `command: ["bash", "/workspace/docker/cross-test-windows.sh"]` | Context — compose overrides the old CMD |
| 76   | `- ./test-files:/workspace/test-files` (linux-aarch64-cross volumes) | **Issue 4** — change to `/build/test-files` |
| 77   | `working_dir: /build` | Context |
| 100  | `- ./test-files:/shared` (macos VM) | NOT changed — different mount point for VM |
| 122  | `- ./test-files:/shared` (windows VM) | NOT changed |
| 143  | `- ./test-files:/shared` (ubuntu-vm) | NOT changed |

**Only lines 52, 64, 76 are modified.** The Layer 2 VM mounts at `/shared` are untouched.

---

### Files NOT Modified

| File | Reason |
|------|--------|
| `docker/cross-macos.Dockerfile` | CMD already invokes validation script (line 84). WORKDIR /workspace at line 82 is correct for this image layout (two-stage build sets WORKDIR /build at line 76, then resets to /workspace at line 82 before CMD). |
| `docker/cross-linux-aarch64.Dockerfile` | Already the reference implementation. WORKDIR /build at line 18. CMD calls script at line 20. |
| `docker/ubuntu.Dockerfile` | Headless test container — no binary export, no CMD (compose provides command). |
| `docker/fedora.Dockerfile` | Same as ubuntu.Dockerfile — no binary export, no CMD. |
| `.github/workflows/ci.yml` | No Docker compose usage — builds directly with cargo. |
| `.github/workflows/release.yml` | Same — no Docker compose. |
| `.github/workflows/deployment.yml` | Site deploy only — no Docker. |
| `.github/workflows/cdk.yml` | Infrastructure only — no Docker. |

---

## 2. Patterns

### Banner Style (all 4 scripts)

```bash
echo "============================================"
echo "  great.sh <description>"
echo "============================================"
echo ""
```

Closing banner:
```bash
echo ""
echo "============================================"
echo "  Cross-compilation complete"
echo "  Binary/Binaries in <path>"
echo "============================================"
```

The toolchain line fits naturally between the opening banner's trailing `echo ""` and the
first step comment. Da Vinci must insert two lines:
```bash
# Print toolchain version for build log traceability
echo "Toolchain: $(rustc --version)"
echo ""
```

### Error Handling

All 4 scripts open with `set -euo pipefail`. Fatal validation errors use:
```bash
echo "FATAL: <message>"
exit 1
```

The `|| true` on line 41 of `test.sh` is the sole exception to `set -e` and is the exact
target of Issue 2.

### Export Block Pattern (cross scripts only)

Standard block structure:
```bash
# Export binary/binaries to shared volume
echo "[4/4] Exporting binary/binaries..."
mkdir -p /workspace/test-files        # <-- change to /build/test-files
dest="/workspace/test-files/..."      # <-- change to /build/test-files/...
cp "$bin" "$dest"
size=$(du -h "$dest" | cut -f1)
echo "  ${dest} (${size})"
```

The macOS script uses a `for` loop over `TARGETS` array. Windows and Linux aarch64 are
single-target with no loop.

### Source Copy Pattern (all 4 scripts)

```bash
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
cd /build        # cross scripts only; test.sh has no cd (compose sets working_dir)
```

`test.sh` differs: it does NOT have `[ -d /workspace/templates ]` guard (line 18 is an
unconditional `cp -r /workspace/templates /build/templates`). Do not normalize this — it
is out of scope.

---

## 3. Dependencies

```
docker-compose.yml
  ├── ubuntu/fedora services
  │     └── command: docker/test.sh (Issues 2, 3)
  │
  ├── macos-cross service
  │     ├── build: docker/cross-macos.Dockerfile (not modified)
  │     ├── volumes: ./test-files:/workspace/test-files  (Issue 4 — line 52)
  │     └── command: docker/cross-test-macos.sh (Issues 3, 4)
  │
  ├── windows-cross service
  │     ├── build: docker/cross-windows.Dockerfile (Issue 1)
  │     ├── volumes: ./test-files:/workspace/test-files  (Issue 4 — line 64)
  │     └── command: docker/cross-test-windows.sh (Issues 3, 4)
  │
  └── linux-aarch64-cross service
        ├── build: docker/cross-linux-aarch64.Dockerfile (not modified)
        ├── volumes: ./test-files:/workspace/test-files  (Issue 4 — line 76)
        └── command: docker/cross-test-linux-aarch64.sh (Issues 3, 4)

test-files/ directory (exists on host)
  ├── great-aarch64-unknown-linux-gnu  (produced by linux-aarch64-cross)
  ├── great-x86_64-apple-darwin        (produced by macos-cross)
  └── great-aarch64-apple-darwin       (produced by macos-cross)
  ... great-x86_64-pc-windows-gnu.exe  (produced by windows-cross, not yet present)
```

The VM services (`macos`, `windows`, `ubuntu-vm`) bind-mount `./test-files:/shared` — they
consume the exported binaries from the cross-compilation containers. These mounts are
UNAFFECTED by the Issue 4 changes (they mount `./test-files` to `/shared`, not
`/workspace/test-files`).

---

## 4. Gotchas

**G1 — cross-windows.Dockerfile WORKDIR conflict is benign.**
Current line 16 is `WORKDIR /workspace`. After the fix it becomes `WORKDIR /build`.
The compose service also sets `working_dir: /build` (line 65). Both pointing to `/build`
is correct and intentional. Confirmed by the cross-linux-aarch64 pattern (Dockerfile
`WORKDIR /build` + compose `working_dir: /build`).

**G2 — cross-macos.Dockerfile has TWO WORKDIR lines.**
Line 76: `WORKDIR /build` (used during dependency pre-fetch stage).
Line 82: `WORKDIR /workspace` (reset before CMD).
This is intentional and is NOT touched by this task.

**G3 — test.sh line 18 is unconditional (no `[ -d ]` guard).**
`cp -r /workspace/templates /build/templates` — differs from cross scripts which guard with
`[ -d /workspace/templates ]`. This is out of scope; do not "fix" it during this task.

**G4 — test-files/ directory already exists on the host.**
The directory contains three binary files from prior cross-compilation runs. The compose
bind-mount at lines 52/64/76 presupposes this directory exists. After changing the mount
from `/workspace/test-files` to `/build/test-files`, Docker will still bind-mount the same
`./test-files` host directory — only the container-side path changes. No host-side directory
creation needed.

**G5 — `du -h "$dest" | cut -f1` pipe.**
These lines in all three export blocks use `$dest` after setting it to the new path. Since
`dest` is set from the variable (not hardcoded), the `du` command will automatically use
the correct new path once the `dest=` assignment is updated. No additional change needed.

**G6 — Issue 4 spec replacement ranges vs. actual line numbers.**
The spec says "Replace lines 46-52" for linux-aarch64 and windows. Socrates confirmed
the comment at line 46 is actually the start of the block (off by one in spec narrative,
but the code blocks in the spec are correct). Da Vinci must match by text content, not
line number. The replacement blocks in the spec are verbatim correct.

**G7 — The macOS script sources osxcross-env.sh before the toolchain print.**
Issue 3 says to insert the toolchain block AFTER line 17 (`echo ""`). Line 10
(`source /etc/profile.d/osxcross-env.sh`) comes before line 17, so `rustc` is already
on PATH from the Dockerfile `ENV PATH="/opt/rust/bin:${PATH}"` regardless. The ordering
relative to `source` does not affect `rustc` availability, but keep the insertion after
line 17 as specified.

**G8 — windows-cross service has no `/build` volume for cargo cache.**
Compare the three cross services:
- `macos-cross`: `cargo-cache-macos:/opt/rust/registry`
- `windows-cross`: `cargo-cache-windows:/root/.cargo/registry`
- `linux-aarch64-cross`: `cargo-cache-linux-aarch64:/root/.cargo/registry`

None of these mounts involve `/build/test-files`, so the bind-mount addition
`./test-files:/build/test-files` does not conflict with any existing mount.

---

## 5. Advisory Items from Socrates

**Concern 1 — Off-by-one line numbers in spec (ADVISORY, not blocking).**

The actual insertion points for Issue 3 are:

| Script | Spec says "after line" | Actual `echo ""` at | `[1/N]` echo at |
|--------|------------------------|---------------------|-----------------|
| `test.sh` | 12 | **12** (correct) | 15 (not 14) |
| `cross-test-macos.sh` | 17 | **17** (correct) | 20 (not 19) |
| `cross-test-windows.sh` | 14 | **14** (correct) | 17 (not 16) |
| `cross-test-linux-aarch64.sh` | 14 | **14** (correct) | 17 (not 16) |

The "after line N" reference is correct. The "before line N" reference is off by 2 in all
cases (there is a blank line and a comment between the echo "" and the [1/N] echo). Da Vinci
should match by text context, not by the "before line N" number.

For Issue 4 linux-aarch64: spec says "Replace lines 47-52" but the comment
`# Export binary to shared volume` starts at **line 46**. Use the code block text, not the
line number, to locate the replacement.

**Concern 2 — cross-macos.Dockerfile line 6 has a misleading bare-cargo usage comment.**
`docker compose run macos-cross cargo build --release --target x86_64-apple-darwin`
This is outside 0020 scope. The macOS CMD already invokes the script, so there is no
security/correctness risk. File a follow-up backlog item if cleanup is desired.

**Concern 3 — cross-linux-aarch64.Dockerfile usage comment mounts workspace read-write.**
Lines 4-5 show `docker run --rm -v $(pwd):/workspace great-cross-aarch64` without `:ro`.
Outside 0020 scope. The CMD already invokes the validation script. Documentation-only issue.

**Concern 4 — `rustup show active-toolchain` omitted from toolchain print.**
The spec chose `rustc --version` only (simpler, sufficient). Nightingale selection mentioned
both. The spec's simplification is reasonable. Da Vinci can add `rustup show active-toolchain`
as a second line if desired, but it is not required by the spec.

---

## 6. Recommended Build Order

Follows spec Section 3 (Fix Order):

| Step | Issue | File(s) | Method |
|------|-------|---------|--------|
| 1 | Issue 4: export paths | `docker/cross-test-macos.sh`, `docker/cross-test-windows.sh`, `docker/cross-test-linux-aarch64.sh`, `docker-compose.yml` | Edit 4 files: 3 script export blocks + header comments + closing banners; 3 one-line compose mount changes |
| 2 | Issue 2: doctor warning | `docker/test.sh` | Replace 1 line (line 41) with 5-line block |
| 3 | Issue 1: Windows Dockerfile CMD | `docker/cross-windows.Dockerfile` | Rewrite file (18 lines → 17 lines: remove comment line 6, change WORKDIR line 16, change CMD line 18) |
| 4 | Issue 3: toolchain version | `docker/test.sh`, `docker/cross-test-macos.sh`, `docker/cross-test-windows.sh`, `docker/cross-test-linux-aarch64.sh` | Insert 3-line block in each script after the opening banner's trailing `echo ""` |

Run `shellcheck docker/test.sh docker/cross-test-macos.sh docker/cross-test-windows.sh docker/cross-test-linux-aarch64.sh` after all edits.

---

*"The most dangerous worldview is the worldview of those who have not viewed the world."*
