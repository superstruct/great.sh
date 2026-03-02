# Spec 0002: TOML Config Parser -- great.toml Schema

**Task:** `.tasks/backlog/0002-config-schema.md`
**Status:** ready
**Type:** feature (foundation)
**Estimated Complexity:** M (single module, schema already partially implemented, enrichment + validation + tests)

---

## Summary

The `great.toml` config schema in `/home/isaac/src/sh.great/src/config/schema.rs` is already substantially implemented. The structs (`GreatConfig`, `ProjectConfig`, `ToolsConfig`, `AgentConfig`, `McpConfig`, `SecretsConfig`, `PlatformConfig`, `PlatformOverride`), validation (`validate()`), and secret-reference extraction (`find_secret_refs()`) are all present and tested. The `load()` pipeline in `/home/isaac/src/sh.great/src/config/mod.rs` works correctly with the current schema.

This spec defines the remaining enrichments needed to bring the schema to completion:

1. **Add `api_key` field to `AgentConfig`** -- agents may reference secrets via `api_key = "${ANTHROPIC_API_KEY}"`, but the field is missing from the struct.
2. **Add `enabled` field to `AgentConfig` and `McpConfig`** -- allow users to disable agents/servers without removing them from config.
3. **Add `version` field to `ProjectConfig`** -- the task requires it.
4. **Expand `find_secret_refs()` to scan `AgentConfig.api_key`** -- currently only scans MCP env values.
5. **Add `Default` impls for structs that lack them** -- `ProjectConfig`, `AgentConfig`, `McpConfig`, `SecretsConfig`, `PlatformOverride`.
6. **Strengthen `validate()`** -- add MCP command validation, transport validation, provider whitelist warnings.
7. **Add new unit tests** -- round-trip with all new fields, `enabled = false` behavior, `api_key` secret scanning, edge cases.

All changes are **additive** (new `Option` fields with serde defaults). No existing TOML files break. No downstream consumers need modification because every new field is `Option` with `#[serde(default)]` or `#[serde(skip_serializing_if)]`.

---

## Files to Modify

| File | Change |
|------|--------|
| `/home/isaac/src/sh.great/src/config/schema.rs` | Add fields to structs, `Default` impls, expand `validate()` and `find_secret_refs()`, add tests |

No new files are created. No changes to `/home/isaac/src/sh.great/src/config/mod.rs` -- the `load()` pipeline is unchanged.

---

## Interfaces

### Full Struct Definitions

Every struct below is the **complete** definition after this spec is applied. Fields marked `[NEW]` are additions; all others are unchanged.

#### `GreatConfig`

```rust
/// Top-level great.toml configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GreatConfig {
    pub project: Option<ProjectConfig>,
    pub tools: Option<ToolsConfig>,
    pub agents: Option<HashMap<String, AgentConfig>>,
    pub mcp: Option<HashMap<String, McpConfig>>,
    pub secrets: Option<SecretsConfig>,
    pub platform: Option<PlatformConfig>,
}
```

No changes. Already derives `Default`.

#### `ProjectConfig`

```rust
/// Project metadata section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    pub name: Option<String>,
    /// [NEW] Project version string (e.g., "1.0.0"). Informational only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub description: Option<String>,
}
```

Changes:
- Add `Default` derive (was missing).
- Add `version: Option<String>` field with `skip_serializing_if`.

#### `ToolsConfig`

```rust
/// Tools section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsConfig {
    #[serde(flatten)]
    pub runtimes: HashMap<String, String>,
    pub cli: Option<HashMap<String, String>>,
}
```

No changes.

#### `AgentConfig`

```rust
/// Configuration for a named AI agent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    /// [NEW] API key or secret reference (e.g., "${ANTHROPIC_API_KEY}").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// [NEW] Whether this agent is active. Defaults to true when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}
```

Changes:
- Add `Default` derive (was missing).
- Add `api_key: Option<String>`.
- Add `enabled: Option<bool>`.

Both fields are `Option` so existing TOML files without them parse correctly. `enabled` defaults to `None` (treated as `true` by consumers).

#### `McpConfig`

```rust
/// Configuration for a named MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub transport: Option<String>,
    pub url: Option<String>,
    /// [NEW] Whether this MCP server is active. Defaults to true when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}
```

Changes:
- Add `enabled: Option<bool>`.

