# Socrates Review: Spec 0002 -- TOML Config Parser Schema Enrichment

**Spec:** `.tasks/ready/0002-config-schema-spec.md`
**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-24
**Round:** 1

---

## VERDICT: APPROVED (with ADVISORY items to address during implementation)

No BLOCKING concerns found. The spec is thorough, well-structured, and implementable without further clarification. The concerns below are ADVISORY -- the builder should address them inline during implementation.

---

## Concerns

### 1. Test count discrepancy

```
{
  "gap": "The spec states '14 existing tests in schema.rs' (line 512) and '14 existing
          tests' in the verification checklist (line 993), but the actual file contains
          exactly 13 #[test] functions.",
  "question": "Where is the 14th test? Is this an off-by-one error or was a test
               removed after the spec was written?",
  "severity": "ADVISORY",
  "recommendation": "Correct the count to 13 in both locations (Testing Strategy section
                     and Verification Checklist). The test names listed in the spec
                     (13 names) are accurate -- only the numeric count is wrong."
}
```

### 2. MCP transport whitelist omits `sse`

```
{
  "gap": "The spec validates transport as 'stdio' or 'http' only. The MCP specification
          also defines 'sse' (Server-Sent Events) as a transport type, which is widely
          used in MCP server implementations for streaming.",
  "question": "Should 'sse' be included in the known transport list alongside 'stdio'
               and 'http'? If a user configures transport = 'sse', they will receive a
               warning that could be confusing.",
  "severity": "ADVISORY",
  "recommendation": "Add 'sse' to the known transports list: ['stdio', 'http', 'sse'].
                     Since this is only a warning (not an error), the impact is low, but
                     false warnings erode user trust in validation output."
}
```

### 3. `load()` bails on first error, swallowing subsequent errors

```
{
  "gap": "The existing load() function iterates messages and bails on the first Error
          variant. With the spec adding 3 new error-producing checks (empty MCP command,
          HTTP transport without URL), a config with multiple errors will only report
          the first one. The spec does not acknowledge this behavior or propose to
          change it.",
  "question": "Is single-error-at-a-time reporting acceptable UX when a config might
               have multiple MCP servers with issues? Should load() collect all errors
               and report them together?",
  "severity": "ADVISORY",
  "recommendation": "This is pre-existing behavior and changing it is out of scope for
                     this spec. Document this as a known limitation. A future iteration
                     could collect all errors and bail with a combined message."
}
```

### 4. Agent provider whitelist will expand -- no extensibility mechanism

```
{
  "gap": "The known agent providers are hardcoded as ['anthropic', 'openai', 'google'].
          As great.sh gains users, providers like 'azure-openai', 'aws-bedrock',
          'ollama', 'together', 'groq' will be common. This produces warnings that
          cannot be suppressed short of a code change.",
  "question": "Is a hardcoded provider list the right mechanism, or should there be a
               way to suppress warnings for custom providers (e.g., a config flag or
               wider default list)?",
  "severity": "ADVISORY",
  "recommendation": "The current approach (warning, not error) is appropriate for an
                     early release. The builder should add a code comment noting the
                     list should be expanded as the ecosystem grows. No spec change
                     needed."
}
```

### 5. `find_secret_refs` regex excludes lowercase secret references

```
{
  "gap": "The regex pattern [A-Z_][A-Z0-9_]* only matches UPPER_CASE references.
          A user writing api_key = '${my_api_key}' or env = { x = '${Postgres_Url}' }
          would get no match and no warning.",
  "question": "Is the uppercase-only constraint intentional and documented to users?
               Is there a convention enforced by the vault module that secret names
               must be uppercase?",
  "severity": "ADVISORY",
  "recommendation": "This appears intentional -- environment variable naming conventions
                     use uppercase. The spec's edge case table (line 471) explicitly
                     documents that '${lower_case}' returns []. The builder should
                     ensure this convention is documented in the sample great.toml or
                     in `great init` output."
}
```

### 6. `test_validate_known_providers_no_warnings` uses agents without `model` fields

