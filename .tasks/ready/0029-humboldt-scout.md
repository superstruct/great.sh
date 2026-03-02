# 0029: Inbuilt MCP Bridge Server -- Humboldt Scout Report

**Scout:** Humboldt (Codebase Scout)
**Spec:** `.tasks/ready/0029-mcp-bridge-spec.md`
**Review:** `.tasks/ready/0029-socrates-review.md`
**Date:** 2026-02-27

---

## 1. Files to Modify -- Exact Insertion Points

### 1.1 `src/cli/mod.rs`

Current state: 79 lines. Module list at lines 1-17, `Command` enum at lines 41-78.

**Insertion 1 -- module declaration** after line 7 (`pub mod mcp;`):
```rust
pub mod mcp_bridge;
```

**Insertion 2 -- Command variant** after line 77 (`Statusline(statusline::Args),`),
before the closing `}` at line 78:
```rust
    /// Start an inbuilt MCP bridge server (stdio JSON-RPC 2.0) -- no Node.js required
    #[command(name = "mcp-bridge")]
    McpBridge(mcp_bridge::Args),
```

### 1.2 `src/main.rs`

Current state: 40 lines. Match arm block at lines 17-39.

**Insertion** after line 38 (`Command::Statusline(args) => cli::statusline::run(args),`):
```rust
        Command::McpBridge(args) => cli::mcp_bridge::run(args),
```

### 1.3 `src/mcp/mod.rs`

Current state: 284 lines. Module is a flat file (no submodule declarations -- it IS the module).

**Insertion** at line 1 (before all existing `use` statements):
```rust
pub mod bridge;
```

This adds the bridge submodule to the `mcp` module. All existing code is unaffected.

### 1.4 `src/config/schema.rs`

Current state: 810 lines. `PlatformOverride` struct ends at line 135. `ConfigMessage` enum begins at line 138. `GreatConfig` struct at lines 10-24. `validate()` method at lines 153-242.

**Insertion A -- `McpBridgeConfig` struct** between line 135 and 137 (between `PlatformOverride` and `ConfigMessage`):
```rust
/// Configuration for the `[mcp-bridge]` section of `great.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct McpBridgeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backends: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
}
```

**Insertion B -- field on `GreatConfig`** after line 23 (`pub platform: Option<PlatformConfig>,`):
```rust
    /// MCP bridge server configuration.
    #[serde(rename = "mcp-bridge", skip_serializing_if = "Option::is_none")]
    pub mcp_bridge: Option<McpBridgeConfig>,
```

**Insertion C -- validation** in `validate()`, after the MCP transport checks (after the closing `}` of the MCP loop, around line 208). Insert before the secrets provider check:
```rust
        // Check: mcp-bridge preset and backends must be known values
        if let Some(bridge) = &self.mcp_bridge {
            if let Some(preset) = &bridge.preset {
                let known_presets = ["minimal", "agent", "research", "full"];
                if !known_presets.contains(&preset.as_str()) {
                    messages.push(ConfigMessage::Warning(format!(
                        "mcp-bridge: unknown preset '{}' -- known presets: {}",
                        preset,
                        known_presets.join(", ")
                    )));
                }
            }
            if let Some(backends) = &bridge.backends {
                let known_backends = ["gemini", "codex", "claude", "grok", "ollama"];
                for b in backends {
                    if !known_backends.contains(&b.as_str()) {
                        messages.push(ConfigMessage::Warning(format!(
                            "mcp-bridge: unknown backend '{}' -- known backends: {}",
                            b,
                            known_backends.join(", ")
                        )));
                    }
                }
            }
        }
```

### 1.5 `src/cli/apply.rs`

Current state: 1007 lines. MCP servers block ends around line 723 (the `println!()` after
the `if changed` block). Bitwarden section begins around line 727.

**Insertion** between the MCP servers println!() (line 723) and the bitwarden section (line 727).
The spec calls this "5a" and labels the bitwarden section "5b". Insert the entire bridge
registration block there -- see spec section 4.1 for the full code.

Key pattern: the block uses `crate::mcp::project_mcp_path()` and `crate::mcp::McpJsonConfig`
(already public in `src/mcp/mod.rs`), and `crate::mcp::McpServerEntry` (also already public).
No new imports needed in apply.rs -- the `crate::mcp::` prefix works with existing mod structure.

**The `save()` method** on `McpJsonConfig` is currently `#[allow(dead_code)]` (line 46 of
`src/mcp/mod.rs`). This block will be the first real caller -- remove the `#[allow(dead_code)]`
attribute from that method when wiring this in.

