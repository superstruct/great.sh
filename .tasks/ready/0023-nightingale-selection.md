# Selection: 0023 — Replace 'beta' with 'alpha' on marketing site and README

- **Curator:** Nightingale (Florence Nightingale, Requirements Curator)
- **Date:** 2026-02-26
- **Source task:** `.tasks/backlog/0023-replace-beta-with-alpha.md`
- **Priority:** P0
- **Type:** content
- **Complexity:** XS
- **Status:** READY — verified, unblocked, all occurrences confirmed

---

## Verification Summary

All 5 occurrences were confirmed present by live grep on 2026-02-26. No new occurrences were found beyond those listed in the task. No occurrences exist in test files, build artifacts, or `.tasks/`.

| File | Line | Current text | Verified |
|------|------|--------------|----------|
| `site/src/components/layout/Nav.tsx` | 38 | `beta` | YES |
| `site/src/components/sections/Hero.tsx` | 27 | `beta — open to testing & feedback` | YES |
| `site/src/components/sections/OpenSource.tsx` | 50 | `great.sh is in beta. Found a bug or have a suggestion?` | YES |
| `README.md` | 3 | `Beta — open to testing and feedback.` | YES |
| `README.md` | 77 | `Beta (v0.1.0). Core features work. We welcome bug reports and feedback.` | YES |

No additional beta occurrences found outside these locations in `site/src/` or `README.md`.

---

## Scope Boundaries

- Touch only the 5 occurrences listed above.
- Preserve all surrounding prose, punctuation, and JSX structure.
- Case rule: sentence-initial "Beta" -> "Alpha"; inline badge/pill "beta" -> "alpha".
- Do not touch: test files, build artifacts, `node_modules/`, `.tasks/`.

---

## Acceptance Criteria (unchanged from task)

- [ ] `grep -ri beta site/src/` returns no matches
- [ ] `grep -i beta README.md` returns no matches
- [ ] `pnpm build:site` exits 0 with no new warnings
- [ ] Nav badge reads "alpha", Hero pill reads "alpha — open to testing & feedback", OpenSource prose reads "great.sh is in alpha."
- [ ] README line 3 reads "Alpha — open to testing and feedback." and line 77 reads "Alpha (v0.1.0)."

---

## Dependencies

None. This task is unblocked and ready to assign immediately.

---

## Notes for the implementing agent

This is a pure text substitution. Five lines across four files. No logic changes, no schema changes, no build changes required. The build verification criterion (`pnpm build:site`) is a safety net, not an expectation of any build-level change.
