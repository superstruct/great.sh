# Scout Report 0002: TOML Config Parser — great.toml Schema

**Scout:** Humboldt
**Date:** 2026-02-24
**Spec:** `.tasks/ready/0002-config-schema-spec.md`
**Socrates verdict:** APPROVED (advisory items only)

---

## Primary Target

**Single file to modify:** `/home/isaac/src/sh.great/src/config/schema.rs`
Lines 1–422. All 7 changes are additive (new `Option` fields + `Default` derives).

No changes required to `/home/isaac/src/sh.great/src/config/mod.rs`.

---

## Current State of `schema.rs`

```
src/config/schema.rs  422 lines
```

### Structs defined (lines 11–131)

| Struct | Line | Derives | Missing derives |
|--------|------|---------|-----------------|
| `GreatConfig` | 11 | Debug, Clone, Serialize, Deserialize, **Default** | none |
| `ProjectConfig` | 28 | Debug, Clone, Serialize, Deserialize | **Default** (MISSING) |
| `ToolsConfig` | 63 | Debug, Clone, Serialize, Deserialize, Default | none |
| `AgentConfig` | 75 | Debug, Clone, Serialize, Deserialize | **Default** (MISSING) |
| `McpConfig` | 84 | Debug, Clone, Serialize, Deserialize | cannot derive Default (command: String) |
| `SecretsConfig` | 100 | Debug, Clone, Serialize, Deserialize | **Default** (MISSING) |
| `PlatformConfig` | 109 | Debug, Clone, Serialize, Deserialize, Default | none |
| `PlatformOverride` | 119 | Debug, Clone, Serialize, Deserialize | **Default** (MISSING) |
| `ConfigMessage` | 127 | Debug, Clone (enum) | N/A |

### Methods on `GreatConfig` (lines 134–197)

- `validate()` — line 141. Runs 2 checks today: agent missing provider+model (Warning), invalid secret name (Error).
- `find_secret_refs()` — line 177. Scans only `mcp.*.env` values currently.

### Existing tests in `schema.rs` (13 tests, lines 200–422)

1. `test_parse_minimal_config` — line 205
2. `test_parse_full_config` — line 215
3. `test_parse_empty_config` — line 255
4. `test_find_secret_refs` — line 262
5. `test_validate_warns_on_empty_agent` — line 277
6. `test_validate_invalid_secret_name` — line 288
7. `test_validate_valid_config_no_messages` — line 303
8. `test_roundtrip_serialize` — line 325
9. `test_tools_cli_only` — line 346
10. `test_mcp_with_transport` — line 361
11. `test_platform_overrides` — line 376
12. `test_find_secret_refs_no_mcps` — line 401
13. `test_find_secret_refs_deduplicates` — line 408

Note: Spec says 14. Actual count is 13. Socrates flagged this as ADVISORY/off-by-one.

---

## Downstream Construction Sites — Exhaustive Map

All sites that must change when new fields are added to `AgentConfig` and `McpConfig`.

### `AgentConfig` construction sites (must add `..Default::default()`)

| File | Line | Code |
|------|------|------|
| `src/cli/init.rs` | 158–164 | `AgentConfig { provider: Some("anthropic"..), model: Some("claude-..") }` |
| `src/cli/init.rs` | 168–174 | `AgentConfig { provider: Some("openai"..), model: Some("codex-mini") }` |
| `src/cli/init.rs` | 178–184 | `AgentConfig { provider: Some("google"..), model: Some("gemini-..") }` |
| `src/cli/template.rs` | 464–469 | `AgentConfig { provider: Some("anthropic"..), model: Some("opus") }` (test) |
| `src/cli/template.rs` | 474–477 | `AgentConfig { provider: Some("anthropic"..), model: Some("sonnet") }` (test) |
| `src/cli/template.rs` | 491–494 | `AgentConfig { provider: Some("anthropic"..), model: None }` (test) |
| `src/cli/template.rs` | 500–503 | `AgentConfig { provider: Some("openai"..), model: Some("gpt-4") }` (test) |

**Total: 7 sites.** All use `{ provider, model }` struct literal without `..Default::default()`. Adding `api_key` and `enabled` fields requires adding `..Default::default()` to each.

### `McpConfig` construction sites (must add `enabled: None`)

