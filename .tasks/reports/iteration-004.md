# Observer Report — Iteration 004

**Date:** 2026-02-24
**Observer:** W. Edwards Deming
**Task:** 0013 — Fix `statusLine` Schema Written by `great loop install`

## Summary

Bugfix: `great loop install` wrote `statusLine` objects missing the required `"type": "command"` discriminator. Claude Code's validator rejected the entire `settings.json`, silently disabling all user settings. Fixed both write paths, added repair logic for existing broken installs, extracted a helper to eliminate the duplication that caused the bug.

## Changes Committed

**Commit:** `277dcc7` — `fix(cli): add missing "type" discriminator to statusLine in settings.json`

| File | Change |
|------|--------|
| `src/cli/loop_cmd.rs` | `statusline_value()` helper, fix 2 JSON literals, repair logic, 4 unit tests |

## Agent Performance

| Agent | Role | Retries | Result |
|-------|------|---------|--------|
| Nightingale | Requirements | 0 | PASS — selected 0013 (P1 bugfix, standalone, high user impact) |
| Lovelace | Spec | 0 | PASS — clean spec, 4 changes specified |
| Socrates | Review gate | 0 | APPROVED — 4 advisory notes, 0 blocking |
| Humboldt | Scout | 0 | PASS — confirmed single-file scope, live broken state on disk |
| Da Vinci | Build | 1 | PASS — R1 built cleanly; R2 replaced 2 tautological tests after Dijkstra |
| Turing | Test | 0 | PASS — 220/220 tests (163 unit + 57 integration), clippy clean |
| Kerckhoffs | Security | 0 | PASS — 0 CRITICAL/HIGH, 2 LOW (TOCTOU benign, non-atomic write) |
| Wirth | Performance | 0 | PASS — no regression, 9.98 MiB baseline recorded |
| Dijkstra | Code review | 1 | R1: REJECT (2 tautological tests), R2: APPROVE (2 WARN advisory) |
| Nielsen | UX | N/A | Skipped — no UI change (CLI install path only) |
| Rams | Visual | N/A | Skipped — no visual component |
| Hopper | Commit | 0 | Committed 277dcc7 |
| Knuth/Gutenberg | Docs | N/A | No doc changes needed for internal bugfix |

## Dijkstra Review Loop

- Round 1: REJECTED — `test_broken_statusline_detected` was tautological (tested test setup, not production code); `test_correct_statusline_not_detected_as_broken` was circular (tested function against itself)
- Round 2: APPROVED — replaced with `test_repair_fixes_broken_statusline` (exercises actual repair decision logic, asserts needs_write, verifies repaired shape and key survival) and `test_correct_statusline_skips_repair` (exercises idempotency path)
- **2 rounds total** (max 2 allowed)

## Bottleneck

**Dijkstra caught real test quality issues.** The Lovelace spec proposed 4 tests but 2 were tautological — they would pass regardless of whether the production code was correct. Dijkstra's review caught this before commit. The fix cycle added ~20 seconds of build time.

**Root cause analysis:** The spec writer (Lovelace) generated tests that verified JSON structure properties rather than the repair behavior. This is a pattern risk for specs that describe data fixes — the tests naturally gravitate toward asserting data shapes rather than exercising the decision logic that operates on data.

## Metrics

- **Files changed:** 1 (`src/cli/loop_cmd.rs`)
- **Lines added:** 132 (helper + repair logic + 4 tests)
- **Lines removed:** 9 (old broken JSON literals + simple guard)
- **Tests added:** 4 new unit tests
- **Tests total:** 220 (163 unit + 57 integration)
- **Agent retries:** 1 (Dijkstra test quality), 0 (build/security/performance)
- **Blocking issues:** 0
- **Non-blocking issues:** 2 Dijkstra WARN (test duplication maintenance hazard, borrow coupling)
- **Build status:** GREEN

## Config Change

**None.** The Dijkstra review loop functioned as designed — it caught the exact category of issue it exists to catch (test quality). The 1-retry cost was minimal. The Lovelace spec pattern of generating tautological tests for data-shape fixes is noted for observation in the next iteration. If it recurs, consider adding a "test exercises production logic, not test setup" checkpoint to the spec template.

## Previous Change Assessment

Iterations 001-003 made no config changes. The pipeline remains stable across 4 iterations with a consistent pattern: spec-phase and review-phase catches prevent build-phase waste. The Dijkstra gate continues to justify its position — this is the second iteration (after 001's Socrates catch) where a review gate caught a real issue before commit.
