# 0029: Inbuilt MCP Bridge Server -- Socrates Review

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Spec:** `.tasks/ready/0029-mcp-bridge-spec.md`
**Backlog:** `.tasks/backlog/0029-inbuilt-mcp-bridge.md`
**Date:** 2026-02-27
**Round:** 1

---

## VERDICT: APPROVED WITH CONDITIONS

There are no strictly BLOCKING issues that prevent implementation from starting, but several WARNING-level concerns require deliberate decisions before the builder proceeds. The spec is unusually thorough for its size and correctly anticipates many failure modes. The primary risks are: (1) introduction of `unsafe` code into a codebase that currently has zero `unsafe` blocks, (2) the `rmcp` 0.16 API is treated as gospel despite explicit "builder must verify" caveats, and (3) a `.unwrap()` in production code violates a hard project convention.

---

## Concerns

### 1. Production `.unwrap()` in `build_command_args()` -- Convention Violation

```
gap: Line 322 of the spec contains `system_prompt.unwrap()` inside `build_command_args()`
     in `src/mcp/bridge/backends.rs`. This is production code, not a test.
question: The CLAUDE.md rule is "No `.unwrap()` in production code -- propagate errors
          with `?`". Why does this spec introduce an `.unwrap()` in a non-test path?
severity: WARNING
recommendation: The code on line 321-323 checks `system_prompt.is_some()` before calling
                `.unwrap()`, so it cannot actually panic. However, this pattern is fragile
                and violates the letter of the convention. Replace with:
                `if let Some(sp) = system_prompt { ... }` which is already the idiomatic
                Rust pattern. The builder (Da Vinci) should fix this during implementation.
```

### 2. First `unsafe` Code in the Entire Codebase

```
gap: The codebase currently has ZERO `unsafe` blocks anywhere in `src/`. This spec
     introduces multiple `unsafe` blocks: `cmd.pre_exec(|| { libc::setpgid(0, 0); Ok(()) })`
     in registry.rs line 509-513, `libc::killpg(pid as libc::pid_t, libc::SIGTERM)` at
     line 608, and `libc::killpg(*pid as libc::pid_t, libc::SIGKILL)` at lines 613, 631.
question: Has the team made a deliberate decision to introduce `unsafe` into the codebase?
          The spec acknowledges it ("The builder should wrap these in well-documented
          functions and add `// SAFETY:` comments") but does not establish a project-wide
          policy. What is the acceptable boundary for `unsafe` in great.sh going forward?
severity: WARNING
recommendation: (1) The spec already notes this and defers to the builder for SAFETY
                comments -- this is acceptable. (2) The team should decide whether to
                add `libc` as an explicit dependency in Cargo.toml rather than relying on
                it being transitively available through tokio. Transitive deps can disappear
                across major versions. (3) Consider whether the alternative approach
                (process-wrap crate, which the backlog originally required) would avoid
                the need for hand-written unsafe entirely. The spec explicitly dropped
                process-wrap to reduce complexity, which is a reasonable trade-off, but
                this means the team is taking on the safety obligation directly.
```

### 3. `libc` Used Without Explicit Cargo.toml Dependency

```
gap: The spec calls `libc::setpgid`, `libc::killpg`, `libc::pid_t`, and `libc::SIGTERM`/
     `libc::SIGKILL` but does NOT add `libc` to `[dependencies]` in Cargo.toml. The spec
     states "The libc crate is a transitive dependency of tokio (on Unix targets) and is
     always available."
question: What happens when tokio changes its dependency graph in a future release and
          libc is no longer a transitive dependency, or when the transitive version is
          incompatible? Using a transitive dep directly is a Cargo anti-pattern.
severity: WARNING
recommendation: Add `libc = "0.2"` explicitly to Cargo.toml with a `[target.'cfg(unix)'.dependencies]`
                section, or alternatively use `nix` (safer wrapper) or `process-wrap` as
                the spec originally considered. The builder should make this call. This is
                a low-risk issue in practice (libc is deeply entrenched in the Rust ecosystem)
                but violates dependency hygiene.
```

### 4. Backlog/Spec Discrepancy: Tool Count ("8" vs 9)

```
gap: The backlog acceptance criteria (line 419-421) say "--preset full exposes all 8 tools"
     then immediately lists 9 tools: prompt, run, wait, list_tasks, get_result, kill_task,
     research, analyze_code, clink. The spec correctly says 9 tools throughout.
question: Is this just a typo in the backlog, or was `clink` intended to be dropped?
severity: QUESTION
recommendation: The spec treats 9 as canonical and the backlog's "8" as a typo. This
                seems correct. No action needed unless the team intended to defer `clink`.
