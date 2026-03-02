# Review: Spec 0034 — Wire mcp-bridge into init wizard and built-in templates

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-28
**Round:** 1

---

## VERDICT: APPROVED

---

## Verification Summary

### Line numbers and code structure -- VERIFIED

The spec claims insertion between line 222 (`}` closing the MCP Servers block) and line 224 (`// Secrets section`) in `src/cli/init.rs`. Confirmed:

- Line 220: `if !mcps.is_empty() {`
- Line 221: `config.mcp = Some(mcps);`
- Line 222: `}`
- Line 223: (empty line)
- Line 224: `// Secrets section`

Insertion point is correct.

### McpBridgeConfig availability -- VERIFIED

`src/cli/init.rs` line 9 imports `use crate::config::schema::*;` which brings `McpBridgeConfig` into scope. The struct at `src/config/schema.rs:144-175` derives `Default` (line 144: `#[derive(Debug, Clone, Serialize, Deserialize, Default)]`). All `Option` fields default to `None`. The `skip_serializing_if = "Option::is_none"` annotations on every field mean only `preset` will appear in TOML output. The `#[serde(rename = "mcp-bridge")]` on `GreatConfig.mcp_bridge` (line 25) ensures correct TOML section name.

### Template current contents -- VERIFIED

All four templates match the spec's "current state" claims exactly:

| Template | Last line | Spec claims last line is | Match |
|----------|-----------|--------------------------|-------|
| `ai-minimal.toml` (14 lines) | `required = ["ANTHROPIC_API_KEY"]` | `required = ["ANTHROPIC_API_KEY"]` | YES |
| `ai-fullstack-ts.toml` (27 lines) | `required = ["ANTHROPIC_API_KEY"]` | `required = ["ANTHROPIC_API_KEY"]` | YES |
| `ai-fullstack-py.toml` (25 lines) | `required = ["ANTHROPIC_API_KEY"]` | `required = ["ANTHROPIC_API_KEY"]` | YES |
| `saas-multi-tenant.toml` (34 lines) | `extra_tools = ["coreutils"]` | `extra_tools = ["coreutils"]` | YES |

### apply.rs consumption -- VERIFIED

`src/cli/apply.rs` lines 727-778 already handle `cfg.mcp_bridge`: they register a `great-bridge` entry in `.mcp.json` with the correct preset and backend args. No changes needed in apply.rs. Spec claim is correct.

### output functions -- VERIFIED

`src/cli/output.rs` confirms `output::header()` (line 24), `output::success()` (line 4), and `output::info()` (line 19) all exist and write to stderr. The spec's inserted code uses exactly these three functions, consistent with the rest of the init wizard.

### Prompt style consistency -- VERIFIED

The spec's wizard prompt follows the exact pattern used throughout `init.rs`:
1. `eprintln!()` for blank line separator
2. `output::header("Section Name")` for section title
3. `eprintln!()` for blank line after header
4. `prompt_yes_no("Question?", default)` for the choice
5. Conditional construction of config struct with `..Default::default()`
6. `output::success()` + `output::info()` for feedback

This is identical to the patterns at lines 66-68 (Tools), 123-125 (Cloud CLIs), 151-153 (AI Agents), 193-195 (MCP Servers). No style deviation.

### Existing test suite -- VERIFIED

`test_templates_parse_as_valid_config` (line 422) deserializes all four templates and asserts `project` and `agents` are `Some`. Since `McpBridgeConfig` is already part of `GreatConfig` and templates currently have no `[mcp-bridge]` stanza (which is fine -- the field is `Option`), adding the stanza will only add a new field to the parsed struct. The test will continue to pass.

### Backlog alignment -- VERIFIED

All five acceptance criteria from the backlog (`/home/isaac/src/sh.great/.tasks/backlog/0034-init-wizard-mcp-bridge.md` lines 19-25) are addressed:
1. Wizard prompt after MCP Servers -- YES (spec section 1)
2. `ai-minimal.toml` with `preset = "minimal"` -- YES (spec section 2a)
3. `ai-fullstack-ts.toml` and `ai-fullstack-py.toml` with `preset = "agent"` -- YES (spec sections 2b, 2c)
4. `saas-multi-tenant.toml` with `preset = "full"` -- YES (spec section 2d)
5. `test_templates_parse_as_valid_config` continues to pass -- YES (spec section 7a)

---

## Concerns

### Concern 1

```
{
  "gap": "init.rs Args struct has no non_interactive field, and main.rs does not forward --non-interactive to init::run(). The spec adds an interactive prompt but does not address --non-interactive.",
  "question": "When a user runs `great --non-interactive init`, should the new MCP bridge prompt be skipped entirely (using the default), or is the existing behavior of init.rs (relying on stdin EOF to return defaults) sufficient?",
  "severity": "ADVISORY",
  "recommendation": "This is a pre-existing gap affecting ALL init wizard prompts, not specific to 0034. The spec correctly notes that piped stdin returns the default (false). However, a future task should wire --non-interactive into init::Args for completeness. No action needed for this spec."
}
```

### Concern 2

```
{
  "gap": "The spec proposes test_default_config_has_no_mcp_bridge (section 7c) which tests GreatConfig::default() has no mcp_bridge. This test validates the schema default, not the wizard behavior.",
  "question": "Is this test adding meaningful coverage beyond what GreatConfig's derive(Default) already guarantees, or is it just documenting intent?",
  "severity": "ADVISORY",
  "recommendation": "The test is harmless and serves as a regression guard if someone changes GreatConfig::default() in the future. Keep it, but note it is a documentation test, not a behavioral test."
}
```

### Concern 3

```
{
  "gap": "The spec does not add an integration test (in tests/cli_smoke.rs) that runs `great init --template ai-minimal` and verifies the output file contains [mcp-bridge].",
  "question": "Is a unit test on include_str! sufficient, or should there be an end-to-end test that exercises the actual init_from_template codepath and checks the written file?",
  "severity": "ADVISORY",
  "recommendation": "The existing tests test_init_from_template_minimal et al. (lines 384-419) already test this codepath by calling init_from_template() and reading the output file. The new test_templates_have_mcp_bridge verifies the content via include_str!. Between these, coverage is adequate. An additional cli_smoke test would be belt-and-suspenders but is not required."
}
```

---

## Summary

Clean, small-scoped spec with verified line numbers, verified template contents, verified struct availability, correct prompt style, and complete backlog coverage. All three concerns are ADVISORY (pre-existing gap, test philosophy, test coverage depth). No BLOCKING issues found.
