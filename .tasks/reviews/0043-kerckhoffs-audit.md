# 0043 Security Audit -- MCP `issues.push()` Bug Fix

**Auditor:** Auguste Kerckhoffs
**Date:** 2026-03-04
**Verdict:** PASS -- no CRITICAL or HIGH findings

## Scope

Single-file change to `src/cli/status.rs`: replace closure chain (`and_then`/`map`/`iter().map()`) with explicit `if-let` loop so that `issues.push()` can be called when an MCP server command is missing from PATH.

## Diff Summary

- Removed: `config.and_then(|cfg| cfg.mcp.as_ref().map(|mcps| mcps.iter().map(...).collect()))` (lines 357-369 old)
- Added: `if let Some(cfg) = config { if let Some(mcps) = cfg.mcp.as_ref() { for (name, m) in mcps { ... } } }` (lines 357-382 new)
- Net change: +13 lines, no new dependencies, no new files.

## Checklist

| # | Check | Result | Notes |
|---|-------|--------|-------|
| 1 | Credential leakage | PASS | Only `name` (TOML key) and `m.command` (binary name) are interpolated. No `env`, `api_key`, `url`, or secret refs exposed. |
| 2 | Injection safety | PASS | `format!()` produces a `String` pushed into `Vec<String>`. Serde's `to_string_pretty()` handles JSON escaping. No shell, HTML, or SQL context. |
| 3 | New dependencies | PASS | No changes to `Cargo.toml`. |
| 4 | Pattern consistency | PASS | Matches existing `if-let` pattern used by tools block (L311-355) and secrets block (L396-421). |
| 5 | Input validation | PASS | `command_exists()` uses the `which` crate (no shell spawning). Server name and command are read-only from parsed TOML. |
| 6 | File permissions | N/A | No file I/O changes. |
| 7 | Supply chain | PASS | No new crates. |

## Findings

None.
