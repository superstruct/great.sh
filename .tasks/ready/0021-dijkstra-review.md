# 0021: Dijkstra Code Review — Fix `loop/` Directory Missing from Cross-Build Context

**Reviewer:** Edsger Dijkstra (Code Reviewer)
**Files reviewed:**
- `docker/cross-test-macos.sh`
- `docker/cross-test-windows.sh`
- `docker/cross-test-linux-aarch64.sh`
- `docker/test.sh`

**Date:** 2026-02-25

---

```
VERDICT: APPROVED

Issues:
- (none)

Summary: Four minimal, correctly placed, pattern-consistent insertions that resolve
         a hard compile-time dependency omission with no extraneous changes.
```

---

## Verification Record

### Placement

Each new line was verified against the spec insertion points:

| File | Spec: insert after | New line at | Before | Correct? |
|---|---|---|---|---|
| `docker/cross-test-macos.sh` | line 23 (`templates` conditional) | line 24 | `cp /workspace/Cargo.toml` | YES |
| `docker/cross-test-windows.sh` | line 20 (`templates` conditional) | line 21 | `cp /workspace/Cargo.toml` | YES |
| `docker/cross-test-linux-aarch64.sh` | line 20 (`templates` conditional) | line 21 | `cp /workspace/Cargo.toml` | YES |
| `docker/test.sh` | line 18 (`templates` unconditional) | line 19 | `cp /workspace/Cargo.toml` | YES |

### Pattern consistency

The three cross-compilation scripts guard both the `templates` and `loop` copies
with `[ -d /workspace/X ]`. The new `loop` line matches this convention exactly:

```bash
# docker/cross-test-macos.sh lines 23-25 (representative)
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
```

`docker/test.sh` uses unconditional copies for `templates`. The new `loop` line
is also unconditional, matching the established style of that script:

```bash
# docker/test.sh lines 18-20
cp -r /workspace/templates /build/templates
cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
```

### Error handling

All four scripts operate under `set -euo pipefail`. No new error handling is
needed: a failed `cp` aborts the script with a non-zero exit code. The
`[ -d ... ]` guards in the cross-test scripts prevent false failure when the
directory is absent from the mount, consistent with the `templates` guard already
present.

### Abstraction and complexity

Each change is a single `cp` invocation. No new logic, no new variables, no
conditional branches beyond the `[ -d ... ]` guard that the existing template
copy already established. Cyclomatic complexity is unchanged.

### Naming

The path `loop/` is consistent with the directory name in the repository root and
with the `templates/` copy it follows. No naming concerns.

### Extraneous changes

None. The diff is four insertions, zero deletions, confined to the copy sections
of the four files. No surrounding lines were modified, no whitespace was altered,
no comments were added or removed beyond what the task requires.

---

## Scope note (advisory, not blocking)

Socrates raised a valid advisory: `test.sh` uses an unconditional copy while the
cross-test scripts use a conditional guard. This asymmetry already existed for
`templates/` and is preserved faithfully here. The behavior difference (immediate
abort in `test.sh` vs deferred Rust compile error in cross-test scripts if `loop/`
is absent) is a pre-existing stylistic divergence between these scripts, not
introduced by this change. It does not block approval.