### 1.6 `src/cli/doctor.rs`

Current state: 799 lines. The `run()` function's check dispatch is at lines 70-96.
`check_mcp_servers` call is at line 89-90. The function ends at line 96.
`check_mcp_servers` function is at lines 572-603.

**Insertion A -- call site** after line 90 (`check_mcp_servers(&mut result, cfg);`),
still inside the `if let Some(ref cfg) = loaded_config` block:
```rust
    // 7b. MCP Bridge backend checks
    check_mcp_bridge(&mut result);
```

IMPORTANT: Per Socrates concern #12, the spec calls `check_mcp_bridge` OUTSIDE the
`if let Some(ref cfg) = loaded_config` guard. The builder must decide:
- Option A (recommended): keep it inside the guard, guarded by `cfg.mcp_bridge.is_some()`
- Option B: call it unconditionally but change the "not in .mcp.json" warning to info
  when no [mcp-bridge] config exists

The current call in the spec is unconditional. The scout recommends Option A for
UX cleanliness.

**Insertion B -- function body** after `check_mcp_servers` (after line 603):
Full function as specified in spec section 4.2. Uses `crate::mcp::project_mcp_path()`
and `crate::mcp::McpJsonConfig::load()` -- both already public.

### 1.7 `Cargo.toml`

Current state: 31 lines. Dependencies block at lines 10-25. `which = "7"` is the last
dependency at line 25.

**Add after line 25:**
```toml
rmcp = { version = "0.16", features = ["server", "transport-io"] }
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
schemars = "1.0"
```

