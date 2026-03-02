# 0032 Socrates Review: Marketing Site MCP Bridge Feature

**Spec:** `.tasks/ready/0032-site-mcp-bridge-spec.md`
**Backlog:** `.tasks/backlog/0032-site-mcp-bridge-feature.md`
**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-28
**Round:** 1

---

## VERDICT: REJECTED

---

## Concerns

### 1. TOML section header uses wrong key format

```
{
  "gap": "Spec shows `[mcp_bridge]` (underscore) in sampleToml, but the Rust schema at src/config/schema.rs:25 has `#[serde(rename = \"mcp-bridge\")]`, meaning the actual TOML section header must be `[mcp-bridge]` (hyphen). The sampleToml will show users an invalid config key that will be silently ignored by the parser.",
  "question": "The spec explicitly calls out that Rust uses `mcp_bridge` with underscore -- but did anyone verify this against the serde rename annotation? What TOML key does the parser actually accept?",
  "severity": "BLOCKING",
  "recommendation": "Change `[mcp_bridge]` to `[mcp-bridge]` in sampleToml. Note: the backlog item at line 32 also has this wrong (`[mcp_bridge]`), so this is an inherited error that the spec must correct, not propagate."
}
```

### 2. TOML inner field uses wrong key format

```
{
  "gap": "Spec shows `default_backend = \"gemini\"` (underscore) inside the bridge stanza, but McpBridgeConfig at src/config/schema.rs:145 has `#[serde(rename_all = \"kebab-case\")]`, meaning the TOML key must be `default-backend` (hyphen). Same issue for `timeout_secs` -> `timeout-secs`, `auto_approve` -> `auto-approve`, `allowed_dirs` -> `allowed-dirs`.",
  "question": "Given the `rename_all = kebab-case` annotation on McpBridgeConfig, should the sample TOML not use `default-backend` instead of `default_backend`?",
  "severity": "BLOCKING",
  "recommendation": "Change `default_backend = \"gemini\"` to `default-backend = \"gemini\"` in sampleToml. The `preset` field is a single word and is unaffected by the rename."
}
```

### 3. Demo output version `v0.3.0` is fabricated and will age poorly

```
{
  "gap": "The mcpBridgeOutput string ends with `Server: great-mcp-bridge v0.3.0`, but Cargo.toml shows `version = \"0.1.0\"`. No other demo string in commands.ts includes a version number. The bridge's get_info() in server.rs uses `env!(\"CARGO_PKG_VERSION\")` which would output `0.1.0` in the real binary.",
  "question": "Why does the demo include a version number that no other demo uses, and why is it `0.3.0` when the crate is at `0.1.0`? Will this confuse users who run the actual command and see a different version?",
  "severity": "ADVISORY",
  "recommendation": "Either remove the version from the demo output entirely (matching the style of loopInstallOutput and initWizardOutput, neither of which show versions), or use the actual current version `0.1.0`."
}
```

### 4. Grid layout with 5 cards leaves orphan -- no visual design decision recorded

```
{
  "gap": "Spec acknowledges the 5th card will sit alone in the last row of the 2-column grid (`md:grid-cols-2`), calls it 'acceptable and actually desirable,' and then offers an optional `md:col-span-2` alternative. The spec says the optional approach is NOT part of required changes, but does not record who made this design decision or whether Rams/Nielsen were consulted.",
  "question": "Was this a deliberate visual design decision or a 'good enough' assumption? An orphan card in a 2-column grid can look incomplete rather than prominent.",
  "severity": "ADVISORY",
  "recommendation": "This is fine as-is for a data-only task. If the team wants the 5th card to span both columns (or to switch to a 3-column grid at larger breakpoints), that is a separate UX task. The spec correctly identifies the tradeoff -- no action needed beyond noting the decision was made by the spec author, not a designer."
}
```

### 5. `Unplug` icon is confirmed available but semantically questionable

```
{
  "gap": "The Unplug icon in lucide-react (confirmed at site/node_modules/lucide-react/dynamicIconImports.d.ts line 1524) visually represents a disconnected plug, not a bridge or connection. The spec says it 'visually represents a connection/bridge between systems (two connectors linking together)' -- but Unplug literally means the opposite of connected.",
  "question": "Was `Cable`, `Plug`, `PlugZap`, `Network`, or `Link2` considered? An icon named 'Unplug' representing a bridge feature is semantically inverted.",
  "severity": "ADVISORY",
  "recommendation": "Builder should evaluate alternatives. `Cable` or `Network` may be better semantic fits. This is a judgment call, not a correctness issue -- the icon will render without error regardless."
}
```

### 6. Content accuracy: tool and backend counts are correct

```
{
  "gap": "None -- verified against shipped code.",
  "question": "N/A",
  "severity": "ADVISORY",
  "recommendation": "Confirmed: 9 tools (server.rs: prompt, run, wait, list_tasks, get_result, kill_task, research, analyze_code, clink), 5 backends (backends.rs: gemini, codex, claude, grok, ollama), 4 presets (tools.rs: minimal, agent, research, full). All claims accurate."
}
```

### 7. `mcpBridgeOutput` is exported but unused -- no lint risk?

```
{
  "gap": "Spec notes this export is not wired to any component and will be used in a future 0033 task. TypeScript allows unused exports without error.",
  "question": "Will `pnpm build:site` (which runs `tsc -b`) flag unused exports? If the tsconfig has `noUnusedLocals` it could fail.",
  "severity": "ADVISORY",
  "recommendation": "Confirmed: TS strict mode does not flag unused exports (only unused locals/parameters). The existing `loopInstallOutput` export in commands.ts would already fail if this were an issue. No risk."
}
```

---

## Summary

The spec is well-structured and proportionate for a data-only site task, but it propagates an **incorrect TOML key format** from the backlog: the Rust schema uses `#[serde(rename = "mcp-bridge")]` and `#[serde(rename_all = "kebab-case")]`, so the sample TOML must use `[mcp-bridge]` and `default-backend`, not `[mcp_bridge]` and `default_backend`. Marketing copy showing invalid config keys that would be silently dropped by the parser is a user-facing correctness issue. Fix the two BLOCKING concerns and this spec is ready.

---

## Evidence

| Claim | Source | Verified |
|-------|--------|----------|
| Section header `[mcp-bridge]` | `src/config/schema.rs:25` -- `#[serde(rename = "mcp-bridge")]` | Yes -- hyphen, not underscore |
| Inner fields kebab-case | `src/config/schema.rs:145` -- `#[serde(rename_all = "kebab-case")]` | Yes -- `default-backend`, not `default_backend` |
| 9 tools total | `src/mcp/bridge/tools.rs:176-185` -- Full preset vec | Yes |
| 5 backends | `src/mcp/bridge/backends.rs:31-77` -- BACKEND_SPECS array | Yes |
| 4 presets | `src/mcp/bridge/tools.rs:132-141` -- Preset enum | Yes |
| `Unplug` in lucide-react | `site/node_modules/lucide-react/dynamicIconImports.d.ts:1524` | Yes |
| lucide-react version | `site/package.json:16` -- `"lucide-react": "^0.511.0"` | Yes |
| Grid class | `Features.tsx:26` -- `grid grid-cols-1 md:grid-cols-2 gap-6` | Yes -- 5th card orphans in col 1 |
| Crate version | `Cargo.toml:3` -- `version = "0.1.0"` | Yes -- not 0.3.0 |
| Agent preset tools | `src/mcp/bridge/tools.rs:158-165` | Yes -- matches spec's mcpBridgeOutput |
