# 0021: Fix `loop/` Directory Missing from Cross-Compilation Build Context

**Priority:** P1
**Type:** bugfix
**Status:** ready
**Created:** 2026-02-25
**Spec author:** Lovelace

---

## Summary

The three cross-compilation shell scripts (`cross-test-macos.sh`, `cross-test-windows.sh`,
`cross-test-linux-aarch64.sh`) copy source files from the read-only `/workspace` volume mount
to a writable `/build` directory before invoking `cargo build`. They copy `src/`, `tests/`,
`templates/` (conditionally), `Cargo.toml`, and `Cargo.lock` -- but omit `loop/`. Because
`src/cli/loop_cmd.rs` contains 22 `include_str!()` macros that reference files under
`../../loop/` (relative to the source file, resolving to the repo-root `loop/` directory),
the build fails with 22 "No such file or directory" errors after all 170+ dependency crates
have already compiled successfully.

Additionally, `docker/test.sh` (used by the `ubuntu` and `fedora` services) has the same
omission and must also be fixed.

## Root Cause Analysis

### The include_str!() contract

Rust's `include_str!()` resolves paths relative to the file containing the macro invocation.
For `src/cli/loop_cmd.rs`, the path `../../loop/agents/nightingale.md` resolves to
`<crate-root>/loop/agents/nightingale.md`. The compiler reads these files at compile time.
If any path is missing, compilation fails immediately with a hard error -- there is no
fallback.

### The 22 embedded files

From `src/cli/loop_cmd.rs`:

| Category | Count | Path pattern (from crate root) |
|---|---|---|
| Agent personas | 15 | `loop/agents/{nightingale,lovelace,socrates,humboldt,davinci,vonbraun,turing,kerckhoffs,rams,nielsen,knuth,gutenberg,hopper,dijkstra,wirth}.md` |
| Slash commands | 5 | `loop/commands/{loop,bugfix,deploy,discover,backlog}.md` |
| Teams config | 1 | `loop/teams-config.json` |
| Observer template | 1 | `loop/observer-template.md` |
| **Total** | **22** | |

### Why the copy step misses loop/

All four test scripts share a copy pattern that was written before the `loop/` directory
existed. When `loop/` was added to the repo and `loop_cmd.rs` began embedding its files,
the test scripts were never updated. The `templates/` directory has a conditional copy in
the cross-compilation scripts (`[ -d /workspace/templates ] && cp -r ...`) showing that
the pattern was designed to be extended -- it simply was not.

### Why ubuntu/fedora are also affected

`docker/test.sh` (lines 16-20) performs the same copy pattern and also omits `loop/`.
The ubuntu and fedora services use this script. They will fail identically when run.

## Exact Changes

### File 1: `docker/cross-test-macos.sh`

**Insert after line 23** (after the `[ -d /workspace/templates ]` conditional copy,
before `cp /workspace/Cargo.toml`):

```bash
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
```

The resulting copy section (lines 21-26) will read:

```bash
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
```

### File 2: `docker/cross-test-windows.sh`

**Insert after line 20** (after the `[ -d /workspace/templates ]` conditional copy,
before `cp /workspace/Cargo.toml`):

```bash
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
```

The resulting copy section (lines 18-23) will read:

```bash
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
```

### File 3: `docker/cross-test-linux-aarch64.sh`

**Insert after line 20** (after the `[ -d /workspace/templates ]` conditional copy,
before `cp /workspace/Cargo.toml`):

```bash
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
```

The resulting copy section (lines 18-23) will read:

```bash
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
```

### File 4: `docker/test.sh`

**Insert after line 18** (after `cp -r /workspace/templates /build/templates`,
before `cp /workspace/Cargo.toml`):

```bash
cp -r /workspace/loop /build/loop
```

Note: `test.sh` uses unconditional copies for `templates` (no `[ -d ... ]` guard), so
the `loop/` copy should also be unconditional here to match the existing style. The `loop/`
directory is always present in the repo; it is a hard build dependency.

