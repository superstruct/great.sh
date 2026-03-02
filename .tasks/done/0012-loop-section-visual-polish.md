# 0012: Loop Section Visual Polish

**Priority:** P3
**Created:** 2026-02-21
**Type:** ui
**Module:** `site/`
**Status:** backlog

## Problem

Pre-existing visual issues in the Loop section identified during 0011 review. These were NOT introduced by 0011 (content accuracy fix) but should be addressed.

## Issues

### 1. Phase 3 card wrapping at desktop
Phase 3 has 6 agents. Cards wrap to two rows at 1280px container width. The second row may appear visually incomplete.

### 2. Double-hyphen phase labels
Phase labels use `--` (ASCII) while the heading uses `—` (em dash). Typographic inconsistency.

### 3. Mobile flow connectors
At 375px, arrow (`→`) and plus (`+`) connectors sit beside stacked cards rather than between them. The sequential vs parallel distinction is lost.

## Acceptance Criteria

- [ ] Phase labels use em dash (`—`) consistently
- [ ] Mobile layout hides or repositions flow connectors for stacked cards
- [ ] Phase 3 cards display fully at all desktop widths

## Notes

- Filed from Rams (visual review) during 0011 loop iteration
- All issues are pre-existing, not regressions
