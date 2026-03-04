# 0041 — Humboldt Scout Report

- **Task:** 0041 — `great mcp test <name>` shows wrong error when no `[mcp]` section exists
- **Scout:** Humboldt
- **Date:** 2026-03-04
- **Complexity:** XS — 1 production line change, 4 new integration tests

---

## 1. File Map

### Production change (1 file, 1 line)

**`/home/isaac/src/sh.great/src/cli/mcp.rs`**

```
Line 183: if mcps.is_empty() {
```

Change to:

```
Line 183: if mcps.is_empty() && name.is_none() {
```

Full context of the affected block (lines 181-196):

```rust
let mcps = cfg.mcp.unwrap_or_default();   // line 181

if mcps.is_empty() {                       // line 183 — BUG HERE
    output::warning("No MCP servers declared in great.toml.");
    return Ok(());
}

let servers_to_test: Vec<...> = match name {  // line 188
    Some(n) => match mcps.get_key_value(n) {
        Some(pair) => vec![pair],
        None => {
            output::error(&format!("MCP server '{}' not found in great.toml", n));
            return Ok(());                    // line 192 — unreachable when mcps empty
        }
    },
    None => mcps.iter().collect(),
};
```

The guard at line 183 is unconditional: when `mcps` is empty and `name` is
`Some("xyz")`, it fires the generic warning and returns before the name-specific
error at line 192 can execute. Adding `&& name.is_none()` gates the early-exit
on the "no specific server was requested" case only.

### Test additions (1 file, appended after line 495)

**`/home/isaac/src/sh.great/tests/cli_smoke.rs`**

- Current length: **2105 lines**
- Existing MCP section: lines 448-495 (header + 3 tests: `mcp_list_no_config`,
  `mcp_add_no_config`, `mcp_add_creates_entry`)
- Insert 4 new tests after line 495 (before the `// Vault` section at line 497)

No existing `mcp test` tests exist in the file. Confirmed by grep.

---

## 2. Patterns to Follow

### Test structure (from lines 452-495)

```rust
#[test]
fn mcp_list_no_config() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["mcp", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No MCP servers configured"));
}
```

- `TempDir::new().unwrap()` — always use a fresh temp dir, never cwd
- `great().current_dir(dir.path())` — all subcommand tests use this helper
- `.args(["mcp", "test", ...])` — slice literal, not vec
- `.assert().success()` — `run_test()` returns `Ok(())` on all paths (exit 0)
- `.stderr(predicate::str::contains(...))` — all `output::*` functions write to stderr

### Toml fixture pattern (from lines 477-482)

```rust
std::fs::write(
    dir.path().join("great.toml"),
    "[project]\nname = \"test\"\n",
)
.unwrap();
```

Or multi-line with raw string literal (both styles appear; raw string preferred
when content exceeds one line).

### Imports already present (lines 1-3)

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
```

All three needed helpers are already imported. No new imports needed.

---

## 3. Dependencies Confirmed

From `/home/isaac/src/sh.great/Cargo.toml` `[dev-dependencies]`:

| Crate | Version | Needed for |
|---|---|---|
| `assert_cmd` | 2.0 | `Command::cargo_bin` + `.assert()` |
| `predicates` | 3.0 | `predicate::str::contains()` |
| `tempfile` | 3.0 | `TempDir::new()` |

All three already present. No Cargo.toml changes required.

---

## 4. Output Function Signatures

From `/home/isaac/src/sh.great/src/cli/output.rs`:

```rust
pub fn warning(msg: &str)   // eprintln! with yellow "!" prefix -> stderr
pub fn error(msg: &str)     // eprintln! with red "x" prefix -> stderr
```

Both write to **stderr**. Tests correctly use `.stderr(...)` not `.stdout(...)`.

The exact strings to assert against (from `run_test()` source):

- Generic (no name): `"No MCP servers declared in great.toml."` — emitted by `output::warning`
- Name-specific:     `"MCP server '{}' not found in great.toml"` — emitted by `output::error`
- Header:            `"Testing MCP Servers"` — emitted by `output::header` at line 168

---

## 5. Behavioral Matrix (from spec)

| Scenario | `name` | `mcps` | Before | After |
|---|---|---|---|---|
| Named, no `[mcp]` section | `Some("xyz")` | empty | generic warning (bug) | name-specific error (fixed) |
| Named, server exists but not this one | `Some("xyz")` | non-empty | name-specific error | same |
| No name, no `[mcp]` section | `None` | empty | generic warning | same |
| No name, servers exist | `None` | non-empty | tests all | same |

---

## 6. Risks and Surprises

**No surprises.** The codebase is clean here.

- The fix is pure control-flow on in-memory data; no I/O, no platform branching.
- `run_test()` already has the correct name-specific error message at line 192
  — the bug is only that it is unreachable. The fix unblocks that path.
- All four test scenarios use fixtures that `great.toml` discovery will resolve
  via `config::discover_config()` walking up from `dir.path()`. Since `TempDir`
  is isolated, there is no risk of picking up a real `great.toml` from the host
  filesystem (the temp dir has no ancestors with one in CI).
- AC4 (`mcp_test_no_name_with_servers_tests_all`) uses `command = "echo"` which
  exists on all platforms. The test only asserts the header is printed, not that
  the server test passes — safe regardless of echo's exit behavior under
  `mcp::test_server()`.

**One technical debt observation (pre-existing, not introduced here):**
`run_test()` always returns `Ok(())` even on error paths. Task 0040 tracks
adding non-zero exit codes. This spec explicitly leaves that behavior unchanged.

---

## 7. Recommended Build Order

1. Edit `/home/isaac/src/sh.great/src/cli/mcp.rs` line 183:
   - `if mcps.is_empty() {` → `if mcps.is_empty() && name.is_none() {`
2. Append 4 tests to `/home/isaac/src/sh.great/tests/cli_smoke.rs` after line 495
   (the MCP section, before `// Vault`)
3. `cargo clippy` — no new warnings expected (the added condition uses existing
   `name` binding; no dead code introduced)
4. `cargo test mcp_test` — run the 4 new tests in isolation first
5. `cargo test` — full suite

---

## 8. Summary

| Item | Value |
|---|---|
| Production files changed | 1 |
| Production lines changed | 1 |
| Test file | `tests/cli_smoke.rs` |
| New tests | 4 |
| New dependencies | 0 |
| New imports | 0 |
| Platform risk | None |
| Ambiguities | None |
