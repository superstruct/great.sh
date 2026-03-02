# Security Audit: 0005 Doctor Command Expansion

**Auditor:** Kerckhoffs
**Date:** 2026-02-24
**Verdict:** PASS (no CRITICAL or HIGH findings)
**Files audited:**
- `src/cli/doctor.rs` (working tree, includes Da Vinci's uncommitted diff)
- `src/cli/util.rs` (new file, shared `get_command_version`)
- `src/cli/status.rs` (working tree, refactored to use `util::`)
- `src/cli/bootstrap.rs` (unchanged, called by doctor `--fix`)
- `src/cli/tuning.rs` (unchanged, called by doctor `--fix`)
- `src/cli/output.rs` (unchanged)
- `src/config/schema.rs` (unchanged)
- `src/platform/package_manager.rs` (unchanged, called by doctor `--fix`)
- `src/platform/detection.rs` (`command_exists`)
- `src/mcp/mod.rs` (unchanged)

---

## Checklist

### 1. No `.unwrap()` in production code

**PASS.** Zero `.unwrap()` calls found in `doctor.rs`, `status.rs`, `bootstrap.rs`, or `tuning.rs` production code. The `.unwrap_or("")` on `lines().next()` in `util.rs:19` is safe (infallible on first element). The previous `.unwrap_or_default()` on `path.to_str()` in `check_config()` has been correctly replaced with an explicit `match` that returns `None` on non-UTF-8 paths (lines +468-480 of diff).

### 2. No command injection

**PASS.** All `Command::new()` calls in `doctor.rs` use hardcoded strings:
- Line 128: `Command::new("bash")` -- Homebrew installer (hardcoded URL)
- Line 576: `Command::new("xcode-select")` -- hardcoded arg `-p`
- Line 598: `Command::new("dpkg")` -- hardcoded args `["-s", "build-essential"]`
- Line 641: `Command::new("docker")` -- hardcoded arg `info`

Config-sourced values (`mcp.command`, tool names from `[tools]`) are only passed to:
- `command_exists()` which uses `which::which()` -- safe PATH lookup, no shell spawning
- `util::get_command_version()` which uses `Command::new(cmd).arg("--version")` -- the `cmd` is used as the direct binary name, not interpreted by a shell. The binary is resolved via the OS exec path, not via `sh -c`. This is safe for command names coming from `great.toml`.

The `check_mcp_servers()` function (new, lines +541-577 of diff) only calls `command_exists(&mcp.command)` -- no process spawning for untrusted MCP commands.

### 3. `--fix` mode shell profile writes

**PASS.** The `FixAction::AddLocalBinToPath` handler (lines 151-178) writes a hardcoded string literal to the shell profile:
```rust
let line = "\n# Added by great doctor --fix\nexport PATH=\"$HOME/.local/bin:$PATH\"\n";
```
No user-controlled or config-sourced data is interpolated into the profile write. The file is opened in append mode, so it cannot truncate existing content.

### 4. MCP server check does not leak sensitive config data

**PASS.** The new `check_mcp_servers()` function only outputs:
- Server name (from TOML key)
- `mcp.command` (the binary name, e.g., "npx")
- `mcp.transport` (e.g., "stdio")

It does **not** print `mcp.args`, `mcp.env`, or `mcp.url` -- these could contain secret references. The `mcp.enabled` field is checked to skip disabled servers. Good.

### 5. Credential handling

**PASS.** The `check_ai_agents()` function (lines 415-454) checks `std::env::var(key).is_ok()` for API key presence but never reads or prints the value. The secret reference checker in `check_config()` similarly only reports whether secrets resolve, not their values.

### 6. AgentConfig Debug derive (pre-existing, noted in audit 0002)

**INFO (pre-existing).** `AgentConfig` still derives `Debug` in `schema.rs:77`, which would expose `api_key` if the struct were debug-printed. No new code in the doctor command debug-prints agent configs, so this remains MEDIUM priority per audit 0002 -- not a regression from this changeset.

---

## Findings

### L1: `util::get_command_version` executes config-sourced binary names (LOW)

**File:** `src/cli/util.rs:10`
**Risk:** A malicious `great.toml` could declare a tool name like `../../tmp/evil` in `[tools]` which would be passed to `Command::new()`. However, `Command::new()` does not interpret shell metacharacters and the OS will simply fail to find such a binary (or find it via PATH, which requires it to already be installed). The `command_exists()` guard (which runs first in both `doctor.rs` and `status.rs`) would also return false for non-existent binaries.
**Severity:** LOW. No shell interpretation, PATH-only resolution.
**Action:** None required. If paranoid, validate tool names against `^[a-zA-Z0-9._-]+$` at config parse time.

### L2: Pre-existing Homebrew installer uses `curl | bash` pattern (LOW)

**File:** `src/cli/doctor.rs:128-129`, `src/cli/bootstrap.rs:359-361`
**Risk:** The Homebrew and Claude Code installers use `curl -fsSL <url> | bash`. This is the official install method for both tools and is gated behind `--fix` mode (user opt-in). The URLs are hardcoded, not config-sourced.
**Severity:** LOW. Standard practice for these tools, user must opt in with `--fix`.
**Action:** None required. Already documented in prior audits.

### L3: `FixAction::InstallSystemPrerequisite` increments `fixed` unconditionally (LOW)

**File:** `src/cli/doctor.rs:189`
**Risk:** The `InstallSystemPrerequisite` match arm increments `fixed += 1` regardless of whether the `bootstrap::ensure_*` call actually succeeded. The bootstrap functions print errors but don't return Result. This is a correctness issue, not a security issue.
**Severity:** LOW. Cosmetic -- the "Fixed N of M" summary may overcount.
**Action:** Consider having bootstrap functions return `Result<bool>`.

---

## Summary

| Severity | Count | Blocks commit? |
|----------|-------|----------------|
| CRITICAL | 0     | --             |
| HIGH     | 0     | --             |
| MEDIUM   | 0     | --             |
| LOW      | 3     | No             |

**Verdict: PASS.** The doctor command changes are safe to commit. The refactoring to `util.rs` is clean, the new MCP server check is properly scoped (no sensitive data leakage), and `--fix` mode writes only hardcoded content. No `.unwrap()` in production code. No command injection vectors.
