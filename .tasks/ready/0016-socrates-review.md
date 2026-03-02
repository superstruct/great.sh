# Spec 0016 Review: `great loop install` -- Overwrite Safety and `--force` Flag

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-24
**Spec:** `/home/isaac/src/sh.great/.tasks/ready/0016-overwrite-safety-spec.md`
**Source:** `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`

---

## VERDICT: APPROVED

---

## Concerns

### 1. stderr flush before blocking stdin read

```
{
  "gap": "The `eprint!(\"Overwrite? [y/N] \")` call does not explicitly flush stderr before calling `read_line`. On some platforms and terminal emulators, stderr may be fully buffered when redirected, meaning the prompt could appear after the user has already typed input -- or not at all.",
  "question": "Should the spec require an explicit `std::io::stderr().flush()?` call after the `eprint!` and before `read_line`?",
  "severity": "ADVISORY",
  "recommendation": "Add `use std::io::Write; std::io::stderr().flush().context(\"failed to flush stderr\")?;` after the `eprint!` line. This is a defensive best practice. In the specific flow here, we already confirmed stdin IS a TTY, and stderr to a TTY is typically unbuffered on Unix, so this is unlikely to cause real problems -- but explicit flushing costs nothing and eliminates the ambiguity."
}
```

### 2. `create_dir_all` executes before the overwrite check

```
{
  "gap": "The spec places the overwrite check AFTER directory creation (`create_dir_all` for agents/, commands/, teams/loop/). If the user declines the overwrite, no files are written, but the directories may have been freshly created as a side effect. The bail message says 'no files were modified' which is technically true (directories are not files), but new directories may have been created.",
  "question": "Is creating directories before the abort check acceptable, or should the directory creation also be gated behind the overwrite check? Could this leave empty directory scaffolding if the user declines on a partial install?",
  "severity": "ADVISORY",
  "recommendation": "This is acceptable. The directories are harmless (empty directories have no semantic meaning), `collect_existing_paths` needs the directories to exist for `Path::exists()` to work correctly, and `create_dir_all` is idempotent. The spec correctly notes this ordering is intentional. No change needed, but the spec could add an explicit note that directory creation is intentionally not gated."
}
```

### 3. Stale comment on line 104: "All 4 slash-command files" vs actual 5

```
{
  "gap": "Line 104 of loop_cmd.rs says `/// All 4 slash-command files shipped with the great.sh Loop.` but the COMMANDS array contains 5 entries (loop, bugfix, deploy, discover, backlog). This is a pre-existing bug, not introduced by this spec, but the spec's test `test_collect_existing_paths_full_install` asserts `existing.len() == 21` which depends on the 5-command count being correct.",
  "question": "Should this spec include a fix for the stale comment on line 104 while it is in the area, or is that a separate task?",
  "severity": "ADVISORY",
  "recommendation": "The builder should fix the comment to say '5' while making changes in this file. It is a one-line change in the same file and prevents future confusion. If the spec author prefers not to, the test assertion of 21 (15 + 5 + 1) still correctly validates the actual count."
}
```

### 4. `confirm_overwrite` display path uses separate `dirs::home_dir()` call

```
{
  "gap": "The `confirm_overwrite` function calls `dirs::home_dir()` independently to compute the `~/` display prefix, while `run_install` calls it separately to compute `claude_dir`. These two calls will return the same value in practice (both read $HOME), but the spec does not pass the already-resolved home path into `confirm_overwrite`.",
  "question": "On Linux, `dirs::home_dir()` reads $HOME, so the integration tests that override HOME via `.env(\"HOME\", dir.path())` will get consistent behavior between `run_install` and `confirm_overwrite`. But on macOS, `dirs::home_dir()` can fall back to `getpwuid_r` if HOME is unset. Is there a scenario where the two calls return different values, causing the `strip_prefix` to fail and display absolute paths instead of `~/` paths?",
  "severity": "ADVISORY",
  "recommendation": "The current approach is fine for all realistic scenarios. The fallback (displaying absolute paths) is correct, just less pretty. If the spec author wants perfect consistency, `confirm_overwrite` could accept a `home: &Path` parameter. But this is cosmetic -- it affects only the display of paths in the prompt, not correctness."
}
```

### 5. Integration test `loop_install_force_overwrites_existing` asserts on stderr

```
{
  "gap": "The integration test `loop_install_force_overwrites_existing` asserts `.stderr(predicate::str::contains(\"--force: overwriting existing files\"))`. Looking at the spec's Change 6, the message is printed via `output::info(\"(--force: overwriting existing files)\")`. The assertion string does NOT include the parentheses. Need to verify that `output::info` wraps the message in a way that the substring match still works.",
  "question": "Does `output::info` pass the string through verbatim (preserving the parentheses), or does it strip/transform it? The assertion `contains(\"--force: overwriting existing files\")` would match `(--force: overwriting existing files)` since it is a substring. Is this intentionally testing a substring to be resilient to formatting changes?",
  "severity": "ADVISORY",
  "recommendation": "The substring match is correct -- `\"--force: overwriting existing files\"` is contained within `\"(--force: overwriting existing files)\"`. No change needed. The test is deliberately resilient to prefix/suffix formatting."
}
```

### 6. No test for `--force --project` combined flags

```
{
  "gap": "The edge case table lists `--force --project` as a scenario, but neither the unit tests nor the integration tests exercise this combination. The `--project` flag writes to `.tasks/` in the current working directory, which is a different path from the `--force`-governed `~/.claude/` files.",
  "question": "Should there be an integration test that verifies `great loop install --force --project` succeeds and writes both `~/.claude/` files and `.tasks/` directory?",
  "severity": "ADVISORY",
  "recommendation": "Add one integration test that exercises the combined path. This is low risk since the two flags are orthogonal, but the edge case table explicitly calls it out, so a test would provide evidence for the claim."
}
```

### 7. `bail!` import -- spec is ambiguous about which approach to use

```
{
  "gap": "Change 1 says to add `bail` to the `anyhow` import, then also mentions `use std::io::IsTerminal;` but says 'The spec uses the trait import approach for clarity.' However, Change 5 shows `use std::io::IsTerminal;` inside the function body (scoped import). The `bail!` import is added at the top level. These are both fine, but the spec says two different things about IsTerminal in Change 1 vs Change 5.",
  "question": "Is the builder expected to add `use std::io::IsTerminal;` at the module level (Change 1) OR use a scoped import inside `confirm_overwrite` (Change 5)? The spec shows both options without a clear directive.",
  "severity": "ADVISORY",
  "recommendation": "The code in Change 5 is the authoritative version since it is the exact code to insert. The builder should use the scoped import as shown in Change 5. Change 1's mention of IsTerminal is informational/alternative. The spec could be clearer, but the builder can resolve this by following the code blocks."
}
```

---

## Lines of Questioning -- Answered

### Q1: Is the `confirm_overwrite` logic correct? Does TTY detection happen before any stdin read?

**Yes.** The `is_terminal()` check on line 267 of the proposed code occurs before the `read_line` call on line 277-278. The control flow is: print file list -> check TTY -> if not TTY, return `Ok(false)` immediately -> if TTY, show prompt -> read line -> compare. This ordering is correct and prevents consuming piped input that was not intended as a confirmation.

### Q2: Does `collect_existing_paths` cover exactly the right 21 files?

**Yes.** I verified against the source:
- `AGENTS` array: 15 entries (nightingale, lovelace, socrates, humboldt, davinci, vonbraun, turing, kerckhoffs, rams, nielsen, knuth, gutenberg, hopper, dijkstra, wirth)
- `COMMANDS` array: 5 entries (loop, bugfix, deploy, discover, backlog)
- 1 teams config (teams/loop/config.json)
- Total: 21

The function iterates the same `AGENTS` and `COMMANDS` constants used by `run_install`, so the sets are guaranteed to be identical. No managed file can be missed or spuriously included.

### Q3: Is the abort path safe -- no files written before the check?

**Yes.** The overwrite check is inserted after `create_dir_all` (which is idempotent and only creates directories) and before the `// Write agent files` block. If the user declines, `bail!` fires and no `std::fs::write` calls for agent/command/config files have executed. The claim "no files were modified" is accurate.

