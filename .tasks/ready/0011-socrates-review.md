# 0011: Socrates Review -- Update Site for great loop

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-20
**Spec:** `.tasks/ready/0011-update-site-great-loop-spec.md`
**Task:** `.tasks/backlog/0011-update-site-great-loop.md`

---

## Verdict: PASS (with non-blocking notes)

The spec is thorough, implementable, and accurate against the source files. All TypeScript types match, imports resolve, component props align, and the agent data is correct against `loop_cmd.rs`. The build order ensures a green build at every step. I found no blocking issues.

---

## Issues Found

### BLOCKING: None

---

### NON-BLOCKING

#### NB-1: Phase structure in Loop.tsx diverges from loop.md (cosmetic, not incorrect)

**What:** The spec's `Loop.tsx` collapses the 5-phase model from `loop/commands/loop.md` into 3 visual phases:
- Phase 1 Sequential: Nightingale, Lovelace, Socrates, Humboldt
- Phase 2 Parallel: Da Vinci, Turing, Kerckhoffs, Nielsen
- Phase 3 Finish: Rams, Hopper, Knuth, Gutenberg, Deming

The actual `loop.md` has: Phase 1 (sequential), Phase 2 (parallel team), Phase 3 (gate check -- no agents, just review), Phase 4 (finish: Rams, Hopper, Knuth+Gutenberg), Phase 5 (clean up + observer).

**Why it matters:** A marketing page simplification is fine, but the spec puts Deming (observer) as the last agent in "Phase 3 -- Finish", while in reality Deming is the team lead running the whole loop (not a finish-phase agent). This is a minor conceptual inaccuracy in the marketing representation.

**Suggested fix:** Optional. Could label Deming as "Observer (team lead)" instead of just "Observe" to avoid implying Deming only acts at the end. Or leave as-is -- marketing simplification is acceptable.

#### NB-2: Von Braun is missing from Loop.tsx phases

**What:** The AGENTS array in `loop_cmd.rs` has 13 agents: nightingale, lovelace, socrates, humboldt, davinci, **vonbraun**, turing, kerckhoffs, rams, nielsen, knuth, gutenberg, hopper. The spec's `phases` array in `Loop.tsx` lists only 13 names total (4 + 4 + 5) but includes Deming (who is not one of the 13 agents -- Deming is the team lead). Von Braun (deploy) is absent from all three phases.

**Why it matters:** The spec says "All 13 agents plus Deming (observer, phase 3) are shown -- 14 names total, matching the '14 roles' in the install output." This is correct in intent (14 roles shown), but Von Braun is one of the 13 installed agents and is completely omitted. A user who runs `great loop status` will see Von Braun listed but find no mention on the marketing page.

**Suggested fix:** Add Von Braun to Phase 3 (Finish) or Phase 2 (Parallel), per the actual loop flow. The task description mentions Von Braun in the CLAUDE.md flow: "Von Braun (deploy)". Looking at `loop.md`, Von Braun does not appear in the phases either -- so Von Braun may be a deploy-only agent used by `/deploy` command, not `/loop`. If so, the spec should add a brief note explaining why only 12 of 13 agents appear in the loop flow, plus Deming as lead = 13 visible names rather than 14. **The "14 roles" claim in the terminal output and the visual should be reconciled.**

#### NB-3: loopInstallOutput terminal mock vs actual CLI output

**What:** The spec's `loopInstallOutput` shows a clean happy path where settings.json is written fresh. The actual `run_install()` in `loop_cmd.rs` has a branch: if `settings.json` already exists and lacks the env var, it prints a warning instead of a success checkmark. The spec also omits the "All Claude: Opus + Sonnet + Haiku" line that the real CLI prints (line 260 of `loop_cmd.rs`).

**Why it matters:** Marketing terminal mocks are aspirational, not literal. This is acceptable. Noted for completeness only.

**Suggested fix:** None required. The mock represents the ideal first-run experience.

#### NB-4: Hero tagline removes "Cloud-synced" -- still a selling point?

**What:** Current: "One command. Fully configured. Cloud-synced. Open source." New: "One command. 13 AI agents. Fully configured. Open source." This drops "Cloud-synced" to make room for "13 AI agents."

**Why it matters:** Cloud sync is still a feature (the Sync step is #4 in HowItWorks, there is a features card for it). Removing it from the hero tagline is a positioning choice, not a bug. Just flagging it as intentional.

**Suggested fix:** None. The trade-off is reasonable -- the loop is the differentiator.

#### NB-5: `[check]` placeholder in terminal mock

**What:** The spec says "Use `[check]` as a plain text stand-in for the checkmark character, matching the pattern already used in `initWizardOutput`." This is correct -- `initWizardOutput` already uses `[check]` on lines 25-28 of `commands.ts`.

**Why it matters:** No issue. Confirming the spec is correct here. The actual CLI uses colored checkmarks via `output::success()` but the terminal mock has always used `[check]` as a text stand-in.

---

## Verification Summary

| Check | Result |
|-------|--------|
| Agent names match `loop_cmd.rs` AGENTS array | PASS (13 agents confirmed) |
| Slash commands match COMMANDS array | PASS (loop, bugfix, deploy, discover) |
| Feature interface matches spec's object shape | PASS (title, description, icon) |
| ComparisonRow interface matches spec's object shape | PASS (all fields boolean) |
| AnimatedSection accepts `id` prop | PASS |
| TerminalWindow accepts `title` prop | PASS |
| Container component exists at expected path | PASS |
| `motion` import from `motion/react` matches existing pattern | PASS |
| Nav uses `navLinks` array (mobile reads same array) | PASS |
| Footer architecton block matches spec's "current" snapshot | PASS (lines 28-48 match) |
| Hero sub-tagline on line 50 matches spec's "current" snapshot | PASS |
| HowItWorks heading says "four steps" | PASS |
| App.tsx component order matches spec's "current" snapshot | PASS |
| Build order keeps tree green at each step | PASS |
| No missing npm dependencies (motion already installed) | PASS |

---

## Conclusion

This is a well-crafted spec. The exact code snippets, line references, build order, and edge case analysis are all solid. The only substantive question is NB-2 (Von Braun omission), which the builder should resolve before or during implementation -- either add Von Braun to a phase or adjust the "14 roles" visual claim. Everything else is ready to build.

**APPROVED for implementation.**
