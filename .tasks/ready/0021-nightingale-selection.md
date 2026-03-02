# Nightingale Selection: Task 0021 — Fix `loop/` Directory Missing from Cross-Build Context

**Selected:** 2026-02-25
**Selected by:** Florence Nightingale (Requirements Curator)
**Priority:** P1
**Type:** bugfix
**Estimated size:** XS (three-line change across three shell scripts)

---

## Selection Rationale

Four candidate tasks were investigated. Two P0 candidates are already complete:

| Task | Claimed State | Verified State |
|------|---------------|----------------|
| 0009 `great apply` | "stub" | DONE — 972-line full implementation with tool mapping, dry-run, spinners, idempotent provisioning |
| 0010 GROUP J (integration tests) | "only 4 smoke tests" | DONE — 90+ test functions in `tests/cli_smoke.rs` |
| 0010 GROUP A (tool install mapping) | "8 tools need special paths" | DONE — `tool_install_spec()` at line 272 of `apply.rs` covers all 8 tools |
| 0021 (loop dir bug, P1) | "backlog" | CONFIRMED ACTIVE — all three cross-test scripts are missing the `loop/` copy step |

The 0021 (diff output channel redesign, P2) and 0022 (diff counter consistency, P2) tasks are valid but rank below a confirmed P1 build breakage.

**0021 (loop dir fix)** is the highest-priority unblocked task that is genuinely not yet done. It is a confirmed build failure: the `loop/` directory is present in the repo with 22 files that are `include_str!()`-embedded at compile time, but the cross-compilation test scripts never copy `loop/` into the writable `/build` directory. This causes all three cross-build targets to fail with 22 "No such file or directory" errors after compiling 170+ dependencies.

---

## Task Definition

### Problem

`src/cli/loop_cmd.rs` embeds 22 files from the `loop/` directory using `include_str!()` macros. The paths resolve relative to the source file: e.g. `../../loop/agents/nightingale.md`.

All three cross-compilation test scripts copy the source tree from the read-only `/workspace` mount to the writable `/build` directory before building. The copy section in each script is:

```bash
cp -r /workspace/src   /build/src
cp -r /workspace/tests /build/tests
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
cp /workspace/Cargo.toml  /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
```

`loop/` is never copied. The build compiles all 170+ dependencies successfully, then fails on the final `great-sh` crate linkage with 22 missing file errors.

### Files Affected

- `/home/isaac/src/sh.great/docker/cross-test-macos.sh` — line 25 area (copy section)
- `/home/isaac/src/sh.great/docker/cross-test-windows.sh` — line 22 area (copy section)
- `/home/isaac/src/sh.great/docker/cross-test-linux-aarch64.sh` — line 22 area (copy section)

### Fix

Add one line to each script's copy section, immediately after the `templates` conditional:

```bash
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
```

The guard `[ -d /workspace/loop ]` matches the style of the existing `templates` guard. No Dockerfile changes, no `.dockerignore` changes, no Rust source changes are needed.

---

## Acceptance Criteria

- [ ] `docker/cross-test-macos.sh` includes `[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop` in its copy section
- [ ] `docker/cross-test-windows.sh` includes `[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop` in its copy section
- [ ] `docker/cross-test-linux-aarch64.sh` includes `[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop` in its copy section
- [ ] `docker compose run --build macos-cross` completes without error and produces valid Mach-O x86_64 and aarch64 binaries
- [ ] `docker compose run --build windows-cross` completes without error and produces a valid PE32+ binary

---

## Dependencies Satisfied

None required. This is a self-contained shell-script change. The `loop/` directory already exists at the repo root with all 22 files. No new dependencies are introduced.

---

## Backlog Maintenance Notes

The following backlog tasks should be closed as already-implemented before the next iteration:

- **0009** (`great apply` command): Fully implemented. `src/cli/apply.rs` is 972 lines with dry-run, spinners, idempotent provisioning, and tool mapping. Mark done.
- **0010 GROUP J** (integration tests): `tests/cli_smoke.rs` contains 90+ test functions — well past the 12-test acceptance threshold. GROUP J is done.
- **0010 GROUP A** (tool install mapping): `tool_install_spec()` at `apply.rs:272-316` covers all 8 special-case tools (cdk, aws, az, gcloud, pnpm, uv, starship, bitwarden-cli). GROUP A is done.

Remaining open work in 0010: Groups B, C, D, E, F, G, H, I, K.
