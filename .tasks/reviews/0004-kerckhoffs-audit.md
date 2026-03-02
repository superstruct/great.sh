# 0004: Status Command -- Security Audit

**Auditor:** Auguste Kerckhoffs
**Date:** 2026-02-24
**Files reviewed:**
- `/home/isaac/src/sh.great/src/cli/status.rs` (471 lines)
- `/home/isaac/src/sh.great/tests/cli_smoke.rs` (status tests, lines 1214-1472)
- `/home/isaac/src/sh.great/src/config/schema.rs` (AgentConfig, SecretsConfig)
- `/home/isaac/src/sh.great/src/cli/output.rs` (output helpers)
- `/home/isaac/src/sh.great/src/platform/detection.rs` (command_exists)

**Verdict: PASS -- no CRITICAL or HIGH findings. Commit is not blocked.**

---

## Audit Checklist

### 1. Secret Leakage (CRITICAL if found)

**Status: CLEAR**

- `SecretStatus` struct (line 62-66) contains only `name: String` and `is_set: bool`. No field exists to hold a secret value.
- `run_json()` line 378: `std::env::var(key).is_ok()` -- the `Result<String>` is immediately reduced to a `bool`. The actual secret value is never bound to a variable, never stored in any struct, and never serialized.
- Human-readable path (line 263): same pattern -- `std::env::var(key).is_ok()` only.
- `AgentStatus` struct (line 53-60) deliberately excludes `api_key` and `enabled` from `AgentConfig`. Only `name`, `provider`, and `model` are serialized.
- `GreatConfig` (which contains `AgentConfig.api_key`) is never serialized directly by the status command. Purpose-built DTOs are used throughout.
- Human-readable agent output (line 221-223) prints only provider and model, never `api_key`.

### 2. No .unwrap() in Production Code (HIGH if found)

**Status: CLEAR (with one acceptable use)**

- The old `path.to_str().unwrap_or_default()` has been replaced with proper `?`-propagated error handling via `ok_or_else()` at lines 100-107. Good fix.
- `version.lines().next().unwrap_or("")` at line 435 in `get_tool_version()`: this is safe -- `.lines()` on any string (even empty) always yields at least one element via the iterator. Technically not reachable as a panic. Acceptable.
- `full.split_whitespace().last().unwrap_or(full)` at line 457 in `print_tool_status()`: safe -- if `split_whitespace()` yields nothing, `unwrap_or(full)` provides the fallback. No panic possible.
- No bare `.unwrap()` calls exist in any production code path in this file.

### 3. Command Injection (CRITICAL if found)

**Status: CLEAR**

- `command_exists()` (detection.rs:124) uses `which::which(cmd)` -- pure PATH lookup, no shell spawning.
- `get_tool_version()` (line 425-444) uses `std::process::Command::new(tool).arg("--version")`. This is NOT shell invocation -- `Command::new()` executes a binary directly via `execvp`, not through `sh -c`. The `--version` is a separate argument, not concatenated.
- Tool names originate from the user's own `great.toml` file, which is a local trust boundary. A user who controls their config file already has arbitrary code execution on their own machine. No privilege escalation vector exists.
- MCP commands follow the same pattern: `command_exists(&mcp.command)` uses `which`, no execution.

### 4. Error Messages -- Information Disclosure (MEDIUM if found)

**Status: CLEAR (with one LOW note)**

- Non-UTF-8 path error (line 104): `"config path contains non-UTF-8 characters: {path}"` -- exposes the user's own filesystem path. Not a secret.
- Config parse error (line 113): `"Failed to parse config: {e}"` -- TOML parse errors could theoretically include file content snippets. However: (a) this only fires for malformed TOML, (b) output goes to stderr only, (c) the user already has read access to their own config. LOW.
- No stack traces, internal struct dumps, or memory addresses are exposed in any error path.

### 5. serde_json Serialization -- Internal State Exposure

**Status: CLEAR**

- `StatusReport` is a purpose-built DTO with exactly the fields intended for public consumption.
- `GreatConfig` is never passed to `serde_json::to_string`. The code constructs `StatusReport` from individual fields, giving full control over what is serialized.
- `#[serde(skip_serializing_if = "Option::is_none")]` is correctly applied to optional sections, preventing `null` clutter.
- `config_path` is exposed in JSON output. This is the path to `great.toml`, which is not sensitive.
- `is_root: bool` is exposed. This is diagnostic information, not a secret.
- `shell` path is exposed (e.g., `/bin/zsh`). Not sensitive.

### 6. Test Coverage for Security Properties

- `status_json_with_secrets` (test line 1334): Confirms secrets appear as `{name, is_set}` only. However, the test does not explicitly assert the absence of the secret value `"value"` from stdout. This is a **test gap** but the structural guarantee (SecretStatus has no field for the value) makes it a LOW concern.
- `status_json_always_exits_zero_even_with_issues` (test line 1421): Confirms JSON mode does not leak failure state via exit code.
- All 16 status tests pass.

---

## Findings

### LOW-1: TOML parse errors may echo config file content

**Location:** `/home/isaac/src/sh.great/src/cli/status.rs:113`
**Severity:** LOW
**Description:** When config parsing fails, the TOML error message is printed to stderr. If a user has a literal API key in their `api_key` field (not a `${REF}`) and the TOML is malformed, the error could include fragments of the secret in stderr output. This is mitigated by: (a) only affecting malformed configs, (b) only going to stderr on the user's own terminal, (c) `api_key` with literal values is already flagged as bad practice.
**Action:** No fix required. Document that `api_key` should use `${SECRET_REF}` syntax, not literal values.

### LOW-2: Secret-absence test does not assert value exclusion

**Location:** `/home/isaac/src/sh.great/tests/cli_smoke.rs:1334-1371`
**Severity:** LOW
**Description:** The `status_json_with_secrets` test sets a secret env var with value `"value"` but does not assert that the string `"value"` is absent from stdout. The structural guarantee (SecretStatus lacks a value field) makes leakage impossible, but an explicit negative assertion would be defense-in-depth.
**Action:** P3 -- consider adding `assert!(!stdout.contains("\"value\""))` or similar in a future test hardening pass.

### LOW-3: `process::exit(1)` bypasses Drop cleanup

**Location:** `/home/isaac/src/sh.great/src/cli/status.rs:280`
**Severity:** LOW
**Description:** `std::process::exit(1)` terminates immediately without running destructors. In the current code this is safe because `run()` holds no resources requiring cleanup (no open files, no temp dirs, no locks). The comment at lines 276-278 correctly documents the rationale. If future changes add resources to `run()`, this could cause issues.
**Action:** No fix required. The comment is sufficient. Monitor if resources are added to `run()` in the future.

---

## Summary

| Category | Result |
|----------|--------|
| Secret leakage | CLEAR |
| .unwrap() in production | CLEAR |
| Command injection | CLEAR |
| Error information disclosure | CLEAR (1 LOW) |
| Serialization safety | CLEAR |
| Test coverage | Adequate (1 LOW gap) |
| **Overall** | **PASS -- commit not blocked** |
