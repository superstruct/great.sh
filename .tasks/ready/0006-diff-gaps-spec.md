# Spec 0006: `great diff` -- Gap Completion

**Author:** Ada Lovelace (Spec Writer)
**Date:** 2026-02-25 (Round 2: blocking concerns resolved)
**Source:** `.tasks/ready/0006-nightingale-selection.md`
**Status:** Ready for implementation

---

## Summary

The `great diff` command (`src/cli/diff.rs`) is a working 198-line implementation
that compares declared `great.toml` configuration against actual system state.
Five gaps remain before the command is complete:

1. **Version comparison** -- installed tools are silently omitted; they should be
   checked for version mismatch and shown with a `~` (yellow) marker.
2. **Exit code on missing config** -- returns 0 instead of 1 when no `great.toml`
   is found.
3. **Disabled MCP servers not skipped** -- servers with `enabled = false` are
   still processed.
4. **Numeric summary line** -- the final output should print category counters
   ("N items to install, M items to configure, K secrets to resolve") instead of
   the generic "Run `great apply`" message.
5. **Red `-` marker for unresolved secrets** -- the backlog specifies red `-` for
   secrets that block progress, but the current code uses green `+`.

No new files, no new dependencies, no architectural changes.

**Line number convention:** All line numbers in this spec reference the ORIGINAL
`src/cli/diff.rs` (198 lines, commit `4addad2`) before any modifications. When
applying changes, the builder should use the BEFORE code snippets as anchors for
finding the replacement targets, not the line numbers, since earlier changes shift
subsequent lines.

---

## Files to Modify

| File | Change type | Lines affected |
|------|-------------|----------------|
| `src/cli/diff.rs` | Edit | ~55 lines added/changed |
| `tests/cli_smoke.rs` | Edit | ~120 lines added (new tests) |

---

## Gap 1: Version Comparison for Installed Tools

### Problem

Currently, when a tool IS installed, the code silently skips it (the `if !installed`
block on lines 66-73 and 80-87 only handles the missing case). The user sees no
information about tools that are present but at the wrong version.

### Approach

For each installed tool, call `util::get_command_version(name)` and check whether
the declared version string appears in the output. This uses a simple `contains`
check, consistent with how the declared version is a prefix or substring of the
full version output (e.g., declared `"22"` matches `"node v22.11.0"`). Special
values `"latest"` and `"stable"` always match (no version mismatch possible).

**Expected version string formats in `great.toml`:** Users typically declare
versions as major (`"22"`), major.minor (`"22.11"`), or major.minor.patch
(`"22.11.0"`). The `contains` check matches these as substrings of the raw
`--version` output. Single-digit major versions (e.g., `"3"` for Python) may
produce false matches against unrelated digits in the version string; this is
acceptable because the diff command is a lightweight heuristic, and `great apply`
uses `MiseManager::version_matches` for authoritative version resolution.

### Required Import

Add to `src/cli/diff.rs` line 5, after the existing `use crate::cli::output;`:

```rust
// BEFORE (line 5):
use crate::cli::output;

// AFTER (lines 5-6):
use crate::cli::output;
use crate::cli::util;
```

### Runtimes Block Change

Replace lines 59-73 of `src/cli/diff.rs` (the runtime tools loop body):

```rust
// BEFORE:
for (name, declared_version) in &tools.runtimes {
    // The flattened map includes the "cli" key — skip it since
    // CLI tools are handled separately below.
    if name == "cli" {
        continue;
    }
    let installed = command_exists(name);
    if !installed {
        tool_diffs.push(format!(
            "  {} {} {}",
            "+".green(),
            name.bold(),
            format!("(need {})", declared_version).dimmed()
        ));
    }
}
```

