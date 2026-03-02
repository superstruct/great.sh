# Humboldt Scout Report — Task 0022: Diff Counter/Marker Consistency

**Scout:** Alexander von Humboldt
**Date:** 2026-02-26
**Spec:** `.tasks/ready/0022-diff-counter-spec.md`

---

## Files to Modify

| File | Lines Changed | Nature |
|------|---------------|--------|
| `src/cli/diff.rs` | 1, 143, 155-165, 178-227 | Bug fixes + import |
| `tests/cli_smoke.rs` | After line 304 | 4 new test functions |

No files to create. No files to delete. No `Cargo.toml` changes.

---

## File 1: `src/cli/diff.rs` (248 lines total)

### Current Imports (lines 1-8)

```rust
use anyhow::Result;
use clap::Args as ClapArgs;
use colored::Colorize;

use crate::cli::output;
use crate::cli::util;
use crate::config;
use crate::platform::command_exists;
```

**Required addition:** Insert `use std::collections::BTreeSet;` after line 1
(before `use clap::Args as ClapArgs;`). There is no existing `std::collections`
import — no conflict possible.

### Counter Variable Declarations (lines 54-57)

```rust
let mut has_diff = false;
let mut install_count: usize = 0;    // line 55
let mut configure_count: usize = 0; // line 56
let mut secrets_count: usize = 0;   // line 57
```

All three are `usize`, declared together in `run()`, before any section loops.

### Counter Increment Sites and Paired Markers

#### Tools section (lines 59-128) — CORRECT, no changes needed

| Lines | Counter | Marker | Color | Condition |
|-------|---------|--------|-------|-----------|
| 70, 97 | `install_count += 1` | `"+"` | green | command not found |
| 80, 107 | `configure_count += 1` | `"~"` | yellow | version mismatch |

Both runtime tools (lines 64-90) and CLI tools (lines 93-118) follow the same
pattern. Marker and counter are consistent throughout.

#### MCP Servers section (lines 130-176) — BUG AT LINE 143

**Bug 1 site — line 143:**
```rust
// Lines 140-150: missing-command path
let cmd_available = command_exists(&mcp.command);
if !cmd_available {
    configure_count += 1;   // LINE 143 — BUG: should be install_count
    mcp_diffs.push(format!(
        "  {} {} {}",
        "+".green(),         // marker says install, counter says configure
        name.bold(),
        format!("({} — not found)", mcp.command).dimmed()
    ));
}
```

**Lines 152-165: `.mcp.json` check — CORRECT, do not change:**
```rust
if cmd_available {
    configure_count += 1;   // line 157 — correct, ~ marker used
    mcp_diffs.push(format!(
        "  {} {} {}",
        "~".yellow(),        // configure marker, configure counter — consistent
        ...
    ));
}
```

**Section header at line 170 — Bug 3 site:**
```rust
output::header("MCP Servers — need configuration:");  // change to "MCP Servers"
```

#### Secrets section (lines 178-227) — BUG (duplicate counting)

**Bug 2: Two independent loops both feed `secrets_count`**

Loop 1 (lines 179-203) — iterates `cfg.secrets.required`:
```rust
// lines 183-193
for key in required {
    if std::env::var(key).is_err() {
        secrets_count += 1;         // line 185
        secret_diffs.push(format!(
            "  {} {} {}",
            "-".red(),
            key.bold(),
            "(not set in environment)".dimmed()
        ));
    }
}
// Section header at line 197: "Secrets — need to set:"
```

Loop 2 (lines 207-227) — iterates `cfg.find_secret_refs()`:
```rust
// lines 209-226
for ref_name in &secret_refs {
    if std::env::var(ref_name).is_err() {
        // NO dedup check against loop 1
        secrets_count += 1;         // line 218
        println!(
            "  {} {} {}",
            "-".red(),
            name.bold(),
            "(referenced in MCP env, not set)".dimmed()
        );
    }
}
// Section header at line 216: "Secret References — unresolved:"
```

Both loops independently increment `secrets_count`. A key in both
`secrets.required` AND returned by `find_secret_refs()` is counted twice.
The two loops also produce two separate rendered sections in output.

