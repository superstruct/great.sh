# 0026: `great diff` Output Channel Redesign -- Technical Spec

**Task:** 0026 -- `great diff` Output Channel Redesign
**Complexity:** S
**Files:** `src/cli/output.rs`, `src/cli/diff.rs`, `tests/cli_smoke.rs`
**Dependencies:** None
**New crates:** None

---

## 1. Problem Statement

`great diff` produces output on two channels simultaneously:

- **stderr** (via `output::header()`, `output::info()`, `output::success()`): section headers
  ("great diff", "Tools", "MCP Servers", "Secrets"), the config path info line, and the
  final summary ("nothing to do" or "N to install, ...").
- **stdout** (via `println!`): the actual diff lines (`+`, `~`, `-` markers).

This split means:

1. `great diff 2>/dev/null` produces orphaned diff lines with no section context.
2. `great diff 1>/dev/null` produces headers with no content beneath them.
3. `great diff | grep "+"` silently drops all section headers and the summary.
4. CI pipelines that capture stdout get incomplete output.

`diff` is a read-only, pipeline-oriented command. Its entire output is data. It should
write everything to stdout, matching standard Unix `diff` behavior. Only fatal errors
(missing config, parse failure) belong on stderr.

---

## 2. Solution Design

Add three new public functions to `src/cli/output.rs` that mirror the existing stderr
helpers but write to stdout. Update all call sites in `src/cli/diff.rs` to use the new
stdout variants. Leave the single `output::error()` call (missing config) on stderr.
No other command is touched.

### 2.1 Why stdout variants, not a channel parameter

A channel parameter (`fn header(msg, channel)`) or a `Writer` trait would require
touching every existing call site across all commands to pass the default. The additive
approach (new `_stdout` functions) has zero blast radius outside `diff.rs`.

---

## 3. File-by-File Changes

### 3.1 `src/cli/output.rs`

Add three new functions after the existing helpers. Each is identical to its stderr
counterpart except it uses `println!` instead of `eprintln!`.

```rust
/// Print a bold header/section title to stdout.
///
/// Use this in pipeline-oriented commands (e.g., `diff`) where all output
/// is data and belongs on stdout. Interactive commands should use `header()`.
pub fn header_stdout(msg: &str) {
    println!("{}", msg.bold());
}

/// Print an informational message to stdout with a blue info prefix.
///
/// Stdout variant of `info()` for pipeline-oriented commands.
pub fn info_stdout(msg: &str) {
    println!("{} {}", "\u{2139}".blue(), msg);
}

/// Print a success message to stdout with a green checkmark prefix.
///
/// Stdout variant of `success()` for pipeline-oriented commands.
pub fn success_stdout(msg: &str) {
    println!("{} {}", "\u{2713}".green(), msg);
}
```

**Placement:** Insert after line 26 (the closing brace of `header()`), before the
`spinner()` function. This groups all output helpers together, with the stdout variants
immediately following their stderr counterparts' section.

**No other changes to output.rs.** The existing `header`, `info`, `success`, `warning`,
`error` functions remain unchanged.

### 3.2 `src/cli/diff.rs`

Seven call sites change. Each replaces an `output::` stderr call with its `_stdout`
variant. The `output::error` call on line 40 is NOT changed.

| Line | Current Call | Replacement |
|------|-------------|-------------|
| 49 | `output::header("great diff");` | `output::header_stdout("great diff");` |
| 50 | `output::info(&format!("Comparing {} against system state", config_path.display()));` | `output::info_stdout(&format!("Comparing {} against system state", config_path.display()));` |
| 123 | `output::header("Tools");` | `output::header_stdout("Tools");` |
| 170 | `output::header("MCP Servers");` | `output::header_stdout("MCP Servers");` |
| 218 | `output::header("Secrets");` | `output::header_stdout("Secrets");` |
| 226 | `output::success("Environment matches configuration -- nothing to do.");` | `output::success_stdout("Environment matches configuration -- nothing to do.");` |
| 239 | `output::info(&format!("{} -- run \`great apply\` to reconcile.", summary));` | `output::info_stdout(&format!("{} -- run \`great apply\` to reconcile.", summary));` |

**Preserved on stderr (no change):**

| Line | Call | Reason |
|------|------|--------|
| 40 | `output::error("No great.toml found. Run \`great init\` to create one.");` | Fatal error before exit(1); must appear on stderr per Unix convention. |

