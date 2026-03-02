# Nightingale Selection — Task 0027

**Date:** 2026-02-27
**Iteration:** 024
**Selected task:** `0027-wire-non-interactive-flag.md`
**Priority:** P2
**Complexity:** S

---

## Discovery Evidence

### Sources examined

| Source | Finding |
|--------|---------|
| `src/cli/sudo.rs:61` | Explicit TODO: `non_interactive` parameter not yet accepted |
| `src/main.rs` | `cli.non_interactive` parsed by clap but never read or forwarded |
| `src/cli/apply.rs:421` | `ensure_sudo_cached(info.is_root)` — one argument, no `non_interactive` |
| `src/cli/doctor.rs:112` | `ensure_sudo_cached(info.is_root)` — same gap |
| `src/cli/doctor.rs:97` | `available_managers(false)` — hardcoded `false`, ignores actual flag |
| `src/cli/mod.rs:35` | `pub non_interactive: bool` declared `global = true` on `Cli` |
| Iteration-023 observer report | 3 non-blocking advisories; none are P0/P1 |

### Commands run

```
grep -rn 'TODO\|FIXME\|HACK' src/
grep -rn 'non_interactive' src/
```

No compiler errors, no clippy warnings discovered in the TODO/FIXME scan — only the one
TODO in `sudo.rs` pointing directly at this gap.

---

## Why this task

The `--non-interactive` flag is a first-class contract with CI and automation users.
A flag that is parsed but has no effect is worse than no flag at all — it creates false
confidence. The gap is confirmed by evidence (TODO comment + dead parsed value in `main.rs`),
is bounded (4 files, ~20 lines of change), and has clear testable criteria.

The three iteration-023 advisories (Dockerfile comments, closing banner paths) are all P3
polish and would each be a trivial one-liner. Batching them as a single P3 task is an
option for a future iteration; they are not the highest-value next action.

---

## Alternatives considered

| Option | Reason not selected |
|--------|---------------------|
| Fix `cross-macos.Dockerfile` usage comment (Socrates advisory) | P3 one-liner; lower value than a correctness bug |
| Fix `cross-linux-aarch64.Dockerfile` usage comment (Socrates advisory) | Same — P3, one-liner |
| Fix closing banner container paths (Nielsen P3) | P3 UX polish, three lines across two scripts |
| New feature work | No P0/P1 gaps found in the codebase scan |

---

## Scope confirmation

Files touched by this task:

1. `/home/isaac/src/sh.great/src/main.rs` — extract and forward `non_interactive`
2. `/home/isaac/src/sh.great/src/cli/apply.rs` — add `non_interactive` field to `Args`; pass to `ensure_sudo_cached`
3. `/home/isaac/src/sh.great/src/cli/doctor.rs` — same; fix hardcoded `false` in `available_managers` call
4. `/home/isaac/src/sh.great/src/cli/sudo.rs` — extend signature, remove TODO, update tests
