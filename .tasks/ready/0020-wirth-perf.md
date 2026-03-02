# 0020: Performance Assessment — Docker Cross-Compilation UX Improvements

**Agent:** Niklaus Wirth (Performance Sentinel)
**Iteration:** 023
**Date:** 2026-02-27
**Spec:** `.tasks/ready/0020-docker-ux-spec.md`

---

```
VERDICT: PASS (with WARN on pre-existing state inconsistency)

Measurements:
- artifact_size: 10,871,272 bytes — UNCHANGED (no Rust source changes)
- benchmark: N/A — no bench suite; cross-compilation scripts are not hot paths
- new_dependencies: 0

Regressions:
- [WARN] cross-test-macos.sh export path partially pre-applied but compose mount
         not yet updated — /build/test-files (script) vs /workspace/test-files
         (compose line 52). Binaries written inside container do not reach host.
         This is a pre-existing inconsistency, not introduced by task 0020, but
         Da Vinci must fix both atomically.

Summary: All six file changes are pure shell/YAML edits with negligible
         performance cost; the Rust binary is unaffected; one pre-existing path
         inconsistency in the macOS cross script requires atomic correction.
```

---

## 1. Scope

Task 0020 modifies six configuration and shell script files only. No Rust source
code is touched, so the compiled binary (`target/release/great`, currently
10,871,272 bytes) is unaffected. All five performance questions are answered
below through static analysis and file measurement.

---

## 2. File Measurements

### Current state (pre-task)

| File | Lines | Bytes |
|------|------:|------:|
| `docker/cross-windows.Dockerfile` | 18 | 500 |
| `docker/test.sh` | 47 | 1,343 |
| `docker/cross-test-macos.sh` | 81 | 2,601 |
| `docker/cross-test-windows.sh` | 58 | 1,753 |
| `docker/cross-test-linux-aarch64.sh` | 58 | 1,803 |
| `docker-compose.yml` | 152 | 4,130 |
| **Total** | **414** | **12,130** |

### Projected state (post-task, per spec)

| File | Delta Lines | Delta Bytes | Notes |
|------|------------:|------------:|-------|
| `docker/cross-windows.Dockerfile` | -1 | ~-50 | Remove bare `cargo build` comment; change WORKDIR + CMD |
| `docker/test.sh` | +5 | ~+140 | doctor_rc block (+3 lines) + toolchain print (+2 lines) |
| `docker/cross-test-macos.sh` | +2 | ~+42 | Toolchain print (+2 lines); banner fix; header comment fix |
| `docker/cross-test-windows.sh` | +2 | ~+51 | Export path fix; toolchain print (+2 lines); banner + header |
| `docker/cross-test-linux-aarch64.sh` | +2 | ~+51 | Same as windows script |
| `docker-compose.yml` | 0 | ~-30 | Three path strings shortened by 10 chars each |
| **Total** | **+10** | **~+204** | **+1.68% across these six files** |

Total script corpus grows by approximately 204 bytes (+1.68%). This is noise.

---

## 3. Question-by-Question Analysis

### Q1: Does the export path change from `/workspace/test-files` to `/build/test-files` affect I/O performance?

**No measurable impact.**

Both paths are local filesystem paths inside the container. The kernel resolves
them through the same VFS layer. The bind-mount is established at container
startup before any script runs; the path string that `mkdir` and `cp` use has
no bearing on throughput. The binary files being copied are 10-11 MB each; the
copy is I/O-bound on disk bandwidth, not on path resolution.

The change is a correctness fix (write to the writable `/build` tree rather than
the accidentally-writable overlay at `/workspace/test-files`). Zero performance
delta.

### Q2: Does adding `rustc --version` to 4 scripts add measurable startup latency?

**Yes, by a trivially bounded amount.**

`rustc --version` forks a subprocess, loads the `rustc` binary into memory (which
will already be warm in the page cache since the container just ran a full build),
reads the version string, and exits. On any reasonable container hardware this
completes in under 50 ms. The cross-compilation builds themselves take 2-10
minutes each. The version print adds well under 0.1% to total container runtime.

The invocation is a command substitution inside `echo`:
```bash
echo "Toolchain: $(rustc --version)"
```
This is evaluated once, at the top of the script, not in a loop. It is also
diagnostically valuable for CI log traceability.

**PASS — not measurable against compilation time.**

### Q3: Does the `doctor_rc` variable capture pattern have any performance difference vs `|| true`?

**No. Both are O(1) shell operations.**

The current code:
```bash
${BIN} doctor 2>&1 || true
```

The replacement:
```bash
doctor_rc=0
${BIN} doctor 2>&1 || doctor_rc=$?
if [ "$doctor_rc" -ne 0 ]; then
    echo "[WARN] great doctor exited non-zero (exit ${doctor_rc})"
fi
```

Both patterns execute `${BIN} doctor` exactly once. The `|| true` and
`|| doctor_rc=$?` clauses are shell built-ins that execute in microseconds. The
`if [ ... ]` test on an integer is also a built-in. The total added overhead
is three shell built-in evaluations — unmeasurable in the context of a
subprocess invocation that takes ~100 ms.

