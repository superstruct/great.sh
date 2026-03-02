# 0031 -- Socrates Review: Loop and MCP Bridge Smoke Tests

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Spec:** `/home/isaac/src/sh.great/.tasks/ready/0031-loop-mcp-bridge-tests-spec.md`
**Backlog:** `/home/isaac/src/sh.great/.tasks/backlog/0031-loop-and-mcp-bridge-smoke-tests.md`
**Date:** 2026-02-28

---

## VERDICT: APPROVED

---

## Verification Summary

**Assertions verified against source code:**

| Test | Assertion | Source verification | Correct? |
|------|-----------|-------------------|----------|
| 1: `loop_help_shows_subcommands` | stdout contains "install" and "status" | `LoopCommand` enum at `loop_cmd.rs:19-34` has `Install`, `Status`, `Uninstall` variants; clap 4 renders these as lowercase in help | Yes |
| 2: `loop_status_fresh_home_reports_not_installed` | exit 0, stderr contains "not installed" | `run_status()` at `loop_cmd.rs:557-655` always returns `Ok(())`; uses `output::error("Agent personas: not installed")` at line 569 (stderr via eprintln) | Yes |
| 3: `loop_uninstall_fresh_home_is_noop` | exit 0 | `run_uninstall()` at `loop_cmd.rs:658-712` guards all deletes with `.exists()` checks, returns `Ok(())` | Yes |
| 4: `mcp_bridge_unknown_preset_shows_error_message` | failure, stderr contains "invalid preset" | `mcp_bridge.rs:105-108` uses `anyhow::Context` with string `"invalid preset '...' — use: ..."` | Yes |
| 5: `loop_install_force_writes_hook_script` | `$HOME/.claude/hooks/great-loop/update-state.sh` exists | `run_install()` writes at `loop_cmd.rs:354-355` | Yes |
| 6: `loop_install_force_writes_settings_json` | settings.json contains "hooks" and "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS" | Created at `loop_cmd.rs:465-495` with both keys in the `json!` literal | Yes |

**TempDir isolation:** `dirs::home_dir()` reads `$HOME` on both Linux and macOS. `.env("HOME", dir.path())` is the established pattern (4 existing tests use it). `TempDir` auto-cleans on drop. No leaks.

**Existing coverage table:** Spec claims 6 existing tests. Verified all 6 exist at lines 1474, 1483, 1497, 1518, 1878, 1887 of `cli_smoke.rs`.

**Current test count:** 92 existing `#[test]` functions in `cli_smoke.rs`. Spec adds 6 for total of 98.

---

## Concerns

```
{
  "gap": "Backlog AC says 'stdout contains Install and Status' (capitalized); spec asserts lowercase 'install' and 'status'",
  "question": "Does clap 4 help output show the capitalized doc-comment or the lowercase variant name for subcommand listings?",
  "severity": "ADVISORY",
  "recommendation": "Clap 4 renders subcommand names from the variant name (lowercased), not the doc-comment. The spec's lowercase assertion is correct. The backlog AC is slightly imprecise but the substring 'install' also matches 'Install' since the description line starts with 'Install the great.sh'. Either way, the test passes. No action needed."
}
```

```
{
  "gap": "Spec comment at line 130 shows error message with double-hyphen '--' but actual code at mcp_bridge.rs:106 uses em dash (U+2014)",
  "question": "Could the comment mislead the builder into asserting for '--' instead of the em dash?",
  "severity": "ADVISORY",
  "recommendation": "The test assertion only checks for 'invalid preset' (no dash), so the comment is cosmetically wrong but the test is correct. Builder should not change the assertion based on the comment."
}
```

```
{
  "gap": "Tests 5 and 6 each spawn a full `loop install --force` independently -- two separate TempDirs, two binary invocations",
  "question": "Should these be combined into one test to halve the install invocations, or is the isolation more valuable?",
  "severity": "ADVISORY",
  "recommendation": "Spec explicitly allows builder to combine or keep separate. Both approaches are valid for smoke tests. The 2-second estimated overhead is negligible."
}
```

---

## Summary

Clean, well-scoped spec. All 6 test assertions are verified correct against the actual source code. TempDir isolation follows the established pattern. No BLOCKING concerns.
