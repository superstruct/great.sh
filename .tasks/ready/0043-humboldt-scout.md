# 0043 Humboldt Scout Report — Status MCP Test Coverage and JSON Bug Fix

**Scout:** Alexander von Humboldt
**Date:** 2026-03-04
**Spec:** `.tasks/ready/0043-status-mcp-test-spec.md`
**Socrates verdict:** APPROVED (7 advisory notes, no blockers)

---

## Bug Location — Confirmed

**File:** `src/cli/status.rs`
**Lines 357–369** — the `let mcp = config.and_then(...)` block in `run_json()`.

```rust
// CURRENT (lines 357-369) — BUG: never pushes to issues
let mcp = config.and_then(|cfg| {
    cfg.mcp.as_ref().map(|mcps| {
        mcps.iter()
            .map(|(name, m)| McpStatus {
                name: name.clone(),
                command: m.command.clone(),
                args: m.args.clone(),
                command_available: command_exists(&m.command),
                transport: m.transport.clone(),
            })
            .collect()
    })
});
```

`command_available` is computed and stored in the struct, but if it is `false`
no `issues.push()` is called. The `has_issues` field at line 416 reads
`!issues.is_empty()`, so it stays `false` even with a missing MCP command.

---

## Surrounding Context in `run_json()`

| Lines | Block | Pushes to `issues`? |
|---|---|---|
| 303 | `let mut issues: Vec<String> = Vec::new();` | — initializer |
| 305–307 | no-config guard | yes |
| 311–355 | `let tools = if let Some(cfg) = config { ... }` | yes, on `!installed` |
| **357–369** | **`let mcp = config.and_then(...)` — BUG** | **NO** |
| 371–381 | `let agents = config.and_then(...)` | no (agents have no availability check) |
| 383–408 | `let secrets = if let Some(cfg) = config { ... }` | yes, on `!is_set` |
| 410–422 | `let report = StatusReport { ..., has_issues: !issues.is_empty(), issues, ... }` | — consumer |
| 424 | `println!("{}", serde_json::to_string_pretty(&report)?)` | — output |

**Pattern to follow:** lines 311–355 (tools block). Explicit `if let Some(cfg) = config`
allowing mutable borrow of `issues` inside the loop.

---

## `McpConfig` Struct — Required vs Optional Fields

**File:** `src/config/schema.rs`, lines 95–111

```rust
pub struct McpConfig {
    pub command: String,               // REQUIRED — non-optional
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub transport: Option<String>,
    pub url: Option<String>,
    pub enabled: Option<bool>,         // skip_serializing_if = "Option::is_none"
}
```

`McpConfig` has no `#[derive(Default)]` — cannot use `..Default::default()`.
Only `command` is required. All other fields are `Option`. Table key hyphens
are valid TOML (e.g., `[mcp.fake-server]`).

---

## `McpStatus` Serialization Struct

**File:** `src/cli/status.rs`, lines 42–51

```rust
struct McpStatus {
    name: String,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Vec<String>>,
    command_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    transport: Option<String>,
}
```

JSON key names are `name`, `command`, `args`, `command_available`, `transport`.
The spec test accesses `s["name"]` and `s["command_available"]` — both correct.

---

## Human-Readable Mode MCP Check — Lines 229–257

```rust
// src/cli/status.rs lines 229-257 (run())
if let Some(mcps) = &cfg.mcp {
    for (name, mcp) in mcps {
        let cmd_available = command_exists(&mcp.command);
        if cmd_available {
            // ... success output ...
        } else {
            output::error(&format!("  {} ({} -- not found)", name, mcp.command));
            has_issues = true;   // line 254
        }
    }
}
// line 279: if has_issues { output::info("Run `great doctor`...") }
```

Test 1 asserts `stderr contains "not found"` and `stderr contains "great doctor"`.
Both are exercised by this existing path. The "great doctor" hint fires at line 279
whenever `has_issues` is true, which includes the MCP-unavailable case.

Socrates advisory 3: also assert `stderr contains "fake-server"` for specificity.
The spec's test 1 does not assert the name, but line 253 already includes `name`
in the format string — the builder may add this assertion.

