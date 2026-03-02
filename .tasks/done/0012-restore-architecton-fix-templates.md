# 0012: Restore architecton.ai Footer Link + Fix Templates Section Positioning

**Priority:** P0 (correction — misrepresentation and missing brand link)
**Type:** bugfix
**Module:** `site/src/components/layout/Footer.tsx`, `site/src/components/sections/Templates.tsx`, `site/src/data/templates.ts`
**Status:** backlog
**Estimated Complexity:** Small

## Context

Two errors were introduced or surfaced during the iteration 001 site build.

**Error 1 — architecton.ai removed from Footer in error.**
The Footer previously linked to architecton.ai, the paid template marketplace for great.sh. That link was removed during the 0011 site build on the grounds that it was "inconsistent with project branding." That call was wrong. architecton.ai is the commercial partner/marketplace for great.sh templates and is a deliberate part of the product model. The Footer must restore this link.

The current Footer (`site/src/components/layout/Footer.tsx`) has:
- GitHub link
- "Open source · Apache-2.0 License" text
- "Built by Superstruct" attribution

architecton.ai should appear as a named link in the footer navigation row (alongside GitHub), labelled clearly (e.g., "Templates via architecton.ai" or simply "architecton.ai").

**Error 2 — Templates section misrepresents templates as free/included.**
The current `site/src/components/sections/Templates.tsx` heading reads "Start with a template" with the subheading "Curated environment configs encoding best-practice AI dev setups. Use as-is or customize." The data in `site/src/data/templates.ts` lists five production-quality templates (AI Full Stack TS, AI Full Stack Python, AI Data Science, AI DevOps, AI Minimal) as if they are bundled with the CLI.

In reality:
- The only templates shipped inside this repo are bare test/example templates (bare loop + install).
- Production templates (like the five listed) are paid offerings via architecton.ai or third parties.

The section must not imply these templates are free or included. Options include:
- Replacing the data file entries with the actual included bare templates, and pointing users to architecton.ai for the full catalogue.
- Retaining the section as a "preview" of what is available via architecton.ai, with clear attribution and a CTA link.

The chosen approach should not mislead a visitor into believing they get these templates for free with `great install`.

## Current State

| File | Current Problem |
|------|----------------|
| `site/src/components/layout/Footer.tsx` | architecton.ai link absent entirely |
| `site/src/components/sections/Templates.tsx` | Heading/subheading implies templates are free and included; no architecton.ai mention |
| `site/src/data/templates.ts` | Lists 5 paid/marketplace templates as if bundled with the CLI |

## Acceptance Criteria

- [ ] `Footer.tsx` contains a visible link to `https://architecton.ai` in the footer navigation row, with legible label text (e.g., "architecton.ai" or "Templates"), styled consistently with the existing GitHub link.
- [ ] `Templates.tsx` heading or subheading copy makes clear that the listed templates are available via architecton.ai (not bundled free), and includes a CTA link to `https://architecton.ai`.
- [ ] No text on the page states or implies that production-quality environment templates are free or included in the open-source CLI install.
- [ ] `pnpm build:site` passes with zero TypeScript errors after both changes.
- [ ] Visual review confirms the Footer link and Templates section CTA are visible and legible on the dark background (`#0a0a0a`), using accent green `#22c55e` or standard link hover treatment.

## Files That Need to Change

- `/home/isaac/src/sh.great/site/src/components/layout/Footer.tsx` — add architecton.ai link to nav row
- `/home/isaac/src/sh.great/site/src/components/sections/Templates.tsx` — update heading, subheading, add architecton.ai CTA
- `/home/isaac/src/sh.great/site/src/data/templates.ts` — either replace entries with the actual included bare templates, or add a `paid: true` / `source: 'architecton.ai'` marker and update the component to display it

## Dependencies

- None. These are self-contained copy and link changes.

## Out of Scope

- Adding a full architecton.ai marketplace integration or dynamic template fetching.
- Changing the template card component design beyond copy and attribution.
- Modifying the `great template` CLI subcommand behaviour.
