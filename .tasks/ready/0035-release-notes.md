# Release Notes — Task 0035: Init Wizard MCP Bridge UX Polish

**Date:** 2026-02-28
**Scope:** `src/cli/init.rs`
**Cargo version:** 0.1.0 (no version bump; polish iteration)

---

## Summary

Four small UX defects in `great init` have been corrected. The `--help` text
now lists all four available templates, the MCP bridge preset menu shows tool
counts so users can make an informed choice, the wizard selects a sensible
preset based on the number of configured agents, and the success message no
longer wraps the preset name in inconsistent single quotes.

---

## User-facing changes

### `--template` lists all four templates

Previously `great init --help` advertised only three templates. The
`saas-multi-tenant` template was already handled by the code but absent from
the flag description.

```
great init --template saas-multi-tenant
```

All four template names now appear in the `--template` option description:

```
--template <TEMPLATE>
    Template to initialize from (ai-fullstack-ts, ai-fullstack-py,
    ai-minimal, saas-multi-tenant)
```

### MCP bridge preset info shows tool counts

When the interactive wizard enables the MCP bridge it now prints:

```
  Presets: minimal (1 tool) | agent (6 tools) | research (8 tools) | full (9 tools)
```

Previously the line listed bare preset names with no context. The tool counts
map directly to the groups exposed via `tools/list`:

| Preset     | Tools |
|------------|-------|
| `minimal`  | 1 (prompt only) |
| `agent`    | 6 (prompt, run, wait, list_tasks, get_result, kill_task) |
| `research` | 8 (agent + research, analyze_code) |
| `full`     | 9 (research + clink) |

The preset written to `great.toml` is unchanged; this is display-only
information. To change the preset after init, edit `great.toml`:

```toml
[mcp-bridge]
preset = "research"
```

### Smart preset auto-selection based on agent count

When the MCP bridge is enabled, the wizard previously always wrote
`preset = "minimal"` regardless of how many agents were configured. It now
selects the default based on agent count:

| Agents configured | Default preset |
|-------------------|----------------|
| Claude only (1 agent) | `minimal` |
| Claude + Codex or Gemini (2+ agents) | `agent` |

The `"agent"` preset adds multi-backend dispatch tools (`run`, `wait`,
`list_tasks`, `get_result`, `kill_task`) that are only useful when more than
one backend is available. Selecting it automatically avoids a configuration
mismatch that would otherwise require a manual edit.

The heuristic is agent-count-based in the wizard context only. Template files
use complexity-based preset selection (for example, `ai-fullstack-ts` and
`ai-fullstack-py` default to `"agent"` even with a single agent configured,
because fullstack projects benefit from longer-running tasks). These semantics
differ intentionally.

### Consistent success message formatting

The success message no longer wraps the preset name in single quotes:

```
# Before
MCP bridge enabled with 'minimal' preset

# After
MCP bridge enabled with minimal preset
```

No other wizard message used that quoting style.

---

## No migration needed

No configuration file changes are required. The `--template` and wizard changes
are purely additive: existing `great.toml` files are unaffected, and users who
have already run `great init` will not see any difference.

The only observable behavioral change is the preset that gets written to a
newly created `great.toml` when both MCP bridge and a second agent are enabled
in the same wizard session. That file is created fresh and reviewed before
`great apply` is run, so there is no risk to existing environments.
