# Task 0035: Init Wizard MCP Bridge UX Polish

**Priority:** P2
**Complexity:** S
**Type:** bugfix/polish
**Module:** `src/cli/init.rs`
**Status:** Backlog

## Problem

Four small defects in `src/cli/init.rs` accumulated as non-blocking follow-ups from iterations 031 and 030. They all touch the MCP bridge onboarding path and together degrade the user experience for anyone going through `great init`:

1. **Preset descriptions absent** (P2, Nielsen iter-031): The info message on line 238 reads `"  Change preset in great.toml: minimal, agent, research, full"` — bare names with no description. A first-time user cannot evaluate which preset is right without reading external docs.

2. **`--template` help text incomplete** (P3, Nielsen iter-031): The doc comment on line 15 lists `(ai-fullstack-ts, ai-fullstack-py, ai-minimal)` — `saas-multi-tenant` is missing. Running `great init --help` silently omits a valid template, and `init_from_template()` already handles it at line 308.

3. **Wizard hardcodes `"minimal"` preset regardless of agent context** (INFO, Dijkstra iter-031): Lines 233–238 always write `preset = "minimal"` even if the user opted into Codex and Gemini earlier in the same wizard run. When multiple agents are configured, `"agent"` is the semantically correct default (6 tools including multi-backend dispatch). The logic is: `if agents.len() > 1 { "agent" } else { "minimal" }`.

4. **Single-quote inconsistency** (INFO, Rams iter-031): Line 237 `output::success("MCP bridge enabled with 'minimal' preset")` uses single quotes around the preset name, but the immediately following line 238 uses no quotes. All wizard success/info messages should use consistent formatting — unquoted preset names match the style used everywhere else in the wizard.

## Acceptance Criteria

- [ ] `great init --help` lists all four templates: `ai-fullstack-ts`, `ai-fullstack-py`, `ai-minimal`, `saas-multi-tenant`
- [ ] When MCP bridge is enabled, the info message includes a one-line description for each preset, e.g. `"  minimal (1 tool) | agent (6 tools) | research (8 tools) | full (9 tools)"`
- [ ] When MCP bridge is enabled and the user configured more than one agent (Claude + at least one of Codex/Gemini), `great.toml` is written with `preset = "agent"` instead of `preset = "minimal"`
- [ ] The success message on enable reads without single quotes around the preset name, consistent with the info line style, e.g. `"MCP bridge enabled with minimal preset"` or `"MCP bridge enabled — preset: minimal"`
- [ ] `cargo test` passes (329 tests); `cargo clippy` is clean

## Dependencies

None. All changes are in `src/cli/init.rs` lines 15, 233–238.

## Notes

- Exact current text, line 15: `/// Template to initialize from (ai-fullstack-ts, ai-fullstack-py, ai-minimal)`
- Exact current text, line 237: `output::success("MCP bridge enabled with 'minimal' preset");`
- Exact current text, line 238: `output::info("  Change preset in great.toml: minimal, agent, research, full");`
- Preset tool counts from `site/src/components/sections/Bridge.tsx` line 14: minimal=1, agent=6, research=8, full=9 — use these to populate the descriptions
- Agent count check: `config.agents.as_ref().map(|a| a.len()).unwrap_or(0) > 1` (Claude is always inserted before this block)
- The preset chosen for the success message must reflect the dynamic selection (minimal vs agent), not a hardcoded string
- Da Vinci should update the two wizard tests added in iter-031 if either verifies the preset value or the success string