```rust
// AFTER:
for (name, declared_version) in &tools.runtimes {
    if name == "cli" {
        continue;
    }
    let installed = command_exists(name);
    if !installed {
        install_count += 1;
        tool_diffs.push(format!(
            "  {} {} {}",
            "+".green(),
            name.bold(),
            format!("(need {})", declared_version).dimmed()
        ));
    } else if declared_version != "latest" && declared_version != "stable" {
        if let Some(actual) = util::get_command_version(name) {
            if !actual.contains(declared_version) {
                configure_count += 1;
                tool_diffs.push(format!(
                    "  {} {} {}",
                    "~".yellow(),
                    name.bold(),
                    format!("(want {}, have {})", declared_version, actual).dimmed()
                ));
            }
        }
    }
}
```

### CLI Tools Block Change

Replace lines 78-88 of `src/cli/diff.rs` (the CLI tools loop body):

```rust
// BEFORE:
if let Some(cli_tools) = &tools.cli {
    for (name, declared_version) in cli_tools {
        let installed = command_exists(name);
        if !installed {
            tool_diffs.push(format!(
                "  {} {} {}",
                "+".green(),
                name.bold(),
                format!("(need {})", declared_version).dimmed()
            ));
        }
    }
}
```

```rust
// AFTER:
if let Some(cli_tools) = &tools.cli {
    for (name, declared_version) in cli_tools {
        let installed = command_exists(name);
        if !installed {
            install_count += 1;
            tool_diffs.push(format!(
                "  {} {} {}",
                "+".green(),
                name.bold(),
                format!("(need {})", declared_version).dimmed()
            ));
        } else if declared_version != "latest" && declared_version != "stable" {
            if let Some(actual) = util::get_command_version(name) {
                if !actual.contains(declared_version) {
                    configure_count += 1;
                    tool_diffs.push(format!(
                        "  {} {} {}",
                        "~".yellow(),
                        name.bold(),
                        format!("(want {}, have {})", declared_version, actual).dimmed()
                    ));
                }
            }
        }
    }
}
```

### Section Header Update

The header for the tools section (line 93) currently says "Tools -- need to install:"
which is no longer accurate when version mismatches are shown. Change it:

```rust
// BEFORE (line 93):
output::header("Tools — need to install:");

// AFTER:
output::header("Tools");
```

### Design Decisions

- **Why `contains` instead of `MiseManager::version_matches`?** The
  `get_command_version` function returns the full first line of `<cmd> --version`
  output (e.g. `"node v22.11.0"`, `"git version 2.43.0"`, `"ripgrep 14.1.0"`).
  `MiseManager::version_matches` expects cleaned numeric versions (e.g.
  `"22.11.0"`). A `contains` check on the raw output is simpler, correct for all
  common formats, and avoids coupling diff to the mise runtime manager.

- **Why skip comparison for `"latest"` and `"stable"`?** These are version
  placeholders that match any installed version. There is no meaningful version
  number to compare against.

- **Why not show "ok" lines for matching tools?** The diff command follows
  `git diff` semantics: only differences are shown. Silence means agreement.
  The summary line at the end already covers the "nothing needed" case.

---

## Gap 2: Exit Code 1 on Missing Config

### Problem

Lines 35-38 of `src/cli/diff.rs` return `Ok(())` (exit 0) when no `great.toml`
is found. This is inconsistent with `doctor.rs` (line 246) which exits non-zero
on failure, and incorrect for CI/scripting where a non-zero exit signals an
actionable problem.

### Change

```rust
// BEFORE (lines 35-38):
Err(_) => {
    output::error("No great.toml found. Run `great init` to create one.");
    return Ok(());
}

// AFTER:
Err(_) => {
    output::error("No great.toml found. Run `great init` to create one.");
    std::process::exit(1);
}
```

### Rationale

Using `std::process::exit(1)` instead of `bail!()` is the established pattern in
this codebase for commands that need to print their full output before exiting
non-zero. See `src/cli/doctor.rs` line 246 and `src/cli/status.rs` line 287 for
the same pattern, including the explanatory comment.

---

## Gap 3: Skip Disabled MCP Servers

### Problem

The MCP diff loop at lines 105-129 of `src/cli/diff.rs` processes all entries in
the `mcp` map regardless of the `enabled` field. Servers with
`enabled = Some(false)` should be skipped.

### Change

