# Scout Report 0035: Init Wizard MCP Bridge UX Polish

**Scout:** Alexander von Humboldt
**Date:** 2026-02-28
**Spec:** `.tasks/ready/0035-init-wizard-polish-spec.md`
**Socrates verdict:** APPROVED WITH ADVISORY

---

## Scope

All changes are confined to one file: `src/cli/init.rs` (532 lines).
No new files, no new dependencies, no API changes.

---

## File Map: `src/cli/init.rs`

### Change 1 — Fix 1: `--template` doc comment (line 15)

**Current (exact text):**
```rust
    /// Template to initialize from (ai-fullstack-ts, ai-fullstack-py, ai-minimal)
```

**Required:** Append `, saas-multi-tenant` to the list.
- `init_from_template()` at line 308 already handles `"saas-multi-tenant"` —
  the help text simply omits it. No logic change.

---

### Change 2 — Fixes 2+3+4: MCP Bridge block (lines 229-239)

**Current block (lines 229-239):**
```rust
    if prompt_yes_no(
        "Enable built-in MCP bridge (routes MCP servers to all AI agents)?",
        false,
    )? {
        config.mcp_bridge = Some(McpBridgeConfig {
            preset: Some("minimal".to_string()),
            ..Default::default()
        });
        output::success("MCP bridge enabled with 'minimal' preset");
        output::info("  Change preset in great.toml: minimal, agent, research, full");
    }
```

**Required replacement (combined diff from spec lines 157-169):**
```rust
    if prompt_yes_no(
        "Enable built-in MCP bridge (routes MCP servers to all AI agents)?",
        false,
    )? {
        // Preset heuristic is agent-count-based (wizard context).
        // Templates use complexity-based presets (fullstack projects get "agent"
        // even with a single agent). These semantics differ intentionally —
        // do not "fix" one to match the other.
        let preset = if config.agents.as_ref().map_or(0, |a| a.len()) > 1 {
            "agent"
        } else {
            "minimal"
        };
        config.mcp_bridge = Some(McpBridgeConfig {
            preset: Some(preset.to_string()),
            ..Default::default()
        });
        output::success(&format!("MCP bridge enabled with {} preset", preset));
        output::info("  Presets: minimal (1 tool) | agent (6 tools) | research (8 tools) | full (9 tools)");
    }
```

The `let preset = ...` binding (Fix 3) must appear before the `output::success`
call (Fix 4), which references it. Apply all three as one edit.

---

## Agent Collection Logic

| Line | Detail |
|------|--------|
| 155 | `let mut agents = HashMap::new();` — type is `HashMap<String, AgentConfig>` |
| 158-165 | Claude inserted unconditionally (always present) |
| 168-177 | Codex inserted if user answers "y" |
| 179-188 | Gemini inserted if user answers "y" |
| 190 | `config.agents = Some(agents);` — HashMap moved into config |

`config.agents` type: `Option<HashMap<String, AgentConfig>>` (from `config/schema.rs`).

At line 229 (MCP Bridge block), `config.agents` is always `Some(...)` because
line 190 runs unconditionally. Claude is always present, so `len() >= 1` is
guaranteed. The `map_or(0, ...)` at Fix 3 is a defensive fallback only.

---

## Preset Tool Count Verification

Source: `src/mcp/bridge/tools.rs`, `Preset::tool_names()` lines 153-186.

| Preset | Tools | Count | Verified |
|--------|-------|-------|---------|
| `Minimal` | `["prompt"]` | **1** | YES |
| `Agent` | `["prompt", "run", "wait", "list_tasks", "get_result", "kill_task"]` | **6** | YES |
| `Research` | Agent + `["research", "analyze_code"]` | **8** | YES |
| `Full` | Research + `["clink"]` | **9** | YES |

Test `test_preset_tool_counts` at `src/mcp/bridge/tools.rs:194-199` already
asserts these exact counts. The counts are not in flux.

---