The resulting copy section (lines 16-21) will read:

```bash
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
cp -r /workspace/templates /build/templates
cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
```

## Files Modified

| File | Change |
|---|---|
| `docker/cross-test-macos.sh` | Add 1 line after line 23 |
| `docker/cross-test-windows.sh` | Add 1 line after line 20 |
| `docker/cross-test-linux-aarch64.sh` | Add 1 line after line 20 |
| `docker/test.sh` | Add 1 line after line 18 |

No Rust source changes. No Dockerfile changes. No `.dockerignore` changes. No
`docker-compose.yml` changes.

## Edge Cases

### loop/ directory absent from workspace

The three cross-compilation scripts use a `[ -d /workspace/loop ]` guard, so if
someone strips `loop/` from the volume mount, the copy is skipped and the build fails
with the original 22 compile errors. This is the correct behavior -- without the source
files, the build cannot succeed, and Rust's error messages clearly identify the missing
paths.

### Partial loop/ directory (missing some .md files)

If `loop/` exists but is incomplete, `cp -r` copies whatever is present and the
build fails with compile errors only for the specific missing files. No data corruption
risk -- `include_str!()` is read-only at compile time.

### Symlinks inside loop/

`cp -r` follows symlinks by default on Linux. The `loop/` directory contains only
regular files (15 + 5 markdown files, 1 JSON, 1 markdown template). No symlinks
exist, so this is a non-issue.

### Platform differences

All four scripts run inside Linux containers regardless of the host OS. `cp -r` and
`[ -d ... ]` are POSIX-standard and behave identically across all container base
images used (Ubuntu, Fedora, Debian).

## Error Handling

No new error handling is required. The change is a `cp` command inside a `set -euo pipefail`
script. If the copy fails (e.g., permission error, disk full), the script aborts immediately
with a non-zero exit code. The `[ -d ... ]` guard prevents failure when the directory is
absent.

## Security Considerations

None. The change copies read-only source files within an isolated Docker container. No
secrets, credentials, or network access are involved. The files being copied (markdown and
JSON) are checked into the public repository.

## Testing Strategy

### Manual verification (required)

Each cross-compilation target must be tested:

```bash
# macOS (builds x86_64 + aarch64 Mach-O binaries)
docker compose run --build macos-cross

# Windows (builds x86_64 PE32+ binary)
docker compose run --build windows-cross

# Linux ARM64 (builds aarch64 ELF binary)
docker compose run --build linux-aarch64-cross

# Ubuntu native (builds + runs test suite)
docker compose up --build ubuntu

# Fedora native (builds + runs test suite)
docker compose up --build fedora
```

Each command must:
1. Complete without error (exit code 0)
2. Produce the expected binary format (validated by `file` command inside the script)
3. Show no "No such file or directory" errors in the build output

### Verification that no Rust changes are needed

```bash
cargo clippy
cargo test
```

Both must pass with zero changes to Rust source files.

### Smoke test for include_str!() correctness

After a successful cross-build, the binary already validates via `file` output matching.
For the ubuntu/fedora services, the test script also runs `great --version` and
`great --help` as smoke tests, which exercise the compiled binary (including the
embedded `loop/` content).

## Implementation Notes

- All four files should be changed in a single commit.
- The commit message should reference this task: e.g.,
  `fix(docker): copy loop/ dir into cross-compilation build context`
- No changes to the GitHub Actions release workflow are needed; it builds natively
  on the runner, not via these Docker scripts.

## Risk Assessment

**Risk: Minimal.**

- The change adds one `cp` line per script. The pattern is identical to the existing
  `templates` copy.
- No Rust code is modified.
- No Docker images are rebuilt (only the entrypoint scripts change).
- If the line is wrong, the failure mode is the same as today (compile error) -- no
  silent data corruption or security regression.
- The `[ -d ... ]` guard in the cross-compilation scripts makes the copy safe even if
  `loop/` were ever removed from the repo.
