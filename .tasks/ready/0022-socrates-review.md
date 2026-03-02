# Socrates Review -- Spec 0022: Align `great diff` Counter Buckets with Visual Markers

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-26
**Spec:** `.tasks/ready/0022-diff-counter-spec.md`
**Round:** 1

---

## VERDICT: APPROVED

---

## Verification Results

### Line Number Accuracy

Every line number reference in the spec was verified against `src/cli/diff.rs` (248 lines):

| Spec claim | Actual code | Verdict |
|------------|-------------|---------|
| Line 143: `configure_count += 1` in MCP missing-command path | Line 143: `configure_count += 1;` | CORRECT |
| Line 122: `output::header("Tools")` | Line 122: `output::header("Tools");` | CORRECT |
| Line 170: MCP header string | Line 170: `output::header("MCP Servers — need configuration:");` | CORRECT |
| Line 197: Secrets header string | Line 197: `output::header("Secrets — need to set:");` | CORRECT |
| Line 216: Secret References header | Line 216: `output::header("Secret References — unresolved:");` | CORRECT |
| Lines 179-203: First secrets loop | Lines 179-203: `if let Some(secrets)` block | CORRECT |
| Lines 207-227: Second secrets loop (`find_secret_refs`) | Lines 207-227: scan and display block | CORRECT |
| Lines 152-165: `.mcp.json` existence check | Lines 152-165: guarded by `if cmd_available` | CORRECT |

### `find_secret_refs()` Behavior

Verified in `src/config/schema.rs` lines 246-277. The spec correctly states it scans:
1. Agent `api_key` fields (lines 251-258) for `${SECRET_NAME}` patterns
2. MCP `env` values (lines 262-271) for the same pattern

The regex `\$\{([A-Z_][A-Z0-9_]*)\}` only matches uppercase names with underscores. The function returns a sorted, deduplicated `Vec<String>` via `sort()` + `dedup()`. All claims match.

### Existing Test Assertions

Verified all 8 existing diff tests in `tests/cli_smoke.rs`:

| Test | Lines | Asserts on | Header text asserted? | Impact of fix |
|------|-------|------------|----------------------|---------------|
| `diff_no_config_exits_nonzero` | 123-131 | Exit code + stderr "great.toml" | No | None |
| `diff_satisfied_config_exits_zero` | 134-155 | Exit code + stderr "nothing to do" | No | None |
| `diff_missing_tool_shows_plus` | 158-179 | stdout tool name + stderr "great apply" | No | None |
| `diff_disabled_mcp_skipped` | 182-204 | stdout/stderr NOT containing server name | No | None |
| `diff_version_mismatch_shows_tilde` | 207-228 | stdout "git" + "want 99.99.99" | No | None |
| `diff_with_custom_config_path` | 231-252 | stderr containing filename | No | None |
| `diff_summary_shows_counts` | 255-280 | stderr "1 to install" + "1 secrets to resolve" | No | None |
| `diff_unresolved_secret_shows_red_minus` | 283-304 | stdout secret name + "not set in environment" | No | None |

No existing test asserts on any header string. The spec's claim that all 8 tests pass unchanged is correct.

### `.mcp.json` Check Independence

The spec claims the `.mcp.json` check (lines 152-165) is independent of the Bug 1 fix. Verified: the `.mcp.json` block runs unconditionally, but the `configure_count += 1` at line 157 is guarded by `if cmd_available`. When `cmd_available` is false (the Bug 1 path), the inner guard prevents a second increment. The fix to line 143 does not interact with this path. CORRECT.

---

## Concerns

### Concern 1 (ADVISORY): Em dash vs double-dash inconsistency in spec text

```
{
  "gap": "The spec's Issue 3 table (lines 93-97) shows header strings with '--' (double hyphen),
         but the actual code uses '---' (em dash U+2014). The Fix 3 table (lines 203-208) correctly
         shows the em dash. An implementer reading only the Issue 3 table could be confused.",
  "question": "Is the double-hyphen notation in the Issue 3 table intentional shorthand, or a
               transcription error?",
  "severity": "ADVISORY",
  "recommendation": "Since the Fix 3 table (the actionable section) correctly shows the em dash,
                     and the fix instruction at line 248 says to change to bare 'MCP Servers'
                     (removing the em dash entirely), this does not affect correctness.
                     No action required."
}
```

### Concern 2 (ADVISORY): Test 3 will also trigger `configure_count` for `.mcp.json`