## Test Functions in `mod tests` (lines 378-531)

| Test function | Lines | What it covers |
|---------------|-------|----------------|
| `test_detect_project_name_returns_string` | 382-385 | `detect_project_name()` returns non-empty string |
| `test_init_from_template_unknown` | 388-398 | Unknown template returns Ok, no file created |
| `test_init_from_template_minimal` | 401-411 | `ai-minimal` template writes valid TOML with `[agents.claude]` |
| `test_init_from_template_fullstack_ts` | 413-424 | `ai-fullstack-ts` template contains node/typescript keywords |
| `test_init_from_template_fullstack_py` | 427-436 | `ai-fullstack-py` template contains python keyword |
| `test_templates_parse_as_valid_config` | 439-473 | All 4 templates deserialize into `GreatConfig`, have `[project]` and agents |
| `test_templates_have_mcp_bridge` | 476-513 | All 4 templates have `[mcp-bridge]` with expected preset values |
| `test_default_config_has_no_mcp_bridge` | 516-522 | `GreatConfig::default()` has no mcp_bridge (opt-in) |
| `test_default_config_serializes` | 524-530 | Default config serializes to TOML without error |

**No existing test covers the interactive wizard's preset selection logic** —
Socrates flagged this. Spec approves it as-is; Socrates suggests (advisory, not
blocking) extracting the `if/else` into a named pure helper for testability.

---

## Socrates Advisory: Code Comment Required

Socrates identified a semantic divergence:

- **Templates** `ai-fullstack-ts` and `ai-fullstack-py` use preset `"agent"`
  even though they define only 1 agent (claude). Preset chosen by project complexity.
- **Wizard** uses `"agent"` only when `len() > 1`. Preset chosen by agent count.

A user who runs `--template ai-fullstack-ts` gets `"agent"` (6 tools).
A user who runs the wizard with only Claude gets `"minimal"` (1 tool).

**Da Vinci must include a code comment** at the `let preset =` binding that
documents this intentional divergence. The comment is included in the
replacement block above. Do not remove it.

---

## Dependency Map

```
src/cli/init.rs
  uses: crate::cli::output          (output::header, output::info, output::success)
  uses: crate::config::schema::*    (GreatConfig, McpBridgeConfig, AgentConfig, etc.)
  uses: crate::platform             (platform::detect_platform_info, Platform enum)
  embeds: templates/*.toml          (include_str! macros at lines 305-308)
  tests use: tempfile (dev dep)
```

No changes needed to any imported module. `McpBridgeConfig` already has
`preset: Option<String>` — no schema change required.

---

## Build Order

1. **Fix 1** (line 15): Independent doc comment edit. Apply first, verify separately.
2. **Fixes 2+3+4** (lines 232-239): Apply as one combined edit. Fix 4 references
   the `preset` binding introduced by Fix 3; they cannot be separated.

---

## Current Test Count

```
cargo test total: 329 tests
  unit + doc tests: 231 passed
  integration tests: 97 passed, 1 ignored
  cli smoke tests:   1 passed
```

Backlog AC5 claims 329 — this is verified correct as of 2026-02-28.

---

## Risks

| Risk | Severity | Note |
|------|----------|------|
| Fix 4 applied before Fix 3 | HIGH | `preset` variable would be undefined. Apply as one edit. |
| Wrong line numbers after earlier edits | LOW | File is 532 lines, no edits in progress on this branch |
| Template tests break | NONE | Templates are not modified by this task |
| New clippy warnings | NONE | `let preset = if ... { } else { }` is idiomatic; `format!` with `&str` is standard |

---

## Files to Modify

| File | Lines | Change type |
|------|-------|-------------|
| `/home/isaac/src/sh.great/src/cli/init.rs` | 15 | Doc comment text |
| `/home/isaac/src/sh.great/src/cli/init.rs` | 229-239 | Logic + display strings |

No files created. No files deleted. No Cargo.toml changes.
