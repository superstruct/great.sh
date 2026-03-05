# Security Audit: 0045 -- `--only` and `--skip` flags for `great apply`

| Field | Value |
|---|---|
| Auditor | Kerckhoffs |
| Date | 2026-03-05 |
| Verdict | **PASS** -- no CRITICAL or HIGH findings |
| Files reviewed | `src/cli/apply.rs` (lines 352-960), `tests/cli_smoke.rs` (lines 857-962) |

---

## Audit Checklist

### 1. Input Validation -- PASS

`ApplyCategory` is a `#[derive(ValueEnum)]` enum with four fixed variants: `Tools`, `Mcp`, `Agents`, `Secrets`. Clap rejects unknown values at parse time with exit code 2 and an error listing valid values. No custom string parsing involved. The integration test `apply_invalid_category` (cli_smoke.rs:948) confirms this.

No injection vector exists -- values never flow into shell commands, file paths, or SQL.

### 2. Category Bypass -- PASS (INFO)

`--skip secrets` allows bypassing the secrets-not-set warnings. Reviewed the secrets check (apply.rs:932-951): it only calls `std::env::var()` to detect missing env vars and prints advisory warnings. No secrets are read, logged, or exposed. No enforcement logic is bypassed. This is intentional per spec section 8.

**INFO**: If a future iteration adds secret enforcement (e.g., blocking apply when secrets are missing), the `--skip secrets` escape hatch should be re-evaluated. Currently safe.

### 3. Logic Correctness of `should_apply` -- PASS

The function at apply.rs:396-404:
```rust
fn should_apply(category: ApplyCategory, only: &[ApplyCategory], skip: &[ApplyCategory]) -> bool {
    if !only.is_empty() { return only.contains(&category); }
    if !skip.is_empty() { return !skip.contains(&category); }
    true
}
```

Logic is correct:
- `only` non-empty: allowlist semantics (only listed categories run). Correct.
- `skip` non-empty: denylist semantics (listed categories are excluded). Correct.
- Both empty: everything runs (preserves default behaviour). Correct.
- Both non-empty: impossible at runtime due to `conflicts_with`, but the function handles it defensively by giving `only` priority. Correct.

Unit tests cover all four paths: no filters, only-match, skip-match, only-multiple (apply.rs:1116-1157).

### 4. Mutual Exclusion -- PASS

`conflicts_with = "skip"` is set on the `only` field (apply.rs:382). In clap, `conflicts_with` is bidirectional -- setting it on one side is sufficient. The integration test `apply_only_and_skip_conflict` (cli_smoke.rs:931-945) verifies both directions produce exit code 2 with "cannot be used with" in stderr.

Note: The `skip` field at apply.rs:386 does not redundantly declare `conflicts_with = "only"`. This is correct clap usage -- not a bug.

### 5. Default Behaviour (No Regression) -- PASS

When neither `--only` nor `--skip` is passed, both `args.only` and `args.skip` are empty `Vec`s. `should_apply()` returns `true` for all categories. All sections execute unconditionally, matching pre-change behaviour.

The unconditional sections (config loading at step 1, platform detection at step 2, dry-run banner, and final summary) remain outside all category gates. Correct.

### 6. Privilege Escalation: Sudo Under `--only tools` -- PASS (INFO)

Sudo credential caching (apply.rs:442-466) is inside the `Tools` gate. This is correct: sudo is needed only for tool installation (apt, homebrew). Running `--only tools` triggers sudo, which is the expected and necessary behaviour.

Running `--only mcp` or `--only secrets` does NOT trigger sudo caching. Correct: no privilege escalation from MCP or secrets categories.

**INFO**: `--only tools` without `--dry-run` will prompt for sudo password. This is documented behaviour, not a vulnerability. The `--dry-run` flag correctly prevents sudo caching (`needs_sudo` checks `!args.dry_run`).

---

## Additional Observations

### L1 (LOW): `ApplyCategory` derives `Debug`

`ApplyCategory` at apply.rs:353 derives `Debug`. This is harmless since the enum contains no secret data -- just category labels (Tools, Mcp, Agents, Secrets). No action needed.

### L2 (LOW): No test for `--skip` with all four categories

The test suite does not cover `--skip tools,mcp,agents,secrets` (skip everything). Per spec section 6, this should exit 0 with only header/summary. The existing tests are sufficient for security purposes, but this edge case could be added for completeness.

---

## Verdict

**PASS**. No CRITICAL or HIGH findings. The implementation is clean, follows the spec, and introduces no new attack surface. The `should_apply` logic is correct and well-tested. Input validation is handled entirely by clap's type system. No secrets, credentials, or user-controlled strings flow through the new code paths.

No findings block commit.