```

### 5. Backlog/Spec Discrepancy: Naming Convention (`MpcBridge` vs `McpBridge`)

```
gap: The backlog consistently uses the typo `MpcBridge` and `MpcBridgeConfig` (lines 52, 67,
     169, 444-445, 454). The spec corrects this to `McpBridge` and `McpBridgeConfig` throughout.
question: Is this correction intentional and agreed upon?
severity: QUESTION
recommendation: The spec's correction is obviously right. "MPC" is not a thing; "MCP"
                (Model Context Protocol) is. No action needed, just noting the silent fix.
```

### 6. Preset Name Discrepancy: Backlog Says "chat", Spec Says "minimal"

```
gap: The backlog's Args description (line 73) lists presets as "minimal, chat, agent, full".
     The spec's CLI Args struct (line 1385-1388) lists "minimal, chat, agent, full" in the
     doc string but actually implements only "minimal, agent, research, full" in the Preset
     enum (tools.rs). The "chat" preset name from the backlog is silently renamed to "minimal"
     and a new "research" preset is added that the backlog did not mention.
question: Was the rename from "chat" to "minimal" and the addition of "research" as a
          fourth preset discussed with the requirements owner? The backlog has 4 presets
          (minimal, chat, agent, full) and the spec has 4 (minimal, agent, research, full)
          but they are different sets.
severity: WARNING
recommendation: The spec's preset hierarchy (minimal -> agent -> research -> full) is more
                logical than the backlog's (minimal -> chat -> agent -> full) since "chat"
                and "minimal" would be redundant if both expose only the `prompt` tool. But
                this should be explicitly acknowledged as a requirements change, not silently
                introduced. The CLI help text at line 1385 still says "chat" which is now
                invalid -- this will confuse users. Fix the help text to say
                "minimal, agent, research, full".
```

### 7. `rmcp` API Assumptions -- Multiple "Builder Must Verify" Disclaimers

```
gap: The spec contains at least 8 explicit "builder must verify" notes (Section 2.2 lines
     1282-1344, Risks section items 1-8). These are not normal spec caveats -- they indicate
     genuine uncertainty about whether the specified code will compile. Key unknowns:
     (a) Whether `#[tool(param)] params: Parameters<T>` is the correct macro syntax
     (b) Whether `list_tools()` can be overridden when `#[tool_handler]` is also used
     (c) Whether `CallToolResult::success()` / `CallToolResult::error()` exist with those signatures
     (d) Whether `ProtocolVersion::LATEST` is a constant or method
     (e) Whether `Implementation` has a `title` field
question: Given that the spec acknowledges significant API uncertainty, what is the
          fallback if rmcp 0.16's actual API diverges from the spec's assumptions?
          How much implementation latitude does the builder have to deviate from the spec?
severity: WARNING
recommendation: This is acceptable for an XL task with a third-party dependency. The spec
                is transparent about the uncertainties. The builder should be given explicit
                latitude to adjust types, method signatures, and macro usage as needed to
                match rmcp's actual API, so long as the functional behavior matches R1-R7.
                Recommend adding a note: "The builder may deviate from exact type signatures
                shown in this spec where rmcp's actual API requires it, provided the
                behavioral contract (tools exposed, error handling, lifecycle) is preserved."
```

### 8. Tokio Runtime Creation -- Pre-existing Pattern But Growing Risk

```
gap: The spec creates a new `tokio::runtime::Runtime` in `mcp_bridge.rs` run(). This is
     the same pattern used by `update.rs` (line 26) and `template.rs` (line 187). However,
     this will be the third location creating a runtime.
question: At what point should the project migrate `main()` to `#[tokio::main]` async fn
          instead of creating per-subcommand runtimes? With 3 runtime creation sites, is
          there risk of nested runtime errors if someone calls bridge code from apply.rs?
severity: QUESTION
recommendation: No action needed for this task. The spec correctly identifies the risk
                (section 3.1, paragraph after the code block) and the current isolation
                is safe. But this is technical debt accumulating -- the team should plan
                a migration to async main when a 4th subcommand needs a runtime.
```

### 9. Security: `--dangerously-skip-permissions` Is Hardcoded, Not Opt-in

```
gap: The `BackendConfig` for Claude hardcodes `auto_approve_flag: Some("--dangerously-skip-permissions")`
     in the BACKEND_SPECS constant (line 221). This flag is always passed when the Claude
     backend is used through the bridge. The spec's security section (item 4) argues this is
     acceptable because "it is the user's explicit choice to enable this backend."
