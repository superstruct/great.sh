# Socrates Review: Spec 0023 — Replace "beta" with "alpha"

**Round:** 1
**Verdict:** APPROVED

---

## Elenchus Record

### Line-number verification (all five occurrences)

Each claimed file and line was read directly from disk. Results:

| # | File | Spec line | Actual line | Content match |
|---|------|-----------|-------------|---------------|
| 1 | `site/src/components/layout/Nav.tsx` | 38 | 38 | `            beta` — CONFIRMED |
| 2 | `site/src/components/sections/Hero.tsx` | 27 | 27 | `            beta — open to testing &amp; feedback` — CONFIRMED |
| 3 | `site/src/components/sections/OpenSource.tsx` | 50 | 50 | `            great.sh is in beta. Found a bug or have a suggestion?{' '}` — CONFIRMED |
| 4 | `README.md` | 3 | 3 | `**The managed AI dev environment.** Beta — open to testing and feedback.` — CONFIRMED |
| 5 | `README.md` | 77 | 77 | `Beta (v0.1.0). Core features work. We welcome bug reports and feedback.` — CONFIRMED |

All five line numbers match exactly. All five replacement strings are verbatim correct.

### Exhaustive search for other "beta" occurrences

Full `[Bb]eta` grep across `site/`, `README.md`, and `src/`:

**site/src/**: exactly the three component lines enumerated in the spec. No other occurrences in `site/src/data/`, `site/src/hooks/`, or any other site file.

**README.md**: exactly lines 3 and 77. No other occurrences.

**src/ (Rust source — do NOT change):** Six occurrences, all technical identifiers or test fixtures:
- `src/cli/apply.rs:960` — env var value `"beta"` in a test fixture
- `src/cli/apply.rs:962` — test assertion `"alpha and beta"` against that fixture
- `src/mcp/mod.rs:231` — test MCP server name `"beta"`
- `src/mcp/mod.rs:237` — test assertion against that server name
- `src/platform/runtime.rs:169` — doc comment, semver pre-release suffix `"2.0.0-beta"`
- `src/platform/runtime.rs:310` — test case for semver pre-release suffix

None of these are user-facing release-stage labels. The spec correctly excludes them and does not name them. The Humboldt scout report (`0023-humboldt-scout.md`) explicitly documents each one in a do-not-change table — the implementation agent has the context needed to avoid touching them.

### Capitalization correctness

- Nav.tsx line 38: inline badge — lowercase `beta` -> lowercase `alpha`. CORRECT.
- Hero.tsx line 27: inline pill — lowercase `beta` -> lowercase `alpha`. CORRECT.
- OpenSource.tsx line 50: inline sentence — lowercase `beta` -> lowercase `alpha`. CORRECT.
- README.md line 3: sentence-initial — capital `Beta` -> capital `Alpha`. CORRECT.
- README.md line 77: sentence-initial — capital `Beta` -> capital `Alpha`. CORRECT.

The spec's capitalization rule is correctly stated and consistently applied across all five instances.

### HTML entity preservation

Hero.tsx line 27 contains `&amp;` which must not be modified. The spec explicitly notes this. The replacement is word-scoped (`beta` -> `alpha`), so the entity is unaffected.

### JSX trailing expression preservation

OpenSource.tsx line 50 ends with `{' '}`. The spec explicitly notes this. The replacement does not touch it.

### Build risk

- No import, prop, type, constant, CSS class name, or `data-testid` in any file references "beta" as an identifier.
- The three changed TSX files contain only display text. TypeScript type-checking is unaffected.
- `pnpm build:site` will pass.

### Acceptance criteria completeness

The grep command in Testing Strategy step 3 (`grep -ri "beta" site/src/components/ README.md`) covers all four files. The acceptance-criteria checkbox on line 99 uses an explicit file list that includes both README.md occurrences. The backlog's own acceptance criteria (`grep -ri beta site/src/` and `grep -i beta README.md`) are fully satisfied by the spec's criterion. No gap.

---

## Concerns

None. No BLOCKING or ADVISORY concerns identified.

---

## Summary

All five replacements have been independently verified against disk; line numbers, replacement strings, and capitalization rules are exact; no additional "beta" occurrences exist in `site/src/` or `README.md`; Rust source occurrences are correctly excluded as technical identifiers; and the site build is unaffected. This spec is implementable without any further clarification.
