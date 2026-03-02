# Spec Review: 0035 — Init Wizard MCP Bridge UX Polish

**Reviewer:** Socrates
**Spec:** `.tasks/ready/0035-init-wizard-polish-spec.md`
**Backlog:** `.tasks/backlog/0035-init-wizard-mcp-bridge-polish.md`
**Round:** 1
**Date:** 2026-02-28

---

## VERDICT: APPROVED WITH ADVISORY

---

## Verification Summary

### Line numbers and code — ALL MATCH

| Spec claim | Actual | Status |
|------------|--------|--------|
| Line 15: doc comment missing `saas-multi-tenant` | `src/cli/init.rs:15` — exact match | VERIFIED |
| Line 237: single-quoted `'minimal'` | `src/cli/init.rs:237` — exact match | VERIFIED |
| Line 238: bare preset names, no descriptions | `src/cli/init.rs:238` — exact match | VERIFIED |
| Lines 233-236: hardcoded `"minimal"` preset | `src/cli/init.rs:233-236` — exact match | VERIFIED |
| `config.agents = Some(agents)` at line 190 | `src/cli/init.rs:190` — exact match | VERIFIED |
| Claude always inserted at line 158 | `src/cli/init.rs:158` — exact match | VERIFIED |
| `init_from_template()` handles `saas-multi-tenant` at line 308 | `src/cli/init.rs:308` — exact match | VERIFIED |
| `test_templates_have_mcp_bridge` at lines 476-513 | `src/cli/init.rs:476-513` — exact match | VERIFIED |

### Tool counts — ALL MATCH

Verified against `src/mcp/bridge/tools.rs` lines 153-186:

| Preset | Spec claims | `tool_names()` returns | Status |
|--------|-------------|----------------------|--------|
| minimal | 1 (prompt) | `vec!["prompt"]` | MATCH |
| agent | 6 (prompt, run, wait, list_tasks, get_result, kill_task) | exact match | MATCH |
| research | 8 (agent + research, analyze_code) | exact match | MATCH |
| full | 9 (research + clink) | exact match | MATCH |

### Edge cases — COMPLETE

The spec's edge case table at line 186 covers all reachable states:
- 1 agent (Claude only) -> `"minimal"` -- correct
- 2 agents (Claude + Codex or Gemini) -> `"agent"` -- correct
- 3 agents (all three) -> `"agent"` -- correct
- `config.agents` is `None` -> `"minimal"` via `map_or(0, ...)` -- defensive, correct
- MCP bridge not enabled -> block skipped entirely -- correct

### Error handling — SOUND

No new error paths. `preset` is a `&str` literal, `format!` is infallible, no `.unwrap()` introduced.

---

## Concerns

### Concern 1: Wizard preset logic diverges from template preset semantics

```
{
  "gap": "Templates ai-fullstack-ts and ai-fullstack-py both define only 1 agent (claude) yet use preset 'agent' (6 tools). The wizard's dynamic logic assigns 'minimal' (1 tool) for any single-agent config. A user who runs the wizard with only Claude and enables the MCP bridge gets a strictly less capable default than someone who uses --template ai-fullstack-ts.",
  "question": "Is the intent that template presets reflect the project's COMPLEXITY (fullstack projects benefit from run/wait/kill_task regardless of agent count) while the wizard's heuristic reflects AGENT COUNT? If so, is this semantic split acceptable to UX, or should the wizard also consider whether the user selected multiple runtimes or CLI tools?",
  "severity": "ADVISORY",
  "recommendation": "Document in a code comment at the preset selection block that the wizard heuristic is agent-count-based while templates are complexity-based. This prevents a future maintainer from 'fixing' one to match the other. No code change required — the current logic is defensible as a conservative default."
}
```

### Concern 2: No automated test for dynamic preset selection

```
{
  "gap": "The spec acknowledges that no existing test covers the wizard's runtime preset selection because the wizard requires interactive stdin. Fix 3 introduces branching logic (agent count > 1) that has zero automated test coverage. The only verification is manual.",
  "question": "Can the preset selection logic be extracted into a pure function (e.g., fn select_default_preset(agent_count: usize) -> &'static str) that IS unit-testable, even if the interactive wizard itself is not?",
  "severity": "ADVISORY",
  "recommendation": "The spec is implementable as-is. However, Da Vinci should consider extracting the two-line if/else into a named helper with a #[cfg(test)] unit test. This is a suggestion for the builder, not a blocking requirement — the logic is trivially simple."
}
```

### Concern 3: Backlog AC5 says 329 tests — spec should not hardcode test count

```
{
  "gap": "Backlog AC5 says 'cargo test passes (329 tests)'. The spec's automated checks section says 'cargo test' and 'cargo clippy' without a count. This is correct — test counts drift. But the backlog's count may be stale.",
  "question": "Is the current test count actually 329, or has it changed since the backlog was written?",
  "severity": "ADVISORY",
  "recommendation": "No action needed in the spec — it correctly avoids hardcoding the count. Da Vinci should run cargo test and confirm the count matches or exceeds backlog's claim. If it differs, note it in the commit message."
}
```

---

## What the spec gets right

1. **Surgical precision**: Four fixes, one file, exact line numbers, all verified correct against actual source.
2. **Combined diff**: The combined before/after block for lines 232-239 is correct and prevents ordering mistakes during implementation.
3. **Build order**: Fix 1 is independent; Fixes 2-4 are correctly grouped as a single edit since Fix 4 depends on Fix 3's binding.
4. **Defensive coding**: `map_or(0, |a| a.len())` handles the impossible-but-safe `None` case without `.unwrap()`.
5. **No scope creep**: The spec does not touch templates, tests, or any other file. Pure init.rs changes.

---

## Summary

A clean, well-scoped S-complexity spec with all line numbers verified correct against actual source. The only substantive finding is a semantic divergence between template presets (complexity-based) and wizard presets (agent-count-based), which is ADVISORY — the wizard's conservative default is defensible. Approved for implementation.
