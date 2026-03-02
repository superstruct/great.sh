# Spec 0022: Align `great diff` Counter Buckets with Visual Markers

**Size:** S (one file changed, one file tests added/modified, zero new deps)
**Source:** `.tasks/backlog/0022-diff-counter-marker-consistency.md`

---

## Summary

`src/cli/diff.rs` has two counting bugs and an inconsistency in section headers:

1. **MCP bucket mismatch (line 143):** When an MCP server's command binary is
   missing, the code renders a `+` (green/install) marker but increments
   `configure_count` instead of `install_count`.

2. **Duplicate secrets count (lines 179--227):** A secret that appears in both
   `secrets.required` and the output of `find_secret_refs()` is counted twice
   in `secrets_count`, inflating the summary.

3. **Section header inconsistency (lines 122, 170, 197, 216):** Four section
   headers use different styles. This spec normalizes them.

---

## Bug Analysis

### Bug 1: MCP bucket mismatch

**Where:** `src/cli/diff.rs`, lines 140--150.

```rust
// Line 141-143: current code
let cmd_available = command_exists(&mcp.command);
if !cmd_available {
    configure_count += 1;          // <-- BUG: increments configure, not install
    mcp_diffs.push(format!(
        "  {} {} {}",
        "+".green(),               // <-- marker says "install"
        name.bold(),
        format!("({} -- not found)", mcp.command).dimmed()
    ));
}
```

The visual marker `+` (green) communicates "needs to be installed." The counter
should be `install_count`, not `configure_count`.

**Invariant to enforce:** Every diff line's marker determines its summary bucket:

| Marker | Color  | Meaning                              | Counter bucket    |
|--------|--------|--------------------------------------|-------------------|
| `+`    | green  | Not present; needs installation      | `install_count`   |
| `~`    | yellow | Present but wrong version/config     | `configure_count` |
| `-`    | red    | Blocked (missing secret/credential)  | `secrets_count`   |

This invariant is already correct for the Tools section (lines 59--128). It is
violated only in the MCP section at line 143.

### Bug 2: Duplicate secrets count

**Where:** `src/cli/diff.rs`, two independent loops that both feed `secrets_count`.

**Loop 1 -- lines 179--203:** Iterates `cfg.secrets.required` (a `Vec<String>`).
For each key not found in `std::env::var`, increments `secrets_count` by 1.

**Loop 2 -- lines 207--227:** Calls `cfg.find_secret_refs()` which scans MCP env
values and agent `api_key` fields for `${SECRET_NAME}` patterns. Returns a
sorted, deduplicated `Vec<String>`. For each ref not found in `std::env::var`,
increments `secrets_count` by 1.

**Overlap scenario:** A `great.toml` like:

```toml
[secrets]
required = ["ANTHROPIC_API_KEY"]

[agents.claude]
api_key = "${ANTHROPIC_API_KEY}"
```

`ANTHROPIC_API_KEY` appears in `secrets.required` AND is returned by
`find_secret_refs()`. If the env var is unset, it gets counted twice: once
on line 185 and once on line 218.

**Root cause:** The two loops share no deduplication state. Each independently
checks `std::env::var` and independently increments `secrets_count`.

### Issue 3: Section header inconsistency

Current headers:

| Line | Header text                          | Style       |
|------|--------------------------------------|-------------|
| 122  | `"Tools"`                            | Bare noun   |
| 170  | `"MCP Servers -- need configuration:"` | Noun + action |
| 197  | `"Secrets -- need to set:"`          | Noun + action |
| 216  | `"Secret References -- unresolved:"` | Noun + action |

The Tools header is the outlier. However, the existing integration test
`diff_missing_tool_shows_plus` (line 177 of `cli_smoke.rs`) does NOT assert
on the header text -- it only asserts on the tool name and `great apply`.
Similarly, no existing test asserts on the exact header strings for MCP or
Secrets sections. Therefore, normalizing headers will not break existing tests.

---

## Fix Design

### Fix 1: MCP `install_count` correction

At line 143 of `src/cli/diff.rs`, change `configure_count += 1` to
`install_count += 1`.

That is the entire fix for Bug 1. One token change.

