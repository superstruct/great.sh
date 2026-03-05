# 0045 -- Socrates Review: `--only` and `--skip` flags for `great apply`

| Field | Value |
|---|---|
| Task ID | 0045 |
| Reviewer | Socrates |
| Date | 2026-03-05 |
| Spec reviewed | `0045-apply-only-skip-spec.md` (Lovelace) |

---

## VERDICT: APPROVED

---

## Strengths

- The `ApplyCategory` as a `clap::ValueEnum` enum is the right design. It rejects invalid categories at parse time with zero custom code, and it is exhaustive at compile time. This is strictly better than the raw `Vec<String>` suggested by Nightingale.
- The `should_apply` helper is trivially testable and keeps the gating logic in one place rather than scattered across 10+ section boundaries.
- Mutual exclusion via `conflicts_with` is clean. Clap handles the error message and exit code.
- Edge case coverage in section 6 is thorough -- duplicates, skip-everything, comma-with-spaces, and the no-op `agents` category are all addressed.
- The spec correctly identifies that config loading, platform detection, dry-run banner, and summary must remain unconditional.
- Test plan covers both the happy paths and the error paths (conflict, invalid category), with unit tests for the helper function.
- The section-to-category mapping is well-documented with line ranges that I verified against the actual `apply.rs` source.

---

## Concerns

### 1. The `tools` category is very coarse-grained

```
{
  "gap": "tools gates 11 distinct sections (sudo, prereqs, homebrew, runtimes, CLI tools, bitwarden-cli, starship, nerd font, platform tools, docker, Claude Code, system tuning)",
  "question": "Is there a user scenario where someone wants to install runtimes but NOT docker, or wants Claude Code but NOT system tuning? If so, should there be sub-categories (e.g., tools.runtimes, tools.docker)?",
  "severity": "ADVISORY",
  "recommendation": "Document that this is an intentional V1 simplification and that sub-categories may be added later. No code change needed now."
}
```

### 2. Bitwarden-CLI gating crosses category boundaries

```
{
  "gap": "Section 5b (bitwarden-cli install) is gated under `tools` per the spec, but it is triggered by the `[secrets]` config section and exists solely to support secrets functionality.",
  "question": "If a user runs `--only secrets`, should they expect bitwarden-cli to be installed as a prerequisite for secrets? Currently it would be skipped because it is under `tools`.",
  "severity": "ADVISORY",
  "recommendation": "Consider gating 5b under BOTH tools and secrets (i.e., run if either category is active), or document this as a known limitation where users must `--only tools,secrets` to get the full secrets workflow."
}
```

### 3. Starship/nerd-font gating may surprise users

```
{
  "gap": "Section 5c (starship config + nerd font) is gated under `tools`, but it runs based on whether starship is in [tools.cli]. If `--skip tools` is used, starship config is skipped even though the tool is already installed.",
  "question": "Is this the intended behavior? A user who already has starship installed and runs `--skip tools` to avoid slow tool installs might expect starship config to still be applied.",
  "severity": "ADVISORY",
  "recommendation": "Acceptable for V1. Just note that starship configuration is considered part of tool provisioning, not a separate category."
}
```

### 4. No integration test asserts section output is absent for `--skip`

```
{
  "gap": "Test 4 (apply_skip_tools_dry_run) only asserts exit code 0. It does not assert that tools-related output is absent from stdout, unlike Test 3 which checks for the absence of 'CLI Tools' and 'MCP Servers'.",
  "question": "Should Test 4 also assert that tool-related section headers are absent from stdout, to verify the skip actually worked?",
  "severity": "ADVISORY",
  "recommendation": "Add a negative assertion to Test 4 (e.g., stdout does NOT contain 'CLI Tools' or 'Runtimes')."
}
```

### 5. No test for multi-value `--only` (comma-separated or repeated)

```
{
  "gap": "The spec describes `--only tools,mcp` and `--only tools --only mcp` as valid usage, but no integration test exercises either form.",
  "question": "Is this covered by clap's own test suite, or should there be at least one integration test confirming multi-value accumulation works end-to-end?",
  "severity": "ADVISORY",
  "recommendation": "Add one integration test for `--only tools,mcp --dry-run` to confirm both sections run and others are skipped."
}
```

### 6. The gating wraps unevenly around the tools section due to nesting

```
{
  "gap": "Sections 3 and 4 (runtimes and CLI tools) are nested inside `if let Some(tools) = &cfg.tools`. The spec says to gate at the outermost level of each section, but the `should_apply` check must go INSIDE the `if let Some(tools)` block, not outside it, to avoid changing the control flow when tools config is absent.",
  "question": "Does the implementer understand that the tools gate wraps around the BODY of each conditional, not around the config-presence check? i.e., the pattern is `if let Some(tools) = &cfg.tools { if should_apply(Tools, ...) { ... } }`?",
  "severity": "ADVISORY",
  "recommendation": "Clarify in section 3.1 that for sections already inside config-presence guards (like `if let Some(tools)`), the `should_apply` check goes inside the existing guard, not outside it. Alternatively, a single large `if should_apply(Tools, ...)` block can wrap everything from section 2a through 10, with the config-presence checks remaining inside."
}
```

### 7. Agents category is a no-op placeholder with no future implementation path

```
{
  "gap": "The spec defines `agents` as a valid category that does nothing. There is no reference to when or how agent file provisioning will be added.",
  "question": "Is there a risk that users pass `--only agents` expecting something to happen and get confused by silent success? Should the command print a message like 'agents: no provisioning configured'?",
  "severity": "ADVISORY",
  "recommendation": "Consider printing an informational message when the agents section runs but has no work to do, so users know the category was recognized but is currently empty."
}
```

---

## Summary

A well-structured spec with clear acceptance criteria, thorough edge case analysis, and a sound design choice (typed enum over raw strings). The seven advisory concerns are refinements, not blockers -- the spec is implementable as-is and will not break existing behavior when no flags are passed.