question: Is merely having `claude` on PATH truly an "explicit choice" to skip all permission
          checks? A user who installs Claude Code for interactive use might not expect that
          adding `[mcp-bridge]` to their great.toml would invoke it with full auto-approval.
          Should there be a separate opt-in for dangerous flags, or at minimum a warning
          during `great doctor` or `great apply`?
severity: WARNING
recommendation: Consider making auto-approve behavior configurable per-backend in the
                `[mcp-bridge]` config, with a default of true to match the spec's current
                behavior but allowing users to disable it. Alternatively, add a warning
                line in `great doctor`'s MCP Bridge section: "claude backend uses
                --dangerously-skip-permissions (full auto-approval)". The current spec
                has no such warning.
```

### 10. File Access in `research`/`analyze_code` -- Path Traversal Not Addressed

```
gap: The `research` tool accepts arbitrary file paths from the MCP client and reads them.
     The `analyze_code` tool auto-detects whether `code_or_path` is a file by checking
     `std::path::Path::new(&params.0.code_or_path).exists()`. The spec's security section
     (item 3) dismisses this as "the AI assistant already has filesystem access."
question: What prevents the MCP client (which could be any JSON-RPC client, not necessarily
          Claude Code) from reading sensitive files like `/etc/shadow`, `~/.ssh/id_rsa`,
          or `~/.aws/credentials`? The argument that "the AI assistant already has filesystem
          access" only holds when the bridge is used exclusively by Claude Code running as
          the same user. If the bridge is exposed to any other client, this is a
          privilege-equivalent exposure.
severity: WARNING
recommendation: The spec's argument is sound for the intended use case (single-user,
                local bridge). However, the spec should explicitly document this threat
                model assumption: "The bridge is designed for single-user, same-machine use
                only. It MUST NOT be exposed over a network or to untrusted MCP clients."
                Consider adding an optional `--allowed-dirs` flag for defense-in-depth.
                This is not blocking for v1 but should be documented as a known limitation.
```

### 11. `schemars` Version Compatibility -- 1.0 Is Relatively New

```
gap: The spec requires `schemars = "1.0"`. The `schemars` 1.0 release is relatively recent
     (late 2025) and represents a major version bump from 0.8.x which was the long-standing
     stable version. The spec notes (risk item 5): "rmcp 0.16 uses schemars 1.0."
question: Has the builder verified that rmcp 0.16 actually requires schemars 1.0 and not
          0.8? If rmcp reexports schemars 0.8, adding schemars 1.0 explicitly will cause
          two incompatible versions and derive macro failures. This is the single most
          likely compile-time failure in the spec.
severity: WARNING
recommendation: The builder's first step after adding dependencies to Cargo.toml should be
                `cargo check` to verify schemars version alignment. If rmcp 0.16 uses
                schemars 0.8, the spec's `schemars = "1.0"` must change to `schemars = "0.8"`.
                The build order (step 1: add deps, cargo check) already covers this.
```

### 12. Doctor Check Does Not Depend on Config

```
gap: The spec's `check_mcp_bridge()` function signature is `fn check_mcp_bridge(result: &mut DiagnosticResult)`
     with no config parameter. It is called unconditionally (line 1590: "check_mcp_bridge(&mut result)"),
     not guarded by `if let Some(ref cfg) = loaded_config` like `check_mcp_servers` is.
     This means the bridge check runs even when there is no great.toml or no [mcp-bridge]
     section. Meanwhile, the apply integration (section 4.1) only triggers when
     `cfg.mcp_bridge.is_some()`.
question: Is it intentional that doctor always checks bridge backends even when the user
          has not opted into the bridge? This could produce confusing output (5 "info: not found"
          lines + 1 "warning: not in .mcp.json") for users who do not use the bridge at all.
severity: WARNING
recommendation: Either (a) guard the check behind `if cfg.mcp_bridge.is_some()` like the
                apply integration, or (b) make it always run but change the "not in .mcp.json"
                from a warning to info when no [mcp-bridge] config exists. Option (a) is
                cleaner. The builder should decide based on UX preference.
```

### 13. Graceful Shutdown -- `shutdown_all()` Not Called on Exit

```
gap: The spec mentions (edge cases, "Stdin EOF" section) that "The bridge then calls
     registry.shutdown_all() to kill lingering processes." However, the `start_bridge()`
     function (lines 1255-1278) does NOT call `shutdown_all()` after `service.waiting()`
     returns. It simply returns `Ok(())`.