### Fix 2: Secrets deduplication

Replace the two independent loops (lines 179--227) with a single unified
collection that deduplicates before counting.

**Strategy:**

1. Create a `BTreeSet<String>` called `all_missing_secrets` before either
   secrets loop.
2. In the first loop (over `secrets.required`), for each key where
   `std::env::var(key).is_err()`, insert it into `all_missing_secrets` and
   push the display line into `secret_diffs` as today.
3. In the second loop (over `find_secret_refs()`), for each ref where
   `std::env::var(ref_name).is_err()`, check if the ref is already in
   `all_missing_secrets`. If it is, skip it (already counted). If it is
   not, insert it and push the display line.
4. After both loops, set `secrets_count = all_missing_secrets.len()`.

Using `BTreeSet` (not `HashSet`) keeps output deterministic without an extra
sort step. `std::collections::BTreeSet` is in the standard library; no new
crate dependency needed.

**Detailed pseudocode for the unified block (replaces lines 178--227):**

```
let mut all_missing_secrets: BTreeSet<String> = BTreeSet::new();
let mut secret_diffs: Vec<String> = Vec::new();

// Phase 1: secrets.required
if let Some(secrets) = &cfg.secrets {
    if let Some(required) = &secrets.required {
        for key in required {
            if std::env::var(key).is_err() {
                all_missing_secrets.insert(key.clone());
                secret_diffs.push(format!(
                    "  {} {} {}",
                    "-".red(),
                    key.bold(),
                    "(not set in environment)".dimmed()
                ));
            }
        }
    }
}

// Phase 2: find_secret_refs (MCP env + agent api_key)
let secret_refs = cfg.find_secret_refs();
for ref_name in &secret_refs {
    if std::env::var(ref_name).is_err() && !all_missing_secrets.contains(ref_name) {
        all_missing_secrets.insert(ref_name.clone());
        secret_diffs.push(format!(
            "  {} {} {}",
            "-".red(),
            ref_name.bold(),
            "(referenced in config, not set)".dimmed()
        ));
    }
}

secrets_count = all_missing_secrets.len();

// Display: single unified section (replaces two separate sections)
if !secret_diffs.is_empty() {
    has_diff = true;
    output::header("Secrets");
    for diff in &secret_diffs {
        println!("{}", diff);
    }
    println!();
}
```

Key behavioral changes:
- The two separate sections ("Secrets -- need to set:" and
  "Secret References -- unresolved:") merge into one section called "Secrets".
- Duplicate keys are counted exactly once. The first occurrence (from
  `secrets.required`) takes priority in the display; the second occurrence
  (from `find_secret_refs`) is silently skipped.
- The descriptive suffix for refs changes from "(referenced in MCP env, not set)"
  to "(referenced in config, not set)" since `find_secret_refs()` also scans
  agent `api_key` fields, not just MCP env.

### Fix 3: Section header normalization

Adopt bare-noun style for all section headers. This is the simpler style and
matches the existing "Tools" header.

| Before                                 | After          |
|----------------------------------------|----------------|
| `"Tools"` (line 122)                   | `"Tools"`      |
| `"MCP Servers -- need configuration:"` (line 170) | `"MCP Servers"` |
| `"Secrets -- need to set:"` (line 197) | `"Secrets"`    |
| `"Secret References -- unresolved:"` (line 216) | *(merged into "Secrets")* |

Only the MCP header changes (line 170). The two secrets headers are replaced by
the single "Secrets" header from Fix 2. The Tools header is already correct.

---

## Interfaces

No new public interfaces. No function signature changes. The `pub fn run(args: Args) -> Result<()>` signature is unchanged.

**New import required at top of `src/cli/diff.rs`:**

```rust
use std::collections::BTreeSet;
```

---

## Files to Modify

| File | Change |
|------|--------|
| `src/cli/diff.rs` | Bug fixes 1, 2, 3 as described above |
| `tests/cli_smoke.rs` | New tests + adjust existing test assertions if needed |

No files to create or delete. No dependency changes in `Cargo.toml`.

---

## Implementation Build Order