| File | Line | Code |
|------|------|------|
| `src/cli/init.rs` | 199–210 | `McpConfig { command: "npx", args: Some(..), env: None, transport: None, url: None }` |
| `src/cli/template.rs` | 520–528 | `McpConfig { command: "existing-cmd", args: None, env: None, transport: None, url: None }` (test) |
| `src/cli/template.rs` | 534–540 | `McpConfig { command: "template-cmd", .. }` (test) |
| `src/cli/template.rs` | 544–550 | `McpConfig { command: "db-cmd", .. }` (test) |
| `src/mcp/mod.rs` | 142–148 | `McpConfig { command: "npx", args: Some(..), env: None, transport: None, url: None }` (test) |
| `src/mcp/mod.rs` | 168–175 | `McpConfig { command: "node", args: Some(..), env: Some(..), transport: None, url: None }` (test) |
| `src/mcp/mod.rs` | 212–218 | `McpConfig { command: "cmd-a", args: None, env: None, transport: None, url: None }` (test) |
| `src/mcp/mod.rs` | 219–225 | `McpConfig { command: "cmd-b", .. }` (test) |
| `src/mcp/mod.rs` | 249–256 | `McpConfig { command: "echo", args: Some(..), env: None, transport: None, url: None }` (test) |

**Total: 9 sites.** `McpConfig` cannot derive `Default` (non-optional `command: String`). Each literal needs `enabled: None` added.

### Read-only consumers (no changes needed)

These files call `config::load()` or read `GreatConfig` fields. They are unaffected by additive `Option` fields.

| File | What it accesses |
|------|-----------------|
| `src/cli/apply.rs` | `cfg.tools`, `cfg.mcp`, `cfg.secrets`, `cfg.platform` — field reads only |
| `src/cli/diff.rs` | `cfg.tools`, `cfg.mcp`, `cfg.secrets`, `cfg.find_secret_refs()` — field reads only |
| `src/cli/doctor.rs` | `config::schema::ConfigMessage::Warning/Error` — enum pattern match only (lines 468–471) |
| `src/cli/mcp.rs` | `crate::config::schema::McpConfig` — reads `command`, `args`, `env` only (line 188) |
| `src/cli/status.rs` | `config::load()` — calls load, reads result |
| `src/cli/sync.rs` | `toml::from_str::<crate::config::schema::GreatConfig>` — deserialization only (line 77) |
| `src/cli/template.rs` | `merge_configs()` reads all fields — struct spread via `or()`, no construction |
| `src/platform/runtime.rs` | `&crate::config::schema::ToolsConfig` — reads `runtimes` map (line 148) |
| `src/mcp/mod.rs` | `McpConfig` used in `add_server()`, `test_server()` — reads `command`, `args`, `env` |

---

## Crate Versions (Cargo.toml)

```toml
serde = { version = "1.0", features = ["derive"] }   # line 14
toml = "0.8"                                           # line 16
regex = "1.0"                                          # line 23
toml_edit = "0.22"                                     # line 24
```

All required crates are present. No new dependencies needed.

---

## Template Files (must remain parseable)

All four templates are embedded via `include_str!` in `src/cli/init.rs` and `src/cli/template.rs`. Existing tests in `init.rs` (`test_templates_parse_as_valid_config`, line 418) already verify they parse as `GreatConfig`.

| Template | Path | Fields used |
|----------|------|-------------|
| `ai-minimal` | `templates/ai-minimal.toml` | project, tools.cli, agents, secrets |
| `ai-fullstack-ts` | `templates/ai-fullstack-ts.toml` | project, tools (runtimes+cli), agents, mcp, secrets |
| `ai-fullstack-py` | `templates/ai-fullstack-py.toml` | project, tools (runtimes+cli), agents, mcp, secrets |
| `saas-multi-tenant` | `templates/saas-multi-tenant.toml` | project, tools, agents, mcp, secrets, platform.macos |

None use `version`, `api_key`, or `enabled` — additive changes are safe. These templates will continue to parse after the spec changes.

---

## Integration Tests

**File:** `/home/isaac/src/sh.great/tests/cli_smoke.rs`

Config-related integration tests:
- `apply_dry_run_with_config` (line 325) — writes `[tools.cli]\nripgrep = "latest"`, calls `great apply --dry-run`
- `apply_dry_run_shows_prerequisites` (line 347) — writes minimal config, checks stderr output
- `apply_no_config_fails` (line 365) — no config, expects failure
- `mcp_add_creates_entry` (line 199) — writes `[project]\nname = "test"`, calls `great mcp add filesystem`

None of these tests construct `AgentConfig` or `McpConfig` directly. No changes needed to integration tests.

**Unit tests in `config/mod.rs`** (5 tests, lines 77–119):
- All use `config::load()` with temporary TOML files
- No direct struct construction
- No changes needed

---

## Dependency Map

