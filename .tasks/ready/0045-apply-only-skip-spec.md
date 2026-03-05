# 0045 -- Specification: `--only` and `--skip` flags for `great apply`

| Field | Value |
|---|---|
| Task ID | 0045 |
| Spec author | Lovelace |
| Date | 2026-03-05 |
| Status | ready |
| Estimated complexity | M |

---

## 1. Overview

Add two mutually exclusive, multi-value flags to `great apply` that let users
selectively run or skip provisioning categories. This enables faster iteration
when working on a single concern (e.g., only configuring MCP servers) and
speeds up CI pipelines that only need a subset of provisioning.

Four categories are defined: `tools`, `mcp`, `agents`, `secrets`. Each maps
to one or more numbered sections in the current `apply::run()` function.

---

## 2. CLI Interface

### 2.1 New type: `ApplyCategory`

A `clap::ValueEnum` enum with four variants. Placed in `src/cli/apply.rs`
above the `Args` struct (before line 354).

```
enum ApplyCategory {
    Tools,    // displayed as "tools"
    Mcp,      // displayed as "mcp"
    Agents,   // displayed as "agents"
    Secrets,  // displayed as "secrets"
}
```

Requirements:
- Derive `Clone`, `Debug`, `PartialEq`, `Eq`, and `clap::ValueEnum`.
- clap `ValueEnum` generates the lowercase string representations automatically.
- Using a typed enum rather than raw strings means clap rejects unknown
  categories at parse time with a standard error message, e.g.:
  `error: invalid value 'foo' for '--only <CATEGORY>'`.

### 2.2 New fields on `Args`

Add two fields to the `Args` struct (currently at line 354) after the
existing `yes` field and before the `non_interactive` skip field:

```
/// Only apply these categories (tools, mcp, agents, secrets). Repeatable.
#[arg(long, value_delimiter = ',', conflicts_with = "skip")]
pub only: Vec<ApplyCategory>,

/// Skip these categories (tools, mcp, agents, secrets). Repeatable.
#[arg(long, value_delimiter = ',', conflicts_with = "only")]
pub skip: Vec<ApplyCategory>,
```

Behaviour:
- `--only tools` applies only tools. `--only tools,mcp` applies both.
- `--only tools --only mcp` also works (clap `Vec` accumulates).
- `--skip tools` applies everything except tools.
- `--only tools --skip mcp` exits with code 2 (clap conflict error) and
  prints a message like: `error: the argument '--only' cannot be used with '--skip'`.
- When neither flag is supplied, all categories run (current behaviour preserved).

### 2.3 Helper method: `should_apply`

Add a private free function (or an `Args` method) in `src/cli/apply.rs`:

```
fn should_apply(category: ApplyCategory, only: &[ApplyCategory], skip: &[ApplyCategory]) -> bool
```

Logic:
- If `only` is non-empty: return `only.contains(&category)`.
- If `skip` is non-empty: return `!skip.contains(&category)`.
- Otherwise: return `true`.

Since `only` and `skip` are `conflicts_with` each other, they will never
both be non-empty simultaneously. The function handles the degenerate case
defensively anyway.

---

## 3. Category-to-Section Mapping

The `run()` function (line 380) has numbered sections. Each maps to a
category as follows:

| Category | Sections in `run()` | Line range (approx.) | Description |
|---|---|---|---|
| `tools` | 2a-2c (sudo, prereqs, homebrew), 3 (runtimes), 4 (CLI tools), 5b (bitwarden-cli), 5c (starship + nerd font), 7 (platform tools), 8 (docker), 9 (Claude Code), 10 (system tuning) | 408-898 | All tool installation and system bootstrapping |
| `mcp` | 5 (MCP servers), 5a (MCP bridge) | 649-778 | MCP server config and bridge registration |
| `agents` | (no current section) | N/A | Reserved for future loop-agent file installation |
| `secrets` | 6 (secrets check) | 821-838 | Required secrets validation |

### 3.1 Gating strategy

Wrap each section boundary with a `should_apply` check. The pattern is:

```
if should_apply(ApplyCategory::Tools, &args.only, &args.skip) {
    // existing tools provisioning code
}
```

Sections that are unconditional today (config loading at step 1, platform
detection at step 2, dry-run banner, and the final summary) remain
unconditional -- they always run regardless of filters.

Detailed gating points in `run()`:

1. **`ApplyCategory::Tools` gate** -- wrap around:
   - The sudo credential caching block (line 408-431). This only exists to
     support tool installation, so it is gated under `tools`.
   - `bootstrap::ensure_prerequisites()` (line 434).
   - The Homebrew installation block (lines 440-492).
   - Section 3: runtimes via mise (lines 495-570).
   - Section 4: CLI tools (lines 573-646).
   - Section 5b: bitwarden-cli auto-install (lines 781-803). This installs
     a tool; it belongs under `tools` not `secrets`.
   - Section 5c: starship + nerd font (lines 806-818).
   - Section 7: platform-specific tools (lines 841-886).
   - Section 8: docker (line 889).
   - Section 9: Claude Code (lines 892-894).
   - Section 10: system tuning (line 897).

2. **`ApplyCategory::Mcp` gate** -- wrap around:
   - Section 5: MCP server configuration (lines 649-724).
   - Section 5a: MCP bridge registration (lines 727-778).

3. **`ApplyCategory::Agents` gate** -- currently a no-op. Insert an empty
   gated block with a comment indicating this is reserved for future agent
   file provisioning. Place it after the MCP gate and before secrets. If
   `--only agents` is used, the command should exit 0 successfully with
   just the header/summary and no errors. This satisfies the acceptance
   criterion that `--only agents --dry-run` exits 0 without error.

4. **`ApplyCategory::Secrets` gate** -- wrap around:
   - Section 6: secrets check (lines 821-838).

### 3.2 Agents placeholder

Since `apply` does not currently install agent files, the `agents` gate
body should be:

```
if should_apply(ApplyCategory::Agents, &args.only, &args.skip) {
    // Reserved: loop-agent file provisioning will be added here.
}
```

This is intentionally a no-op. The category must still be accepted by the
parser so that `--only agents` does not error. This matches the acceptance
criteria.

---

## 4. Implementation Plan (Build Order)

All changes are in two files. No new files are created.

### Step 1: Define `ApplyCategory` enum

File: `src/cli/apply.rs`
Location: insert before the `Args` struct (before line 354).

Derive `Clone`, `Debug`, `PartialEq`, `Eq`, `clap::ValueEnum`.

### Step 2: Add `only` and `skip` fields to `Args`

File: `src/cli/apply.rs`
Location: inside the `Args` struct, after the `yes` field (after line 366),
before the `non_interactive` field.

### Step 3: Add `should_apply` helper

File: `src/cli/apply.rs`
Location: after the `Args` struct, before `pub fn run()`.

### Step 4: Gate provisioning sections in `run()`

File: `src/cli/apply.rs`
Location: inside `pub fn run()`.

Wrap the section blocks as described in section 3.1 above. Each gate is a
single `if should_apply(...)` check at the outermost level of each section.
Do not nest gates inside helper functions.

Preserve the unconditional blocks: config loading, platform detection,
dry-run banner, and final summary.

### Step 5: Add integration tests

File: `tests/cli_smoke.rs`

---

## 5. Test Plan

### 5.1 Integration tests (add to `tests/cli_smoke.rs`)

All tests use `--dry-run` to avoid side effects. They operate in a temp
directory with a minimal `great.toml`.

**Test 1: `apply_only_tools_dry_run`**
- Args: `["apply", "--only", "tools", "--dry-run"]`
- Write a minimal `great.toml` with `[project]` and `[tools.runtimes]`.
- Assert: exit code 0.
- Assert: stdout contains "Dry run" (proving the command ran).

**Test 2: `apply_only_mcp_dry_run`**
- Args: `["apply", "--only", "mcp", "--dry-run"]`
- Write a minimal `great.toml` with `[project]` and `[mcp.context7]`.
- Assert: exit code 0.

**Test 3: `apply_only_agents_dry_run`**
- Args: `["apply", "--only", "agents", "--dry-run"]`
- Write a minimal `great.toml` with `[project]`.
- Assert: exit code 0.
- Assert: stdout does NOT contain "CLI Tools" or "MCP Servers" (proving
  other sections were skipped).

**Test 4: `apply_skip_tools_dry_run`**
- Args: `["apply", "--skip", "tools", "--dry-run"]`
- Write a minimal `great.toml` with `[project]`.
- Assert: exit code 0.

**Test 5: `apply_only_and_skip_conflict`**
- Args: `["apply", "--only", "tools", "--skip", "mcp"]`
- Assert: exit code 2 (clap argument error).
- Assert: stderr contains "cannot be used with".

