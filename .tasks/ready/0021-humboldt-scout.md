# 0021: Humboldt Scout Report — Fix `loop/` Missing from Cross-Build Context

**Scout:** Humboldt (Codebase Scout)
**Task:** 0021 — Fix `loop/` directory missing from cross-compilation build context
**Date:** 2026-02-25

---

## Terrain Summary

Four shell scripts copy workspace source into a writable `/build` directory before
invoking `cargo build`. All four omit `loop/`. Because `src/cli/loop_cmd.rs` contains
22 `include_str!()` calls referencing files under `loop/`, every Docker build fails at
the Rust compile step after spending time compiling 170+ dependency crates.

The fix is one line per file, four total lines. No Rust source changes. No Dockerfile
changes. No docker-compose.yml changes.

---

## File Map

### File 1: `docker/cross-test-macos.sh`

**Full path:** `/home/isaac/src/sh.great/docker/cross-test-macos.sh`
**Line count:** 79 lines
**Used by:** `macos-cross` service in `docker-compose.yml` (line 54)

**Exact insertion point — after line 23, before line 24:**

```
line 23:  [ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
line 24:  cp /workspace/Cargo.toml /build/Cargo.toml     <- new line inserts HERE
```

**Line to insert:**
```bash
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
```

**Resulting copy block (lines 21-26 after edit):**
```bash
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
```

---

### File 2: `docker/cross-test-windows.sh`

**Full path:** `/home/isaac/src/sh.great/docker/cross-test-windows.sh`
**Line count:** 57 lines (58 after edit)
**Used by:** `windows-cross` service in `docker-compose.yml` (line 66)

**Exact insertion point — after line 20, before line 21:**

```
line 20:  [ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
line 21:  cp /workspace/Cargo.toml /build/Cargo.toml     <- new line inserts HERE
```

**Line to insert:**
```bash
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
```

**Resulting copy block (lines 18-23 after edit):**
```bash
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
```

---

### File 3: `docker/cross-test-linux-aarch64.sh`

**Full path:** `/home/isaac/src/sh.great/docker/cross-test-linux-aarch64.sh`
**Line count:** 57 lines (58 after edit)
**Used by:** `linux-aarch64-cross` service in `docker-compose.yml` (line 78)

**Exact insertion point — after line 20, before line 21:**

```
line 20:  [ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
line 21:  cp /workspace/Cargo.toml /build/Cargo.toml     <- new line inserts HERE
```

**Line to insert:**
```bash
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
```

**Resulting copy block (lines 18-23 after edit):**
```bash
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
```

---

### File 4: `docker/test.sh`

**Full path:** `/home/isaac/src/sh.great/docker/test.sh`
**Line count:** 46 lines (47 after edit)
**Used by:** `ubuntu` service (docker-compose.yml line 28) and `fedora` service (line 40)

**Exact insertion point — after line 18, before line 19:**

```
line 18:  cp -r /workspace/templates /build/templates
line 19:  cp /workspace/Cargo.toml /build/Cargo.toml     <- new line inserts HERE
```

**Line to insert (unconditional — matches existing test.sh style):**
```bash
cp -r /workspace/loop /build/loop
```

**Resulting copy block (lines 16-21 after edit):**
```bash
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
cp -r /workspace/templates /build/templates
cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
```

---

## The loop/ Directory

**Full path:** `/home/isaac/src/sh.great/loop/`
**Total files:** 22

```
loop/agents/davinci.md
loop/agents/dijkstra.md
loop/agents/gutenberg.md
loop/agents/hopper.md
loop/agents/humboldt.md
loop/agents/kerckhoffs.md
loop/agents/knuth.md
loop/agents/lovelace.md
loop/agents/nielsen.md
loop/agents/nightingale.md
loop/agents/rams.md
loop/agents/socrates.md
loop/agents/turing.md
loop/agents/vonbraun.md
loop/agents/wirth.md          <- 15 agent files
loop/commands/backlog.md
loop/commands/bugfix.md
loop/commands/deploy.md
loop/commands/discover.md
loop/commands/loop.md          <- 5 command files
loop/observer-template.md      <- 1 template
loop/teams-config.json         <- 1 config
```

**include_str!() count in `src/cli/loop_cmd.rs`:** 22 confirmed via `grep -c`.

Breakdown: 15 agents (lines 48-105) + 5 commands (lines 110-129) + 1 teams config
(line 133) + 1 observer template (line 136). Every file in `loop/` has a corresponding
`include_str!()` call. No orphaned files. No missing includes.

---

## docker-compose.yml Service Map

**Full path:** `/home/isaac/src/sh.great/docker-compose.yml`
**Line count:** 152 lines

| Service | Script | Dockerfile |
|---|---|---|
| `ubuntu` | `docker/test.sh` | `docker/ubuntu.Dockerfile` |
| `fedora` | `docker/test.sh` | `docker/fedora.Dockerfile` |
| `macos-cross` | `docker/cross-test-macos.sh` | `docker/cross-macos.Dockerfile` |
| `windows-cross` | `docker/cross-test-windows.sh` | `docker/cross-windows.Dockerfile` |
| `linux-aarch64-cross` | `docker/cross-test-linux-aarch64.sh` | `docker/cross-linux-aarch64.Dockerfile` |

