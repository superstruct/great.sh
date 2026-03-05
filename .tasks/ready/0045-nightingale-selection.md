# 0045 — Nightingale Selection: `apply --only` and `--skip` flags

| Field | Value |
|---|---|
| Task ID | 0045 |
| Priority | P2 |
| Type | feature |
| Module | `src/cli/apply.rs` |
| Status | selected |
| Selected by | Nightingale |
| Date | 2026-03-05 |

## Decision

Implement option 1: add `--only` and `--skip` flags to `great apply`.

Rationale: the flags represent real user value (selective provisioning is a common need on CI and when
iterating on a single concern). Deleting the test stubs removes accountability and leaves a gap in the
test contract. Implementing the flags is the right call.

## Scope Clarification

Add two optional, multi-value flags to `apply`:

- `--only <category>` — run only the specified category(ies); all others are skipped.
- `--skip <category>` — skip the specified category(ies); all others run normally.
- Both flags accept one or more values from the set: `tools`, `mcp`, `agents`, `secrets`.
- `--only` and `--skip` are mutually exclusive (clap `conflicts_with`).

The `agents` category maps to loop-agent file installation (the `great loop` integration step,
currently step 10 / bootstrap). `tools` covers runtimes + CLI tools (steps 3–4). `mcp` covers
MCP server configuration and MCP bridge (steps 5–5a). `secrets` covers the secrets check (step 6).

Note on current `run()` structure: the function runs up to 10 sequential provisioning sections.
The filter logic gates each labelled section behind a helper (e.g., `fn should_run(category,
only, skip) -> bool`) and is applied at the entry point of each section block, not deep inside
helper functions, to keep the change surface small.

## Acceptance Criteria

1. `great apply --only tools --dry-run` exits 0 and prints the tools section; MCP, agents, and secrets sections are absent from stdout.
2. `great apply --only mcp --dry-run` exits 0 and prints the MCP section; tools section is absent.
3. `great apply --only agents --dry-run` exits 0; no error about an unknown category.
4. `great apply --skip tools --dry-run` exits 0 and prints MCP + secrets sections; tools section is absent.
5. `great apply --only tools --skip mcp` exits with code 2 (clap conflict error).

## Files That Need to Change

- `src/cli/apply.rs` — add `--only` and `--skip` to `Args`, add `should_run()` helper, gate each provisioning section.
- `tests/cli_smoke.rs` — add the four integration tests corresponding to the failing cases in the task.

## Dependencies

None. No other tasks are blocking.

## Out of Scope

- Changing any provisioning logic within the sections themselves.
- Adding new categories beyond `tools`, `mcp`, `agents`, `secrets`.
- Persisting filter selections to `great.toml`.
- Applying filters to `great diff` or `great status`.

## Notes for Implementer (Lovelace / Da Vinci)

The `Args` struct at line 354 of `src/cli/apply.rs` currently has three fields: `config`, `dry_run`,
`yes`, and the skipped `non_interactive`. Add:

```rust
/// Run only these categories (tools, mcp, agents, secrets).
#[arg(long, value_delimiter = ',', conflicts_with = "skip")]
pub only: Vec<String>,

/// Skip these categories (tools, mcp, agents, secrets).
#[arg(long, value_delimiter = ',', conflicts_with = "only")]
pub skip: Vec<String>,
```

A simple helper resolves the filter at each section boundary:

```rust
fn should_run(category: &str, only: &[String], skip: &[String]) -> bool {
    if !only.is_empty() {
        return only.iter().any(|c| c == category);
    }
    !skip.iter().any(|c| c == category)
}
```

Valid category strings to document and validate: `tools`, `mcp`, `agents`, `secrets`.
Consider emitting a warning (not an error) when an unrecognised category name is passed, to
keep the UX forgiving.
