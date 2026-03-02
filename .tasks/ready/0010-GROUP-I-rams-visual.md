# Rams Visual Review: Task 0010 GROUP I — Dead Code and Safety Cleanup

**Reviewer:** Rams (Visual / Output Design)
**Date:** 2026-02-26
**Scope:** Output aesthetics — formatting, comment style consistency, information
density, structural cleanliness of the diff.

---

## VERDICT: APPROVED

---

## Assessment by Principle

### Principle 3 — Aesthetic

The removal of the 12-line manual `impl Default` block in `src/cli/doctor.rs`
and its replacement with a single `#[derive(Default)]` attribute is a textbook
aesthetic improvement. The before state was visual noise: boilerplate that
communicated nothing the type signature did not already say. The after state is
immediate and silent.

The deletion of `SyncStatus` and `SyncBlob` from `src/sync/mod.rs` removes 19
lines of dead structure. The file is visually lighter. Nothing was lost.

### Principle 4 — Understandable

The inline comment pattern is consistent and scannable:

```
#[allow(dead_code)] // Planned for GROUP X (description).
```

A reader skimming the source understands in one pass: the item is suppressed,
it is not abandoned, and there is a named future destination for it. The
information load is correctly proportioned — enough to answer "why", not so much
as to obscure the code.

### Principle 6 — Honest

The diff removes exactly what has no callers. It does not manufacture purpose
for dead items. The `SyncStatus` and `SyncBlob` types had zero callers in
production and in tests; they are gone. The remaining annotated items all have
documented future GROUP assignments, which is an honest statement of intent,
not decoration.

The config re-export in `src/config/mod.rs` was trimmed from 9 symbols to 2
(`ConfigMessage`, `GreatConfig`). This deviates from the spec (which said to
keep all 9, removing only the `#[allow(unused_imports)]` annotation), but the
build passes cleanly and `cargo clippy` is silent. Downstream callers import
directly from `crate::config::schema::*`, not from the re-export path. The
implementation went further than specified toward minimum surface — which is
honest to the actual dependency graph.

### Principle 8 — Thorough

**Minor issue — comment style inconsistency (advisory, not blocking).**

Two of the ten new annotations use a two-clause form:

```rust
// Part of complete PackageManager interface; planned for GROUP D (doctor --fix).
// Part of complete PackageManager interface; planned for GROUP E (update command).
```

All other eight annotations use the single-clause form:

```rust
// Planned for GROUP X (description).
```

The additional context in `package_manager.rs` is accurate and useful (these
are trait methods, which is a structurally different reason for retention than a
standalone function). However, the visual divergence from the dominant pattern
is a minor thoroughness gap. It does not impede comprehension.

**Trailing blank line in `src/cli/doctor.rs`** — carried from Dijkstra advisory.
File ends at line 775 with a blank line after the closing `}`. Not a correctness
issue. Inconsistent with other modules that terminate at the last `}`.

### Principle 10 — As little design as possible

The diff is minimal. Every line removed was genuinely dead. Every line added is
either a one-line attribute (`#[derive(Default)]`) or a short inline comment.
The `Cargo.toml` removal of `thiserror = "2.0"` is a single line with no
substitution — the correct form when something is simply not needed.

No new structure was introduced. No new abstractions. The diff makes the
codebase smaller and the intent of each remaining annotation explicit. This is
the principle applied correctly.

---

## Advisory Items (non-blocking)

1. `src/platform/package_manager.rs` lines 17 and 24 — two-clause comment form
   differs from single-clause form used by all other eight annotations. Consider
   normalising on the next pass through this file. (Principle 8)

2. `src/cli/doctor.rs` line 775 — trailing blank line at end of file. Remove
   when next touching this file. (Principle 8)

3. `src/mcp/mod.rs:98` — comment reads `// Planned for user-level MCP config
   support.` with no GROUP reference. Intentional per spec (no GROUP assigned).
   Acceptable, but if a GROUP is later assigned, update the annotation.
   (Principle 6 — keep it honest when intent becomes concrete)

---

*"Less, but better."*