```
schema.rs (changes here)
    |
    +-- config/mod.rs (re-exports, unchanged)
            |
            +-- cli/init.rs     [AgentConfig x3, McpConfig x1 -- update literals]
            +-- cli/apply.rs    [reads fields -- no change]
            +-- cli/diff.rs     [reads fields -- no change]
            +-- cli/mcp.rs      [reads McpConfig -- no change]
            +-- cli/sync.rs     [deserializes GreatConfig -- no change]
            +-- cli/doctor.rs   [matches ConfigMessage -- no change]
            +-- cli/status.rs   [calls load() -- no change]
            +-- cli/template.rs [AgentConfig x4, McpConfig x3 in tests -- update literals]
            +-- mcp/mod.rs      [McpConfig x5 in tests -- update literals]
            +-- platform/runtime.rs [reads ToolsConfig -- no change]
```

---

## Risks

**Risk 1: Struct literal exhaustiveness** (LOW)
Rust struct literals without `..` are exhaustive. Adding `api_key` and `enabled` to `AgentConfig` and `enabled` to `McpConfig` will cause compile errors at all construction sites until updated. This is a feature: the compiler will identify every site that needs updating. Use `cargo build` after Step 1 to get a complete error list.

**Risk 2: `merge_configs` in `template.rs`** (LOW)
`merge_configs()` (line 280) uses struct spread to rebuild `GreatConfig`. It destructures `existing.tools` and `template.tools` into `ToolsConfig { runtimes, cli }`. Since `ToolsConfig` has no new fields, this is unaffected. The `agents` and `mcp` maps are merged via `e.entry(k).or_insert(v)` — the new fields in `AgentConfig` and `McpConfig` are preserved as-is from whichever config wins. No change needed.

**Risk 3: Test count in spec** (LOW)
Spec states 14 existing schema.rs tests; actual count is 13. Socrates flagged. This is documentation-only — no implementation risk.

**Risk 4: `sse` transport** (LOW)
Spec's transport whitelist is `["stdio", "http"]`. Socrates advises adding `"sse"`. Builder should add it to avoid false warnings for SSE-based MCP servers.

**Risk 5: `validate()` on disabled entries** (INFORMATIONAL)
Disabled agents/MCPs still get validated. Acceptable for this iteration per Socrates. Add a `// TODO` comment.

---

## Recommended Build Order

1. **Edit `schema.rs` — add fields and derives** (Step 1 of spec)
   - Add `Default` to `ProjectConfig`, `AgentConfig`, `SecretsConfig`, `PlatformOverride`
   - Add `version: Option<String>` to `ProjectConfig` (with `#[serde(skip_serializing_if = "Option::is_none")]`)
   - Add `api_key: Option<String>` and `enabled: Option<bool>` to `AgentConfig`
   - Add `enabled: Option<bool>` to `McpConfig`
   - Run `cargo build` — compiler will flag all construction sites

2. **Fix construction sites** (mandated by Step 4 of spec)
   - 7 `AgentConfig` sites: add `..Default::default()`
   - 9 `McpConfig` sites: add `enabled: None`
   - Verify: `cargo build` passes

3. **Expand `find_secret_refs()`** (Step 2 of spec)
   - Add agent `api_key` scan before MCP scan
   - Verify: existing 13 schema.rs tests still pass

4. **Strengthen `validate()`** (Step 3 of spec)
   - Add 5 new validation checks (MCP empty command, unknown transport, http+no-url, unknown secrets provider, unknown agent provider)
   - Add `"sse"` to transport whitelist per Socrates advisory
   - Verify: all existing tests pass

5. **Add 19 new tests** (Step 5 of spec)
   - All test bodies are fully specified in the spec
   - Verify: `cargo test --lib config` passes

6. **Full verification**
   - `cargo test` — all tests pass
   - `cargo clippy -- -D warnings` — zero warnings

---

## Patterns to Follow

- `#[serde(skip_serializing_if = "Option::is_none")]` is the project pattern for optional fields (see `McpConfig.transport`, `McpConfig.url` at lines 92–95).
- All `Option` fields default to `None` on deserialization — no `#[serde(default)]` needed at field level.
- `Regex::new(...).expect("valid regex")` is the project pattern for compile-time-constant regexes (see `find_secret_refs()` line 179 and `apply.rs::resolve_secret_refs` line 925).
- Secret regex pattern is `r"\$\{([A-Z_][A-Z0-9_]*)\}"` — used consistently in `schema.rs`, `apply.rs`, and `mcp/mod.rs`.
- `ConfigMessage::Warning(String)` / `ConfigMessage::Error(String)` — existing error enum, no changes needed.

---

## Technical Debt Flagged

- `apply.rs` duplicates `resolve_secret_refs()` (line 924) which is also in `mcp/mod.rs::resolve_env()` (line 79). Both implement the same regex substitution logic. Not in scope for this task but should be unified in a future iteration.
- The regex is compiled inside every call to `find_secret_refs()` and `resolve_secret_refs()`. Should be a `once_cell::Lazy` or `std::sync::OnceLock` in a future iteration.