### Summary Line (lines 229-244)

```rust
if !has_diff {
    output::success("Environment matches configuration — nothing to do.");
} else {
    let mut parts = Vec::new();
    if install_count > 0 {
        parts.push(format!("{} to install", install_count));      // line 234
    }
    if configure_count > 0 {
        parts.push(format!("{} to configure", configure_count));  // line 237
    }
    if secrets_count > 0 {
        parts.push(format!("{} secrets to resolve", secrets_count)); // line 240
    }
    let summary = parts.join(", ");
    output::info(&format!("{} — run `great apply` to reconcile.", summary));
}
```

Summary format: `"N to install, N to configure, N secrets to resolve — run \`great apply\` to reconcile."`

The summary is written to stderr via `output::info`. Tests assert on stderr.
Diff item lines (tool names, secret names) are written to stdout via `println!`.

---

## File 2: `tests/cli_smoke.rs` — Diff Tests Map

### Existing diff tests (all in the `// Diff` section)

| Test name | Lines | Config TOML used | Assertions |
|-----------|-------|------------------|-----------|
| `diff_no_config_exits_nonzero` | 122-131 | none (empty TempDir) | `.failure()`, stderr contains `"great.toml"` |
| `diff_satisfied_config_exits_zero` | 133-155 | `[tools.cli] git = "latest"` | `.success()`, stderr contains `"nothing to do"` |
| `diff_missing_tool_shows_plus` | 157-179 | `[tools.cli] nonexistent_tool_xyz_88888 = "1.0.0"` | stdout contains `"nonexistent_tool_xyz_88888"`, stderr contains `"great apply"` |
| `diff_disabled_mcp_skipped` | 181-204 | `[mcp.disabled-server] command = "nonexistent_cmd_xyz_77777" enabled = false` | stdout NOT contains `"disabled-server"`, stderr NOT contains `"disabled-server"` |
| `diff_version_mismatch_shows_tilde` | 206-228 | `[tools.cli] git = "99.99.99"` | stdout contains `"git"`, stdout contains `"want 99.99.99"` |
| `diff_with_custom_config_path` | 230-252 | `[tools.cli] git = "latest"` via `--config` flag | stderr contains `"custom.toml"` |
| `diff_summary_shows_counts` | 254-280 | `nonexistent_tool_xyz_99999` + `required = ["NONEXISTENT_SECRET_XYZ_99999"]` | stderr contains `"1 to install"`, stderr contains `"1 secrets to resolve"`, stderr contains `"great apply"` |
| `diff_unresolved_secret_shows_red_minus` | 282-304 | `required = ["NONEXISTENT_SECRET_XYZ_88888"]` | stdout contains `"NONEXISTENT_SECRET_XYZ_88888"`, stdout contains `"not set in environment"` |

### Section boundary

The diff section ends at line 304. Next section is `// Template` beginning at
line 306.

**Test insertion point: after line 304, before line 306.**

### Unique tool name pattern used by existing tests

Existing tests use suffix `_xyz_NNNNN` with 5-digit suffixes: `88888`, `77777`,
`99999`. New tests must use distinct 5-digit suffixes to avoid env collision:
- `77777` — used by `diff_disabled_mcp_skipped`
- `88888` — used by `diff_missing_tool_shows_plus` and `diff_unresolved_secret_shows_red_minus`
- `99999` — used by `diff_summary_shows_counts`

Spec prescribes: `66666` for test 2, `55555` for test 3, `44444` for test 4,
`77777` for test 1's MCP command (safe because `diff_disabled_mcp_skipped` uses
it only when `enabled = false`, and test 1's MCP is enabled — the command name
is used in a different assertion context). The builder should use the exact
names from the spec to match acceptance criteria.

---

## File 3: `src/config/schema.rs` — `find_secret_refs()` (lines 246-277)