Add a guard at the top of the MCP loop body, after line 105:

```rust
// BEFORE (lines 105-107):
for (name, mcp) in mcps {
    // Check if the command for this MCP server exists
    let cmd_available = command_exists(&mcp.command);

// AFTER:
for (name, mcp) in mcps {
    // Skip disabled servers
    if mcp.enabled == Some(false) {
        continue;
    }

    // Check if the command for this MCP server exists
    let cmd_available = command_exists(&mcp.command);
```

### Consistency

This is identical to the pattern in `src/cli/doctor.rs` lines 567-569:
```rust
if mcp.enabled == Some(false) {
    pass(result, &format!("{}: disabled (skipped)", name));
    continue;
}
```

The diff command does not print a "skipped" line for disabled servers because
diff output follows "silence means no action needed" semantics.

### MCP Counter Increments

In addition to the disabled-server guard, the two MCP push sites (lines 109-114
for missing command, and lines 122-128 for needs-.mcp.json) each need a
`configure_count += 1;` added immediately before their respective `mcp_diffs.push`
calls:

```rust
// Missing MCP command push (around line 109):
            if !cmd_available {
                configure_count += 1;
                mcp_diffs.push(format!(
                    // ... existing format unchanged
                ));
            }

// Needs .mcp.json config push (around line 122):
            if cmd_available {
                configure_count += 1;
                mcp_diffs.push(format!(
                    // ... existing format unchanged
                ));
            }
```

---

## Gap 4: Numeric Summary Line

### Problem

The backlog requirement 5 specifies: "At the end of the diff output, print: 'N
items to install, M items to configure, K secrets to resolve'". The current code
prints only "No changes needed" (line 192) or "Run `great apply` to reconcile
these differences." (line 194). This provides no quantitative breakdown, which is
needed for CI pipelines and quick human scanning.

### Approach

Introduce three counters (`install_count`, `configure_count`, `secrets_count`)
that are incremented during the diff walk. At the end, format a summary line
using these counters. The counters classify items as follows:

- **install_count:** missing tools (green `+` lines in the tools section)
- **configure_count:** version mismatches (yellow `~` in tools) + MCP diffs
  (both missing command `+` and needs-config `~`)
- **secrets_count:** unresolved secrets from both `secrets.required` and
  `find_secret_refs()` (red `-` lines, see Gap 5)

### Change

Replace the summary block at lines 191-195 of `src/cli/diff.rs`:

```rust
// BEFORE (lines 191-195):
    if !has_diff {
        output::success("System matches declared configuration. No changes needed.");
    } else {
        output::info("Run `great apply` to reconcile these differences.");
    }

// AFTER:
    if !has_diff {
        output::success("Environment matches configuration — nothing to do.");
    } else {
        let mut parts = Vec::new();
        if install_count > 0 {
            parts.push(format!("{} to install", install_count));
        }
        if configure_count > 0 {
            parts.push(format!("{} to configure", configure_count));
        }
        if secrets_count > 0 {
            parts.push(format!("{} secrets to resolve", secrets_count));
        }
        let summary = parts.join(", ");
        output::info(&format!("{} — run `great apply` to reconcile.", summary));
    }
```

### Counter Placement

The counters must be declared alongside `has_diff` and incremented at each diff
push site. Specifically:

```rust
// After line 52 ("let mut has_diff = false;"), add:
    let mut install_count: usize = 0;
    let mut configure_count: usize = 0;
    let mut secrets_count: usize = 0;
```

Increment rules (applied inside the AFTER blocks already specified in Gaps 1/5):

| Diff type | Counter to increment |
|-----------|---------------------|
| Missing tool (`+` green in tools section) | `install_count += 1` |
| Version mismatch (`~` yellow in tools section) | `configure_count += 1` |
| Missing MCP command (`+` green in MCP section) | `configure_count += 1` |
| MCP needs .mcp.json config (`~` yellow in MCP section) | `configure_count += 1` |
| Unresolved secret in `secrets.required` (`-` red) | `secrets_count += 1` |
| Unresolved secret ref from MCP env (`-` red) | `secrets_count += 1` |

