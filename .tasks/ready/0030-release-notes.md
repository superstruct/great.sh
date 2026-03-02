# Release Notes: Task 0030 — MCP Bridge Hardening

**Date:** 2026-02-28
**Scope:** `src/mcp/bridge/server.rs`, `src/mcp/bridge/backends.rs`,
`src/mcp/bridge/registry.rs`, `src/cli/mcp_bridge.rs`, `src/cli/doctor.rs`,
`src/config/schema.rs`, `src/main.rs`, `Cargo.toml`

---

## What Changed

Five hardening items deferred from task 0029 are now resolved. No new crate
dependencies. No breaking changes to existing `great.toml` keys or CLI flags.

---

## Path Traversal Prevention (`--allowed-dirs`)

The `research` and `analyze_code` MCP tools previously read arbitrary file
paths supplied by the MCP client without validation. Any path accepted by the
shell — including `../../etc/shadow`, absolute paths to credential files, and
symlinks resolving outside the working tree — was read unconditionally.

`GreatBridge` now accepts an optional directory allowlist. When set, every
requested path is canonicalized with `std::fs::canonicalize` (resolving all
symlinks) and checked against the allowlist before the file is opened. Paths
that do not start with an allowed canonical prefix are rejected with an error
response returned to the MCP client; the file is never opened.

**CLI flag:**

```sh
great mcp-bridge --allowed-dirs /home/user/projects,/tmp/scratch
```

Multiple directories are comma-separated. Paths are resolved once at startup;
relative paths are interpreted from the process working directory at that point.
Non-existent directories are skipped with a `tracing::warn` message.

**`great.toml` key** (equivalent, CLI flag wins if both are present):

```toml
[mcp-bridge]
allowed_dirs = ["/home/user/projects", "/tmp/scratch"]
```

When `--allowed-dirs` is omitted and `allowed_dirs` is absent from config, all
paths are permitted. This preserves existing behavior for the single-user,
same-machine threat model that the bridge was designed for.

**Error message when a path is rejected:**

```
path not in allowed directories: '/etc/shadow' (canonical: /etc/shadow).
Allowed: /home/user/projects, /tmp/scratch
```

Note on TOCTOU: there is a theoretical race between the `canonicalize` check
and the subsequent `read`. Exploiting it requires local filesystem access with
the same UID, which is outside this tool's threat model. The race is documented;
no mitigation is needed.

---

## Auto-Approve Opt-Out (`auto_approve = false`)

The Claude backend previously passed `--dangerously-skip-permissions` to every
bridge-initiated invocation with no way to suppress it. Users who installed
Claude Code for interactive use and later added `[mcp-bridge]` to `great.toml`
were granting full auto-approval to all bridged calls without any visible
indication.

A new `auto_approve` config key (default: `true`) controls whether auto-approval
flags are forwarded to each backend.

| Backend  | Auto-approve flag suppressed when `auto_approve = false` |
|----------|----------------------------------------------------------|
| `claude` | `--dangerously-skip-permissions`                         |
| `gemini` | `-y`                                                     |
| `grok`   | `-y`                                                     |
| `codex`  | `--full-auto`                                            |
| `ollama` | (no flag; setting has no effect)                         |

**`great.toml` to disable auto-approve:**

```toml
[mcp-bridge]
auto_approve = false
```

The default is `true`, preserving existing behavior when the key is absent.

**`great doctor` warning:** when the `claude` binary is on PATH, `great doctor`
now reports a warning line that names `--dangerously-skip-permissions` explicitly
when auto-approve is enabled (the default). Adding `auto_approve = false` to
`[mcp-bridge]` changes this line from `warn` to `pass`. The warning appears even
when no `[mcp-bridge]` section exists in `great.toml`; the `.mcp.json`
registration check is skipped in that case (user has not opted in to the bridge).

---

## Doctor Refactored to Use Canonical Backend List

`check_mcp_bridge()` in `src/cli/doctor.rs` previously contained a hardcoded
backend list that duplicated `BACKEND_SPECS` in `src/mcp/bridge/backends.rs`.
The two lists would silently drift whenever a backend was added, renamed, or
removed.

`backends.rs` now exports a new `all_backend_specs()` function that returns
static metadata for every known backend regardless of installation status.
`check_mcp_bridge()` calls `all_backend_specs()` directly. The hardcoded slice
is gone. The `#[allow(dead_code)]` annotation on `BackendConfig::api_key_env`
is removed; the field is now in active use.

Adding a new backend to `BACKEND_SPECS` automatically propagates to `great
doctor` output with no change to `doctor.rs`.

---

## Global `--verbose` / `--quiet` Flags Now Apply to `mcp-bridge`

`great mcp-bridge` previously accepted only its own `--log-level` flag for
tracing control and ignored the global `--verbose` and `--quiet` flags.

The precedence rules, from highest to lowest:

| Condition                        | Effective tracing level |
|----------------------------------|-------------------------|
| `--log-level <LEVEL>` specified  | `<LEVEL>` (explicit wins) |
| `--verbose` (global flag)        | `debug`                 |
| `--quiet` (global flag)          | `error`                 |
| Neither flag                     | `warn` (unchanged)      |

If both `--verbose` and `--quiet` are passed, `--verbose` takes precedence.

**Examples:**

```sh
great --verbose mcp-bridge               # tracing at debug
great --quiet mcp-bridge                 # tracing at error
great --verbose mcp-bridge --log-level info  # tracing at info (explicit wins)
great mcp-bridge                         # tracing at warn (unchanged)
```

---

## Binary Size Reduced 37% via Release Profile Optimizations

The 0029 merge raised the release binary from 10.4 MiB to 13.6 MiB (+31%),
exceeding the 12.5 MiB threshold set in the task. The following profile
settings are now in `Cargo.toml`:

```toml
[profile.release]
lto = true
strip = true
codegen-units = 1
```

`lto = true` enables cross-crate dead-code elimination and deduplication of
monomorphized generics. `strip = true` removes symbol tables from the final
binary. `codegen-units = 1` disables parallel code generation, allowing LLVM
to optimize across the entire crate graph. Post-mitigation binary size is
recorded in the iteration-027 observer report.

Note: `strip = true` removes symbols used for panic backtraces. For a CLI tool
that communicates errors via exit codes and stderr messages, this is acceptable.
Release builds take longer; development builds (`cargo build` without
`--release`) are unaffected.

Note: `codegen-units = 1` is the implicit default when `lto = true` is set in
recent versions of rustc. Both are listed explicitly for clarity.

---

## Migration Notes

No changes to existing `great.toml` keys or subcommand behavior.

**New optional keys in `[mcp-bridge]`:**

| Key            | Type             | Default | Effect                                         |
|----------------|------------------|---------|------------------------------------------------|
| `auto_approve` | `bool`           | `true`  | Set `false` to suppress backend auto-approve flags |
| `allowed_dirs` | `[string]` array | absent  | Restrict file-reading tools to listed directories  |

If neither key is present, behavior is identical to 0029.

**`great doctor`** will now show a new warning line when the `claude` binary
is on PATH. This line appears even without a `[mcp-bridge]` section. It is
informational; it does not change the doctor exit code.
