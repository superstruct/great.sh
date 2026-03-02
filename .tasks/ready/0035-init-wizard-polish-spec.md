# Spec 0035: Init Wizard MCP Bridge UX Polish

**Task:** `.tasks/backlog/0035-init-wizard-mcp-bridge-polish.md`
**Complexity:** S (single file, four textual changes)
**File:** `src/cli/init.rs`

---

## Summary

Four small defects in the `great init` wizard degrade the MCP bridge onboarding UX.
This spec describes exact code changes for each fix, all confined to `src/cli/init.rs`.
No new dependencies, no new files, no API changes.

---

## Fix 1: `--template` help text incomplete (line 15)

The `#[arg(long)]` doc comment omits the `saas-multi-tenant` template. The
`init_from_template()` function at line 308 already handles it; the help text
simply does not advertise it.

### Current code (line 15)

```rust
    /// Template to initialize from (ai-fullstack-ts, ai-fullstack-py, ai-minimal)
```

### Replacement

```rust
    /// Template to initialize from (ai-fullstack-ts, ai-fullstack-py, ai-minimal, saas-multi-tenant)
```

### Verification

After building, `cargo run -- init --help` must list all four template names
in the `--template` option description.

---

## Fix 2: Preset descriptions absent (line 238)

The info message after enabling the MCP bridge lists bare preset names with no
context. A first-time user cannot evaluate which preset to pick. Replace with
a format that shows the tool count per preset.

Tool counts are verified against `src/mcp/bridge/tools.rs` lines 153-186:

| Preset     | Tools | Count |
|------------|-------|-------|
| `minimal`  | prompt | 1 |
| `agent`    | prompt, run, wait, list_tasks, get_result, kill_task | 6 |
| `research` | agent + research, analyze_code | 8 |
| `full`     | research + clink | 9 |

### Current code (line 238)

```rust
        output::info("  Change preset in great.toml: minimal, agent, research, full");
```

### Replacement

```rust
        output::info("  Presets: minimal (1 tool) | agent (6 tools) | research (8 tools) | full (9 tools)");
```

---

## Fix 3: Smart preset selection (lines 233-236)

The wizard hardcodes `preset = "minimal"` regardless of agent context. When the
user configured multiple agents (Claude + Codex and/or Gemini), the `"agent"`
preset (6 tools including multi-backend dispatch via `run`) is the semantically
correct default.

### Decision logic

At line 232, `config.agents` is `Some(HashMap)` because the local `agents`
HashMap was moved into `config.agents` at line 190. Claude is always present
(inserted at line 158), so `len() >= 1` is guaranteed. The check is:

```
if config.agents.as_ref().map_or(0, |a| a.len()) > 1 => "agent"
else => "minimal"
```

### Current code (lines 233-236)

```rust
        config.mcp_bridge = Some(McpBridgeConfig {
            preset: Some("minimal".to_string()),
            ..Default::default()
        });
```

### Replacement

```rust
        let preset = if config.agents.as_ref().map_or(0, |a| a.len()) > 1 {
            "agent"
        } else {
            "minimal"
        };
        config.mcp_bridge = Some(McpBridgeConfig {
            preset: Some(preset.to_string()),
            ..Default::default()
        });
```

---

## Fix 4: Single-quote inconsistency (line 237)

The success message wraps the preset name in single quotes, but no other wizard
message uses that style. Remove the quotes and use the dynamic `preset` variable
from Fix 3.

### Current code (line 237)

```rust
        output::success("MCP bridge enabled with 'minimal' preset");
```

### Replacement

```rust
        output::success(&format!("MCP bridge enabled with {} preset", preset));
```

Note: This line must come after the `let preset = ...` binding from Fix 3.

---

## Combined diff: lines 232-239

For clarity, here is the full before/after for the MCP Bridge `if` block
(lines 232-239):

### Before

```rust
    )? {
        config.mcp_bridge = Some(McpBridgeConfig {
            preset: Some("minimal".to_string()),
            ..Default::default()
        });
        output::success("MCP bridge enabled with 'minimal' preset");
        output::info("  Change preset in great.toml: minimal, agent, research, full");
    }
```

### After

```rust
    )? {
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

---

## Implementation build order

All four fixes are in the same file. Apply in this order:

1. **Fix 1** (line 15) -- doc comment, no dependencies
2. **Fixes 2 + 3 + 4** (lines 232-239) -- apply as one combined edit using the
   "Combined diff" above, since Fix 4 depends on Fix 3's `preset` binding

---

## Edge cases

| Scenario | Expected preset | Rationale |
|----------|----------------|-----------|
| User accepts only Claude (default) | `"minimal"` | Single agent, 1 tool is sufficient |
| User adds Codex only | `"agent"` | 2 agents, multi-backend dispatch tools useful |
| User adds Gemini only | `"agent"` | 2 agents, same reasoning |
| User adds both Codex and Gemini | `"agent"` | 3 agents, `"agent"` is still the right default |
| `config.agents` is `None` (impossible in current flow) | `"minimal"` | `map_or(0, ...)` returns 0, safe fallback |
| MCP bridge not enabled | No preset written | Block is skipped entirely |

---

## Error handling

No new error paths. The `preset` variable is a `&str` literal (`"agent"` or
`"minimal"`), so there is no possibility of invalid values. The `format!` macro
is infallible.

---

## Security considerations

None. This change affects only user-facing display text and a configuration
default. No file paths, network calls, or secret handling are modified.

---

## Testing strategy

### Existing tests -- no changes needed

The `test_templates_have_mcp_bridge` test (lines 476-513 of `src/cli/init.rs`)
verifies template preset values, not the wizard's dynamic preset logic. It
asserts that `ai-minimal` has preset `"minimal"`, `ai-fullstack-ts`/`py` have
`"agent"`, and `saas-multi-tenant` has `"full"`. These template files are
unchanged by this task, so this test requires no update.

The `init_help_shows_initialize` smoke test (line 45 of `tests/cli_smoke.rs`)
checks for the word "Initialize" in `--help` output. It does not assert on
template names, so it passes without modification.

No existing test checks the wizard's runtime preset selection or the success
message string, because the wizard requires interactive stdin. The existing
tests exercise template-based init, which is not modified here.

### Manual verification

```bash
# Fix 1: help text shows all four templates
cargo run -- init --help 2>&1 | grep -o 'saas-multi-tenant'
# Expected: saas-multi-tenant

# Fix 2-4: interactive wizard (requires stdin)
# Run `cargo run -- init --force` in a temp directory,
# accept defaults for everything, enable MCP bridge.
# Verify: success message says "MCP bridge enabled with minimal preset"
# Verify: info line says "Presets: minimal (1 tool) | agent (6 tools) | ..."

# Fix 3: with multiple agents
# Same as above, but say "y" to Codex.
# Verify: success message says "MCP bridge enabled with agent preset"
# Verify: great.toml contains preset = "agent"
```

### Automated checks

```bash
cargo test          # All existing tests pass (no regressions)
cargo clippy        # No new warnings
```

---

## Files to modify

| File | Lines | Change |
|------|-------|--------|
| `src/cli/init.rs` | 15 | Add `saas-multi-tenant` to doc comment |
| `src/cli/init.rs` | 232-239 | Dynamic preset selection + updated messages |

No files created. No files deleted.

---

## Platform considerations

All changes are pure string/logic changes with no platform-specific behavior.
macOS ARM64/x86_64, Ubuntu, and WSL2 are all unaffected.
