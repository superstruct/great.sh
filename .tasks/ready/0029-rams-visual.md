# Task 0029 — Design Review: MCP Bridge CLI output

**Reviewer:** Dieter Rams
**Date:** 2026-02-27
**Files reviewed:**
- `src/cli/mcp_bridge.rs`
- `src/cli/doctor.rs` (lines 614–670, `check_mcp_bridge`)
- `src/cli/apply.rs` (lines 726–780, bridge registration block)
- `src/cli/output.rs`
- `src/cli/mod.rs`

**Commands run:**
- `great mcp-bridge --help`
- `great doctor`

---

## Verdict: REJECTED

One issue violates Principle 6 (Honest). Two issues violate Principle 8
(Thorough). All three are correctable without structural changes.

---

## Issues

### Issue 1 — Principle 6 (Honest): global flags in help text have no effect

`great mcp-bridge --help` shows:

```
  -v, --verbose                Increase output verbosity
  -q, --quiet                  Suppress all output except errors
      --non-interactive        Disable interactive prompts (for CI/automation)
```

These are global flags inherited from the root `Cli` struct via `global = true`.
Clap propagates them to all subcommands. However, `mcp_bridge::run` does not
receive or read them — the `Args` struct contains only `preset`, `backends`,
`timeout`, and `log_level`. The flags appear in the help text but alter nothing.

This is dishonest. A user running:

```
great mcp-bridge --quiet --preset full
```

will receive no indication that `--quiet` was silently ignored. The bridge
emits its own `--log-level` flag for verbosity control, which duplicates the
surface further without integrating it.

**Required correction:** Either wire `--verbose` to set `--log-level debug` and
`--quiet` to set `--log-level off` within `mcp_bridge::run`, or suppress these
inherited flags from the mcp-bridge help display entirely. The `--log-level`
flag already provides the correct granularity for this subcommand — the global
flags add confusion, not utility.

---

### Issue 2 — Principle 8 (Thorough): `check_mcp_bridge` uses double-hyphen instead of em-dash

`doctor.rs` line 665:

```rust
"great-bridge: not in .mcp.json -- run `great apply` to register",
```

All other advisory messages in doctor.rs and apply.rs use ` — ` (em-dash with
spaces) as the separator between state and action. For example:

```rust
"great.toml: not found — run `great init` to create one"  // line 557
"mise: not installed — recommended for managing..."         // line 427
```

The bridge check uses `--` (double hyphen), inconsistent with every other
message in the file. The em-dash is the established typographic convention
throughout the codebase for this "state — action" pattern.

**Required correction:** Change `--` to ` — ` at `doctor.rs` line 665.

---

### Issue 3 — Principle 8 (Thorough): help text description uses double-hyphen

`mcp_bridge.rs` line 10 (the doc comment rendered as the subcommand description):

```rust
/// Start an inbuilt MCP bridge server (stdio JSON-RPC 2.0) -- no Node.js required.
```

This appears verbatim in `great mcp-bridge --help` as:

```
Start an inbuilt MCP bridge server (stdio JSON-RPC 2.0) -- no Node.js required
```

And in `great --help` as the subcommand summary. The `--` should be an em-dash.
Compare to `mod.rs` line 80:

```rust
/// Start an inbuilt MCP bridge server (stdio JSON-RPC 2.0) -- no Node.js required
```

The same string appears in both places (doc comment and command attribute) and
both use the double-hyphen. Both require correction for consistency with the
rest of the help surface.

**Required correction:** Change `--` to ` —` in `mcp_bridge.rs` line 10 and
in `mod.rs` line 80.

---

## What is working well

**`check_mcp_bridge` follows section structure correctly.** It uses
`output::header("MCP Bridge")`, iterates backends with the established
`pass`/`warn` helpers, and ends with `println!()` for section separation.
This is visually identical to `check_mcp_servers`, `check_ai_agents`, and
`check_shell`. No structural inconsistency.

**The apply block is appropriately minimal.** Lines 726–780 follow the
established `output::header` + `output::success`/`output::info`/`output::error`
+ `println!()` pattern used by all other apply sections. The three-state
output (already registered / would register / registered) mirrors the runtime
and MCP server blocks exactly. Well-designed.

**`--log-level` is the correct control for a stdio bridge.** The bridge is a
server process — it never interacts with the user after startup. A dedicated
log-level flag scoped to stderr output is appropriate. The five-level range
(`off`, `error`, `warn`, `info`, `debug`) is sufficient and the default of
`warn` is correct for production use.

**The `any_found` guard in `check_mcp_bridge` is appropriate.** When no
backends exist at all, it emits a single consolidated warning rather than five
individual "not found" warnings. This reduces noise without losing information.
Principle 10 applied well.

---

## Summary

| # | Principle | File | Line | Severity |
|---|-----------|------|------|----------|
| 1 | 6 — Honest | `src/cli/mod.rs`, `src/cli/mcp_bridge.rs` | — | Blocking |
| 2 | 8 — Thorough | `src/cli/doctor.rs` | 665 | Blocking |
| 3 | 8 — Thorough | `src/cli/mcp_bridge.rs` + `src/cli/mod.rs` | 10 + 80 | Blocking |

All three are single-line corrections. No architectural change is required.

---

*Less, but better.*
