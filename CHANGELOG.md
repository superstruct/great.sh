# Changelog

## Loop plugin 0.4.0 — 2026-07-11

Recalibrated the great.sh Loop for models with stronger self-direction: fewer
roles, evidence-gated termination.

### Changed

- Collapsed the 16-persona roster into 4 functional roles: **builder**,
  **verifier** (adversarial — tries to prove the change broken or insecure),
  **reviewer** (read-only quality), and an optional **scout**. The personas'
  checklists (testing, security audit, regression watch, performance, code
  structure, UX heuristics, output design, docs accuracy) are folded into the
  role prompts as review dimensions.
- Removed fixed inter-role round caps (3 plan rounds, 3 build-fix cycles,
  2 security, 2 UX, …). A phase now ends when its exit criteria are met —
  quality gates green plus no CONFIRMED blocking findings — with a single
  generous safety ceiling as a stuck-loop backstop.
- Verification is artifact-driven: a finding without a cited reproduction is
  PLAUSIBLE, not CONFIRMED, and only CONFIRMED CRITICAL/HIGH findings block.
  The builder answers findings with rerun evidence, not re-argument.
- Per-role model tiering now defaults to inheriting the session model. Pinning
  a tier per role remains supported in the teams config — e.g. Opus for
  security-audit-heavy verification, since Fable-class cyber safety
  classifiers can refuse security-probing work mid-audit.
- Spec writing and plan review moved from dedicated agents into the team-lead
  session, which self-reviews the spec against the retired reviewer's gap
  checklist before building.

### Removed

- 15 persona agent files (`nightingale`, `lovelace`, `socrates`, `humboldt`,
  `davinci`, `vonbraun`, `turing`, `kerckhoffs`, `rams`, `nielsen`, `knuth`,
  `gutenberg`, `hopper`, `dijkstra`, `wirth`). Legacy pre-plugin installs of
  these files are still cleaned up by `great loop install`.