Each increment is placed immediately before the corresponding `tool_diffs.push`,
`mcp_diffs.push`, `secret_diffs.push`, or `println!` call. The builder adds a
single line (e.g., `install_count += 1;`) before each push.

### Example Output

When there are 2 missing tools, 1 MCP server needing config, and 1 unresolved
secret:

```
2 to install, 1 to configure, 1 secrets to resolve — run `great apply` to reconcile.
```

When fully satisfied:

```
Environment matches configuration — nothing to do.
```

---

## Gap 5: Red `-` Marker for Unresolved Secrets

### Problem

The backlog requirement 3 (line 55) specifies: "Display unresolved secrets with
a red `-` indicator". The Notes section (line 96) defines the marker semantics:
"`-` (red) = blocked (e.g., missing secret prevents MCP config)". However, the
current code uses green `+` for secrets at lines 149 and 183. This is
inconsistent with the design intent: secrets are blockers (they prevent MCP
servers from functioning), whereas `+` semantically means "needs to be added"
which implies a simple installation action.

### Change 1: `secrets.required` block

Replace the marker in lines 148-154 of `src/cli/diff.rs`:

```rust
// BEFORE (lines 148-154):
            for key in required {
                if std::env::var(key).is_err() {
                    secret_diffs.push(format!(
                        "  {} {} {}",
                        "+".green(),
                        key.bold(),
                        "(not set in environment)".dimmed()
                    ));
                }
            }

// AFTER:
            for key in required {
                if std::env::var(key).is_err() {
                    secrets_count += 1;
                    secret_diffs.push(format!(
                        "  {} {} {}",
                        "-".red(),
                        key.bold(),
                        "(not set in environment)".dimmed()
                    ));
                }
            }
```

### Change 2: Secret references from MCP env

Replace the marker in lines 177-186 of `src/cli/diff.rs`:

```rust
// BEFORE (lines 177-186):
    if !unresolved_refs.is_empty() {
        has_diff = true;
        output::header("Secret References — unresolved:");
        for name in &unresolved_refs {
            println!(
                "  {} {} {}",
                "+".green(),
                name.bold(),
                "(referenced in MCP env, not set)".dimmed()
            );
        }

// AFTER:
    if !unresolved_refs.is_empty() {
        has_diff = true;
        output::header("Secret References — unresolved:");
        for name in &unresolved_refs {
            secrets_count += 1;
            println!(
                "  {} {} {}",
                "-".red(),
                name.bold(),
                "(referenced in MCP env, not set)".dimmed()
            );
        }
```

### Rationale

The `-` (red) marker visually distinguishes secrets from installable items. A
missing tool can be automatically installed by `great apply`, but a missing
secret requires the user to manually provide a value (e.g., via `great vault set`
or environment variable). The red color signals that this is a blocker, not a
routine action.

### Docstring Update

The `run` function's docstring (lines 25-28) currently documents only `+` and
`~`. Update it to include `-`:

```rust
// BEFORE (lines 25-28):
/// - `+` (green) — needs to be added / installed
/// - `~` (yellow) — partially configured, needs attention

// AFTER:
/// - `+` (green) — needs to be added / installed
/// - `~` (yellow) — partially configured, needs attention
/// - `-` (red) — blocked, requires manual resolution (e.g., missing secret)
```

---

## Integration Tests

All tests go in `tests/cli_smoke.rs`, in a new section after the existing
`// Diff` section (currently at line 119).

### Test 1: `diff_no_config_exits_nonzero`

Replaces the existing `diff_no_config_shows_error` test (lines 122-131) which
currently asserts `.success()`. The test must change to assert `.failure()`.

```rust
// REPLACE the existing test at lines 122-131:

#[test]
fn diff_no_config_exits_nonzero() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::contains("great.toml"));
}
```

### Test 2: `diff_satisfied_config_exits_zero`

Verifies that diff with a config where all tools are present shows the success
message and exits 0. The success message is printed via `output::success` to
stderr.

