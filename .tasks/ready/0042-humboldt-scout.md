# Scout Report 0042: `great status` Doctor Hint

**Scout:** Alexander von Humboldt
**Date:** 2026-03-04
**Spec:** `.tasks/ready/0042-status-doctor-hint-spec.md`
**Complexity:** XS — two files, five insertion points, zero new deps

---

## File Map

### `src/cli/status.rs` (438 lines total)

**Accumulator insertion point:**

- Line 125: comment `// -- Human-readable mode --------------------------------------------`
- Line 126: `output::header("great status");`
- Insert `let mut has_issues = false;` between line 125 and 126.

**Issue site (a) — runtimes loop, missing tool:**

- Lines 172–189: `for (name, version) in &tools.runtimes` loop
- Line 176: `let installed = command_exists(name);`
- Lines 182–188: `print_tool_status(name, version, installed, actual_version.as_deref(), args.verbose);`
- Insert `if !installed { has_issues = true; }` after line 188 (after `print_tool_status` call, before the closing `}`).

**Issue site (b) — cli tools loop, missing tool:**

- Lines 191–207: `if let Some(cli_tools) = &tools.cli` block
- Line 193: `let installed = command_exists(name);`
- Lines 199–205: `print_tool_status(name, version, installed, actual_version.as_deref(), args.verbose);`
- Insert `if !installed { has_issues = true; }` after line 205 (after `print_tool_status` call, before the closing `}`).

**Issue site (c) — MCP section, unavailable command:**

- Lines 222–249: MCP section `if let Some(mcps) = &cfg.mcp`
- Line 246: `output::error(&format!("  {} ({} -- not found)", name, mcp.command));` (inside `else` branch)
- Insert `has_issues = true;` on line 247 (immediately after the `output::error` call, still inside the `else` block).

**Issue site (d) — secrets section, missing secret:**

- Lines 252–264: secrets section `if let Some(secrets) = &cfg.secrets`
- Line 260: `output::error(&format!("  {} -- missing", key));` (inside `else` branch)
- Insert `has_issues = true;` on line 261 (immediately after the `output::error` call, still inside the `else` block).

**Hint print point:**

- Line 267: `println!();`  ← trailing blank line, keep as-is
- Lines 269–272: exit-0 comment block (added task 0040)
- Line 274: `Ok(())`
- Insert the hint between line 267 and the comment block:

```rust
if has_issues {
    output::info("Tip: use 'great doctor' for exit-code health checks in CI.");
}
```

The `output::info()` function (confirmed below) writes to stderr with a blue `ℹ` prefix — consistent with the hint pattern used in `init.rs`, `sync.rs`, and `doctor.rs`.

**JSON path — no changes needed:**

- Line 121–123: `if args.json { return run_json(...); }` — returns before the human-readable block, so `has_issues` is never in scope here. `run_json()` (lines 285–413) is untouched.

---

### `src/cli/output.rs` (65 lines total)

- `info()` function: lines 19–21
- Signature: `pub fn info(msg: &str)`
- Implementation: `eprintln!("{} {}", "ℹ".blue(), msg);`
- Confirmed: writes to **stderr**, colored blue. Correct function to use for the hint.

---

### `tests/cli_smoke.rs` — target tests

| Test | Lines | Change needed |
|---|---|---|
| `status_exit_zero_even_with_missing_tools` | 1937–1959 | Add `.stderr(predicate::str::contains("great doctor"))` |
| `status_exit_zero_even_with_missing_secrets` | 1962–1985 | Add `.stderr(predicate::str::contains("great doctor"))` |
| `status_human_and_json_exit_codes_match` | 2015–2046 | No change (exit code only, no stderr assertion) |
| `status_no_config_exits_zero` | 2049–2056 | No change (implicit: no issues = no hint) |
| `status_no_doctor_hint_when_clean` | NEW | Insert after line 2056 |

**New test placement:** after `status_no_config_exits_zero` closes at line 2056, before `status_verbose_with_config_shows_capabilities` at line 2058.

The new test uses `predicate::str::contains(...).not()` — the `predicates` crate is already a dev-dependency; `.not()` is available on all `Predicate` implementations.

---

## Existing Hint Patterns (confirmed)

All actionable hints in the codebase use `output::info()` with backtick-wrapped command names in the message string:

| File | Line | Pattern |
|---|---|---|
| `src/cli/init.rs` | 336 | `output::info("Run \`great apply\` to provision your environment.")` |
| `src/cli/sync.rs` | 105 | `output::info("Run \`great apply\` to provision the restored environment.")` |
| `src/cli/template.rs` | 171 | `output::info("Run \`great apply\` to provision your environment.")` |
| `src/cli/doctor.rs` | 251 | `output::warning("Run \`great doctor --fix\` to attempt automatic fixes.")` |
| `src/mcp/bridge/server.rs` | 26 | string contains `Run \`great doctor\` to check backend availability` |

The spec's proposed hint `"Tip: use 'great doctor' for exit-code health checks in CI."` uses single quotes rather than backticks. This is a minor deviation from the backtick convention used everywhere else in the codebase. Da Vinci should note this and consider `"Tip: run \`great doctor\` for exit-code health checks in CI."` for consistency — but this is a style call, not a blocker.

---

## Dependency Map

No new crates or imports required. `output::info` is already in scope via `use crate::cli::{output, util};` at line 5 of `status.rs`.

---

## Risks

1. **Borrow checker — none.** `has_issues` is a simple `bool` on the stack. The loops that set it already own their bindings. No conflict with the existing `cfg` borrows.

2. **`print_tool_status` encapsulation.** The helper function (lines 420–437) does not return a value indicating install status. The spec correctly avoids threading a mutable reference through it — instead reading `installed` at the call site, which is already in scope. No refactor needed.

3. **MCP section borrow pattern.** The `else` branch at line 245 (`} else {`) is a simple if-else on `cmd_available` (a `bool`). Adding `has_issues = true;` inside is safe — no borrow issues.

4. **Test predicate order.** The `assert_cmd` fluent API chains `.stderr()` calls as AND — all must match. Adding a second `.stderr()` predicate is additive and cannot break the first assertion.

5. **No-config path.** Per spec edge cases: when `config_path_str` is `None`, the `if let Some(cfg) = &config` block at line 166 is skipped entirely, so `has_issues` is never set to `true`. The hint will not appear. This is the intended behavior.

---

## Recommended Build Order

1. Edit `src/cli/status.rs` — five insertions in one pass (accumulator + four issue sites + hint print).
2. Edit `tests/cli_smoke.rs` — two test updates + one new test.
3. `cargo clippy -- -D warnings` — verify zero new warnings.
4. `cargo test` — all tests pass.
5. Visual check: `cargo run -- status` in a directory with a config referencing a missing tool.