```
{
  "gap": "Test 3 (diff_secret_dedup_required_and_ref) uses command='echo' which exists, so
         cmd_available=true. The TempDir has no .mcp.json, so lines 156-164 will increment
         configure_count by 1 for the MCP server. The test only asserts on secrets_count
         ('1 secrets to resolve') and absence of '2 secrets'. It does NOT assert on the
         absence of 'to configure', so the extra configure_count is harmless.",
  "question": "Should Test 3 also assert that 'to configure' appears with count 1, to fully
               document the MCP server's configure_count side effect?",
  "severity": "ADVISORY",
  "recommendation": "This is a test quality improvement, not a correctness issue. The test
                     as specified will pass. Asserting on the configure_count would make the
                     test more complete but is not required for this bugfix. The implementer
                     can decide."
}
```

### Concern 3 (ADVISORY): `echo` command availability with `which::which()`

```
{
  "gap": "The spec proposes using 'echo' as the MCP command in Tests 3 and 4 because it
         exists on all platforms. While /bin/echo or /usr/bin/echo exists on macOS and Linux,
         the which crate resolves from PATH. On some minimal CI containers, echo may only
         be a shell builtin (not a standalone binary on PATH).",
  "question": "Has which::which('echo') been verified to return Ok on all CI environments
               used by this project?",
  "severity": "ADVISORY",
  "recommendation": "The existing test 'test_command_exists_positive' (detection.rs:423)
                     uses 'ls', not 'echo'. Both are standard on Linux/macOS. In practice,
                     /bin/echo exists on every standard Linux and macOS installation, and
                     the project's CI environments (ubuntu.Dockerfile, fedora.Dockerfile)
                     use full base images, not minimal/scratch. Risk is very low. If
                     paranoid, use 'ls' instead of 'echo'."
}
```

### Concern 4 (ADVISORY): Merging two sections loses provenance information

```
{
  "gap": "The current code shows secrets from secrets.required under 'Secrets -- need to set:'
         and refs from find_secret_refs() under 'Secret References -- unresolved:'. The fix
         merges both into a single 'Secrets' section. A user who sees 'ANTHROPIC_API_KEY
         (not set in environment)' cannot tell whether it was explicitly required or discovered
         via config scanning.",
  "question": "Is the loss of provenance distinction acceptable to users who may want to know
               WHY a secret is required?",
  "severity": "ADVISORY",
  "recommendation": "The backlog (0022, requirement 3) says 'Deduplicate secrets that appear
                     in both secrets.required and find_secret_refs'. Merging is a valid approach.
                     The suffix text ('not set in environment' vs 'referenced in config, not set')
                     still provides differentiation. This is a UX trade-off, not a bug."
}
```

### Concern 5 (ADVISORY): BTreeSet is unnecessary -- Vec dedup would suffice

```
{
  "gap": "The spec uses BTreeSet for 'deterministic output without an extra sort step.'
         However, the BTreeSet is only used for membership checking (contains/insert).
         Display order comes from the insertion order of secret_diffs Vec, not from the
         BTreeSet iteration order. A HashSet would work identically for the contains() check.",
  "question": "Since BTreeSet ordering is never consumed (only contains/insert/len are used),
               is the BTreeSet choice justified over HashSet?",
  "severity": "ADVISORY",
  "recommendation": "Both work. BTreeSet has slightly worse constant factors for insert/contains
                     (O(log n) vs O(1)) but secrets lists are tiny (< 20 items). The choice is
                     inconsequential. No change needed."
}
```

---

## Cross-Reference Check

| Backlog requirement | Spec coverage | Verdict |
|---------------------|---------------|---------|
| Define classification rules (install/configure/secrets) | Table in Bug Analysis section (lines 50-54) | Covered |
| Each marker matches its summary bucket | Fix 1 (line 143 change) + acceptance criteria | Covered |
| Deduplicate secrets | Fix 2 (BTreeSet dedup) + acceptance criteria + Test 3 | Covered |
| Consider unifying section headers | Fix 3 (bare-noun style) + acceptance criteria | Covered |
| Integration tests updated | 4 new tests specified + all 8 existing tests verified | Covered |

The Nightingale selection suggested `HashSet`; the spec chose `BTreeSet` with rationale. This is an acceptable deviation.

---

## Summary

This is a clean, well-scoped bugfix spec. All line number references are accurate. All existing test assertions were verified to be unaffected. The fix design is minimal and correct. The three fixes (counter bucket, dedup, header normalization) are independent and low-risk. No BLOCKING concerns found. APPROVED for implementation.