Note: `McpConfig` cannot derive `Default` because `command: String` would produce an empty string, which is semantically invalid. Instead, implement `Default` manually to make downstream code (like `merge_configs` in `template.rs`) continue to compile if needed. However, since `McpConfig` is always constructed with an explicit `command` value in all existing code, we do **not** add `Default` -- the compiler enforces that `command` is always provided. Existing code in `init.rs` and `template.rs` constructs `McpConfig` with explicit field values and does not rely on `Default`.

#### `SecretsConfig`

```rust
/// Secret and credential management configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecretsConfig {
    pub provider: Option<String>,
    pub required: Option<Vec<String>>,
}
```

Changes:
- Add `Default` derive (was missing).

#### `PlatformConfig`

```rust
/// Platform-specific override container.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformConfig {
    pub macos: Option<PlatformOverride>,
    pub wsl2: Option<PlatformOverride>,
    pub linux: Option<PlatformOverride>,
}
```

No changes. Already derives `Default`.

#### `PlatformOverride`

```rust
/// Platform-specific overrides.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformOverride {
    pub extra_tools: Option<Vec<String>>,
}
```

Changes:
- Add `Default` derive (was missing).

#### `ConfigMessage`

```rust
/// A validation message.
#[derive(Debug, Clone)]
pub enum ConfigMessage {
    Warning(String),
    Error(String),
}
```

No changes.

---

## Implementation Approach

### Build Order

All changes are in `/home/isaac/src/sh.great/src/config/schema.rs`. Apply in this sequence:

**Step 1: Add new fields to structs**

Add `version` to `ProjectConfig`, `api_key` and `enabled` to `AgentConfig`, `enabled` to `McpConfig`. Add `Default` derives to `ProjectConfig`, `AgentConfig`, `SecretsConfig`, `PlatformOverride`. Add `#[serde(skip_serializing_if = "Option::is_none")]` to all new `Option` fields.

Verify: `cargo build` passes. All existing tests pass. All embedded templates (`templates/*.toml`) still parse.

**Step 2: Expand `find_secret_refs()` to scan agent `api_key` fields**

Currently `find_secret_refs()` only scans `mcp.*.env` values. After this change it also scans `agents.*.api_key` values for `${SECRET_NAME}` patterns.

Updated method body:

```rust
pub fn find_secret_refs(&self) -> Vec<String> {
    let mut refs = Vec::new();
    let re = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").expect("valid regex");

    // Scan agent api_key fields
    if let Some(agents) = &self.agents {
        for agent in agents.values() {
            if let Some(api_key) = &agent.api_key {
                for cap in re.captures_iter(api_key) {
                    refs.push(cap[1].to_string());
                }
            }
        }
    }

    // Scan MCP env values
    if let Some(mcps) = &self.mcp {
        for mcp in mcps.values() {
            if let Some(env) = &mcp.env {
                for value in env.values() {
                    for cap in re.captures_iter(value) {
                        refs.push(cap[1].to_string());
                    }
                }
            }
        }
    }

    refs.sort();
    refs.dedup();
    refs
}
```

Verify: existing `find_secret_refs` tests pass, new tests for agent scanning pass.

**Step 3: Strengthen `validate()`**

Add these checks to the existing `validate()` method, after the current checks:

```rust
// Check: MCP servers must have a non-empty command
if let Some(mcps) = &self.mcp {
    for (name, mcp) in mcps {
        if mcp.command.trim().is_empty() {
            messages.push(ConfigMessage::Error(format!(
                "mcp '{}': 'command' must not be empty",
                name
            )));
        }
        // Check: if transport is specified, it must be "stdio" or "http"
        if let Some(transport) = &mcp.transport {
            if transport != "stdio" && transport != "http" {
                messages.push(ConfigMessage::Warning(format!(
                    "mcp '{}': unknown transport '{}' -- expected 'stdio' or 'http'",
                    name, transport
                )));
            }
        }
        // Check: http transport requires a url
        if mcp.transport.as_deref() == Some("http") && mcp.url.is_none() {
            messages.push(ConfigMessage::Error(format!(
                "mcp '{}': transport 'http' requires a 'url' field",
                name
            )));
        }
    }
}

// Check: if secrets.provider is set, warn on unknown providers
if let Some(secrets) = &self.secrets {
    if let Some(provider) = &secrets.provider {
        let known = ["env", "1password", "bitwarden", "keychain"];
        if !known.contains(&provider.as_str()) {
            messages.push(ConfigMessage::Warning(format!(
                "secrets: unknown provider '{}' -- known providers: {}",
                provider,
                known.join(", ")
            )));
        }
    }
}

// Check: agent provider whitelist (warning, not error)
if let Some(agents) = &self.agents {
    for (name, agent) in agents {
        if let Some(provider) = &agent.provider {
            let known = ["anthropic", "openai", "google"];
            if !known.contains(&provider.as_str()) {
                messages.push(ConfigMessage::Warning(format!(
                    "agent '{}': unknown provider '{}' -- known providers: {}",
                    name, provider, known.join(", ")
                )));
            }
        }
    }
}
```

