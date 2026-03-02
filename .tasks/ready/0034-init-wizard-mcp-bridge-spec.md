# Spec 0034: Wire mcp-bridge into the init wizard and built-in templates

**Status:** Ready
**Complexity:** S (small)
**Author:** Lovelace (Spec Writer)
**Date:** 2026-02-28

## Summary

The `great mcp-bridge` feature -- a pure-Rust, zero-Node MCP bridge to five AI backends -- is invisible to new users. Neither the interactive init wizard nor the four built-in templates emit a `[mcp-bridge]` stanza. This spec adds an opt-in wizard prompt and appends the correct `[mcp-bridge]` section to each template file.

No changes to `src/config/schema.rs` or `src/cli/apply.rs` are required. `McpBridgeConfig` already deserializes from `[mcp-bridge]` via `#[serde(rename = "mcp-bridge")]`, and `apply.rs` already reads `cfg.mcp_bridge` at line 727.

## Files to modify

| File | Change |
|------|--------|
| `src/cli/init.rs` | Add MCP Bridge wizard prompt (lines 222-223, between MCP Servers and Secrets sections) |
| `templates/ai-minimal.toml` | Append `[mcp-bridge]` stanza with `preset = "minimal"` |
| `templates/ai-fullstack-ts.toml` | Append `[mcp-bridge]` stanza with `preset = "agent"` |
| `templates/ai-fullstack-py.toml` | Append `[mcp-bridge]` stanza with `preset = "agent"` |
| `templates/saas-multi-tenant.toml` | Append `[mcp-bridge]` stanza with `preset = "full"` |

No new files are created.

## 1. Interactive wizard change (`src/cli/init.rs`)

### Location

Insert the new section between the end of the MCP Servers block (after line 222, `config.mcp = Some(mcps);`) and the start of the Secrets block (line 224, `// Secrets section`).

### Code to insert

Insert the following block immediately after line 222 (`}`) and before line 224 (`// Secrets section`):

```rust
    // MCP Bridge section
    eprintln!();
    output::header("MCP Bridge");
    eprintln!();

    if prompt_yes_no(
        "Enable the built-in MCP bridge (great mcp-bridge)?",
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

### Design rationale

- **Default is `false`**: Users without any AI CLI tools installed get no benefit. Opt-in avoids confusion.
- **Preset is `"minimal"`**: The wizard targets first-time users who likely have a single agent (Claude). The `minimal` preset exposes only the essential tools. Users can upgrade to `agent`/`full` by editing `great.toml` later.
- **Placement after MCP Servers**: Logically groups MCP-related configuration. The bridge is conceptually an MCP server itself (it serves MCP over stdio), so it belongs adjacent to the MCP Servers section.
- **Uses `McpBridgeConfig { ..Default::default() }`**: All other fields (`backends`, `default_backend`, `timeout_secs`, `auto_approve`, `allowed_dirs`) default to `None` and are skipped during serialization (`skip_serializing_if = "Option::is_none"`), producing a clean TOML output with only `preset = "minimal"`.

### Expected TOML output when user answers "y"

The `toml::to_string_pretty` serializer will emit (field ordering is determined by struct field order in `GreatConfig`, which places `mcp_bridge` last):

```toml
[mcp-bridge]
preset = "minimal"
```

### Expected TOML output when user answers "n" or presses Enter

No `[mcp-bridge]` stanza is emitted (the field remains `None` and `skip_serializing_if = "Option::is_none"` suppresses it).

## 2. Template changes

### 2a. `templates/ai-minimal.toml`

Append after the final line (`required = ["ANTHROPIC_API_KEY"]`):

```toml

[mcp-bridge]
preset = "minimal"
```

Full file after change:

```toml
[project]
name = "my-project"

[tools.cli]
gh = "latest"

[agents.claude]
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[secrets]
provider = "env"
required = ["ANTHROPIC_API_KEY"]

[mcp-bridge]
preset = "minimal"
```

### 2b. `templates/ai-fullstack-ts.toml`

Append after the final line (`required = ["ANTHROPIC_API_KEY"]`):

```toml

