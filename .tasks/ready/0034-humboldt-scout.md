# Scout Report 0034: Wire mcp-bridge into `great init` and Built-in Templates

**Scout:** Alexander von Humboldt
**Date:** 2026-02-28
**Task:** Wire `mcp-bridge` into `great init` wizard and four built-in templates
**Spec:** `.tasks/ready/0034-init-wizard-mcp-bridge-spec.md` (APPROVED by Socrates)

---

## 1. Files to Modify

### 1a. `src/cli/init.rs` (366 lines)

**Role:** Interactive wizard — inserts new MCP Bridge section.

**Wizard flow (in order):**
1. Lines 36-46: Existing config guard + `--template` fast-path
2. Lines 49-54: Platform detection
3. Lines 59-64: Project name (prompt)
4. Lines 66-148: Tools section — runtimes, CLI tools, package managers, Starship
5. Lines 123-137: Cloud CLIs sub-section
6. Lines 150-190: AI Agents section
7. Lines 192-222: MCP Servers section
8. **INSERTION POINT** — after line 222, before line 224
9. Lines 224-249: Secrets section
10. Lines 252-267: Platform overrides
11. Lines 269-282: Serialize + write + print next steps

**Exact insertion point:**
```
Line 220:     if !mcps.is_empty() {
Line 221:         config.mcp = Some(mcps);
Line 222:     }
Line 223: (empty)
Line 224:     // Secrets section     <-- new block goes between 222 and 224
```

**Insert this block at line 223 (between the `}` and `// Secrets section`):**
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

**`prompt_yes_no` function:**
- Lines 336-350 in `src/cli/init.rs`
- Signature: `fn prompt_yes_no(question: &str, default_yes: bool) -> Result<bool>`
- Private to the module; no import needed
- `default_yes: false` means Enter key returns `false` (opt-in)

**Imports:**
- Line 9: `use crate::config::schema::*;` — wildcard import brings `McpBridgeConfig` into scope already
- No new import lines needed

**Output functions used in the wizard (all write to stderr):**
- `output::header(msg)` — bold text, line 24 of `src/cli/output.rs`
- `output::success(msg)` — green checkmark, line 4
- `output::info(msg)` — blue info, line 19
- `eprintln!()` — blank line separator (pattern used at lines 54, 68, 123, 151, 193)

**Section pattern (all 5 sections follow this structure):**
```rust
    eprintln!();
    output::header("Section Name");
    eprintln!();
    // prompt(s) ...
```

**Existing tests in `mod tests` block (lines 361-465):**

| Test name | Line | What it checks |
|-----------|------|----------------|
| `test_detect_project_name_returns_string` | 365 | `detect_project_name()` non-empty |
| `test_init_from_template_unknown` | 371 | Unknown template returns Ok, no file written |
| `test_init_from_template_minimal` | 384 | File created, contains `[project]` and `[agents.claude]` |
| `test_init_from_template_fullstack_ts` | 397 | Contains `node`, `typescript`, `Full-stack TypeScript` |
| `test_init_from_template_fullstack_py` | 409 | Contains `python`, `Full-stack Python` |
| `test_templates_parse_as_valid_config` | 422 | All 4 templates deserialize; `project` and `agents` are `Some` |
| `test_default_config_serializes` | 458 | `GreatConfig::default()` serializes without error |

Two new tests must be added after `test_templates_parse_as_valid_config` (after line 456):
1. `test_templates_have_mcp_bridge` — asserts each template parses with correct `preset`
2. `test_default_config_has_no_mcp_bridge` — asserts `GreatConfig::default().mcp_bridge.is_none()`

---

### 1b. `src/config/schema.rs` — READ-ONLY (no changes)

**`McpBridgeConfig` struct (lines 144-175):**
- Derives: `Debug, Clone, Serialize, Deserialize, Default`
- `#[serde(rename_all = "kebab-case")]` — field `default_backend` serializes as `default-backend`
- All 6 fields are `Option<T>` with `skip_serializing_if = "Option::is_none"`:
  - `backends: Option<Vec<String>>`
  - `default_backend: Option<String>`
  - `timeout_secs: Option<u64>`
  - `preset: Option<String>` — only field used in task 0034
  - `auto_approve: Option<bool>`
  - `allowed_dirs: Option<Vec<String>>`

**`GreatConfig.mcp_bridge` field (lines 25-26):**
```rust
#[serde(rename = "mcp-bridge", skip_serializing_if = "Option::is_none")]
pub mcp_bridge: Option<McpBridgeConfig>,
```
- TOML section name is `[mcp-bridge]` (via `rename`)
- Field is last in `GreatConfig` struct (lines 11-27), so it serializes last in TOML output
- `Default` derive sets it to `None`

---

### 1c. `templates/ai-minimal.toml` (14 lines)

**Current last line (line 14):** `required = ["ANTHROPIC_API_KEY"]`

**Append:**
```toml

[mcp-bridge]
preset = "minimal"
```

**Result:** 17 lines total. No existing `[mcp-bridge]` stanza.

---

### 1d. `templates/ai-fullstack-ts.toml` (27 lines)

**Current last line (line 27):** `required = ["ANTHROPIC_API_KEY"]`

**Append:**
```toml

[mcp-bridge]
preset = "agent"
```

**Result:** 30 lines total. No existing `[mcp-bridge]` stanza.

---

### 1e. `templates/ai-fullstack-py.toml` (25 lines)

**Current last line (line 25):** `required = ["ANTHROPIC_API_KEY"]`

**Append:**
```toml

[mcp-bridge]
preset = "agent"
```