The spec correctly notes this pattern is safe under `set -e` on bash 4.4+.

**PASS — zero performance difference.**

### Q4: Do the compose mount path changes affect container startup time?

**No.**

Docker bind-mount setup is performed by the container runtime before the
entrypoint executes. The mount path string (`/workspace/test-files` vs
`/build/test-files`) is resolved once at mount time by the kernel VFS layer.
The string length difference (24 chars vs 18 chars) is below any measurable
threshold. Container startup overhead is dominated by image layer loading
and cgroup setup, not by mount path resolution.

Additionally, changing from a two-layer mount (`:ro` workspace + writable
overlay at `/workspace/test-files`) to a single writable mount at
`/build/test-files` eliminates a kernel VFS overlay stacking operation. This
is a marginal improvement, not a regression.

**PASS — negligible improvement, not a regression.**

### Q5: Does the Windows Dockerfile rewrite affect image build time or size?

**No meaningful impact.**

The changes to `docker/cross-windows.Dockerfile` are:
1. Remove one comment line from the usage block.
2. Change `WORKDIR /workspace` to `WORKDIR /build`.
3. Change the `CMD` from a bare `cargo build` invocation to
   `["bash", "/workspace/docker/cross-test-windows.sh"]`.

None of these changes affect any `RUN` layer. Docker image layers are
determined by `RUN`, `COPY`, and `ADD` instructions. `WORKDIR` and `CMD`
are metadata written into the image manifest; they add zero bytes to image
content and take microseconds to process during build. The two `RUN` layers
(apt-get install + rustup target add) are unchanged and will be served from
cache on any warmed builder.

Projected image layer delta: **0 bytes** to content layers; ~20 bytes to
manifest metadata.

**PASS — no image size regression.**

---

## 4. Pre-Existing Inconsistency: macOS Export Path (WARN)

During measurement, a pre-existing inconsistency was detected in the current
working tree that is directly relevant to task 0020:

**`docker/cross-test-macos.sh`** already has the export path updated to
`/build/test-files` (lines 67, 71) and the header comment updated (line 5),
but the closing banner (line 80) still reads `/workspace/test-files/`.

**`docker-compose.yml`** still mounts `./test-files:/workspace/test-files`
for the `macos-cross` service (line 52), not `/build/test-files`.

This means in the current state, the macOS cross-compilation script writes
binaries to `/build/test-files/` inside the container, but the host-side
`./test-files` directory is bind-mounted at `/workspace/test-files/` — a
path the script no longer writes to. The exported binaries do not reach the
host. This is a **data correctness failure** for the macOS cross service.

This inconsistency exists in the pre-task working tree, not as a result of
task 0020 changes. Da Vinci must correct it atomically:

1. Fix the macOS script banner (line 80): `/workspace/test-files/` -> `/build/test-files/`
2. Fix all three compose mount paths (lines 52, 64, 76): `:/workspace/test-files` -> `:/build/test-files`

These are already included in the task 0020 spec (AC-6, AC-9). The spec's
recommended implementation order (Issue 4 first) is correct — do the compose
mount fix before or simultaneously with the script path fix.

---

## 5. Dependency Check

Zero new dependencies introduced. All six modified files are shell scripts and
YAML configuration. No `Cargo.toml` changes, no `package.json` changes.

Current direct Rust dependencies: 15 (unchanged).
Transitive dependency count: unchanged.

---

## 6. Allocation and Resource Pattern Review

No hot-path code is modified. The changes that exist are:

- Shell variable assignment (`doctor_rc=0`): stack allocation, no heap.
- Integer comparison (`[ "$doctor_rc" -ne 0 ]`): built-in, no allocation.
- One additional `echo` call: writes to stdout fd, no allocation beyond the
  string literal.
- `rustc --version` subprocess: bounded, single-shot, outside any loop.

No unbounded allocations, no O(n²) patterns, no missing pagination. These are
configuration files for container-based builds, not application hot paths.

---

## 7. Summary

| Check | Result |
|-------|--------|
| Binary size delta | 0 bytes — no Rust changes |
| Script corpus delta | +204 bytes (+1.68%) across 6 files |
| New dependencies | 0 |
| Export path I/O | No delta — same kernel VFS semantics |
| `rustc --version` latency | <50 ms per container run — negligible |
| `doctor_rc` capture pattern | Identical cost to `\|\| true` |
| Compose mount startup | Negligible; one fewer VFS overlay layer |
| Dockerfile build/size | 0 bytes to image content layers |
| Pre-existing path inconsistency | WARN — macOS script and compose mount |
|   | are out of sync; Da Vinci must fix both |

**VERDICT: PASS.** Task 0020 introduces no performance regressions. All changes
are shell control flow, comment text, and YAML path strings. The Rust binary is
untouched. The pre-existing macOS export path inconsistency is a correctness
concern (not performance) and is already addressed by the spec's AC-6 and AC-9
requirements. Da Vinci must apply those changes atomically.
