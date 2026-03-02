# Release Notes — Task 0034

## Feature: MCP Bridge in Init Wizard

`great init` now asks whether you want to enable the built-in MCP bridge during
project setup. The MCP bridge routes your configured MCP servers to every AI
agent in your environment — Claude, Codex, Gemini — without duplicating server
configuration per agent.

The prompt defaults to **no**, so existing workflows are unaffected. If you opt
in, `great init` writes a `[mcp-bridge]` section with `preset = "minimal"` to
your `great.toml`. You can change the preset at any time to one of four levels:

| Preset     | Intended use                                 |
|------------|----------------------------------------------|
| `minimal`  | Single-agent projects, lightweight setup     |
| `agent`    | Multi-agent development environments         |
| `research` | Research and exploration workflows           |
| `full`     | All bridges enabled; maximum connectivity    |

All four built-in templates now ship with a `[mcp-bridge]` section pre-set to
the appropriate level for that template's complexity, so teams that start from a
template get the bridge configured correctly out of the box.

### Changes

- `great init` interactive wizard: added "Enable built-in MCP bridge?" prompt
  (opt-in, defaults to no); when accepted, writes `[mcp-bridge]` with
  `preset = "minimal"` to the generated `great.toml`
- Template `ai-minimal`: added `[mcp-bridge]` with `preset = "minimal"`
- Template `ai-fullstack-ts`: added `[mcp-bridge]` with `preset = "agent"`
- Template `ai-fullstack-py`: added `[mcp-bridge]` with `preset = "agent"`
- Template `saas-multi-tenant`: added `[mcp-bridge]` with `preset = "full"`

### Migration

No action required for existing projects. The `[mcp-bridge]` section is opt-in
and has no effect until you add it to your `great.toml` and run `great apply`.
To enable the bridge on an existing project, add the following to `great.toml`
and choose the preset that matches your workflow:

```toml
[mcp-bridge]
preset = "minimal"
```