[mcp-bridge]
preset = "agent"
```

Full file after change:

```toml
[project]
name = "my-project"
description = "Full-stack TypeScript AI project"

[tools]
node = "22"
deno = "latest"

[tools.cli]
typescript = "latest"
pnpm = "latest"
gh = "latest"
bat = "latest"
uv = "latest"
starship = "latest"

[agents.claude]
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[mcp.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "."]

[secrets]
provider = "env"
required = ["ANTHROPIC_API_KEY"]

[mcp-bridge]
preset = "agent"
```

### 2c. `templates/ai-fullstack-py.toml`

Append after the final line (`required = ["ANTHROPIC_API_KEY"]`):

```toml

[mcp-bridge]
preset = "agent"
```

Full file after change:

```toml
[project]
name = "my-project"
description = "Full-stack Python AI project"

[tools]
python = "3.12"
node = "22"

[tools.cli]
uv = "latest"
gh = "latest"
bat = "latest"
starship = "latest"

[agents.claude]
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[mcp.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "."]

[secrets]
provider = "env"
required = ["ANTHROPIC_API_KEY"]

[mcp-bridge]
preset = "agent"
```

### 2d. `templates/saas-multi-tenant.toml`

Append after the final line (`extra_tools = ["coreutils"]`):

```toml

[mcp-bridge]
preset = "full"
```

Full file after change:

```toml
[project]
name = "my-saas"
description = "Multi-tenant SaaS with Hasura, AWS, and AI agents"

[tools]
node = "22"
python = "3.12"

# Templates can declare their own CLI tools — `great apply` installs them
# alongside any tools the user has added manually to their great.toml.
[tools.cli]
pnpm = "latest"
uv = "latest"
gh = "latest"
bat = "latest"
hasura-cli = "latest"
aws = "latest"
cdk = "latest"
starship = "latest"

[agents.claude]
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[mcp.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "."]

[secrets]
provider = "env"
required = ["ANTHROPIC_API_KEY", "HASURA_ADMIN_SECRET"]

[platform.macos]
extra_tools = ["coreutils"]

[mcp-bridge]
preset = "full"
```

## 3. Edge cases

| Edge case | Handling |
|-----------|----------|
| Non-interactive stdin (piped `/dev/null`) | `prompt_yes_no` returns the default (`false`); no `[mcp-bridge]` stanza emitted. Correct behavior -- non-interactive should be conservative. |
| `--template` flag used | Templates are verbatim `include_str!` -- the stanza is baked into the file. No wizard interaction occurs. |
| User already has `[mcp-bridge]` in an existing `great.toml` | `great init` refuses to overwrite without `--force`. With `--force`, the entire file is replaced. No merge logic exists or is needed. |
| `McpBridgeConfig` fields evolve in future | The `..Default::default()` pattern in the wizard ensures new `Option` fields default to `None` without code changes. Template files only specify `preset`; unknown fields are silently ignored by serde on deserialization. |

## 4. Error handling

No new error paths are introduced. The only fallible operation in the new code is `prompt_yes_no(...)`, which already uses `?` to propagate I/O errors. `McpBridgeConfig` construction is infallible.

## 5. Security considerations

None. The MCP bridge preset is a declarative configuration choice. No secrets, network calls, or file system access are introduced by this change. The bridge itself is activated later by `great apply`, which is outside the scope of this task.

## 6. Platform differences

None. The wizard prompt and template TOML are platform-independent. The `[mcp-bridge]` stanza is read identically on macOS (ARM64/x86_64), Ubuntu, and WSL2.

## 7. Testing strategy

### 7a. Existing test that must continue to pass

**`test_templates_parse_as_valid_config`** (file: `src/cli/init.rs`, line 422)

This test calls `toml::from_str::<GreatConfig>(content)` on all four embedded templates and asserts that `project` and `agents` are `Some`. After adding `[mcp-bridge]` stanzas, the templates will still deserialize cleanly because `McpBridgeConfig` is already part of `GreatConfig` (line 26 of `src/config/schema.rs`).

No changes to this test are required.

### 7b. New test: templates have mcp_bridge configured

Add the following test to the `mod tests` block in `src/cli/init.rs` (after the existing `test_templates_parse_as_valid_config` test, around line 456):

```rust
    #[test]
    fn test_templates_have_mcp_bridge() {
        let templates: &[(&str, &str, &str)] = &[
            (
                "ai-minimal",
                include_str!("../../templates/ai-minimal.toml"),
                "minimal",
            ),
            (
                "ai-fullstack-ts",
                include_str!("../../templates/ai-fullstack-ts.toml"),
                "agent",
            ),
            (
                "ai-fullstack-py",
                include_str!("../../templates/ai-fullstack-py.toml"),
                "agent",
            ),
            (
                "saas-multi-tenant",
                include_str!("../../templates/saas-multi-tenant.toml"),
                "full",
            ),
        ];
        for (name, content, expected_preset) in templates {
            let config: GreatConfig = toml::from_str(content)
                .unwrap_or_else(|e| panic!("template '{}' failed to parse: {}", name, e));
            let bridge = config.mcp_bridge.unwrap_or_else(|| {
                panic!("template '{}' should have a [mcp-bridge] section", name)
            });
            assert_eq!(
                bridge.preset.as_deref(),
                Some(*expected_preset),
                "template '{}' should have preset '{}'",
                name,
                expected_preset
            );
        }
    }
