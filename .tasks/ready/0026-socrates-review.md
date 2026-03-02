# 0026: `great diff` Output Channel Redesign -- Socrates Review

**Spec:** `.tasks/ready/0026-diff-output-channel-spec.md`
**Backlog:** `.tasks/backlog/0026-diff-output-channel-redesign.md`
**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-27
**Round:** 1

---

## VERDICT: APPROVED

The spec is clear, correct in substance, and implementable as written. The three
issues found are all arithmetic/bookkeeping errors that do not affect the actual
code changes. The call site mapping is accurate, the proposed function signatures
match their stderr counterparts exactly, and the blast radius is correctly
confined to `diff.rs`.

---

## Concerns

### 1. Test count is wrong: 12 tests exist, not 11

```
{
  "gap": "The spec claims 'eleven existing diff tests' (sections 3.3 intro, 8.1,
          and acceptance criteria item 8). The actual file has 12 diff tests.",
  "question": "Is diff_unresolved_secret_shows_red_minus (cli_smoke.rs line 283)
               intentionally omitted, or was it missed during enumeration?",
  "severity": "ADVISORY",
  "recommendation": "Update the count from 'eleven' to 'twelve' in sections 3.3,
                     8.1, and acceptance criteria. Add a 'No change needed' entry
                     for diff_unresolved_secret_shows_red_minus in section 3.3
                     (both assertions are already on stdout)."
}
```

**Evidence:** The 12 tests in `tests/cli_smoke.rs`:

| # | Test function (line) | Needs change? |
|---|---------------------|---------------|
| 1 | `diff_no_config_exits_nonzero` (123) | No |
| 2 | `diff_satisfied_config_exits_zero` (134) | Yes |
| 3 | `diff_missing_tool_shows_plus` (158) | Yes |
| 4 | `diff_disabled_mcp_skipped` (182) | No |
| 5 | `diff_version_mismatch_shows_tilde` (207) | No |
| 6 | `diff_with_custom_config_path` (231) | Yes |
| 7 | `diff_summary_shows_counts` (255) | Yes |
| 8 | `diff_unresolved_secret_shows_red_minus` (283) | **No (MISSING FROM SPEC)** |
| 9 | `diff_mcp_missing_command_counted_as_install` (307) | Yes |
| 10 | `diff_mcp_missing_command_and_missing_tool_install_count` (332) | Yes |
| 11 | `diff_secret_dedup_required_and_ref` (358) | Yes |
| 12 | `diff_secret_ref_only_no_required_section` (386) | Yes |