### Q4: Does `--force` correctly bypass the check in all scenarios?

**Yes.** The guard condition is `!existing.is_empty() && !force`. When `force` is true, this condition is false regardless of `existing`, so `confirm_overwrite` is never called. The `--force` informational message is gated on `force && !existing.is_empty()`, which correctly suppresses it for fresh installs. All five behavioral scenarios in the spec are consistent with this logic.

### Q5: Are the integration tests realistic (fake HOME via env var)?

**Yes.** On Linux, `dirs::home_dir()` reads the `HOME` environment variable. The `assert_cmd` `.env("HOME", dir.path())` sets `HOME` for the child process, so `dirs::home_dir()` resolves to the temp directory. This pattern is already used successfully in the existing test suite (e.g., `status_shows_platform` uses `TempDir`). The integration tests correctly isolate the test from the real `~/.claude/`.

### Q6: Edge case -- `dirs::home_dir()` in display logic vs HOME override?

**Not a problem.** Both `run_install` and `confirm_overwrite` call `dirs::home_dir()`, which reads `HOME` from the environment. Since `HOME` is set to the temp dir for the child process, both calls resolve to the same path. The `strip_prefix` in `confirm_overwrite` will successfully produce `~/` paths. On macOS, `dirs::home_dir()` also respects `HOME` when set. The edge case where `HOME` is unset and `getpwuid_r` is used as fallback is handled by the existing `.context("could not determine home directory")` error in `run_install`, which fires before `confirm_overwrite` is ever reached.

---

## Summary

This is a well-structured spec with correct control flow, accurate file counts, exhaustive edge case coverage, and realistic tests. All 7 concerns are ADVISORY -- no blocking issues found. The spec is implementable as-is without further clarification.
