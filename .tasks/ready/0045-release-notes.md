# Release Notes — Task 0045: `--only` and `--skip` flags for `great apply`

**Date:** 2026-03-05
**Type:** Feature
**Scope:** `great apply`

---

## What changed

`great apply` now accepts two optional flags that limit which provisioning
categories run. This makes the command faster when you only need to reconfigure
one concern, and enables tighter CI pipelines that provision a subset of the
environment.

---

## New flags

```
--only <CATEGORY>    Run only the specified categories; skip all others.
--skip <CATEGORY>    Skip the specified categories; run all others.
```

Valid categories:

| Category  | What it covers |
|-----------|----------------|
| `tools`   | Tool installation: sudo setup, prerequisites, Homebrew, runtimes (mise), CLI tools, Bitwarden CLI, Starship, platform-specific tools, Docker, Claude Code, system tuning |
| `mcp`     | MCP server configuration and MCP bridge registration |
| `agents`  | Loop-agent file provisioning (reserved; currently a no-op) |
| `secrets` | Required secrets validation |

---

## Usage

```sh
# Apply only MCP server configuration
great apply --only mcp

# Apply tools and MCP, skip secrets
great apply --only tools,mcp

# Equivalent using repeated flags
great apply --only tools --only mcp

# Apply everything except tools (useful when tools are already installed)
great apply --skip tools

# Preview what the MCP step would do without making changes
great apply --only mcp --dry-run

# Skip secrets validation on a machine where secrets are managed externally
great apply --skip secrets
```

---

## Constraints

- `--only` and `--skip` are mutually exclusive. Using both in the same
  invocation exits with code 2:

  ```
  error: the argument '--only <CATEGORY>' cannot be used with '--skip <CATEGORY>'
  ```

- An unknown category value exits with code 2:

  ```
  error: invalid value 'foo' for '--only <CATEGORY>'
         [possible values: tools, mcp, agents, secrets]
  ```

- Comma-separated values must not include spaces. `--only tools,mcp` works;
  `--only "tools, mcp"` will fail because `" mcp"` (with the leading space) is
  not a recognised category name. Use `--only tools --only mcp` if in doubt.

- Passing `--only agents` is valid and exits 0. The `agents` category has no
  provisioning steps yet; it is accepted so that pipelines can reference it
  today without breaking when the implementation arrives.

- Config loading and platform detection always run regardless of filters. Only
  the provisioning sections are gated.

---

## Why

Running `great apply` on a machine where tools are already installed
re-runs every installation check on every invocation. When iterating on MCP
configuration, that overhead is wasted work. `--only mcp` reduces a multi-minute
apply to a few seconds. Conversely, `--skip tools` lets a CI step that only
needs secrets validation skip the full tool bootstrap.

---

## Migration

No changes are required for existing usage. When neither flag is supplied,
all categories run exactly as before. The new fields have empty `Vec` defaults.

---

## Files changed

- `src/cli/apply.rs` — `ApplyCategory` enum, `only`/`skip` fields on `Args`,
  `should_apply()` helper, per-section gates in `run()`
- `tests/cli_smoke.rs` — 7 new integration tests covering all flag combinations
  and both conflict orderings