**Also add a Unix-only section** (Socrates concern #3 -- explicit libc dependency):
```toml
[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

The `libc` crate is already a transitive dep (confirmed at version 0.2.182 in `Cargo.lock`)
and `tracing` at 0.1.44 is also already transitive. Adding them explicitly is correct hygiene.

---

## 2. Files to Create

| File | Phase | Role |
|------|-------|------|
| `src/mcp/bridge/mod.rs` | 1 | Module root, re-exports submodules |
| `src/mcp/bridge/backends.rs` | 1 | `BackendConfig`, `discover_backends()`, `build_command_args()` |
| `src/mcp/bridge/registry.rs` | 1 | `TaskState`, `TaskRegistry`, process lifecycle |
| `src/mcp/bridge/tools.rs` | 2 | Parameter structs, `Preset` enum |
| `src/mcp/bridge/server.rs` | 2 | `GreatBridge` rmcp server, `start_bridge()` |
| `src/cli/mcp_bridge.rs` | 3 | CLI `Args`, `pub fn run()` |
| `tests/mcp_bridge_protocol.sh` | 5 | Manual protocol pipe test |

Directory to create: `/home/isaac/src/sh.great/src/mcp/bridge/`

---

## 3. Existing Patterns to Follow

### 3.1 Args struct pattern (from `src/cli/status.rs` lines 73-82, `src/cli/doctor.rs` lines 9-20)

```rust
#[derive(ClapArgs)]
pub struct Args {
    /// Description of flag
    #[arg(long)]
    pub some_flag: bool,

    /// Hidden field set by main.rs (not a CLI arg)
    #[arg(skip)]
    pub non_interactive: bool,
}
```

The `mcp_bridge.rs` Args struct uses `#[arg(long, default_value = "agent")]` for the
preset -- this pattern is correct for clap derive.

### 3.2 `run()` entry point (from `src/cli/update.rs` lines 20-33)

Tokio runtime creation follows this exact pattern:
```rust
pub fn run(args: Args) -> Result<()> {
    // ... sync setup ...
    let rt = tokio::runtime::Runtime::new().context("failed to create tokio runtime")?;
    rt.block_on(async_fn())
}
```

This is the third runtime creation site (update.rs line 26, template.rs line 187 are the
existing two). The pattern is established. No change to `fn main()` needed.

### 3.3 Doctor check function pattern (from `src/cli/doctor.rs` lines 283-365)

Each check function:
1. Starts with `output::header("Section Name")`
2. Calls `pass()`, `warn()`, or `fail()` helpers (lines 268-281) with the `result` ref
3. Ends with `println!()`
4. Never returns Err -- swallows all internal errors, uses `warn`/`fail` instead

### 3.4 Apply section pattern (from `src/cli/apply.rs` lines 649-723)

MCP section pattern:
```rust
if let Some(mcps) = &cfg.mcp {
    if !mcps.is_empty() {
        output::header("MCP Servers");
        // ... do work ...
        println!();
    }
}
```

The bridge registration block follows this with `if cfg.mcp_bridge.is_some()`.

### 3.5 McpJsonConfig usage (from `src/mcp/mod.rs`)

Already used in tests. The `save()` method (line 46) and `add_server()` (line 53) are
both `#[allow(dead_code)]`. The bridge registration will be their first production caller.
Use `mcp_json.mcp_servers.insert(...)` directly (as the spec does) rather than
`add_server()` since we need a `McpServerEntry` not a `McpConfig`.

### 3.6 Error handling

- Production code: `?` propagation, `anyhow::bail!()`, `.context()`
- No `.unwrap()` in production paths (Socrates concern #1)
- The spec's `system_prompt.unwrap()` at `backends.rs` line ~322 is already fixed in
  the spec's own code via `if let Some(sp) = system_prompt { ... }` pattern (lines 307-314)
- The `serde_json::to_string().unwrap_or_else(|_| "[]".to_string())` pattern in server.rs
  is acceptable: the fallback is a safe default, not a panic path

---

## 4. Dependency Analysis

### 4.1 Current `Cargo.toml` (25 deps, 278 transitive in lockfile)

```toml
clap 4.5, anyhow 1.0, serde 1.0, serde_json 1.0, toml 0.8,
tokio 1.0 (full), reqwest 0.12, zip 2, dirs 6.0, colored 3.0,
indicatif 0.17, regex 1.0, toml_edit 0.22, semver 1.0, which 7
```

### 4.2 New dependencies and their status

| Dep | Version | Transitive? | Note |
|-----|---------|-------------|------|
| `rmcp` | 0.16 | No | First appearance -- largest new dep |
| `uuid` | 1.x | No | Not in lockfile -- new |
| `tracing` | 0.1 | YES (0.1.44) | Already transitive via tokio |
| `tracing-subscriber` | 0.3 | No | Not in lockfile -- new |
| `schemars` | 1.0 | Unknown | CRITICAL: verify rmcp 0.16 uses schemars 1.0 not 0.8.x |
| `libc` | 0.2 | YES (0.2.182) | Already transitive via tokio on Unix |

### 4.3 Schemars version risk (Socrates concern #11 -- highest compile risk)

`schemars` 1.0 is a major break from 0.8.x. If rmcp 0.16 uses schemars 0.8 internally
and the workspace declares schemars 1.0 explicitly, derive macro failures will occur at
compile time. The builder's first step after adding deps must be `cargo check`. If it
fails with schemars-related errors, change `schemars = "1.0"` to `schemars = "0.8"`.

### 4.4 Binary size note

Current binary is lean (15 deps, no async proc macro frameworks). Adding rmcp + tracing-subscriber
will be non-trivial. Builder should run `cargo build --release` and measure size delta
before and after for the observer report.

---

## 5. Risks and Surprises

### Risk 1: rmcp 0.16 API is unverified (HIGH)

Eight "builder must verify" items in the spec. The proc macro API
(`#[tool_router]`, `#[tool_handler]`, `#[tool(param)]`, `Parameters<T>`) is inferred
from rmcp documentation but not compiled against. The spec is transparent about this.

**Mitigation:** Builder has explicit latitude to adjust types and signatures while
preserving R1-R7 behavior. Start with Phase 1 (no rmcp) and `cargo check` before
proceeding to Phase 2.

### Risk 2: First `unsafe` code in the codebase (MEDIUM)

`src/` currently has zero files containing `unsafe`. The `pre_exec` / `killpg` calls
in `registry.rs` introduce it. Confirmed: `grep -r "unsafe" src/ | wc -l` = 0.

**Mitigation:** Gate all unsafe behind `#[cfg(unix)]`. Add `// SAFETY:` comments
per Socrates recommendation. Add `libc` explicitly to `[target.'cfg(unix)'.dependencies]`.

### Risk 3: `save()` method `#[allow(dead_code)]` annotation needs cleanup (LOW)

`src/mcp/mod.rs` line 46: `McpJsonConfig::save()` is annotated `#[allow(dead_code)]`.
The bridge registration in `apply.rs` will call it. Remove the dead_code annotation.
Same for `add_server()` at line 53 (though it won't be called directly).

### Risk 4: `shutdown_all()` not called on exit (Socrates concern #13) (MEDIUM)

The `start_bridge()` function in the spec does NOT call `registry.shutdown_all()`
after `service.waiting().await?` returns. Background tokio tasks holding process
handles will be abandoned on exit. The builder must add:
```rust
registry.shutdown_all().await;
```
after the `service.waiting()` await in `server.rs::start_bridge()`.

### Risk 5: `check_mcp_bridge` runs unconditionally (Socrates concern #12) (LOW)

Spec calls `check_mcp_bridge(&mut result)` unconditionally in `doctor.rs::run()`.
This produces noisy output (5 "not found" lines) for users without any AI CLIs.
Recommended fix: guard with `if cfg.mcp_bridge.is_some()` or change to info-level.

### Risk 6: CLI help text says "chat" for a preset that does not exist (LOW)

`src/cli/mcp_bridge.rs` Args doc string says "minimal, chat, agent, full" but the
`Preset` enum has "minimal, agent, research, full". Fix the help text to match.

### Risk 7: `McpJsonConfig::load()` uses `unwrap_or_default()` in apply.rs bridge block

The spec writes:
```rust
let mut mcp_json = crate::mcp::McpJsonConfig::load(&mcp_json_path).unwrap_or_default();
```

`McpJsonConfig::load()` returns `Result<Self>`. This `.unwrap_or_default()` silently
swallows a potential parse error (corrupt `.mcp.json`). Better pattern (consistent
with project convention):
```rust
let mut mcp_json = crate::mcp::McpJsonConfig::load(&mcp_json_path)
    .context("failed to read .mcp.json")?;
```
But this would fail `great apply` if `.mcp.json` is corrupt. The existing MCP block
(apply.rs lines 655-661) handles this via `unwrap_or_default()` too -- so the spec
is consistent with the existing pattern. Accept it.

---

## 6. Dependency Map (Build Order)

```
Phase 1 (no cross-deps, compile-verify first):
  Cargo.toml           -- add deps, cargo check
  config/schema.rs     -- McpBridgeConfig (standalone struct)
  mcp/bridge/backends.rs -- BackendConfig, discover_backends, build_command_args
  mcp/bridge/registry.rs -- TaskRegistry (depends on backends)
  mcp/bridge/mod.rs    -- declares backends, registry, server, tools
  mcp/mod.rs           -- add pub mod bridge

Phase 2 (depends on Phase 1 + rmcp):
  mcp/bridge/tools.rs  -- parameter structs, Preset (schemars derive)
  mcp/bridge/server.rs -- GreatBridge (rmcp macros, depends on tools + registry)

Phase 3 (depends on Phase 2):
  cli/mcp_bridge.rs    -- Args, run() (depends on server, registry, backends, tools)
  cli/mod.rs           -- add pub mod mcp_bridge, McpBridge variant
  main.rs              -- add dispatch arm

Phase 4 (depends on Phase 3):
  cli/apply.rs         -- bridge registration block
  cli/doctor.rs        -- check_mcp_bridge function

Phase 5 (depends on Phase 3):
  tests/cli_smoke.rs   -- mcp_bridge_help and mcp_bridge_unknown_preset tests
  tests/mcp_bridge_protocol.sh -- manual pipe test
```

---

## 7. Technical Debt Flagged

1. **Three tokio runtime creation sites** (update.rs:26, template.rs:187, mcp_bridge.rs:new).
   Socrates notes this should trigger a migration to `#[tokio::main]` when a fourth
   is needed. Track this.

2. **`McpJsonConfig::save()` and `add_server()` were dead code** for the entire codebase
   until this task. The bridge registration is their first real caller.

3. **`check_mcp_bridge` always runs** in the spec's design -- not guarded by config.
   This is inconsistent with `check_mcp_servers` which only runs when the config has
   MCP entries. Builder should align the two patterns.

4. **The existing MCP block in `apply.rs`** (lines 649-723) uses raw `serde_json::Value`
   manipulation rather than `McpJsonConfig` -- a minor inconsistency. The new bridge
   block correctly uses `McpJsonConfig`. The existing block is technical debt but do
   not change it in this task.

---

## 8. Exact Line Count Summary

| File | Current Lines | Key Lines |
|------|--------------|-----------|
| `src/cli/mod.rs` | 79 | Insert after line 7 (module) and after line 77 (variant) |
| `src/main.rs` | 40 | Insert after line 38 |
| `src/mcp/mod.rs` | 284 | Insert at line 1 |
| `src/config/schema.rs` | 810 | Insert at lines 136, 24, and ~208 |
| `src/cli/apply.rs` | 1007 | Insert after line ~723 |
| `src/cli/doctor.rs` | 799 | Insert after line 90 (call) and after line 603 (function) |
| `Cargo.toml` | 31 | Insert after line 25 |
