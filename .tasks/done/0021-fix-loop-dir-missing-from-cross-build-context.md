# 0021: Fix `loop/` Directory Missing from Cross-Compilation Build Context

**Priority:** P1
**Type:** bugfix
**Module:** `docker/cross-test-macos.sh`, `docker/cross-test-windows.sh`, `docker/cross-test-linux-aarch64.sh`
**Status:** backlog
**Created:** 2026-02-25
**Component:** docker / cross-compilation

## Context

`src/cli/loop_cmd.rs` embeds 22 files at compile time using `include_str!()` macros:

- 15 agent personas from `loop/agents/*.md`
- 5 slash-commands from `loop/commands/*.md`
- `loop/teams-config.json`
- `loop/observer-template.md`

The `include_str!()` paths are relative to the source file. From `src/cli/loop_cmd.rs` they
resolve to the repo root `loop/` directory (e.g. `../../loop/agents/nightingale.md`).

All three cross-compilation test scripts share the same copy pattern that prepares the
writable `/build` directory from the read-only `/workspace` mount:

```bash
cp -r /workspace/src   /build/src
cp -r /workspace/tests /build/tests
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
cp /workspace/Cargo.toml  /build/Cargo.toml
cp /workspace/Cargo.lock  /build/Cargo.lock
```

`loop/` is never copied. Cargo compiles 170+ dependencies successfully then fails on the
final `great-sh` crate with 22 "No such file or directory" errors — one per `include_str!`
call.

The fix is a one-line addition to each test script's copy section:

```bash
cp -r /workspace/loop /build/loop
```

No Dockerfile changes and no `.dockerignore` changes are needed — `loop/` is already
present in the workspace volume mount (`.:/workspace:ro`).

## Acceptance Criteria

- [ ] `docker compose run --build macos-cross` completes without error and produces valid Mach-O x86_64 and aarch64 binaries
- [ ] `docker compose run --build windows-cross` completes without error and produces a valid PE32+ binary
- [ ] `docker compose run --build linux-aarch64-cross` completes without error and produces a valid ELF ARM aarch64 binary
- [ ] The copy section in each of the three test scripts includes `cp -r /workspace/loop /build/loop`
- [ ] `cargo clippy` and `cargo test` pass unchanged (no Rust source changes)

## Dependencies

None. Self-contained shell-script change.

## Notes

All three cross-compilation scripts are affected identically; fix all three in a single
commit. The ubuntu and fedora services build from `/workspace` directly (not via a copy
step) and are unaffected.