---

## Test File Insertion Point

**File:** `tests/cli_smoke.rs`
**Total lines:** 2258

Status section runs from line 53 to line 2094. The recommended insertion point
is **after line 2048** (end of `status_human_and_json_exit_codes_match`) and
before line 2050 (start of `status_no_config_exits_zero`).

### Existing status tests for orientation

| Lines | Test name |
|---|---|
| 57–66 | `status_shows_platform` |
| 68–77 | `status_warns_no_config` |
| 79–88 | `status_json_outputs_json` |
| 1821–1841 | `status_json_valid_json` |
| 1843–1860 | `status_json_no_config_still_valid` |
| 1862–1894 | `status_json_with_config_includes_tools` |
| 1896–1934 | `status_json_with_secrets` |
| 1936–1959 | `status_exit_zero_even_with_missing_tools` |
| 1962–1987 | `status_exit_zero_even_with_missing_secrets` |
| 1989–2013 | `status_json_always_exits_zero_even_with_issues` |
| 2016–2048 | `status_human_and_json_exit_codes_match` ← **insert after here** |
| 2050–2057 | `status_no_config_exits_zero` |

### Import situation

`tests/cli_smoke.rs` imports at lines 1–3:
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
```

`serde_json` is used at lines 1833–1834, 1854–1855, 1886–1887, 1921–1922
**without a `use` statement** — called as `serde_json::from_str(...)` directly.
`serde_json` is in `[dependencies]` (not dev-dependencies), so it is available
in test code. **No new imports needed.**

---

## Dependency Map

```
tests/cli_smoke.rs
  └─ invokes `great status --json` via assert_cmd
       └─ src/cli/status.rs :: run_json()
            ├─ src/platform :: command_exists()  [unchanged]
            ├─ src/config :: GreatConfig         [unchanged]
            │    └─ src/config/schema.rs :: McpConfig [unchanged]
            └─ serde_json::to_string_pretty()    [unchanged]
```

No new dependencies. No new files. No new structs.

---

## Risks

| Risk | Severity | Notes |
|---|---|---|
| HashMap iteration order non-deterministic | Low | Tests use `.any()` — correct. Issue string ordering irrelevant for single-server test. |
| `enabled = false` MCP servers | None | Neither `run()` nor `run_json()` checks `enabled`. The fix replicates the same omission as tools — intentional, out of scope. |
| Secrets block uses closures with `issues.push()` | Advisory | Socrates confirmed: the in-closure pattern also compiles (secrets block proves it). The spec's if-let refactor is preferable for consistency but not borrow-checker required. Builder may choose either approach; if-let is recommended. |
| Test 1 "not found" assertion not name-specific | Advisory | Could pass on spurious stderr. Adding `.stderr(predicate::str::contains("fake-server"))` would harden it. |

---

## Recommended Build Order

1. **Fix `src/cli/status.rs` lines 357–369**: replace `let mcp = config.and_then(...)` with
   explicit `if let` block matching the tools pattern (lines 311–355). Push
   `format!("MCP server '{}' command '{}' not found", name, m.command)` when `!available`.

2. **Add Test 1 to `tests/cli_smoke.rs`** after line 2048:
   `status_mcp_missing_command_shows_not_found` — human mode, asserts exit 0,
   stderr "not found", stderr "great doctor". Consider also asserting
   stderr "fake-server" per Socrates advisory 3.

3. **Add Test 2 to `tests/cli_smoke.rs`** directly after Test 1:
   `status_json_mcp_missing_sets_has_issues` — JSON mode, parses output with
   `serde_json::from_str`, asserts `command_available: false`, `has_issues: true`,
   `issues` array mentions "fake-server" and "not found".

4. `cargo test status_mcp_missing_command_shows_not_found status_json_mcp_missing_sets_has_issues`
5. `cargo test` (full suite, no regressions)
6. `cargo clippy`

---

## Technical Debt Noted

No new debt introduced. The existing `enabled` field on `McpConfig` is not
checked in `run()` or `run_json()` — this is pre-existing and out of scope.