All changes are in a single file (`diff.rs`) with no inter-dependencies. The
recommended order for the implementer:

1. Add `use std::collections::BTreeSet;` to the imports (line 1 area).
2. Fix Bug 1: Change `configure_count` to `install_count` on line 143.
3. Fix Bug 2: Replace the two secrets loops (lines 178--227) with the unified
   deduplication block described above.
4. Fix header strings: Change line 170 from `"MCP Servers -- need configuration:"`
   to `"MCP Servers"`.
5. Add/update integration tests in `tests/cli_smoke.rs`.

---

## Edge Cases

### Empty inputs
- **No `[secrets]` section:** `cfg.secrets` is `None`. The first `if let Some`
  short-circuits. `find_secret_refs()` still runs (it scans MCP and agents
  independently). No crash, correct behavior.
- **No `[mcp]` section:** The MCP block is skipped entirely. `find_secret_refs()`
  returns an empty vec if there are no MCP entries and no agent api_keys.
  Correct behavior.
- **Empty `secrets.required = []`:** The inner `for key in required` loop
  iterates zero times. Correct behavior.
- **All secrets are set:** Both loops find zero missing secrets.
  `all_missing_secrets` is empty, `secrets_count` is 0. No section printed.

### Platform differences (macOS ARM64/x86_64, Ubuntu, WSL2)
- `command_exists` uses `which::which()` which works identically on all
  three platforms. No platform-specific behavior changes.
- `std::env::var` is cross-platform. No changes needed.
- The `colored` crate respects `NO_COLOR` and `TERM` on all platforms. The
  fix does not alter color behavior.

### Concurrent access
- Not applicable. `great diff` is a single-threaded, read-only operation.
  It reads config, checks system state, and prints. No file writes, no
  shared mutable state.

### Interaction with `.mcp.json` check
- Lines 152--165 check whether `.mcp.json` exists. This is a separate
  `configure_count` increment for MCP servers whose command IS available but
  whose `.mcp.json` config is missing. This is correctly bucketed as `~`
  (configure) and should NOT be changed. Only the missing-command path
  (line 143) is wrong.

---

## Error Handling

No new error paths are introduced. The existing `anyhow::Result<()>` return
type and `?` propagation are unchanged. The `BTreeSet::insert` and
`BTreeSet::contains` methods are infallible.

The only actionable error message change is the suffix text for secret refs:
- Before: `"(referenced in MCP env, not set)"`
- After: `"(referenced in config, not set)"`

This is more accurate since `find_secret_refs()` scans both MCP env values
AND agent `api_key` fields.

---

## Security Considerations

- **No new inputs:** The fix does not add any new user-facing inputs or
  config fields.
- **No secret values exposed:** The code only checks whether env vars exist
  (`std::env::var(key).is_err()`). It never prints secret values.
- **No new network calls:** `great diff` is fully offline.

---

## Testing Strategy

### Existing tests that must continue passing (no assertion changes expected)

| Test | Asserts on | Impact |
|------|------------|--------|
| `diff_no_config_exits_nonzero` | Exit code + stderr "great.toml" | None |
| `diff_satisfied_config_exits_zero` | Exit code + stderr "nothing to do" | None |
| `diff_missing_tool_shows_plus` | stdout tool name + stderr "great apply" | None |
| `diff_disabled_mcp_skipped` | stdout/stderr NOT containing server name | None |
| `diff_version_mismatch_shows_tilde` | stdout "git" + "want 99.99.99" | None |
| `diff_with_custom_config_path` | stderr containing filename | None |
| `diff_unresolved_secret_shows_red_minus` | stdout secret name + "not set" | None |

### Test requiring assertion update

**`diff_summary_shows_counts`** (line 255 of `cli_smoke.rs`):

The current config in this test declares one nonexistent CLI tool and one
nonexistent secret. It asserts:
- stderr contains `"1 to install"`
- stderr contains `"1 secrets to resolve"`

After the fix, the MCP bucket change does not affect this test (no MCP servers
in the config). The secret dedup does not affect it (secret only in
`secrets.required`, not in `find_secret_refs`). **This test should pass
unchanged.**

### New tests to add

