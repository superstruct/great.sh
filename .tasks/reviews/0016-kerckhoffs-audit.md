# Security Audit: 0016 Overwrite Safety for `great loop install`

**Auditor:** Auguste Kerckhoffs
**Date:** 2026-02-24
**Verdict:** PASS (no CRITICAL/HIGH findings)

---

## Scope

- `/home/isaac/src/sh.great/src/cli/loop_cmd.rs` -- `--force` flag, `collect_existing_paths`, `confirm_overwrite`, overwrite gate in `run_install`, unit tests
- `/home/isaac/src/sh.great/tests/cli_smoke.rs` -- 4 integration tests for overwrite safety

## Build / Test / Lint

| Check | Result |
|-------|--------|
| `cargo test` | 188 unit + 62 integration PASS (1 ignored) |
| `cargo clippy -- -D warnings` | PASS, 0 warnings |
| Compiler warnings | 1 non-fatal: `duplicated attribute` on `#[test]` (see L1) |

---

## Security Analysis

### 1. Stdin Input Injection (PASS)

**Question:** Can user input from stdin be injected into file content or paths?

**Answer:** No. The `confirm_overwrite` function (lines 194-228) reads a single line via `std::io::stdin().read_line()`, trims whitespace, lowercases it, and compares it to the string literals `"y"` and `"yes"`. The result is a boolean. The raw input string (`input`) is never used in any file path, file content, or shell command. It goes out of scope at the end of the function.

### 2. TOCTOU Between `exists()` and Write (ADVISORY)

**Question:** Is there a time-of-check-to-time-of-use gap?

**Answer:** Yes, there is a theoretical window between `collect_existing_paths` calling `Path::exists()` (line 171) and the actual `std::fs::write()` calls (lines 265-287). However:
- The files being written are in `~/.claude/`, a user-owned directory.
- The write targets are deterministic paths derived from compile-time constants (agent/command names).
- A concurrent actor that could create files in `~/.claude/agents/` between the check and the write already has the same privilege level as the running user.
- This is identical to the pre-existing TOCTOU pattern documented in audit 0013 (L2).

**Severity:** ADVISORY (not actionable; consistent with existing codebase pattern)

### 3. `--force` Flag Exploitation (PASS)

**Question:** Can `--force` be exploited to overwrite files outside the managed set?

**Answer:** No. The `--force` flag (line 23) is a boolean that simply skips the confirmation prompt. The set of files written is always the same fixed set regardless of `--force`:
- 15 agent files: `~/.claude/agents/{name}.md` (names from compile-time `AGENTS` array)
- 5 command files: `~/.claude/commands/{name}.md` (names from compile-time `COMMANDS` array)
- 1 config file: `~/.claude/teams/loop/config.json`
- conditionally: `~/.claude/settings.json`

All paths are constructed from hardcoded string literals joined with `Path::join`. No user-controlled input participates in path construction.

### 4. Path Traversal in `collect_existing_paths` (PASS)

**Question:** Can path traversal occur?

**Answer:** No. `collect_existing_paths` (lines 162-189) constructs paths exclusively from:
- `claude_dir` (derived from `dirs::home_dir()`)
- Hardcoded directory names: `"agents"`, `"commands"`, `"teams"`, `"loop"`
- Agent/command names from compile-time `AGENTS` and `COMMANDS` arrays (e.g., `"nightingale"`, `"loop"`)

No user input enters the path construction chain. The `.name` fields are `&'static str` literals defined in the same file (lines 45-130).

### 5. Broken Pipe Handling in `confirm_overwrite` (PASS)

**Question:** Does `confirm_overwrite` handle broken pipe safely?

**Answer:** If `read_line` fails (broken pipe, closed stdin, etc.), the `.context()` wrapper on line 224 converts it to an `anyhow::Error` which propagates via `?` to the caller `run_install`, which returns `Err(...)`, and the CLI exits with a nonzero code and an error message. This is correct behavior -- no panic, no data corruption, no partial writes (the overwrite gate at line 251-256 runs *before* any file writes).

### 6. Non-TTY Detection (PASS)

The non-TTY branch (lines 212-217) correctly prints the `--force` hint to stderr and returns `Ok(false)`, causing `run_install` to `bail!("aborted: no files were modified")` before any writes. This is the safe default: when stdin is not interactive, refuse to overwrite without `--force`. Confirmed by integration test `loop_install_non_tty_existing_files_aborts`.

### 7. No New Dependencies (PASS)

`std::io::IsTerminal` is stdlib (stable since Rust 1.70). `tempfile` is an existing dev-dependency. No changes to `Cargo.toml`.

---

## Findings

### L1: Duplicate `#[test]` Attribute in Integration Tests (LOW)

**File:** `/home/isaac/src/sh.great/tests/cli_smoke.rs`, line 1077

An orphan `#[test]` attribute sits above a section comment block, immediately followed by another `#[test]` on line 1082. Both attributes attach to `loop_install_force_flag_accepted`, causing it to run twice. The compiler emits `warning: duplicated attribute`. This is harmless but wastes CI time and clutters output.

**Fix:** Remove the `#[test]` on line 1077.

### L2: `unwrap_or_default()` in `run_status` (LOW / PRE-EXISTING)

**File:** `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`, line 463

```rust
let contents = std::fs::read_to_string(&settings_path).unwrap_or_default();
```

This silently swallows read errors on `settings.json`. If the file exists but is not readable (e.g., permissions 000), the user sees "Agent Teams env: not found" which is misleading. Not introduced by this PR but adjacent to the changed code.

**Severity:** LOW (pre-existing, no security impact)

---

## Verdict

**PASS** -- No CRITICAL or HIGH findings. The overwrite safety gate is correctly implemented:

1. `collect_existing_paths` scans only hardcoded managed paths (no user input in path construction).
2. The overwrite check runs *before* any file writes.
3. Non-TTY stdin correctly blocks without `--force`.
4. User input from the confirmation prompt is never used beyond a string comparison.
5. `--force` cannot expand the set of written files.
6. No new dependencies introduced.

Two LOW findings (L1: duplicate test attribute, L2: pre-existing `unwrap_or_default`) are advisory and do not block commit.