```rust
pub fn find_secret_refs(&self) -> Vec<String> {
    let mut refs = Vec::new();
    let re = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").expect("valid regex");

    // Scan agent api_key fields for secret references (lines 251-259)
    if let Some(agents) = &self.agents {
        for agent in agents.values() {
            if let Some(api_key) = &agent.api_key {
                for cap in re.captures_iter(api_key) {
                    refs.push(cap[1].to_string());
                }
            }
        }
    }

    // Scan MCP env values for secret references (lines 262-272)
    if let Some(mcps) = &self.mcp {
        for mcp in mcps.values() {
            if let Some(env) = &mcp.env {
                for value in env.values() {
                    for cap in re.captures_iter(value) {
                        refs.push(cap[1].to_string());
                    }
                }
            }
        }
    }

    refs.sort();
    refs.dedup();    // line 275 — Vec::dedup removes consecutive duplicates
    refs             // returns Vec<String>
}
```

**Key facts for the builder:**

- Return type: `Vec<String>` (sorted, deduped)
- Dedup mechanism: `Vec::sort()` then `Vec::dedup()` — removes consecutive
  duplicates after sorting. This handles cross-MCP-server duplicates correctly.
- Scans TWO sources: `agent.api_key` fields AND `mcp.env` values
- This is why the old suffix text `"(referenced in MCP env, not set)"` is
  inaccurate — the fix changes it to `"(referenced in config, not set)"`
- `find_secret_refs()` does NOT check `secrets.required` — that is a separate
  field entirely. The overlap is between `secrets.required` (explicit list) and
  `find_secret_refs()` output (scanned pattern matches). The BTreeSet in the
  fix deduplicates across both sources.

**`SecretsConfig` struct (lines 111-117):**
```rust
pub struct SecretsConfig {
    pub provider: Option<String>,
    pub required: Option<Vec<String>>,  // line 116 — Vec<String>, not a set
}
```

The `required` field is a plain `Vec<String>` with no inherent deduplication.

---

## Dependency Map

```
diff.rs::run()
  ├── config::discover_config()          [src/config/mod.rs — unchanged]
  ├── config::load()                     [src/config/mod.rs — unchanged]
  ├── command_exists()                   [src/platform/mod.rs — unchanged]
  ├── util::get_command_version()        [src/cli/util.rs — unchanged]
  ├── cfg.find_secret_refs()             [src/config/schema.rs — read-only, no changes]
  └── output::{header,info,success,error} [src/cli/output.rs — unchanged]
```

No external crate changes. `BTreeSet` is `std::collections::BTreeSet` — stdlib only.

---

## Risks

**Risk 1 — `diff_unresolved_secret_shows_red_minus` assertion on `"not set in environment"`**

Line 303: `stdout(predicate::str::contains("not set in environment"))`

The Fix 2 unified block changes the display suffix for `secrets.required` items
to remain `"(not set in environment)"` (unchanged). Only the `find_secret_refs()`
suffix changes (from `"(referenced in MCP env, not set)"` to
`"(referenced in config, not set)"`). The test at line 303 uses a config with
only `secrets.required`, no MCP refs. **This test is unaffected.**

**Risk 2 — Section header change for MCP (line 170)**

No existing test asserts on `"MCP Servers"` header text. Confirmed: none of
the 8 existing diff tests contains `"MCP Servers"` in assertions.
Change is safe.

**Risk 3 — Merging two secrets sections into one**

The old two-section output (`"Secrets — need to set:"` and
`"Secret References — unresolved:"`) becomes one `"Secrets"` section. No
existing test asserts on either old header string. **No assertion breakage.**

**Technical debt flagged:** The `output::header()` function renders to stderr
(confirmed by test assertions using `.stderr()`), while item detail lines use
`println!()` which goes to stdout. This split is an existing inconsistency in
the codebase — the builder should not change this behavior, only match it.

---

## Recommended Build Order

1. `src/cli/diff.rs` line 1: add `use std::collections::BTreeSet;`
2. `src/cli/diff.rs` line 143: `configure_count += 1` → `install_count += 1`
3. `src/cli/diff.rs` line 170: change header string to `"MCP Servers"`
4. `src/cli/diff.rs` lines 178-227: replace both secrets loops with unified
   `BTreeSet`-deduped block; single `"Secrets"` section header
5. `tests/cli_smoke.rs` after line 304: add 4 new test functions in order
   matching spec names

Run `cargo test` and `cargo clippy` after each step to catch regressions early.