**Test 6: `apply_invalid_category`**
- Args: `["apply", "--only", "nonsense", "--dry-run"]`
- Assert: exit code 2 (clap argument error).
- Assert: stderr contains "invalid value" (clap's ValueEnum rejection).

### 5.2 Unit tests (add to the `mod tests` block in `src/cli/apply.rs`)

**Test: `test_should_apply_no_filters`**
- `should_apply(Tools, &[], &[])` returns `true`.

**Test: `test_should_apply_only_match`**
- `should_apply(Tools, &[Tools], &[])` returns `true`.
- `should_apply(Mcp, &[Tools], &[])` returns `false`.

**Test: `test_should_apply_skip_match`**
- `should_apply(Tools, &[], &[Tools])` returns `false`.
- `should_apply(Mcp, &[], &[Tools])` returns `true`.

**Test: `test_should_apply_only_multiple`**
- `should_apply(Tools, &[Tools, Mcp], &[])` returns `true`.
- `should_apply(Mcp, &[Tools, Mcp], &[])` returns `true`.
- `should_apply(Secrets, &[Tools, Mcp], &[])` returns `false`.

---

## 6. Edge Cases

| Edge case | Expected behaviour |
|---|---|
| `--only` with no value (`great apply --only`) | clap error: requires a value. Exit 2. |
| `--only` with unknown category (`--only foo`) | clap `ValueEnum` rejects it. Exit 2. Error message lists valid values. |
| `--only tools --only tools` (duplicate) | Works fine. `Vec` contains two `Tools` entries; `contains()` still returns true. No dedup needed. |
| `--only agents` when no agent provisioning exists | Exits 0 successfully. Prints header and summary. No-op body. |
| `--skip tools,mcp,agents,secrets` (skip everything) | Exits 0. Only the header, platform detection, and summary are printed. This is valid; the user asked for nothing. |
| `--only tools,mcp,agents,secrets` (include everything) | Equivalent to no filter. All sections run. |
| Comma-separated with spaces (`--only "tools, mcp"`) | clap `value_delimiter` splits on comma. `" mcp"` (with leading space) will fail ValueEnum parsing. This is standard clap behaviour. The user should use `--only tools,mcp` or `--only tools --only mcp`. |
| `--only` combined with `--dry-run` | Both work together. `--dry-run` is orthogonal to filtering. |
| `--only` combined with `--config <path>` | Both work together. Config is always loaded regardless of category filter. |

---

## 7. Error Handling

| Scenario | Action |
|---|---|
| Unknown category value | clap rejects at parse time with exit code 2 and lists valid values. No custom code needed. |
| `--only` + `--skip` together | clap `conflicts_with` rejects at parse time with exit code 2. No custom code needed. |
| Config file not found | Existing error handling applies (line 388). Occurs before any category gating. |
| A gated section would have errored | If the section is skipped by filter, the error never occurs. This is expected and correct. |

---

## 8. Security Considerations

- No new attack surface. The flags are purely additive filtering on existing
  provisioning logic.
- `--skip secrets` allows a user to bypass the secrets-not-set warnings.
  This is intentional -- the user explicitly asked to skip secrets checking.
  No secrets are exposed; the check is purely advisory.
- No secrets or credentials are involved in the flag parsing itself.

---

## 9. Platform Considerations

| Platform | Notes |
|---|---|
| macOS ARM64 / x86_64 | No platform-specific behaviour for the flags themselves. The gated sections already handle platform differences internally. |
| Ubuntu (native Linux) | Same as above. |
| WSL2 | Same as above. The sudo caching block (gated under `tools`) correctly handles WSL2 already. |

The flags are parsed by clap before any platform detection occurs. Platform
differences only matter within the gated section bodies, which are unchanged.

---

## 10. Files to Modify

| File | Change |
|---|---|
| `src/cli/apply.rs` | Add `ApplyCategory` enum, add `only`/`skip` to `Args`, add `should_apply()`, gate each provisioning section. |
| `tests/cli_smoke.rs` | Add 6 integration tests (tests 1-6 from section 5.1). |

No new files are created. No other files need changes -- `main.rs` already
passes `Args` through to `apply::run()` and clap handles the new fields
automatically.

---

## 11. Acceptance Checklist

- [ ] `great apply --only tools --dry-run` exits 0; MCP/secrets sections absent from stdout.
- [ ] `great apply --only mcp --dry-run` exits 0; tools section absent from stdout.
- [ ] `great apply --only agents --dry-run` exits 0; no error.
- [ ] `great apply --skip tools --dry-run` exits 0; tools section absent from stdout.
- [ ] `great apply --only tools --skip mcp` exits 2 with conflict error.
- [ ] `great apply --only nonsense` exits 2 with invalid value error.
- [ ] `cargo clippy` passes with no new warnings.
- [ ] `cargo test` passes (all existing + new tests).
- [ ] No `.unwrap()` in new production code.
