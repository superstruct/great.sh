# Task 0027 — Design Review: --non-interactive flag wiring

**Reviewer:** Dieter Rams
**Date:** 2026-02-27
**Files reviewed:**
- `src/cli/sudo.rs`
- `src/cli/apply.rs`
- `src/cli/doctor.rs`
- `src/main.rs`

---

## Verdict: APPROVED

---

## Review

This is a plumbing change — four files, one boolean threaded through a dispatch
chain. Reviewed against the ten principles where applicable to code form.

### Principle 8 — Thorough (every detail considered)

**Doc comment phrasing is consistent across both Args structs.**

`apply.rs` lines 368-371:
```rust
/// Set by main.rs from the global --non-interactive flag.
/// Not a CLI argument -- hidden from clap.
#[arg(skip)]
pub non_interactive: bool,
```

`doctor.rs` lines 16-19:
```rust
/// Set by main.rs from the global --non-interactive flag.
/// Not a CLI argument -- hidden from clap.
#[arg(skip)]
pub non_interactive: bool,
```

Comment text, attribute order, and field declaration are identical. No drift.

**`#[arg(skip)]` pattern is the correct mechanism** for fields that must exist
on the struct but must not appear in the CLI help text. Both structs use it
consistently.

**Field placement.** In `apply.rs`, `non_interactive` is appended after `yes`
(the last user-facing field). In `doctor.rs`, it follows `fix`. In both cases
the skipped field sits at the end of the struct, separated from user-visible
fields by the doc comment explaining why it is hidden. This is the right order
— public surface first, internal plumbing last.

### Principle 10 — As little design as possible

**`main.rs` dispatch blocks are minimal and symmetric.**

```rust
Command::Apply(mut args) => {
    args.non_interactive = non_interactive;
    cli::apply::run(args)
}
// ...
Command::Doctor(mut args) => {
    args.non_interactive = non_interactive;
    cli::doctor::run(args)
}
```

Two lines per block, no intermediate variables, no helper function. The pattern
is the same in both cases — adding one would require the other. Commands that
do not consume `non_interactive` (Init, Status, Sync, Vault, Mcp, Update, Diff,
Template, Loop, Statusline) remain as single-expression arms. The asymmetry
between single-expression and block arms is justified, not decorative.

### Principle 6 — Honest

`sudo.rs::ensure_sudo_cached` accepts `non_interactive: bool` and checks it
before the TTY test on line 69:

```rust
if non_interactive || !std::io::stdin().is_terminal() {
    return SudoCacheResult::NonInteractive;
}
```

The ordering is correct: an explicit flag takes precedence over TTY detection.
The function is honest about what it does — both paths return the same variant,
and the variant name `NonInteractive` accurately describes the condition without
implying error.

### Minor observations (non-blocking)

1. `sudo.rs` doc comment uses `--` (double hyphen) for the em-dash position in
   "Wrong password 3 times" comment on line 41: `can continue -- individual
   sudo calls`. This is pre-existing style in the file, not introduced by this
   task. Consistent with codebase norm.

2. `apply.rs` line 368 comment also uses `--` rather than an em-dash: `Not a
   CLI argument -- hidden from clap.` Same as `doctor.rs`. Both structs are
   identical in this regard — consistency is maintained even if the typographic
   choice is informal.

Neither item warrants a correction request. The change is tight, internally
consistent, and adds no unnecessary surface.

---

*Less, but better.*