```

### 7c. New test: wizard default produces no mcp_bridge

Add the following test to verify that the default `GreatConfig` (which simulates a non-interactive wizard run where all defaults are accepted) has no `mcp_bridge`:

```rust
    #[test]
    fn test_default_config_has_no_mcp_bridge() {
        let config = GreatConfig::default();
        assert!(
            config.mcp_bridge.is_none(),
            "default config should not have mcp_bridge (opt-in only)"
        );
    }
```

### 7d. Existing tests unaffected

The following existing tests require no modification and should continue to pass without changes:

- `test_detect_project_name_returns_string` -- no relation to mcp-bridge
- `test_init_from_template_unknown` -- tests unknown template name, unaffected
- `test_init_from_template_minimal` -- asserts `[project]` and `[agents.claude]` exist; still true
- `test_init_from_template_fullstack_ts` -- asserts `node`, `typescript`, `Full-stack TypeScript`; still true
- `test_init_from_template_fullstack_py` -- asserts `python`, `Full-stack Python`; still true
- `test_default_config_serializes` -- default config serializes; `mcp_bridge` is `None` so skipped
- All schema tests in `src/config/schema.rs` -- no schema changes

### 7e. Validation

Run the full test suite after implementation:

```bash
cargo test
cargo clippy -- -D warnings
```

## 8. Build order

This is a single-pass change with no dependencies between steps. However, the recommended order is:

1. **Templates first** (4 files) -- append `[mcp-bridge]` stanzas. This lets you run `cargo test` immediately to confirm `test_templates_parse_as_valid_config` still passes.
2. **Wizard prompt** (`src/cli/init.rs`) -- insert the MCP Bridge section.
3. **New tests** (`src/cli/init.rs`) -- add `test_templates_have_mcp_bridge` and `test_default_config_has_no_mcp_bridge`.
4. **Verify** -- `cargo test` and `cargo clippy`.

## 9. Checklist for the builder

- [ ] Append `[mcp-bridge]` with `preset = "minimal"` to `templates/ai-minimal.toml`
- [ ] Append `[mcp-bridge]` with `preset = "agent"` to `templates/ai-fullstack-ts.toml`
- [ ] Append `[mcp-bridge]` with `preset = "agent"` to `templates/ai-fullstack-py.toml`
- [ ] Append `[mcp-bridge]` with `preset = "full"` to `templates/saas-multi-tenant.toml`
- [ ] Insert MCP Bridge wizard section in `src/cli/init.rs` between MCP Servers and Secrets
- [ ] Add `test_templates_have_mcp_bridge` test
- [ ] Add `test_default_config_has_no_mcp_bridge` test
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes
