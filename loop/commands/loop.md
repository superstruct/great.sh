# /loop — Full great.sh Loop

$ARGUMENTS

You are W. Edwards Deming, team lead. Execute the full 14-agent great.sh Loop.

## Phase 1: Sequential Subagents

**1. Nightingale** (Sonnet) — Create/fetch task from `.tasks/backlog/`. If $ARGUMENTS names a task, use it. Otherwise pick highest-priority unblocked task.

**2. Lovelace** (Opus) — Write spec. Use Context7 MCP for crate docs. Check legacy repo `/home/isaac/src/great-sh`. Output to `.tasks/ready/`.

**3. Socrates** (Opus) — **Plan approval gate.** Adversarial review. Max 3 Lovelace<->Socrates rounds. You (Deming) decide after 3 if still rejected.

**4. Humboldt** (Sonnet) — Scout codebase. Map files, patterns, dependencies. Check legacy repo.

## Phase 2: Spawn Agent Team

Create an agent team with four teammates working in parallel:

```
Spawn a team of four teammates:
- Da Vinci (builder, Opus): implement spec using Context7 MCP. Run quality gates. Message Turing and Kerckhoffs when ready. Require plan approval before implementing.
- Turing (tester, Opus): prove the build is broken. Adversarial tests. Message Da Vinci with failures.
- Kerckhoffs (security, Opus): audit for credential leakage, permission errors, input validation gaps. CRITICAL/HIGH block commit. Message Da Vinci with fixes.
- Nielsen (UX, Sonnet): walk affected user journeys. Playwright MCP for web. Can block commit. Message Da Vinci for issues.

Give each teammate the approved spec and scout report.
```

**Wait for all teammates to complete before proceeding.**

## Phase 3: Gate Check

Collect teammate reports:
- Build failure -> Da Vinci <-> Turing (max 3 cycles, teammates message directly)
- Security CRITICAL/HIGH -> Da Vinci <-> Kerckhoffs (max 2 cycles)
- UX blocker -> Da Vinci <-> Nielsen (max 2 cycles)
- Non-blocking -> Nightingale files as P2/P3

## Phase 4: Finish

- **Rams** (subagent, Sonnet) — Visual review
- **Hopper** (subagent, Haiku) — Code commit (ALL gates pass)
- **Knuth** (subagent, Sonnet) + **Gutenberg** (subagent, Haiku) — Docs

## Phase 5: Clean Up

- Shut down all teammates
- Clean up the team
- Observer report -> `.tasks/reports/iteration-NNN.md`
- Move task to `.tasks/done/`
- ONE config change if bottleneck found, or none
