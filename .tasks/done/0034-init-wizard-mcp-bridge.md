# Task 0034: Wire mcp-bridge into the init wizard and built-in templates

**Priority:** P2
**Complexity:** S
**Type:** feature

## Problem

`great mcp-bridge` is the most recently shipped and most differentiating capability of great.sh — a pure-Rust, zero-Node MCP bridge to five AI backends. Yet it is completely invisible to new users going through the natural onboarding flow:

1. `great init` — interactive wizard never asks about mcp-bridge; generated `great.toml` has no `[mcp-bridge]` stanza.
2. All four built-in templates (`ai-minimal`, `ai-fullstack-ts`, `ai-fullstack-py`, `saas-multi-tenant`) contain no `[mcp-bridge]` stanza.
3. `great apply` therefore never configures the bridge for any new user.

The marketing site (`site/src/data/commands.ts` `sampleToml`) shows `[mcp-bridge]` prominently, but the real CLI does not emit it. This is a gap between promise and delivery.

The fix is small — `great init` already has a `prompt_yes_no` pattern for optional features. Adding a single branch for mcp-bridge and updating the templates is an S-complexity change.

## Acceptance Criteria

- [ ] `great init` wizard asks "Enable the built-in MCP bridge (great mcp-bridge)?" after the MCP Servers section; answering yes writes a `[mcp-bridge]` stanza (`preset = "minimal"`) to the generated `great.toml`.
- [ ] `ai-minimal.toml` template gains a `[mcp-bridge]` section (`preset = "minimal"`).
- [ ] `ai-fullstack-ts.toml` and `ai-fullstack-py.toml` templates gain a `[mcp-bridge]` section (`preset = "agent"`).
- [ ] `saas-multi-tenant.toml` template gains a `[mcp-bridge]` section (`preset = "full"`).
- [ ] `test_templates_parse_as_valid_config` in `src/cli/init.rs` continues to pass (all four templates still deserialize cleanly into `GreatConfig`).

## Notes

- `McpBridgeConfig` is already defined in `src/config/schema.rs` with `#[serde(rename = "mcp-bridge")]`. The serializer will emit the correct `[mcp-bridge]` TOML key automatically.
- `great apply` already reads and applies `config.mcp_bridge`; no changes needed there.
- The wizard should default to `false` (opt-in) since users without any AI CLI tools installed will get no benefit from the bridge.
- Preset guidance: `minimal` for simple single-agent setups, `agent` for multi-agent TypeScript/Python stacks, `full` for the SaaS template which has the most backends.
- Prior iteration context: mcp-bridge was shipped in iter 026 (task 0029), hardened in iter 028 (task 0030), smoke-tested in iter 028 (task 0031), and marketed in iter 029/030 (tasks 0032/0033). This task closes the last onboarding gap.
- `McpBridgeConfig` fields to verify before coding: check `src/config/schema.rs` for the exact struct — it has at minimum `preset` and `default_backend` fields.