The omitted test (#8) needs no changes -- its assertions are already on `.stdout(...)`:
```rust
// cli_smoke.rs lines 302-303
.stdout(predicate::str::contains("NONEXISTENT_SECRET_XYZ_88888"))
.stdout(predicate::str::contains("not set in environment"));
```

**Impact:** None on implementation. The builder will not accidentally change a test
that the spec did not list. But the spec's count is wrong in three places, which
could confuse verification.

---

### 2. Changed-test count is 8, not 9

```
{
  "gap": "The spec claims 'Nine of these tests have assertions that move from
          .stderr(...) to .stdout(...)' (section 3.3 intro and acceptance criteria).
          Counting the actual changes listed in section 3.3: 8 tests change,
          4 remain unchanged (no_config, disabled_mcp, version_mismatch,
          unresolved_secret).",
  "question": "Which is the ninth test that supposedly changes? Or is this an
               arithmetic error?",
  "severity": "ADVISORY",
  "recommendation": "Update 'nine' to 'eight' in the section 3.3 intro sentence
                     and in acceptance criteria item 8. The detailed per-test
                     listings are correct -- only the summary count is wrong."
}
```

**Evidence:** Section 3.3 explicitly marks three tests as "No change" out of 11
listed (should be 4 out of 12 with the missing test). 12 - 4 = 8 changes, not 9.

---

### 3. MCP Servers header line number off by one

```
{
  "gap": "The spec's call site table (section 3.2) says line 170 for
          output::header('MCP Servers'). The actual line is 171.",
  "question": "Was this a transcription error, or was the code modified after the
               spec was drafted?",
  "severity": "ADVISORY",
  "recommendation": "Update '170' to '171' in the table. This is cosmetic -- the
                     builder will match on the string, not the line number -- but
                     accuracy prevents confusion during review."
}
```

**Evidence:**
```
$ rg -n 'output::header\("MCP Servers"\)' src/cli/diff.rs
171:            output::header("MCP Servers");
```

---

## Verification Summary

### Correctness -- PASS

All 8 `output::*` calls in `diff.rs` are accounted for:

| Actual line | Call | Spec disposition |
|-------------|------|-----------------|
| 40 | `output::error(...)` | Preserved on stderr (correct) |
| 49 | `output::header("great diff")` | Changed to `header_stdout` |
| 50 | `output::info(&format!("Comparing..."))` | Changed to `info_stdout` |
| 123 | `output::header("Tools")` | Changed to `header_stdout` |
| 171 | `output::header("MCP Servers")` | Changed to `header_stdout` |
| 218 | `output::header("Secrets")` | Changed to `header_stdout` |
| 226 | `output::success(...)` | Changed to `success_stdout` |
| 239 | `output::info(&format!("{} -- run..."))` | Changed to `info_stdout` |

No `output::*` calls are missed. No `eprintln!` calls exist outside of `output::*`
in `diff.rs`. The seven changes and one preservation are complete and correct.

### Completeness -- PASS

The spec covers every `output::*` call site in `diff.rs`. The `println!` calls
for diff lines (lines 125, 127, 173, 175, 219, 222) are already on stdout and
correctly left unchanged.

### Simplicity -- PASS

The additive approach (new `_stdout` variants) is the simplest option. The
alternative of a channel parameter would require touching every caller across
`doctor.rs`, `vault.rs`, `status.rs`, `apply.rs`, etc. The spec explicitly
justifies this in section 2.1.

### Consistency -- PASS

The proposed `_stdout` functions use identical formatting to their stderr
counterparts:
- `header_stdout`: `println!("{}", msg.bold())` vs `header`: `eprintln!("{}", msg.bold())`
- `info_stdout`: `println!("{} {}", "\u{2139}".blue(), msg)` vs `info`: `eprintln!("{} {}", "\u{2139}".blue(), msg)` -- note the actual `info()` uses the literal character, but U+2139 is the same codepoint.
- `success_stdout`: `println!("{} {}", "\u{2713}".green(), msg)` vs `success`: `eprintln!("{} {}", "\u{2713}".green(), msg)` -- same, U+2713 matches the literal character.

### Testability -- PASS

Every test assertion change is correct. The 8 tests that need `.stderr(...)` to
`.stdout(...)` migrations are accurately identified with correct line numbers.
The 4 tests that need no changes are correctly excluded.

### Blast radius -- PASS

`output::header`, `output::info`, and `output::success` are called from at least
7 other files (`doctor.rs`, `vault.rs`, `apply.rs`, `status.rs`, `init.rs`,
`template.rs`, `loop_cmd.rs`). None of those callers are touched. The new
`_stdout` functions are additive-only.

### Backlog alignment -- PASS

The spec satisfies all 5 acceptance criteria from the backlog:
1. `great diff 2>/dev/null` produces complete stdout output -- covered
2. `great diff 1>/dev/null` produces no output -- covered
3. Missing config error stays on stderr -- covered
4. `cargo clippy` passes -- covered
5. Integration tests updated -- covered

---

## Summary

Spec is accurate in substance and ready for implementation. Three ADVISORY
bookkeeping errors (test count 12 not 11, changed-test count 8 not 9, MCP line
171 not 170) do not affect the correctness of the prescribed code changes.
