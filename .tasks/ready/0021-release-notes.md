# Release Notes: Task 0021 — Fix `loop/` Directory Missing from Cross-Compilation Build Context

**Type:** Bug Fix
**Priority:** P1
**Date:** 2026-02-25
**Commit:** `fix(docker): copy loop/ dir into cross-compilation build context`

---

## Summary

All Docker-based builds were failing with 22 Rust compile errors after spending
time compiling 170+ dependency crates. The `loop/` directory was never copied into
the writable `/build` directory before `cargo build` ran.

---

## What Changed

Four shell scripts each received one new line in their source-copy section,
inserted between the `templates` copy and the `Cargo.toml` copy:

**`docker/cross-test-macos.sh`** (line 24):
```bash
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
```

**`docker/cross-test-windows.sh`** (line 21):
```bash
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
```

**`docker/cross-test-linux-aarch64.sh`** (line 21):
```bash
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
```

**`docker/test.sh`** (line 19):
```bash
cp -r /workspace/loop /build/loop
```

No Rust source changes. No Dockerfile changes. No `docker-compose.yml` changes.

---

## Why

`src/cli/loop_cmd.rs` contains 22 `include_str!()` macros that embed files from
the `loop/` directory at compile time:

| Category         | Count | Location            |
|------------------|-------|---------------------|
| Agent personas   | 15    | `loop/agents/*.md`  |
| Slash commands   |  5    | `loop/commands/*.md`|
| Teams config     |  1    | `loop/teams-config.json` |
| Observer template|  1    | `loop/observer-template.md` |
| **Total**        | **22** |                    |

Rust resolves `include_str!()` paths relative to the source file at compile time.
If any path is absent, the compiler fails immediately with a hard error — there is
no fallback. The four test scripts copy source from the read-only `/workspace`
volume mount to a writable `/build` directory before building. When the `loop/`
directory was added to the repo and `loop_cmd.rs` began embedding its files, these
copy sections were never updated.

---

## Impact

Cross-compilation Docker builds now succeed:

| Service               | Target                                      | Status before | Status after |
|-----------------------|---------------------------------------------|---------------|--------------|
| `macos-cross`         | `x86_64-apple-darwin`, `aarch64-apple-darwin` | FAIL (22 errors) | PASS |
| `windows-cross`       | `x86_64-pc-windows-gnu`                    | FAIL (22 errors) | PASS |
| `linux-aarch64-cross` | `aarch64-unknown-linux-gnu`                | FAIL (22 errors) | PASS |
| `ubuntu`              | native x86_64 Linux                         | FAIL (22 errors) | PASS |
| `fedora`              | native x86_64 Linux                         | FAIL (22 errors) | PASS |

---

## Migration Notes

None. This is a build infrastructure fix. No user-facing behaviour changed. No
configuration changes required. No API changes.

If you have a local fork of any of the four `docker/*.sh` scripts, add the
`loop/` copy line after the `templates` copy in each script's source-copy section.