**Test 1: `diff_mcp_missing_command_counted_as_install`**

Purpose: Verify that an MCP server with a missing command is counted in
`install_count`, not `configure_count`.

```
Config:
  [project]
  name = "test"

  [mcp.fake-server]
  command = "nonexistent_mcp_cmd_xyz_77777"

Assert:
  - stderr contains "1 to install"
  - stderr does NOT contain "to configure"
  - stdout contains "nonexistent_mcp_cmd_xyz_77777"
```

**Test 2: `diff_mcp_missing_command_and_missing_tool_install_count`**

Purpose: Verify that missing tools AND missing MCP commands both accumulate
into `install_count`.

```
Config:
  [project]
  name = "test"

  [tools.cli]
  nonexistent_tool_xyz_66666 = "1.0.0"

  [mcp.fake-server]
  command = "nonexistent_mcp_cmd_xyz_66666"

Assert:
  - stderr contains "2 to install"
```

**Test 3: `diff_secret_dedup_required_and_ref`**

Purpose: Verify that a secret appearing in both `secrets.required` and as
a `${...}` reference in MCP env is counted only once.

```
Config:
  [project]
  name = "test"

  [secrets]
  required = ["DEDUP_TEST_SECRET_XYZ_55555"]

  [mcp.test-server]
  command = "echo"
  env = { KEY = "${DEDUP_TEST_SECRET_XYZ_55555}" }

Assert:
  - stderr contains "1 secrets to resolve"   (not "2 secrets")
  - stderr does NOT contain "2 secrets"
```

Note: `echo` is used as the MCP command because it exists on all platforms
(macOS, Linux, WSL2), so `command_exists("echo")` returns true and we avoid
an extraneous install_count increment.

**Test 4: `diff_secret_ref_only_no_required_section`**

Purpose: Verify that secrets found only via `find_secret_refs()` (no
`secrets.required` section) are still counted.

```
Config:
  [project]
  name = "test"

  [mcp.test-server]
  command = "echo"
  env = { KEY = "${REFONLY_SECRET_XYZ_44444}" }

Assert:
  - stderr contains "1 secrets to resolve"
  - stdout contains "REFONLY_SECRET_XYZ_44444"
```

### Regression verification

After implementing all fixes, run:
```
cargo test
cargo clippy
```

All existing tests must pass. Zero new clippy warnings.

---

## Acceptance Criteria

- [ ] Line 143 of `diff.rs`: `configure_count += 1` changed to `install_count += 1`
- [ ] Every `+` marker line increments `install_count`; every `~` marker line increments `configure_count`; every `-` marker line contributes to `secrets_count`
- [ ] Secrets from `secrets.required` and `find_secret_refs()` are deduplicated via `BTreeSet` before counting
- [ ] `secrets_count` equals the cardinality of the dedup set, not the sum of two independent loops
- [ ] Section headers use consistent bare-noun style: "Tools", "MCP Servers", "Secrets"
- [ ] New test `diff_mcp_missing_command_counted_as_install` passes
- [ ] New test `diff_mcp_missing_command_and_missing_tool_install_count` passes
- [ ] New test `diff_secret_dedup_required_and_ref` passes
- [ ] New test `diff_secret_ref_only_no_required_section` passes
- [ ] All 8 pre-existing diff integration tests pass without assertion changes
- [ ] `cargo clippy` produces zero new warnings
- [ ] No new crate dependencies added

---

## Risk Assessment

**Low risk.** The changes are confined to counter arithmetic and display logic
in a single function. No public API changes, no new dependencies, no async
code, no file I/O changes.

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Existing test `diff_summary_shows_counts` breaks due to count change | Low | Low | Test config has no MCP servers and no overlapping secrets; unaffected by both fixes |
| CI consumers parsing exact header text break | Low | Medium | No known CI consumers parse header text; headers are on stderr, not stdout |
| `BTreeSet` import conflicts with existing imports | None | None | `diff.rs` has no existing `std::collections` import |
| MCP servers with missing command but existing `.mcp.json` misclassified | None | None | The `.mcp.json` check (lines 152-165) only runs when `cmd_available` is true; the missing-command path is independent |
