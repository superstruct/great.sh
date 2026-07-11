# 0055 — Collapse Loop roster to functional roles with evidence-gated termination

| Field | Value |
|---|---|
| Priority | P1 |
| Type | refactor |
| Module | `loop/`, `src/cli/loop_cmd.rs`, `site/` |
| Status | backlog |
| Estimated Complexity | L |

## Problem

The Loop's 16-persona, 5-phase pipeline with fixed inter-role round caps (3 plan
rounds, 3 build-fix cycles, 2 security, 2 UX, …) was calibrated for agent models
that needed external choreography: narrow single-job prompts, hard turn budgets,
and a fixed hand-off order. Current-generation models plan and verify their own
work; the choreography now produces redundant turns (agents re-deriving context
the previous agent already had) and forces re-tuning of caps and role prompts on
every model generation. The durable value of the Loop is its evidence discipline
— no success claims without cited command output — not the number of roles.

## Proposed Change

1. Collapse the 16 personas into 4 functional roles:
   - **builder** — implements the spec, runs quality gates, answers findings
     with evidence (command output), not re-argument
   - **verifier** — adversarial: tries to prove the change broken or insecure;
     absorbs testing, security-audit, regression-watch, and performance
     checklists as review dimensions; findings must cite reproductions
   - **reviewer** — read-only quality review; absorbs code-structure, UX
     heuristics, output-design, and docs-accuracy checklists as review
     dimensions
   - **scout** (optional) — read-only recon for large or unfamiliar change
     surfaces
2. Remove fixed inter-role round caps. A phase ends when its exit criteria are
   met: quality gates green plus verifier reporting no CONFIRMED blocking
   findings. Keep one generous safety ceiling as a stuck-loop backstop, not a
   tuning knob.
3. Verification is artifact-driven: a verifier finding without a cited
   reproduction is PLAUSIBLE, not CONFIRMED; only CONFIRMED findings block.
4. Per-role model tiering config remains supported but defaults to inheriting
   the session model. Keep the documented note that security-focused audit
   work can be pinned to Opus (see f54d20d).
5. Update skills, teams config, plugin manifest, CLI roster constants, README,
   site Loop section; delete retired persona files; note the change in the
   CHANGELOG.

## Acceptance Criteria

- Role count ≤ 4 plus optional scout; retired persona files removed from the
  plugin
- No fixed inter-role round caps remain in skills or agent prompts
- Evidence rules preserved: gates before commits, cited output, observer report
- A dogfood run on a real backlog item completes with fewer total agent turns
  than the historical pipeline, recorded in the iteration report
- `cargo test` green; docs and site consistent with the new roster
