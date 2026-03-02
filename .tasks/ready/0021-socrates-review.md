# 0021: Socrates Review -- Fix `loop/` Directory Missing from Cross-Build Context

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Spec:** `.tasks/ready/0021-cross-build-loop-dir-spec.md`
**Date:** 2026-02-25

---

## VERDICT: APPROVED

---

## Elenchus Summary

I examined five lines of questioning against the spec. The spec is thorough,
accurate, and implementable without further clarification. No blocking concerns
were found. Two advisory observations are noted below.

---

## Line 1: Are the insertion points correct?

I verified every line number in the spec against the actual file contents.

| File | Spec says insert after | Actual content at that line | Next line (before which we insert) | Correct? |
|---|---|---|---|---|
| `docker/cross-test-macos.sh` | line 23 | `[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates` | line 24: `cp /workspace/Cargo.toml /build/Cargo.toml` | YES |
| `docker/cross-test-windows.sh` | line 20 | `[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates` | line 21: `cp /workspace/Cargo.toml /build/Cargo.toml` | YES |
| `docker/cross-test-linux-aarch64.sh` | line 20 | `[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates` | line 21: `cp /workspace/Cargo.toml /build/Cargo.toml` | YES |
| `docker/test.sh` | line 18 | `cp -r /workspace/templates /build/templates` | line 19: `cp /workspace/Cargo.toml /build/Cargo.toml` | YES |

All four insertion points are precise and verified.

## Line 2: Is the conditional guard pattern consistent?

The spec uses `[ -d /workspace/loop ] && cp -r ...` for the three cross-test
scripts and an unconditional `cp -r` for `test.sh`. I examined the existing
patterns:

- **cross-test-*.sh**: `templates` uses `[ -d /workspace/templates ] && cp -r ...`
  (conditional). The spec mirrors this for `loop/`. Consistent.
- **test.sh**: `templates` uses `cp -r /workspace/templates /build/templates`
  (unconditional). The spec mirrors this for `loop/`. Consistent.

The spec also explains the rationale (section "Exact Changes", File 4 note):
`test.sh` uses unconditional copies for `templates`, so `loop/` should also be
unconditional to match the existing style. This is sound reasoning -- the style
difference between scripts is preserved rather than homogenized.

## Line 3: Are there other Docker scripts that copy source?

I enumerated all files under `docker/`:

```
docker/ubuntu.Dockerfile
docker/fedora.Dockerfile
docker/cross-macos.Dockerfile
docker/cross-windows.Dockerfile
docker/cross-linux-aarch64.Dockerfile
docker/test.sh
docker/cross-test-macos.sh
docker/cross-test-windows.sh
docker/cross-test-linux-aarch64.sh
```

The three Dockerfiles that use `COPY` (`ubuntu.Dockerfile`, `fedora.Dockerfile`,
`cross-macos.Dockerfile`) only copy `Cargo.toml` and `Cargo.lock*` for
dependency pre-fetching -- they do not copy source code. Source copying happens
exclusively in the four `.sh` scripts, all of which the spec addresses.

The GitHub Actions release workflow was also checked -- it does not use these
Docker scripts at all (builds natively on the runner).

No missing scripts. The spec's coverage is complete.

## Line 4: Could this fix break anything?

I examined three failure scenarios:

1. **`loop/` absent**: The `[ -d ... ]` guard in cross-test scripts skips the
   copy silently; Rust then fails with 22 clear compile errors. In `test.sh`,
   the unconditional `cp -r` would fail under `set -euo pipefail`, aborting
   immediately. This matches the existing `templates` behavior in `test.sh` --
   if `templates/` were missing, that script would also abort. Acceptable:
   `loop/` is always present in the repo (22 committed files confirmed).

2. **Disk space**: The `loop/` directory contains 22 text files (markdown + 1
   JSON). Negligible disk impact in a container that already compiles 170+ Rust
   crates.

3. **Filename collision**: No `/build/loop` directory exists before the copy
   (the `/build` directory starts empty). No collision risk.

No breakage scenario identified.

## Line 5: Is the acceptance criteria testable?

The spec provides exact `docker compose` commands for all five services. Each
produces observable pass/fail output (exit code + binary format validation).
The "22 files" count (15 agents + 5 commands + 1 JSON + 1 template) was
independently verified against `src/cli/loop_cmd.rs`.

The spec also correctly notes that `cargo clippy` and `cargo test` on the host
must pass unchanged, confirming no Rust source modifications were introduced.

Testable and measurable.

---

## Concerns

```
{
  "gap": "Backlog task 0021 states 'ubuntu and fedora services build from
          /workspace directly (not via a copy step) and are unaffected' but
          the spec correctly contradicts this, showing test.sh does copy.
          The backlog task should be updated or marked superseded to avoid
          future confusion.",
  "question": "Will the backlog task be updated to reflect that ubuntu/fedora
               ARE affected via test.sh?",
  "severity": "ADVISORY",
  "recommendation": "After implementation, update or close the backlog task
                     noting that the spec expanded scope to include test.sh."
}
```

```
{
  "gap": "The spec uses conditional guard for loop/ in cross-test scripts but
          unconditional in test.sh. If a future contributor removes loop/ from
          the repo, test.sh would hard-fail while cross-test scripts would
          silently skip the copy and then fail later with Rust compile errors.
          These are different failure modes for the same root cause.",
  "question": "Is the asymmetric failure mode (immediate abort in test.sh vs
               deferred 22-error Rust failure in cross-test scripts) acceptable,
               or should all four scripts use unconditional copy since loop/ is
               a hard build dependency?",
  "severity": "ADVISORY",
  "recommendation": "The spec's approach of matching each script's existing
                     convention is defensible. However, a comment in each
                     cross-test script noting that loop/ is a hard build
                     dependency (unlike templates/) could prevent future
                     confusion. This is a style preference, not a correctness
                     issue."
}
```

---

## Verified Facts

- **22 `include_str!()` macros** in `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`
  (lines 48-136): 15 agents + 5 commands + 1 JSON + 1 template. Confirmed.
- **22 files** exist under `/home/isaac/src/sh.great/loop/`: 15 in `agents/`,
  5 in `commands/`, plus `teams-config.json` and `observer-template.md`. Confirmed.
- **No `.dockerignore`** at the repo root. The `loop/` directory is available
  to Docker build context without any exclusion. Confirmed.
- **No other copy scripts** exist beyond the four identified. Confirmed.
- **No GitHub Actions workflows** reference these Docker scripts. Confirmed.

---

## Summary

The spec is precise, complete, internally consistent, and ready for
implementation -- a well-scoped one-line-per-file bugfix with verified insertion
points and no blocking concerns.
