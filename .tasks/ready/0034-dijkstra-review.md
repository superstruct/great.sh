# Dijkstra Review — Task 0034: Wire mcp-bridge into init wizard and templates

**Reviewer:** Edsger Dijkstra (Code Reviewer)
**Date:** 2026-02-28
**Files reviewed:**
- `/home/isaac/src/sh.great/src/cli/init.rs`
- `/home/isaac/src/sh.great/templates/ai-minimal.toml`
- `/home/isaac/src/sh.great/templates/ai-fullstack-ts.toml`
- `/home/isaac/src/sh.great/templates/ai-fullstack-py.toml`
- `/home/isaac/src/sh.great/templates/saas-multi-tenant.toml`

---

```
VERDICT: APPROVED

Issues:
- [WARN] src/cli/init.rs:238 — The info string lists four presets
  ("minimal, agent, research, full") but omits the ordering significance:
  the user has no hint that these are cumulative tiers. This is a UX
  deficiency, not a structural defect.

- [WARN] src/cli/init.rs:233-236 — The wizard hardcodes preset "minimal"
  regardless of what the user chose in prior sections (e.g., if they added
  Gemini and Codex, "agent" or "research" would be more appropriate). The
  choice is not wrong but it is conservative to the point of being
  misleading for a multi-agent setup. A future task should revisit
  context-aware preset selection.

Summary: The changes are structurally sound — the wizard section is
minimal, abstraction boundaries are respected, preset strings match the
canonical values in tools.rs exactly, the schema field is properly
annotated with skip_serializing_if, and the two new tests (test_templates_have_mcp_bridge
and test_default_config_has_no_mcp_bridge) directly and correctly assert
the invariants introduced by this task.
```

---

## Detailed findings

### Correctness

The four preset strings written to templates (`"minimal"`, `"agent"`, `"agent"`, `"full"`) are verified to match exactly the strings handled by `Preset::from_str` in `/home/isaac/src/sh.great/src/mcp/bridge/tools.rs` (lines 143–148). No unknown preset can slip through: `GreatConfig::validate` at `src/config/schema.rs:252–261` independently checks the known set `["minimal", "agent", "research", "full"]` and emits a warning on unknown values.

The wizard section (lines 224–239 of `init.rs`) correctly uses `McpBridgeConfig { preset: Some("minimal"), ..Default::default() }`, which serializes under the `[mcp-bridge]` key via the `#[serde(rename = "mcp-bridge")]` annotation on `GreatConfig.mcp_bridge`. Round-trip correctness is guaranteed by the pre-existing `test_templates_parse_as_valid_config` test (which now covers all four templates) and the new `test_templates_have_mcp_bridge` test.

The opt-in default (`false` for `prompt_yes_no`) is correct: MCP bridge is a non-trivial runtime dependency and must not be silently enabled.

### Simplicity

The wizard block (15 lines including spacing) is the minimum expression of the requirement. It introduces no helper, no intermediate struct, no branching beyond the single yes/no gate. This is exemplary.

### Abstraction boundaries

The init module does not reach into `src/mcp/bridge/` directly; it speaks only the config schema language. Serialization and interpretation of the preset happen elsewhere. The boundary is clean.

### Naming

`mcp_bridge`, `McpBridgeConfig`, `preset` — consistent throughout the codebase. No new names are introduced.

### Pattern consistency

The wizard section follows the identical pattern of every prior section: `eprintln!()`, `output::header(...)`, `eprintln!()`, `if prompt_yes_no(...)?`. No deviation. Template files follow the same append-at-end ordering as other optional sections (compare `[platform.macos]` at end of `saas-multi-tenant.toml`).

### Tests

`test_templates_have_mcp_bridge` (lines 475–513): correctly deserializes all four templates and asserts both presence and exact preset value. The use of a slice of triples `(&str, &str, &str)` — name, content, expected preset — is clear and avoids repetition.

`test_default_config_has_no_mcp_bridge` (lines 515–522): correctly asserts the opt-in invariant on `GreatConfig::default()`. Simple and unambiguous.

Both new tests use `unwrap_or_else(|e| panic!(...))` rather than bare `.unwrap()`, which produces actionable error messages identifying which template failed. This is correct practice in `#[cfg(test)]` scope per project conventions.

### Dead code

No unused imports or variables introduced.

### WARN details

**WARN — init.rs:238 (advisory):** The hint string `"minimal, agent, research, full"` presents names without conveying that each is a superset of the previous. A user reading this after the wizard would not know whether to upgrade. This is documentation quality, not structural correctness.

**WARN — init.rs:233 (advisory):** The wizard always emits `preset = "minimal"` when the user confirms the bridge. If the user also confirmed Codex and Gemini earlier in the same wizard run, "agent" would be the semantically correct default (the bridge exists to route work across multiple backends). The code is not incorrect — the user can edit `great.toml` — but the conservative choice conflicts slightly with a multi-agent configuration the user just requested. No blocking action required; recommended for a follow-up task.