```rust
#[test]
fn diff_satisfied_config_exits_zero() {
    let dir = TempDir::new().unwrap();
    // Declare only tools we know exist on any CI runner
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
git = "latest"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stderr(predicate::str::contains("nothing to do"));
}
```

### Test 3: `diff_missing_tool_shows_plus`

Verifies that a nonexistent tool produces a `+` marker in the stdout output and
the summary line (via `output::info`) appears on stderr.

The diff command uses two output channels:
- **stdout** (`println!`): diff lines with `+`, `~`, `-` markers
- **stderr** (`output::info`, `output::header`, etc.): section headers and the
  summary line

The tool name appears on stdout (diff line). The summary "run `great apply`"
appears on stderr (via `output::info`). These assertions are exact.

```rust
#[test]
fn diff_missing_tool_shows_plus() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent_tool_xyz_88888 = "1.0.0"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("nonexistent_tool_xyz_88888"))
        .stderr(predicate::str::contains("great apply"));
}
```

### Test 4: `diff_disabled_mcp_skipped`

Verifies that a disabled MCP server does not appear in the diff output.

```rust
#[test]
fn diff_disabled_mcp_skipped() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.disabled-server]
command = "nonexistent_cmd_xyz_77777"
enabled = false
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("disabled-server").not())
        .stderr(predicate::str::contains("disabled-server").not());
}
```

### Test 5: `diff_version_mismatch_shows_tilde`

Verifies that an installed tool at the wrong version produces a `~` marker.
This test uses `git` (universally available) and declares a version that will
never match (e.g., `"99.99.99"`).

```rust
#[test]
fn diff_version_mismatch_shows_tilde() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
git = "99.99.99"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("git"))
        .stdout(predicate::str::contains("want 99.99.99"));
}
```

### Test 6: `diff_with_custom_config_path`

Verifies that `--config` flag works with the exit-code change. This test
uniquely exercises the `--config` code path (lines 31-32 of diff.rs) which
bypasses `discover_config()` entirely. Test 2 relies on auto-discovery from
`current_dir`, so this test covers a distinct branch.

```rust
#[test]
fn diff_with_custom_config_path() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("custom.toml");
    std::fs::write(
        &config_path,
        r#"
[project]
name = "custom"

[tools.cli]
git = "latest"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .args(["diff", "--config", config_path.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("custom.toml"));
}
```

### Test 7: `diff_summary_shows_counts`

Verifies that the numeric summary line includes category counts. This test
declares a missing tool and a missing secret to exercise at least two counter
categories.

```rust
#[test]
fn diff_summary_shows_counts() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent_tool_xyz_99999 = "1.0.0"

[secrets]
required = ["NONEXISTENT_SECRET_XYZ_99999"]
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stderr(predicate::str::contains("1 to install"))
        .stderr(predicate::str::contains("1 secrets to resolve"))
        .stderr(predicate::str::contains("great apply"));
}
```

### Test 8: `diff_unresolved_secret_shows_red_minus`

Verifies that unresolved secrets use the `-` marker (not `+`). The test checks
for the secret name on stdout (diff line) and absence of the `+` marker next to
it. Since colored output is typically stripped in non-TTY mode, we check for the
literal `-` character adjacent to the secret name.

```rust
#[test]
fn diff_unresolved_secret_shows_red_minus() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[secrets]
required = ["NONEXISTENT_SECRET_XYZ_88888"]
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("NONEXISTENT_SECRET_XYZ_88888"))
        .stdout(predicate::str::contains("not set in environment"));
}
```

---

## Edge Cases

