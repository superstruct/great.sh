# Socrates Review: Spec 0006 -- `great diff` Gap Completion

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-25
**Spec:** `.tasks/ready/0006-diff-gaps-spec.md`
**Backlog:** `.tasks/backlog/0006-diff-command.md`
**Round:** 2

---

## VERDICT: APPROVED

---

## Round 1 Blocking Concerns -- Resolution Status

### 1. Missing Requirement: Numeric Summary Line -- RESOLVED

Gap 4 (spec lines 325-418) now introduces three counters (`install_count`,
`configure_count`, `secrets_count`), declares them after `has_diff` at line 52,
specifies increment rules at each push site, and formats the summary line as
`"N to install, M to configure, K secrets to resolve -- run great apply to
reconcile."` with zero-count categories omitted. This matches the backlog
requirement 5 (line 64): "N items to install, M items to configure, K secrets to
resolve". The counter-to-category mapping table (spec lines 393-399) is clear and
unambiguous. Test 7 (`diff_summary_shows_counts`) exercises two counter categories.

### 2. Test 3 stdout/stderr Contradiction -- RESOLVED

Test 3 (`diff_missing_tool_shows_plus`, spec lines 594-617) now correctly asserts:
- `.stdout(predicate::str::contains("nonexistent_tool_xyz_88888"))` -- diff line via `println!`
- `.stderr(predicate::str::contains("great apply"))` -- summary via `output::info` (eprintln)

The spec also adds a clear channel documentation block (lines 586-589) explaining
that diff lines go to stdout and section headers/summary go to stderr. No hedging
language remains.

### 3. Dropped Backlog Requirement: Red `-` Marker for Secrets -- RESOLVED

Gap 5 (spec lines 422-520) changes `"+".green()` to `"-".red()` in both the
`secrets.required` block (BEFORE matches `src/cli/diff.rs` lines 148-154 exactly)
and the MCP env secret references block (BEFORE matches lines 177-186 exactly).
The docstring update (spec lines 512-519) adds the `-` (red) marker to the marker
legend. Test 8 (`diff_unresolved_secret_shows_red_minus`) verifies the secret name
appears on stdout.

---

## Round 1 Advisory Concerns -- Resolution Status

### 4. False-Negative Risk in `contains()` Version Matching -- RESOLVED

The spec now documents expected version string formats (lines 63-69), explicitly
lists the single-digit major version false-match risk, and explains that the diff
command is a lightweight heuristic while `great apply` uses `MiseManager::version_matches`
for authoritative resolution.

### 5. Spec Line Number References Will Drift -- RESOLVED

The spec adds an explicit convention note (lines 30-34): all line numbers reference
the ORIGINAL file before any modifications, and builders should use BEFORE code
snippets as anchors, not line numbers.

### 6. Test 6 Unique Value -- RESOLVED

The spec now documents (lines 686-688) that Test 6 exercises the `--config` code
path (lines 31-32 of diff.rs) which bypasses `discover_config()` entirely, making
it distinct from Test 2 which relies on auto-discovery.

### 7. `std::process::exit(1)` Skips Destructors -- NO CHANGE NEEDED

Remains advisory. Consistent with established codebase patterns. No resources are
held at the early exit point.

---

## Round 2: New Issue Check

### Cross-Verification: BEFORE Snippet Accuracy (Round 2)

All BEFORE snippets in the revised spec were verified against the actual source at
`/home/isaac/src/sh.great/src/cli/diff.rs` (198 lines, commit `4addad2`):

| Spec BEFORE | Actual source | Match? |
|---|---|---|
| Gap 1 runtimes (lines 59-73) | diff.rs lines 59-74 | Content exact |
| Gap 1 CLI tools (lines 78-88) | diff.rs lines 77-89 | Content exact |
| Gap 1 header (line 93) | diff.rs line 93 | Exact |
| Gap 2 error path (lines 35-38) | diff.rs lines 35-38 | Exact |
| Gap 3 MCP loop (lines 105-107) | diff.rs lines 105-107 | Exact |
| Gap 4 summary (lines 191-195) | diff.rs lines 191-195 | Exact |
| Gap 5 secrets.required (lines 148-154) | diff.rs lines 147-155 | Content exact |
| Gap 5 MCP env refs (lines 177-186) | diff.rs lines 177-186 | Exact |
| Docstring (lines 25-28) | diff.rs lines 27-28 | Exact |

### Import Verification

`use crate::cli::util;` -- confirmed: `util.rs` exists at `/home/isaac/src/sh.great/src/cli/util.rs`
with `pub fn get_command_version(cmd: &str) -> Option<String>` at line 9. The call
`util::get_command_version(name)` in the spec matches this signature (name is `&String`,
which coerces to `&str`).

### Test Assertion Channel Verification

All 8 tests use the correct output channels:

| Test | stdout assertions | stderr assertions | Correct? |
|---|---|---|---|
| 1: no config exits nonzero | (none) | "great.toml" | Yes (output::error -> eprintln) |
| 2: satisfied exits zero | (none) | "nothing to do" | Yes (output::success -> eprintln) |
| 3: missing tool shows plus | tool name | "great apply" | Yes (println vs output::info) |
| 4: disabled MCP skipped | .not() disabled-server | .not() disabled-server | Yes |
| 5: version mismatch | "git", "want 99.99.99" | (none) | Yes (println for diff lines) |
| 6: custom config path | (none) | "custom.toml" | Yes (output::info -> eprintln) |
| 7: summary counts | (none) | "1 to install", "1 secrets to resolve", "great apply" | Yes (output::info -> eprintln) |
| 8: secret red minus | secret name, "not set" | (none) | Yes (println for diff lines) |

### Potential Concern: MCP `.mcp.json` Check Uses Relative Path

```
gap: The existing MCP diff code at line 118 checks `std::path::Path::new(".mcp.json")`
     which resolves relative to the process's cwd, not relative to the config file's
     directory. If the user runs `great diff --config /some/other/dir/great.toml`
     from a different directory, the `.mcp.json` check will look in the wrong
     location. However, this is a pre-existing issue in the current code, not
     introduced by this spec.

severity: ADVISORY

recommendation: No action for this spec. This is a pre-existing limitation noted
               in the backlog (line 97-98: "The .mcp.json check uses a path
               relative to cwd, which is correct: this file lives in the project
               root alongside great.toml"). The backlog considers this intentional.
```

---

## Summary

All three Round 1 blocking concerns are fully resolved. The revised spec adds Gap 4 (numeric summary with counters), Gap 5 (red `-` markers for secrets), and corrects Test 3's output channel assertions. All BEFORE snippets match the actual source code. All test assertions target the correct output channels. The spec is implementable without further clarifying questions. Approved for handoff to the builder.