```
{
  "gap": "The proposed test_validate_known_providers_no_warnings test (line 779-798)
          creates agents with only 'provider' set and no 'model'. Agent 'a' has
          provider='anthropic' but model=None. Per existing validation check #1,
          this should NOT trigger a warning because the check is
          'agent.provider.is_none() && agent.model.is_none()' -- having provider
          alone is sufficient. So the test is correct. But this is subtle and worth
          calling out for the builder.",
  "question": "N/A -- verified this is correct. The existing check requires BOTH
               provider and model to be None to warn.",
  "severity": "ADVISORY",
  "recommendation": "No change needed. The builder should understand that the existing
                     validation only warns when both provider AND model are absent."
}
```

### 7. No validation for `enabled` field interaction with other checks

```
{
  "gap": "The spec adds 'enabled' fields but validation still checks disabled agents
          (e.g., agent 'x' with enabled=false and no provider still triggers a warning
          about missing provider). Is this intended?",
  "question": "Should validate() skip checks on agents/MCPs where enabled = false?
               A disabled agent with no provider is not really a misconfiguration.",
  "severity": "ADVISORY",
  "recommendation": "For this iteration, validating all entries regardless of enabled
                     state is reasonable -- it catches misconfigurations that would
                     surface if the user re-enables the entry. Add a TODO comment if
                     desired."
}
```

### 8. Spec does not mention `#[serde(default)]` on new fields

```
{
  "gap": "The spec states new Option fields need #[serde(skip_serializing_if)] but
          does not mention whether #[serde(default)] is needed on individual fields.
          For Option<T> fields in a struct where the struct itself is the value of an
          Option in the parent, serde already handles absent keys as None without
          requiring field-level #[serde(default)]. The spec is correct that this works
          but does not explain WHY it works.",
  "question": "N/A -- verified: when a table like [agents.claude] is deserialized into
               AgentConfig, serde's toml deserializer maps absent keys to None for
               Option<T> fields without needing explicit #[serde(default)] attributes.",
  "severity": "ADVISORY",
  "recommendation": "No change needed. The builder should understand that Option<T>
                     fields in serde structs default to None for absent TOML keys.
                     Field-level #[serde(default)] is only needed for non-Option types
                     with custom defaults."
}
```

---

## Cross-verification results

| Spec Claim | Verified Against Source | Result |
|------------|----------------------|--------|
| `GreatConfig` already derives `Default` | `/home/isaac/src/sh.great/src/config/schema.rs` line 10 | CORRECT |
| `ProjectConfig` lacks `Default` derive | Line 27 | CORRECT -- only `Debug, Clone, Serialize, Deserialize` |
| `AgentConfig` lacks `Default` derive | Line 74 | CORRECT |
| `SecretsConfig` lacks `Default` derive | Line 99 | CORRECT |
| `PlatformOverride` lacks `Default` derive | Line 119 | CORRECT |
| `PlatformConfig` already derives `Default` | Line 108 | CORRECT |
| `ToolsConfig` already derives `Default` | Line 62 | CORRECT |
| 3 `AgentConfig` construction sites in `init.rs` | Lines 160, 170, 180 | CORRECT |
| 1 `McpConfig` construction in `init.rs` | Line 199 | CORRECT |
| 4 `AgentConfig` construction sites in `template.rs` tests | Lines 464, 474, 491, 501 | CORRECT |
| 3 `McpConfig` construction sites in `template.rs` tests | Lines 520, 534, 544 | CORRECT |
| 5 `McpConfig` construction sites in `mcp/mod.rs` tests | Lines 142, 168, 212, 219, 249 | CORRECT (verified 5 sites) |
| `regex` crate is in dependencies | `Cargo.toml` line 23 | CORRECT |
| Existing tests count = 14 in schema.rs | Actual: 13 `#[test]` functions | INCORRECT -- off by one |
| Existing tests count = 5 in mod.rs | Actual: 5 `#[test]` functions | CORRECT |
| Templates parse without new fields | Checked `ai-fullstack-ts.toml`, `saas-multi-tenant.toml` -- no `version`, `api_key`, or `enabled` fields | CORRECT -- additive changes are safe |

---

## Summary

A well-crafted, implementation-ready spec with accurate downstream impact analysis, comprehensive test coverage (19 new tests), correct serde behavior, and sensible validation additions. The only factual error is a test count of 14 that should be 13. All other concerns are advisory improvements the builder can address inline.