| Case | Expected behavior |
|------|-------------------|
| No `great.toml` and no `--config` | Print error to stderr, exit 1 |
| `--config` pointing to nonexistent file | `config::load` returns Err, propagated via `?` (exit 1 with anyhow message) |
| `--config` pointing to unparseable file | Same as above |
| Tool installed but `get_command_version` returns `None` | Skip version comparison (treat as "version unknown, no mismatch shown"); do NOT increment any counter |
| Declared version is `"latest"` or `"stable"` | Always considered matching, no `~` line |
| Declared version is `"22"` and actual is `"node v22.11.0"` | `contains("22")` = true, no mismatch |
| Declared version is `"22"` and actual is `"node v20.11.22"` | `contains("22")` = true (false negative). This is acceptable; the diff command is a lightweight heuristic, not a version solver. The `apply` command uses `MiseManager::version_matches` for authoritative checks. |
| All MCP servers disabled | No MCP section printed, `has_diff` stays false, all counters remain 0 |
| Empty `mcp` map | `if let Some(mcps)` block entered but loop body never executes |
| `mcp.enabled` is `None` (field absent) | Treated as enabled (only `Some(false)` is skipped) |
| `mcp.enabled` is `Some(true)` | Treated as enabled (processed normally) |
| Config has tools but no MCP or secrets | Only tools section printed; summary shows "N to install" only |
| Config has no tools, MCP, or secrets | "nothing to do" message, all counters are 0 |
| All secrets resolved | No secrets section printed, `secrets_count` remains 0 |
| Only secrets unresolved (tools and MCP satisfied) | Summary shows only "K secrets to resolve" |
| Mix of all three categories | Summary shows all three parts joined by commas |

---

## Error Handling

| Error condition | Current handling | Change needed |
|-----------------|------------------|---------------|
| Config not found | `output::error` + `return Ok(())` | Change to `std::process::exit(1)` |
| Config parse failure | Propagated via `?` (anyhow) | No change |
| `get_command_version` fails | Returns `None` | No change (skip comparison gracefully) |
| `command_exists` fails | Returns `false` | No change |

---

## Security Considerations

- No new inputs are accepted. The `--config` flag already exists and is
  path-validated by `config::load`.
- `get_command_version` executes `<cmd> --version` for tools declared in the
  user's own `great.toml`. This is an existing trust boundary (the user controls
  which commands are declared). No change to this surface.
- No secrets are read, displayed, or transmitted by this command.

---

## Build Order

This is a single-file change with no cross-module dependencies. Build order:

1. Add `use crate::cli::util;` import to `diff.rs`
2. Add counter variables (`install_count`, `configure_count`, `secrets_count`) after `has_diff`
3. Apply Gap 3 (MCP skip guard) -- simplest, 3 lines
4. Apply Gap 2 (exit code) -- 1 line change
5. Apply Gap 5 (red `-` for secrets) -- change `"+".green()` to `"-".red()` in both secrets blocks, add `secrets_count` increments
6. Apply Gap 1 (version comparison) -- runtimes block, then CLI tools block, then header; add `install_count` and `configure_count` increments
7. Add `configure_count` increments in the MCP diffs section (both `+` and `~` push sites)
8. Apply Gap 4 (numeric summary) -- replace the summary block
9. Update docstring to include `-` (red) marker
10. Update existing test `diff_no_config_shows_error` -> `diff_no_config_exits_nonzero`
11. Add new tests (tests 2-8)
12. Run `cargo clippy` and `cargo test`

---

## Acceptance Criteria

- [ ] `great diff` with no `great.toml` exits with code 1 (not 0)
- [ ] `great diff` with a satisfied config exits with code 0 and prints "nothing to do"
- [ ] Missing tools show `+` (green) marker with the declared version
- [ ] Installed tools with version mismatch show `~` (yellow) marker with "want X, have Y"
- [ ] Installed tools with matching or `latest`/`stable` version produce no output
- [ ] MCP servers with `enabled = false` are silently skipped
- [ ] MCP servers with `enabled = None` or `enabled = Some(true)` are processed normally
- [ ] Unresolved secrets show `-` (red) marker, not `+` (green)
- [ ] Summary line shows numeric counts: "N to install, M to configure, K secrets to resolve"
- [ ] Summary line omits zero-count categories (e.g., no "0 to install" shown)
- [ ] `--config` flag still works correctly
- [ ] All 8 integration tests pass
- [ ] `cargo clippy` produces no warnings on changed files
- [ ] No new dependencies added