### 3.3 `tests/cli_smoke.rs`

Nine existing diff tests need assertion channel updates. Tests that currently assert on
`.stderr(predicate::str::contains(...))` for headers and summary must switch to
`.stdout(predicate::str::contains(...))`.

**Test: `diff_no_config_exits_nonzero`** (line 123)
No change. Already asserts on `.stderr(...)` for the error message. This remains correct
because `output::error()` stays on stderr.

**Test: `diff_satisfied_config_exits_zero`** (line 134)
```rust
// BEFORE (line 154):
.stderr(predicate::str::contains("nothing to do"));
// AFTER:
.stdout(predicate::str::contains("nothing to do"));
```

**Test: `diff_missing_tool_shows_plus`** (line 158)
```rust
// BEFORE (line 178):
.stderr(predicate::str::contains("great apply"));
// AFTER:
.stdout(predicate::str::contains("great apply"));
```
The `.stdout(predicate::str::contains("nonexistent_tool_xyz_88888"))` assertion on
line 177 remains unchanged (already on stdout).

**Test: `diff_disabled_mcp_skipped`** (line 182)
No change needed. Both assertions use `.not()` predicates and check both channels.
The disabled server should appear on neither channel -- this remains correct.

**Test: `diff_version_mismatch_shows_tilde`** (line 207)
No change needed. Both assertions are already on `.stdout(...)`.

**Test: `diff_with_custom_config_path`** (line 231)
```rust
// BEFORE (line 251):
.stderr(predicate::str::contains("custom.toml"));
// AFTER:
.stdout(predicate::str::contains("custom.toml"));
```
This assertion checks the info line "Comparing custom.toml against system state"
which moves from stderr to stdout.

**Test: `diff_summary_shows_counts`** (line 255)
```rust
// BEFORE (lines 277-279):
.stderr(predicate::str::contains("1 to install"))
.stderr(predicate::str::contains("1 secrets to resolve"))
.stderr(predicate::str::contains("great apply"));
// AFTER:
.stdout(predicate::str::contains("1 to install"))
.stdout(predicate::str::contains("1 secrets to resolve"))
.stdout(predicate::str::contains("great apply"));
```

**Test: `diff_mcp_missing_command_counted_as_install`** (line 307)
```rust
// BEFORE (lines 326-327):
.stderr(predicate::str::contains("1 to install"))
.stderr(predicate::str::contains("to configure").not())
// AFTER:
.stdout(predicate::str::contains("1 to install"))
.stdout(predicate::str::contains("to configure").not())
```
The `.stdout(...)` assertion on line 328 remains unchanged.

**Test: `diff_mcp_missing_command_and_missing_tool_install_count`** (line 332)
```rust
// BEFORE (line 354):
.stderr(predicate::str::contains("2 to install"));
// AFTER:
.stdout(predicate::str::contains("2 to install"));
```

**Test: `diff_secret_dedup_required_and_ref`** (line 358)
```rust
// BEFORE (lines 381-382):
.stderr(predicate::str::contains("1 secrets to resolve"))
.stderr(predicate::str::contains("2 secrets").not());
// AFTER:
.stdout(predicate::str::contains("1 secrets to resolve"))
.stdout(predicate::str::contains("2 secrets").not());
```

**Test: `diff_secret_ref_only_no_required_section`** (line 386)
```rust
// BEFORE (line 406):
.stderr(predicate::str::contains("1 secrets to resolve"))
// AFTER:
.stdout(predicate::str::contains("1 secrets to resolve"))
```
The `.stdout(...)` assertion on line 407 remains unchanged.

---

## 4. Build Order

This is a single-pass change with no dependency ordering concerns:

1. Add three functions to `src/cli/output.rs`.
2. Update seven call sites in `src/cli/diff.rs`.
3. Update test assertions in `tests/cli_smoke.rs`.
4. Run `cargo clippy` -- expect zero new warnings.
5. Run `cargo test` -- all tests pass.

Steps 1 and 2 can be done in either order (the compiler will catch any mismatch).
Step 3 must follow step 2 or tests will fail.

---

## 5. Edge Cases

### 5.1 Empty diff (no differences)

When the config is fully satisfied, `has_diff` is false. The code reaches line 226
and calls `output::success(...)`. After this change it calls `output::success_stdout(...)`.
This is the only output line produced (besides the header and info line). All three
are now on stdout. `great diff 2>/dev/null` shows the complete "nothing to do" message.