question: Where is `shutdown_all()` actually called? The code as written will exit without
          killing background tasks spawned via `run`. The `kill_on_drop(true)` on individual
          child handles will clean up immediate children, but the spawned background tokio
          tasks hold the `TaskRegistry` arc, and the registry only kills processes explicitly
          via `shutdown_all()`.
severity: WARNING
recommendation: Add `registry.shutdown_all().await;` after `service.waiting().await?;` in
                `start_bridge()`. The backlog mentions using `CancellationToken` + `tokio::select!`
                for graceful shutdown -- the spec simplified this away but lost the cleanup
                call in the process.
```

### 14. Binary Size Impact Not Quantified

```
gap: The spec adds 5 new Cargo dependencies: rmcp (with server + transport-io features
     pulling in async-trait, schemars, etc.), uuid, tracing, tracing-subscriber (with
     env-filter), and schemars. The spec does not estimate the binary size impact.
question: What is the expected binary size increase? The current `great` binary is relatively
          lean. rmcp + tracing + tracing-subscriber are non-trivial. For a CLI that is
          downloaded/installed by end users, binary size matters.
severity: QUESTION
recommendation: The builder should measure `cargo build --release` binary size before and
                after, and report in the observer iteration report. If the increase exceeds
                5 MB, consider whether tracing-subscriber could be replaced with a simpler
                stderr logging approach (eprintln! macros are already used throughout the
                codebase). This is not blocking -- just needs measurement.
```

### 15. Scope: 9 Tools in First Iteration vs Incremental Delivery

```
gap: The spec delivers 9 tools, 7 new source files, and modifications to 8 existing files
     in a single XL task. The backlog estimated XL complexity.
question: Would it be safer to ship a minimal first iteration (prompt + run + wait +
          list_tasks + get_result + kill_task = the "agent" preset) and add research,
          analyze_code, and clink in a follow-up? The agent preset is the default and
          covers the core value proposition.
severity: QUESTION
recommendation: The spec's phased build order already provides natural stopping points.
                If the builder encounters rmcp API difficulties in Phase 2, they could
                ship Phases 1-3 with the agent preset only and defer the research/analysis/
                subagent tools. This does not require a spec change -- just builder judgment.
```

---

## Cross-Reference Summary

| Area | Backlog | Spec | Status |
|------|---------|------|--------|
| Subcommand name | `great mcp-bridge` | `great mcp-bridge` | Match |
| Variant naming | `MpcBridge` (typo) | `McpBridge` (corrected) | Spec fixes typo |
| Tool count | "8" (but lists 9) | 9 | Spec correct, backlog typo |
| Presets | minimal, chat, agent, full | minimal, agent, research, full | Divergent (concern #6) |
| Protocol version | `"2024-11-05"` | `ProtocolVersion::LATEST` | Spec defers to rmcp |
| process-wrap | Required | Dropped (manual libc) | Explicit trade-off |
| JSON-RPC structs | New `protocol.rs` file | Eliminated (rmcp handles) | Good simplification |
| `handlers.rs` | Required | Eliminated (methods on GreatBridge) | Good simplification |
| schemars | Not mentioned | `"1.0"` added | Spec adds (concern #11) |
| Config field | `mcp_bridge: Option<MpcBridgeConfig>` | `mcp_bridge: Option<McpBridgeConfig>` | Typo fix |
| Apply behavior | "skip if already present" | "compare args, overwrite if different" | Spec improves |

---

## Overall Risk Assessment

**Medium-High.** The core architecture is sound and the spec demonstrates deep understanding of the rmcp crate, the MCP protocol, and the existing codebase. However, the heavy reliance on unverified rmcp API assumptions (8 explicit "builder must verify" items) means the builder will need significant latitude to adjust the implementation. The introduction of `unsafe` code and a new dependency paradigm (tracing, schemars) into a previously clean codebase represents a meaningful complexity increase.

The spec is **implementable** with the understanding that:
1. rmcp API adjustments are expected and acceptable
2. The builder adds `libc` explicitly or chooses process-wrap instead
3. The `.unwrap()` at line 322 is replaced with idiomatic pattern matching
4. `shutdown_all()` is wired into the exit path
5. The preset name discrepancy is resolved (remove "chat" from help text)

---

**Summary:** A thorough and well-structured XL spec with 5 WARNINGs requiring builder judgment (unsafe policy, libc dep hygiene, shutdown path, doctor guard, auto-approve opt-in), 1 convention violation (.unwrap), and reasonable uncertainty about rmcp's exact API surface -- all manageable, none blocking.