These are appended after the existing agent/secret checks. Existing validation logic is untouched.

Verify: all existing validate tests pass. New validation tests pass.

**Step 4: Update existing `init.rs` agent construction**

The `init.rs` file constructs `AgentConfig` with explicit fields `{ provider, model }`. After adding `api_key` and `enabled` to the struct, these constructions will fail to compile **only if we do NOT derive `Default`** or add `..Default::default()`. Since we are adding `Default` to `AgentConfig`, the existing code must be updated to use struct update syntax:

```rust
AgentConfig {
    provider: Some("anthropic".to_string()),
    model: Some("claude-sonnet-4-20250514".to_string()),
    ..Default::default()
}
```

This applies to all `AgentConfig` constructions in:
- `/home/isaac/src/sh.great/src/cli/init.rs` (3 occurrences: lines 160-163, 170-173, 180-183)

Similarly, `McpConfig` constructions in `init.rs` (line 199-208) need `enabled: None` added since `McpConfig` does not derive `Default`:

```rust
McpConfig {
    command: "npx".to_string(),
    args: Some(vec![...]),
    env: None,
    transport: None,
    url: None,
    enabled: None,  // [NEW]
}
```

And in `template.rs`, `merge_configs` constructs `ToolsConfig` (line 298) -- this is unchanged since `ToolsConfig` has no new fields.

**Step 5: Add tests**

Add these new test functions to the `#[cfg(test)] mod tests` block in `schema.rs`.

Verify: `cargo test` passes.

---

## Downstream Compatibility

All downstream consumers of the config structs have been audited. Here is the complete list and the impact:

| Consumer | File | Impact |
|----------|------|--------|
| `init.rs` | `AgentConfig` construction (3 sites), `McpConfig` construction (1 site) | Must add `..Default::default()` to `AgentConfig` sites and `enabled: None` to `McpConfig` site |
| `template.rs` | `merge_configs()`, test `AgentConfig` (4 sites), test `McpConfig` (3 sites) | Must add `..Default::default()` to `AgentConfig` sites and `enabled: None` to `McpConfig` sites |
| `mcp.rs` (cli) | Reads `McpConfig` fields | No impact -- only reads `command`, `args`, `env` |
| `mcp/mod.rs` | `McpConfig` construction in tests (5 sites) | Must add `enabled: None` to each `McpConfig { ... }` literal in test code |
| `status.rs`, `diff.rs`, `apply.rs`, `doctor.rs`, `sync.rs` | Call `config::load()` | No impact -- schema changes are additive `Option` fields |
| `platform/runtime.rs` | Reads `ToolsConfig.runtimes` | No impact -- `ToolsConfig` unchanged |

**Strategy for `AgentConfig` construction sites:** Since there are many construction sites in `template.rs` tests (~15 places), the most efficient approach is:

1. Derive `Default` on `AgentConfig` (already planned).
2. Add `..Default::default()` to every `AgentConfig { ... }` literal in `init.rs` and `template.rs` tests.

**Strategy for `McpConfig` construction sites:** Since `McpConfig` has `command: String` (non-optional), we cannot derive `Default`. Instead, add `enabled: None` to every `McpConfig { ... }` literal. There are few of these (1 in `init.rs`, potentially a few in `template.rs` tests).

---

## Validation Logic (Complete)

After this spec, `validate()` performs these checks in order:

| # | Check | Severity | Message |
|---|-------|----------|---------|
| 1 | Agent with no `provider` and no `model` | Warning | `"agent '{name}' has no provider or model specified"` |
| 2 | Secret name with non-alphanumeric/underscore chars | Error | `"invalid secret name '{key}': must be alphanumeric with underscores"` |
| 3 | MCP server with empty `command` | Error | `"mcp '{name}': 'command' must not be empty"` |
| 4 | MCP server with unknown `transport` value | Warning | `"mcp '{name}': unknown transport '{val}' -- expected 'stdio' or 'http'"` |
| 5 | MCP server with `transport = "http"` but no `url` | Error | `"mcp '{name}': transport 'http' requires a 'url' field"` |
| 6 | Unknown secrets provider | Warning | `"secrets: unknown provider '{val}' -- known providers: env, 1password, bitwarden, keychain"` |
| 7 | Unknown agent provider | Warning | `"agent '{name}': unknown provider '{val}' -- known providers: anthropic, openai, google"` |

Checks 1-2 exist today. Checks 3-7 are new.

The `load()` function in `mod.rs` iterates messages: warnings print to stderr via `eprintln!`, errors bail via `anyhow::bail!`. This behavior is unchanged.

---

## Secret Reference Extraction (Complete)

### Regex Pattern

```
\$\{([A-Z_][A-Z0-9_]*)\}
```

Matches `${UPPERCASE_NAME}` patterns. The capture group extracts the name without the `${}` wrapper.

### Scan Locations

After this spec, `find_secret_refs()` scans:

1. `agents.*.api_key` -- agent API key references (NEW)
2. `mcp.*.env.*` -- MCP environment variable values (existing)

### Return Value

`Vec<String>` -- sorted, deduplicated list of secret names.

### Example

Given this TOML:

```toml
[agents.claude]
provider = "anthropic"
api_key = "${ANTHROPIC_API_KEY}"

[agents.codex]
provider = "openai"
api_key = "${OPENAI_API_KEY}"

[mcp.postgres]
command = "npx"
env = { DATABASE_URL = "${POSTGRES_URL}", API_KEY = "${ANTHROPIC_API_KEY}" }
```

`find_secret_refs()` returns: `["ANTHROPIC_API_KEY", "OPENAI_API_KEY", "POSTGRES_URL"]`

Note: `ANTHROPIC_API_KEY` appears in both an agent and an MCP env value, but is deduplicated.

---

## Edge Cases

### Empty / Partial Configs

| Input | Expected Result |
|-------|-----------------|
| Empty string `""` | `GreatConfig::default()` -- all fields `None` |
| Only `[project]` section | `project = Some(...)`, all others `None` |
| Only `[tools.cli]` (no runtime keys) | `tools = Some(ToolsConfig { runtimes: {}, cli: Some({...}) })` |
| Only `[agents.x]` with no fields | Parses as `AgentConfig::default()`, validation warns "no provider or model" |
| `enabled = false` on agent | Parses correctly, consumers check `agent.enabled.unwrap_or(true)` |
| `enabled = false` on MCP server | Parses correctly, consumers check `mcp.enabled.unwrap_or(true)` |

### Invalid TOML

| Input | Expected Result |
|-------|-----------------|
| `"this is not [[[ toml"` | `toml::de::Error` with line/column info, propagated by `load()` as `anyhow::Error` |
| `[mcp.x]\ncommand = 42` | `toml::de::Error` -- type mismatch (expected string, got integer) |
| Unknown top-level key `[foo]` | Silently ignored by serde (no `deny_unknown_fields`), forward compatible |
| Unknown field inside a known section | Silently ignored by serde |

### Secret References

| Input | Expected `find_secret_refs()` |
|-------|-------------------------------|
| No agents, no MCPs | `[]` |
| `api_key = "sk-literal-key"` (no `${}`) | `[]` |
| `api_key = "${}"` (empty name) | `[]` -- regex requires `[A-Z_]` as first char |
| `api_key = "${lower_case}"` | `[]` -- regex requires uppercase |
| `env = { X = "${A}${B}" }` (multiple refs in one value) | `["A", "B"]` |
| Same ref in 3 places | Deduplicated to 1 entry |

### Platform Differences

No platform-specific behavior in the config parser. The `PlatformConfig` struct describes overrides that consumers apply per-platform, but the parser itself is platform-agnostic. Works identically on macOS ARM64/x86_64, Ubuntu, and WSL2.

---

## Error Handling

All errors use `anyhow::Result` propagation through `load()`. No `.unwrap()` in production code.

| Error Source | Handling |
|--------------|----------|
| File not found | `std::fs::read_to_string` returns `io::Error`, propagated via `?` |
| TOML syntax error | `toml::from_str` returns `toml::de::Error` with line/column, propagated via `?` |
| TOML type mismatch | Same as above -- serde reports expected vs. actual type |
| Validation error (fatal) | `load()` calls `anyhow::bail!("config error in {path}: {msg}")` |
| Validation warning | `load()` prints to stderr via `eprintln!("config warning: {msg}")` |
| `Regex::new` failure | Uses `.expect("valid regex")` -- acceptable because the regex is a compile-time constant |

---

## Security Considerations

1. **Secret references are not resolved in this module.** The `find_secret_refs()` method only identifies `${SECRET_NAME}` patterns. Resolution (looking up actual secret values) is the vault module's responsibility. This prevents accidental secret leakage through config serialization.

2. **No secret values in TOML.** The `api_key` field is designed for reference strings like `"${ANTHROPIC_API_KEY}"`, not literal API keys. If a user puts a literal key, `find_secret_refs()` will return an empty list for that field (no `${}` pattern match), which is correct -- the vault is not involved.

3. **Round-trip serialization safety.** `toml::to_string` will serialize `api_key` values as-is. If a user accidentally puts a real secret in the TOML file, that is a user error, not a parser bug. The `great init` wizard does not prompt for actual secret values.

4. **No `deny_unknown_fields`.** This is intentional for forward compatibility -- newer versions of `great.toml` may have fields that older CLI versions do not know about. Unknown fields are silently ignored rather than causing parse failures.

---

## Testing Strategy

### Existing Tests (must continue to pass)

All 14 existing tests in `schema.rs` must pass unchanged:
- `test_parse_minimal_config`
- `test_parse_full_config`
- `test_parse_empty_config`
- `test_find_secret_refs`
- `test_validate_warns_on_empty_agent`
- `test_validate_invalid_secret_name`
- `test_validate_valid_config_no_messages`
- `test_roundtrip_serialize`
- `test_tools_cli_only`
- `test_mcp_with_transport`
- `test_platform_overrides`
- `test_find_secret_refs_no_mcps`
- `test_find_secret_refs_deduplicates`

All 5 existing tests in `mod.rs` must pass unchanged:
- `test_load_valid_config`
- `test_load_missing_file_errors`
- `test_load_invalid_toml_errors`
- `test_data_dir_returns_path`
- `test_config_dir_returns_path`

### New Tests (add to `schema.rs`)

#### `test_agent_api_key_parse`

```rust
#[test]
fn test_agent_api_key_parse() {
    let toml_str = r#"
[agents.claude]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key = "${ANTHROPIC_API_KEY}"
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let agent = config.agents.unwrap().remove("claude").unwrap();
    assert_eq!(agent.api_key.as_deref(), Some("${ANTHROPIC_API_KEY}"));
}
```

#### `test_agent_enabled_field`

```rust
#[test]
fn test_agent_enabled_field() {
    let toml_str = r#"
[agents.claude]
provider = "anthropic"
enabled = true

[agents.codex]
provider = "openai"
enabled = false
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let agents = config.agents.unwrap();
    assert_eq!(agents["claude"].enabled, Some(true));
    assert_eq!(agents["codex"].enabled, Some(false));
}
```

#### `test_agent_enabled_absent_is_none`

```rust
#[test]
fn test_agent_enabled_absent_is_none() {
    let toml_str = r#"
[agents.claude]
provider = "anthropic"
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let agents = config.agents.unwrap();
    assert_eq!(agents["claude"].enabled, None);
}
```

#### `test_mcp_enabled_field`

```rust
#[test]
fn test_mcp_enabled_field() {
    let toml_str = r#"
[mcp.filesystem]
command = "npx"
enabled = false
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let mcps = config.mcp.unwrap();
    assert_eq!(mcps["filesystem"].enabled, Some(false));
}
```

#### `test_project_version_field`

```rust
#[test]
fn test_project_version_field() {
    let toml_str = r#"
[project]
name = "my-project"
version = "2.1.0"
description = "A test project"
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let project = config.project.unwrap();
    assert_eq!(project.version.as_deref(), Some("2.1.0"));
}
```

#### `test_find_secret_refs_from_agent_api_key`

```rust
#[test]
fn test_find_secret_refs_from_agent_api_key() {
    let toml_str = r#"
[agents.claude]
provider = "anthropic"
api_key = "${ANTHROPIC_API_KEY}"

[agents.codex]
provider = "openai"
api_key = "${OPENAI_API_KEY}"
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let refs = config.find_secret_refs();
    assert_eq!(refs, vec!["ANTHROPIC_API_KEY", "OPENAI_API_KEY"]);
}
```

#### `test_find_secret_refs_combined_agents_and_mcp`

```rust
#[test]
fn test_find_secret_refs_combined_agents_and_mcp() {
    let toml_str = r#"
[agents.claude]
api_key = "${ANTHROPIC_API_KEY}"

[mcp.db]
command = "npx"
env = { URL = "${POSTGRES_URL}", KEY = "${ANTHROPIC_API_KEY}" }
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let refs = config.find_secret_refs();
    // ANTHROPIC_API_KEY appears in both agent and mcp, deduplicated
    assert_eq!(refs, vec!["ANTHROPIC_API_KEY", "POSTGRES_URL"]);
}
```

#### `test_find_secret_refs_literal_api_key_no_match`

```rust
#[test]
fn test_find_secret_refs_literal_api_key_no_match() {
    let toml_str = r#"
[agents.claude]
provider = "anthropic"
api_key = "sk-ant-literal-key-not-a-reference"
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let refs = config.find_secret_refs();
    assert!(refs.is_empty());
}
```

#### `test_validate_mcp_empty_command`

```rust
#[test]
fn test_validate_mcp_empty_command() {
    let toml_str = r#"
[mcp.broken]
command = ""
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let messages = config.validate();
    let has_error = messages.iter().any(|m| match m {
        ConfigMessage::Error(e) => e.contains("command") && e.contains("broken"),
        _ => false,
    });
    assert!(has_error, "expected error for empty command: {:?}", messages);
}
```

#### `test_validate_mcp_unknown_transport`

```rust
#[test]
fn test_validate_mcp_unknown_transport() {
    let toml_str = r#"
[mcp.weird]
command = "test"
transport = "grpc"
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let messages = config.validate();
    let has_warning = messages.iter().any(|m| match m {
        ConfigMessage::Warning(w) => w.contains("grpc"),
        _ => false,
    });
    assert!(has_warning, "expected warning for unknown transport: {:?}", messages);
}
```

#### `test_validate_mcp_http_requires_url`

```rust
#[test]
fn test_validate_mcp_http_requires_url() {
    let toml_str = r#"
[mcp.remote]
command = "test"
transport = "http"
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let messages = config.validate();
    let has_error = messages.iter().any(|m| match m {
        ConfigMessage::Error(e) => e.contains("http") && e.contains("url"),
        _ => false,
    });
    assert!(has_error, "expected error for http without url: {:?}", messages);
}
```

#### `test_validate_unknown_secrets_provider`

```rust
#[test]
fn test_validate_unknown_secrets_provider() {
    let toml_str = r#"
[secrets]
provider = "hashicorp-vault"
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let messages = config.validate();
    let has_warning = messages.iter().any(|m| match m {
        ConfigMessage::Warning(w) => w.contains("hashicorp-vault"),
        _ => false,
    });
    assert!(has_warning, "expected warning for unknown secrets provider: {:?}", messages);
}
```

#### `test_validate_unknown_agent_provider`

```rust
#[test]
fn test_validate_unknown_agent_provider() {
    let toml_str = r#"
[agents.custom]
provider = "azure-openai"
model = "gpt-4"
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let messages = config.validate();
    let has_warning = messages.iter().any(|m| match m {
        ConfigMessage::Warning(w) => w.contains("azure-openai"),
        _ => false,
    });
    assert!(has_warning, "expected warning for unknown agent provider: {:?}", messages);
}
```

#### `test_validate_known_providers_no_warnings`

```rust
#[test]
fn test_validate_known_providers_no_warnings() {
    let toml_str = r#"
[agents.a]
provider = "anthropic"

[agents.b]
provider = "openai"

[agents.c]
provider = "google"

[secrets]
provider = "env"
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let messages = config.validate();
    assert!(messages.is_empty(), "known providers should produce no warnings: {:?}", messages);
}
```

#### `test_roundtrip_with_new_fields`

```rust
#[test]
fn test_roundtrip_with_new_fields() {
    let toml_str = r#"
[project]
name = "roundtrip"
version = "1.0.0"

[agents.claude]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key = "${ANTHROPIC_API_KEY}"
enabled = true

[mcp.fs]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem"]
enabled = false
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let serialized = toml::to_string(&config).unwrap();
    let config2: GreatConfig = toml::from_str(&serialized).unwrap();

    let project2 = config2.project.unwrap();
    assert_eq!(project2.version.as_deref(), Some("1.0.0"));

    let agents2 = config2.agents.unwrap();
    assert_eq!(agents2["claude"].api_key.as_deref(), Some("${ANTHROPIC_API_KEY}"));
    assert_eq!(agents2["claude"].enabled, Some(true));

    let mcps2 = config2.mcp.unwrap();
    assert_eq!(mcps2["fs"].enabled, Some(false));
}
```

#### `test_agent_default`

```rust
#[test]
fn test_agent_default() {
    let agent = AgentConfig::default();
    assert!(agent.provider.is_none());
    assert!(agent.model.is_none());
    assert!(agent.api_key.is_none());
    assert!(agent.enabled.is_none());
}
```

#### `test_project_default`

```rust
#[test]
fn test_project_default() {
    let project = ProjectConfig::default();
    assert!(project.name.is_none());
    assert!(project.version.is_none());
    assert!(project.description.is_none());
}
```

#### `test_secrets_default`

```rust
#[test]
fn test_secrets_default() {
    let secrets = SecretsConfig::default();
    assert!(secrets.provider.is_none());
    assert!(secrets.required.is_none());
}
```

#### `test_find_secret_refs_multiple_in_one_value`

```rust
#[test]
fn test_find_secret_refs_multiple_in_one_value() {
    let toml_str = r#"
[mcp.combo]
command = "test"
env = { CONN = "postgres://${DB_USER}:${DB_PASS}@localhost" }
"#;
    let config: GreatConfig = toml::from_str(toml_str).unwrap();
    let refs = config.find_secret_refs();
    assert_eq!(refs, vec!["DB_PASS", "DB_USER"]);
}
```

Total: 19 new tests.

---

## Sample `great.toml` (Complete)

This sample exercises every section and field defined by this spec:

```toml
[project]
name = "my-saas-app"
version = "1.0.0"
description = "Multi-tenant SaaS with AI agents"

[tools]
node = "22"
python = "3.12"
deno = "latest"

[tools.cli]
ripgrep = "latest"
fd-find = "latest"
bat = "latest"
jq = "latest"
gh = "latest"
pnpm = "latest"
uv = "latest"
starship = "latest"
aws = "latest"
cdk = "latest"

[agents.claude]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key = "${ANTHROPIC_API_KEY}"
enabled = true

[agents.codex]
provider = "openai"
model = "codex-mini"
api_key = "${OPENAI_API_KEY}"
enabled = true

[agents.gemini]
provider = "google"
model = "gemini-2.5-pro"
api_key = "${GOOGLE_API_KEY}"
enabled = false

[mcp.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "."]
enabled = true

[mcp.postgres]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-postgres"]
env = { DATABASE_URL = "${POSTGRES_URL}" }
enabled = true

[mcp.remote-api]
command = "great-mcp"
transport = "http"
url = "http://localhost:8080"
enabled = true

[secrets]
provider = "env"
required = ["ANTHROPIC_API_KEY", "OPENAI_API_KEY", "GOOGLE_API_KEY", "POSTGRES_URL"]

[platform.macos]
extra_tools = ["coreutils", "gnu-sed"]

[platform.wsl2]
extra_tools = ["wslu"]

[platform.linux]
extra_tools = ["build-essential"]
```

---

## Verification Checklist

After implementation, run these commands from the project root:

```bash
# 1. All unit tests pass (schema.rs + mod.rs tests)
cargo test --lib config

# 2. All integration tests pass (templates still parse)
cargo test

# 3. Clippy clean
cargo clippy -- -D warnings

# 4. Build succeeds on all targets
cargo build
```

The builder should verify that:
- [ ] `cargo test` passes with 0 failures
- [ ] All 14 existing `schema.rs` tests pass unmodified
- [ ] All 19 new tests pass
- [ ] All 5 `mod.rs` tests pass unmodified
- [ ] All embedded template TOML files still parse (tested by `test_templates_parse_as_valid_config` in `init.rs`)
- [ ] `cargo clippy` produces no warnings
- [ ] `find_secret_refs()` returns `["ANTHROPIC_API_KEY", "OPENAI_API_KEY"]` for a config with those refs in agent `api_key` fields
- [ ] `validate()` returns errors for empty MCP commands and HTTP transport without URL
- [ ] `validate()` returns warnings (not errors) for unknown providers