### 5.2 `great diff` with no config file

The `output::error(...)` call on line 40 stays on stderr. `std::process::exit(1)` fires.
No stdout output is produced. This matches Unix convention: errors on stderr, nonzero exit.

### 5.3 Colored output to a pipe

`colored` respects the `NO_COLOR` environment variable and `isatty()`. When stdout is
piped (e.g., `great diff | grep "+"`), ANSI codes may or may not be emitted depending
on the `colored` crate's terminal detection. This is existing behavior and is not
changed by this task. If the user sets `NO_COLOR=1`, all ANSI escapes are suppressed.

### 5.4 Platform differences (macOS ARM64/x86_64, Ubuntu, WSL2)

No platform-specific behavior involved. `println!` and `eprintln!` are standard Rust
macros that behave identically on all platforms. The `colored` crate handles terminal
detection cross-platform.

### 5.5 Concurrent access

Not applicable. `great diff` is a single-threaded, read-only command. No file locks,
no shared state, no network calls.

---

## 6. Error Handling

Only one error path exists in `diff.rs`:

| Condition | Current behavior | After change |
|-----------|-----------------|--------------|
| `config::discover_config()` fails | `output::error(...)` on stderr, `exit(1)` | Unchanged |
| `config::load(...)` returns `Err` | Propagated via `?` to main, anyhow prints to stderr | Unchanged |

No new error paths are introduced. The three new `output.rs` functions cannot fail
(`println!` panics on write failure, same as the existing `eprintln!` calls -- this is
standard Rust behavior for stdout/stderr write errors and is acceptable for a CLI tool).

---

## 7. Security Considerations

None. This change alters which file descriptor receives output. No new inputs are
accepted, no new data is exposed, no secrets are handled differently.

---

## 8. Testing Strategy

### 8.1 Existing tests (updated assertions)

The eleven existing diff tests in `tests/cli_smoke.rs` cover:

- Missing config exits nonzero with error on stderr
- Satisfied config shows "nothing to do"
- Missing tool shows `+` marker
- Disabled MCP server is skipped
- Version mismatch shows `~` marker
- Custom config path
- Summary install/secret counts
- MCP missing command counted as install
- Combined MCP and tool install count
- Secret deduplication (required + ref)
- Secret ref without required section

Nine of these tests have assertions that move from `.stderr(...)` to `.stdout(...)` as
detailed in section 3.3. Two tests (`diff_no_config_exits_nonzero` and
`diff_disabled_mcp_skipped`) remain unchanged.

### 8.2 Verification commands

After the implementation:

```bash
# All tests pass
cargo test

# No new clippy warnings
cargo clippy

# Manual verification: complete output on stdout, nothing on stderr
cargo run -- diff 2>/dev/null       # should show headers + diff lines + summary
cargo run -- diff 1>/dev/null       # should show nothing (no stderr output)

# Manual verification: error still on stderr
cd /tmp && cargo run -- diff 2>&1 1>/dev/null  # should show "No great.toml found"
```

### 8.3 No new test functions required

The existing test suite already covers every code path in `diff.rs`. The only change
is which channel (`stdout` vs `stderr`) the assertions target. No new test functions
are needed for this S-complexity change.

---

## 9. Acceptance Criteria

- [ ] Three new public functions exist in `src/cli/output.rs`: `header_stdout`,
      `info_stdout`, `success_stdout`. Each uses `println!` instead of `eprintln!`.
- [ ] All seven `output::header()`, `output::info()`, and `output::success()` calls
      in `src/cli/diff.rs` are replaced with their `_stdout` variants.
- [ ] The `output::error()` call for missing config (line 40) remains on stderr.
- [ ] `great diff 2>/dev/null` with a valid `great.toml` produces complete, coherent
      output on stdout: section headers, diff lines, and summary are all present.
- [ ] `great diff 1>/dev/null` with a valid `great.toml` produces no output (stderr
      is reserved for fatal errors only; no informational output goes to stderr).
- [ ] `great diff` against a missing config still prints the error on stderr and
      exits with code 1.
- [ ] `cargo clippy` passes with zero new warnings.
- [ ] All eleven diff integration tests in `tests/cli_smoke.rs` pass with updated
      channel assertions (nine tests change from `.stderr(...)` to `.stdout(...)`).
- [ ] No changes to any other command (`status`, `doctor`, `apply`, `init`, etc.).
