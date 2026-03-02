# 0002: TOML Config Parser -- great.toml Schema

**Priority:** P0 (foundation)
**Type:** feature
**Module:** `src/config/`
**Status:** pending

## Context

The current `src/config/schema.rs` defines a minimal `GreatConfig` with flat `Vec` fields for tools, agents, and mcp. The TOML structure needs to match the product spec's section-based design (`[project]`, `[platform]`, `[tools]`, `[agents.*]`, `[mcp.*]`, `[secrets]`) using proper serde deserialization with HashMap-based named entries for agents and MCP servers.

The existing `src/config/mod.rs` already has working `load()`, `discover_config()`, `data_dir()`, and `config_dir()` functions. The schema upgrade must remain compatible with the existing `load()` pipeline (read file, `toml::from_str`).

The legacy repo at `/home/isaac/src/great-sh/src/core/config.rs` used JSON-based config with a different structure. The new schema is TOML-first and should not follow the legacy layout -- it should follow the `great.toml` spec.

## Requirements

1. **Define the complete `GreatConfig` struct** with serde `Deserialize` and `Serialize` derives. Top-level sections: `project: Option<ProjectConfig>`, `platform: Option<PlatformConfig>`, `tools: Option<ToolsConfig>`, `agents: Option<HashMap<String, AgentConfig>>`, `mcp: Option<HashMap<String, McpConfig>>`, `secrets: Option<SecretsConfig>`. Each section is optional so partial configs are valid.

2. **Define section structs**: `ProjectConfig` (name, version, description), `PlatformConfig` with optional sub-tables `macos: Option<MacOSConfig>` and `wsl2: Option<Wsl2Config>`, `ToolsConfig` with `cli: Option<Vec<ToolEntry>>` for declarative tool lists, `AgentConfig` (provider, model, api_key reference, enabled), `McpConfig` (command, args, env HashMap, enabled), `SecretsConfig` (provider, path). All string fields that may contain `${SECRET_NAME}` references should be typed as `String`.

3. **Implement validation on parse**: after deserialization, run a `validate()` method on `GreatConfig` that returns `anyhow::Result<Vec<String>>` where `Ok(warnings)` means valid with optional warnings (e.g., unknown keys logged as warnings) and `Err` means a required field is missing or invalid with an actionable error message (e.g., "agents.claude: missing 'provider' field -- set to 'anthropic' or 'openai'").

4. **Detect `${SECRET_NAME}` variable references**: add a `find_secret_refs(&self) -> Vec<String>` method that scans all string fields and returns a deduplicated list of secret names referenced. This does not resolve them -- just identifies them for the vault module to resolve later. Use the `regex` crate (already in deps) with pattern `\$\{([A-Z_][A-Z0-9_]*)\}`.

5. **Provide sensible defaults**: implement `Default` for all config structs so that `GreatConfig::default()` produces a minimal valid config. The `load()` function in `mod.rs` should continue to work unchanged -- schema changes are internal to the struct definitions.

## Acceptance Criteria

- [ ] A sample `great.toml` with all sections can be parsed into `GreatConfig` and serialized back to TOML without data loss (round-trip test).
- [ ] Parsing a TOML file with a missing optional section (e.g., no `[secrets]`) succeeds and returns `None` for that field.
- [ ] Parsing a malformed TOML file (syntax error) returns a clear `toml::de::Error` message, not a panic.
- [ ] `find_secret_refs()` correctly extracts `["ANTHROPIC_KEY", "OPENAI_KEY"]` from a config containing `api_key = "${ANTHROPIC_KEY}"` and `api_key = "${OPENAI_KEY}"`.
- [ ] Unit tests cover: valid full config, valid partial config (project-only), invalid TOML syntax, and secret reference extraction.

## Dependencies

- None directly, but task 0001 (platform detection) informs which `PlatformConfig` sub-tables are meaningful.
- Existing `src/config/mod.rs` functions (`load`, `discover_config`, `data_dir`, `config_dir`) must continue to compile and work.

## Notes

- The current schema at `src/config/schema.rs` uses `Vec<ToolConfig>` etc. This needs to change to the HashMap/section-based approach. This is a breaking change to the struct layout but since all subcommands are stubs (`not yet implemented`), there are no downstream consumers to migrate.
- The `toml` crate (0.8) supports `#[serde(deny_unknown_fields)]` but this is too strict for forward compatibility. Instead, use the default (ignore unknown fields) and log warnings in the `validate()` method.
- Keep the `McpConfig` struct compatible with the MCP server JSON format used by Claude and other AI tools -- the `command`, `args`, `env` pattern is standard.
- The `regex` crate is already in `Cargo.toml` dependencies.