**Result:** 28 lines total. No existing `[mcp-bridge]` stanza.

---

### 1f. `templates/saas-multi-tenant.toml` (34 lines)

**Current last line (line 34):** `extra_tools = ["coreutils"]`

**Append:**
```toml

[mcp-bridge]
preset = "full"
```

**Result:** 37 lines total. No existing `[mcp-bridge]` stanza.

**Note:** This template ends with `[platform.macos]` and `extra_tools`, not `[secrets]`. The append goes after `extra_tools = ["coreutils"]`.

---

### 1g. `src/cli/apply.rs` — READ-ONLY, context only

**`cfg.mcp_bridge` is already consumed at lines 727-778:**
```rust
// 5a. Register MCP bridge in .mcp.json
if let Some(ref bridge_cfg) = cfg.mcp_bridge {
    output::header("MCP Bridge");
    // ... builds bridge_args from preset and backends ...
    // ... registers "great-bridge" entry in .mcp.json ...
}
```
No changes to `apply.rs` required for this task.

---

## 2. Dependency Map

```
templates/*.toml
    └── embedded by init.rs via include_str!("../../templates/*.toml")
    └── deserialized by test_templates_parse_as_valid_config → GreatConfig
    └── GreatConfig.mcp_bridge → McpBridgeConfig (schema.rs lines 144-175)

src/cli/init.rs
    ├── uses crate::config::schema::* (wildcard, covers McpBridgeConfig)
    ├── uses crate::cli::output (header, success, info)
    ├── calls prompt_yes_no (local fn, lines 336-350)
    └── assigns config.mcp_bridge = Some(McpBridgeConfig { preset, ..Default::default() })

src/cli/apply.rs
    └── consumes cfg.mcp_bridge at line 727 (existing, no change)
```

---

## 3. Patterns to Follow

**Section separator pattern** (used consistently in init.rs):
```rust
eprintln!();
output::header("Section Name");
eprintln!();
```

**Config struct population pattern** (used at lines 61-64, 169-177, 202-214):
```rust
config.field = Some(StructName {
    field: Some("value".to_string()),
    ..Default::default()
});
```

**Opt-in prompt pattern** (used for Codex, Gemini, Deno, Starship, Cloud CLIs):
```rust
if prompt_yes_no("Enable X?", false)? {
    // configure X
    output::success("X enabled");
    output::info("  Hint about X");
}
```

---

## 4. Gotchas

1. **Wildcard import covers everything.** `use crate::config::schema::*;` at line 9 already imports `McpBridgeConfig`. Do not add a named import — it will conflict.

2. **Trailing newline before `[mcp-bridge]` in templates.** The spec shows a blank line between the last existing line and `[mcp-bridge]`. This is conventional TOML style and matches the existing section separators in all four templates.

3. **`saas-multi-tenant.toml` ends differently.** Three templates end with `[secrets]`, but `saas-multi-tenant.toml` ends with `[platform.macos]` / `extra_tools`. The append logic is identical (add blank line + `[mcp-bridge]`), but the final line differs.

4. **`test_init_from_template_minimal` (line 384) will pass without changes.** It only asserts `[project]` and `[agents.claude]` are present. Adding `[mcp-bridge]` does not disturb those assertions.

5. **TOML serialization field order.** `mcp_bridge` is the last field in `GreatConfig` struct (line 26). `toml::to_string_pretty` serializes struct fields in declaration order, so `[mcp-bridge]` will appear at the end of wizard-generated files — consistent with the template ordering.

6. **`preset = "minimal"` for wizard, not "agent".** The spec is deliberate: new users run the wizard and likely have one agent (Claude). `minimal` preset is safer than `agent`. Template presets differ by template complexity.

7. **No stdin/non-interactive guard needed in new code.** The existing `prompt_yes_no` already returns `default_yes` when stdin is empty (piped `/dev/null`). With `false` as default, non-interactive runs produce no `[mcp-bridge]` stanza.

---

## 5. Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `test_templates_parse_as_valid_config` failing | None — verified: `McpBridgeConfig` is already in `GreatConfig`; the field is `Option` so absent stanza is fine, and present stanza deserializes cleanly | Already mitigated by schema design |
| `saas-multi-tenant.toml` wrong append point | Low | Spec identifies exact last line: `extra_tools = ["coreutils"]` (line 34). Confirmed by reading the file. |
| Duplicate import if named import added | Low | Wildcard at line 9 covers it — do not add `use crate::config::schema::McpBridgeConfig;` |
| clippy warning on unused struct field | None | All fields used (preset is set, rest use Default::default()) |

**Technical debt noted:** `src/cli/init.rs` has no `non_interactive` field in `Args`. The `--non-interactive` global flag (wired in other commands via `apply::Args`) is not forwarded to `init`. This is a pre-existing gap (Socrates Concern 1), out of scope for 0034.

---

## 6. Recommended Build Order

1. **Templates (4 files)** — append `[mcp-bridge]` stanzas. Run `cargo test` immediately; `test_templates_parse_as_valid_config` must still pass.
2. **Wizard prompt** — insert MCP Bridge section in `src/cli/init.rs` between lines 222 and 224.
3. **New tests** — add `test_templates_have_mcp_bridge` and `test_default_config_has_no_mcp_bridge` to `mod tests` in `src/cli/init.rs` (after line 456, before the closing `}`).
4. **Verify** — `cargo test` and `cargo clippy -- -D warnings`.

Total estimated changed lines: ~30 in templates + ~25 in init.rs = ~55 lines. No new files.
