# Release Notes — Task 0022: `great diff` Counter/Marker Consistency

**Date:** 2026-02-26
**Scope:** `src/cli/diff.rs`, `tests/cli_smoke.rs`
**Cargo version:** 0.1.0 (no version bump; bugfix)

---

## Summary

`great diff` reported incorrect summary counts in two cases: an MCP server
with a missing command binary was counted as "to configure" instead of "to
install," and a secret appearing in both `secrets.required` and an MCP env
reference was counted twice. Both are now fixed. Section headers were also
normalized to a consistent bare-noun style.

These are counting bugs only. No diff output lines were silently dropped and
no installation behavior is affected. The fixes make the summary line accurate
for the first time in configurations that include MCP servers or overlapping
secret declarations.

---

## User-facing changes

### MCP server with missing command now counted as "to install"

Previously, when an MCP server's command binary was absent from PATH, the
`+` (green) install marker was shown but the summary counted it under "to
configure." This meant a config like:

```toml
[mcp.my-server]
command = "nonexistent_cmd"
```

would print:

```
  + my-server (nonexistent_cmd -- not found)
```

but then report `1 to configure` rather than `1 to install`. The marker and
the count now agree.

### Secrets are no longer double-counted

A secret that appears in both `secrets.required` and as a `${...}` reference
in an MCP `env` block or agent `api_key` field was previously counted twice
in the summary. For example:

```toml
[secrets]
required = ["ANTHROPIC_API_KEY"]

[agents.claude]
api_key = "${ANTHROPIC_API_KEY}"
```

Previously reported `2 secrets to resolve`. Now reports `1 secrets to resolve`.

The secret is still displayed once. The first occurrence (from
`secrets.required`) takes priority. The redundant discovery via config
scanning is silently deduplicated.

### Unified "Secrets" section

The two separate sections — "Secrets -- need to set:" (for `secrets.required`)
and "Secret References -- unresolved:" (for refs found via config scanning) —
have been merged into a single "Secrets" section. This reduces visual
repetition when a secret appears in both places.

The suffix text for secrets discovered through config scanning changes from
"(referenced in MCP env, not set)" to "(referenced in config, not set)" to
accurately reflect that `great diff` scans both MCP `env` values and agent
`api_key` fields.

### Section headers normalized

All diff section headers now use bare-noun style, matching the existing
"Tools" header:

| Before | After |
|--------|-------|
| `MCP Servers -- need configuration:` | `MCP Servers` |
| `Secrets -- need to set:` | `Secrets` |
| `Secret References -- unresolved:` | *(merged into "Secrets")* |

---

## Technical changes

- `src/cli/diff.rs` line 143: `configure_count += 1` changed to
  `install_count += 1` in the MCP missing-command path. One token change.
- The two independent secrets loops (over `secrets.required` and
  `find_secret_refs()`) replaced by a single unified block using
  `std::collections::BTreeSet` for deduplication. `secrets_count` is now
  set to the cardinality of the dedup set.
- `output::header("MCP Servers -- need configuration:")` changed to
  `output::header("MCP Servers")`.
- The two `output::header` calls for the secrets sections replaced by a single
  `output::header("Secrets")` call after both phases of the unified loop.
- New import: `use std::collections::BTreeSet;`. No new crate dependencies.

The marker-to-bucket invariant is now enforced uniformly:

| Marker | Color  | Summary bucket    |
|--------|--------|-------------------|
| `+`    | green  | `install_count`   |
| `~`    | yellow | `configure_count` |
| `-`    | red    | `secrets_count`   |

---

## Test coverage

4 new integration tests added to `tests/cli_smoke.rs`:

| Test | What it covers |
|------|----------------|
| `diff_mcp_missing_command_counted_as_install` | Missing MCP command increments `install_count`, not `configure_count` |
| `diff_mcp_missing_command_and_missing_tool_install_count` | Missing tool + missing MCP command both accumulate into a single `install_count` |
| `diff_secret_dedup_required_and_ref` | Secret in both `secrets.required` and MCP env counted once, not twice |
| `diff_secret_ref_only_no_required_section` | Secret found only via `find_secret_refs()` still counted and displayed |

All 8 pre-existing `diff_*` integration tests pass without assertion changes.

---

## Migration notes

No breaking changes. No configuration changes required. The summary counts
are now correct where they were previously inflated; any script or CI step
that was asserting on an inflated count (e.g., "2 secrets to resolve" for a
single secret) will need to be updated to expect the correct value.

No `great.toml` schema changes. No new dependencies.