The three VM services (`macos`, `windows`, `ubuntu-vm`) use `dockurr/macos`,
`dockurr/windows`, and `qemux/qemu` images respectively. They do not build Rust source
and are unaffected.

---

## Patterns to Follow

**Pattern 1: Conditional copy for cross-test scripts**

All three `cross-test-*.sh` scripts use `[ -d /workspace/templates ] && cp -r` for the
`templates` directory. The new `loop/` copy line must follow the same guard pattern:

```bash
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
```

**Pattern 2: Unconditional copy for test.sh**

`test.sh` uses `cp -r /workspace/templates /build/templates` with no guard. The new
`loop/` copy line must also be unconditional:

```bash
cp -r /workspace/loop /build/loop
```

**Pattern 3: Ordering within the copy block**

In all four scripts the copy order is: `src` -> `tests` -> `templates` -> (new: `loop`)
-> `Cargo.toml` -> `Cargo.lock`. The new line always goes between `templates` and
`Cargo.toml`. This is consistent with the spec.

**Pattern 4: set -euo pipefail**

All four scripts begin with `set -euo pipefail`. Any `cp` failure aborts the script
immediately. The `[ -d ... ]` guard in the cross-test scripts prevents premature abort
if `loop/` were ever absent from the workspace volume.

---

## Other docker/ Files — No Changes Needed

**Dockerfiles examined:**

- `/home/isaac/src/sh.great/docker/ubuntu.Dockerfile` — only `COPY Cargo.toml` and
  `COPY Cargo.lock*` for dependency layer caching. No source copy. Unaffected.
- `/home/isaac/src/sh.great/docker/fedora.Dockerfile` — same pattern. Unaffected.
- `/home/isaac/src/sh.great/docker/cross-macos.Dockerfile` — same pattern. Unaffected.
- `/home/isaac/src/sh.great/docker/cross-windows.Dockerfile` — same pattern. Unaffected.
- `/home/isaac/src/sh.great/docker/cross-linux-aarch64.Dockerfile` — same pattern. Unaffected.

Source files are never `COPY`'d in Dockerfiles; they are volume-mounted as
`.:/workspace:ro` at runtime. The copy into `/build` happens exclusively in the `.sh`
entrypoint scripts.

No `.dockerignore` exists at the repo root. The `loop/` directory is fully accessible
in the Docker build context and the volume mount.

---

## Dependency Map

```
loop/agents/*.md (15 files)    -\
loop/commands/*.md (5 files)    +-> include_str!() in src/cli/loop_cmd.rs
loop/teams-config.json          |   requires all 22 paths at compile time
loop/observer-template.md      -/

docker-compose.yml
  macos-cross -> cross-test-macos.sh      -> cargo build -> FAILS (missing loop/)
  windows-cross -> cross-test-windows.sh  -> cargo build -> FAILS (missing loop/)
  linux-aarch64-cross -> cross-test-linux-aarch64.sh -> cargo build -> FAILS
  ubuntu -> test.sh                        -> cargo build -> FAILS (missing loop/)
  fedora -> test.sh                        -> cargo build -> FAILS (missing loop/)
```

---

## Risks

**Risk 1 (LOW): Asymmetric failure modes**

The three cross-test scripts use a `[ -d ... ]` guard. If `loop/` were somehow absent
from the workspace mount, they would silently skip the copy and then fail with 22 Rust
compile errors. `test.sh` would abort immediately with a `cp` error. The spec explicitly
accepts this asymmetry (matching each script's existing convention). Socrates also
reviewed and accepted it as advisory-only. No action needed unless the project later
wants uniform behavior.

**Risk 2 (NONE): /build/loop directory collision**

The `/build` directory is always empty at the start of the copy step. No existing
`/build/loop` path exists. `cp -r` will create it. No collision.

**Risk 3 (NONE): Dockerfile layer cache invalidation**

The Dockerfiles do not `COPY loop/`. The shell scripts run at container start time,
not during image build. Changing the `.sh` files does not invalidate any Docker layer
cache. Docker images do not need to be rebuilt.

**Risk 4 (TECHNICAL DEBT): loop/ was added without updating test scripts**

This bug exists because `loop/` was added to the repo and `loop_cmd.rs` began embedding
its files, but the four test scripts were never updated. The `[ -d /workspace/templates ]`
conditional in the cross-test scripts was a hint that extensibility was planned — it was
simply not executed. Going forward, any new top-level directory referenced by `include_str!()`
in Rust source would require the same treatment. There is no automated check for this.

---

## Recommended Build Order

This is a one-commit change. No sequencing required.

1. Edit `docker/cross-test-macos.sh` — insert after line 23
2. Edit `docker/cross-test-windows.sh` — insert after line 20
3. Edit `docker/cross-test-linux-aarch64.sh` — insert after line 20
4. Edit `docker/test.sh` — insert after line 18
5. Commit all four files together with message:
   `fix(docker): copy loop/ dir into cross-compilation build context`

---

## Verification Commands (from spec)

```bash
# After edit, verify locally:
cargo clippy
cargo test

# Docker services (requires Docker):
docker compose run --build macos-cross
docker compose run --build windows-cross
docker compose run --build linux-aarch64-cross
docker compose up --build ubuntu
docker compose up --build fedora
```

All five commands must exit 0 and produce no "No such file or directory" errors.
