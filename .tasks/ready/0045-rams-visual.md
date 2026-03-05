# Visual Design Review: `great apply --only` / `--skip`

**Author:** Dieter Rams (Visual Reviewer)
**Date:** 2026-03-05
**Files reviewed:**
- `/home/isaac/src/sh.great/src/cli/apply.rs`
- `/home/isaac/src/sh.great/src/cli/output.rs`
**Commands tested:** `apply --help`, `apply --only tools --dry-run`,
`apply --only agents --dry-run`, `apply --only mcp --dry-run`,
`apply --only invalid`

---

## Verdict: APPROVED

No blocking defects. Two advisories noted below.

---

## Test Corpus

### Command 1 — `great apply --help`

```
Apply configuration to the current environment

Usage: great apply [OPTIONS]

Options:
      --config <CONFIG>
          Path to configuration file

  -v, --verbose
          Increase output verbosity

      --dry-run
          Preview changes without applying

  -q, --quiet
          Suppress all output except errors

      --non-interactive
          Disable interactive prompts (for CI/automation)

  -y, --yes
          Skip confirmation prompts

      --only <ONLY>
          Only apply these categories (tools, mcp, agents, secrets). Repeatable.
          Mutually exclusive with --skip

          Possible values:
          - tools:   Tool installation and system bootstrapping
          - mcp:     MCP server configuration and bridge registration
          - agents:  Loop-agent file provisioning (reserved for future use)
          - secrets: Required secrets validation

      --skip <SKIP>
          Skip these categories (tools, mcp, agents, secrets). Repeatable.
          Mutually exclusive with --only

          Possible values:
          - tools:   Tool installation and system bootstrapping
          - mcp:     MCP server configuration and bridge registration
          - agents:  Loop-agent file provisioning (reserved for future use)
          - secrets: Required secrets validation

  -h, --help
          Print help (see a summary with '-h')
```

### Command 2 — `apply --only tools --dry-run`

```
great apply

ℹ Config: /tmp/rams-review/great.toml
ℹ Platform: WSL Ubuntu 24.04 (x86_64)

⚠ Dry run mode — no changes will be made

ℹ Filter: only tools

System Prerequisites
✓   curl — already installed
✓   git — already installed
✓   build-essential — already installed
✓   unzip — already installed

CLI Tools
✓   gh — already installed

Docker
✓   Docker — installed and daemon running

Claude Code
✓   Claude Code — already installed

System Tuning
✓   inotify max_user_watches: 1048576 (>= 524288)

ℹ Dry run complete. Run `great apply` without --dry-run to apply changes.
```

### Command 3 — `apply --only agents --dry-run`

```
great apply

ℹ Config: /tmp/rams-review/great.toml
ℹ Platform: WSL Ubuntu 24.04 (x86_64)

⚠ Dry run mode — no changes will be made

ℹ Filter: only agents

ℹ   agents: no provisioning configured (reserved for future use)
ℹ Dry run complete. Run `great apply` without --dry-run to apply changes.
```

### Command 4 — `apply --only mcp --dry-run`

```
great apply

ℹ Config: /tmp/rams-review/great.toml
ℹ Platform: WSL Ubuntu 24.04 (x86_64)

⚠ Dry run mode — no changes will be made

ℹ Filter: only mcp

MCP Bridge
ℹ   great-bridge — would register in .mcp.json

ℹ Dry run complete. Run `great apply` without --dry-run to apply changes.
```

### Command 5 — `apply --only invalid`

```
error: invalid value 'invalid' for '--only <ONLY>'
  [possible values: tools, mcp, agents, secrets]

For more information, try '--help'.
```

---

## Principle-by-Principle Evaluation

### 1. Innovative — PASS

The filter mechanism does not invent new vocabulary. It reuses the existing
four-category taxonomy the command already operates on, expressed through
standard CLI `--only` / `--skip` conventions. Novelty is appropriate to the
scope: a selection predicate on a provisioning pipeline is a well-understood
pattern. No gratuitous novelty, no missed opportunity.

### 2. Useful — PASS

The filter banner (`ℹ Filter: only tools`) appears immediately after the
dry-run warning and before the first section, which is the correct position in
the information hierarchy. It answers the user's first question — "what will
this run do?" — before any results are shown. The no-op message for agents
(`ℹ   agents: no provisioning configured (reserved for future use)`) correctly
appears only when a filter makes the empty category visible, suppressing noise on
unfiltered runs.

The `--only` / `--skip` flags are repeatable and comma-delimited, giving
composable selection without requiring multiple invocations.

### 3. Aesthetic — PASS

The filter banner uses `output::info` (blue ℹ), consistent with how other
preamble lines (`Config:`, `Platform:`) are rendered. Color semantics are
preserved: filter metadata is informational, not a warning or a result. The
banner line is visually recessive relative to the section headers that follow —
correct weight ordering.

### 4. Understandable — PASS

`ℹ Filter: only tools` is unambiguous. The phrase "only tools" mirrors the
flag syntax (`--only tools`) so users can map output back to input instantly.
`ℹ Filter: skipping mcp` follows the same pattern for `--skip`. No
translation required.

The error output for an invalid value (command 5) is handled entirely by clap,
which lists the possible values verbatim. The user does not need to know the
enum name `ApplyCategory` — they see the concrete strings they can type. This
is understandable without documentation.

### 5. Unobtrusive — PASS

The filter banner is a single line. It does not repeat the flag name, the
config path, or any information already shown in the preamble. Sections
suppressed by the filter are completely absent from the output — no
"skipped" placeholder lines. The command stays out of the way.

### 6. Honest — PASS

`agents: no provisioning configured (reserved for future use)` accurately
describes the state: the category exists in the schema, accepts valid input, but
performs no action. The parenthetical `(reserved for future use)` is precise —
it tells the user this is intentional, not a defect in their configuration. The
feature does not overstate its capabilities.

### 7. Long-lasting — PASS

The design couples directly to the enum variant names, which are the canonical
identifiers for these categories across the codebase. If categories are added
or renamed, the filter values and banner text update in one place. No
hand-maintained list of strings; no help text that could drift from the
implementation.

### 8. Thorough — PASS (with advisory)

The `--only` and `--skip` flags are marked `conflicts_with` each other at the
clap level — the mutual exclusion is enforced at parse time, not application
time, and produces a clear clap error message. The empty-category visibility
rule (agents no-op message shown only under a filter) is handled via a
conditional in the source, not left as an unaddressed edge case.

**Advisory A1 (Principle 8 — thoroughness): inconsistent capitalisation in
no-op message.**

The agents no-op message reads:
```
ℹ   agents: no provisioning configured (reserved for future use)
```

The category name `agents` is lowercase. All section headers elsewhere in the
output use title case: `System Prerequisites`, `CLI Tools`, `MCP Bridge`,
`Docker`, `Claude Code`. The two-space indent before `agents` is also
inconsistent with how `output::info` is called here — the `info` primitive
produces `ℹ <message>`, so the two-space indent is embedded in the message
string rather than applied by the output layer. This is consistent with the
pattern used throughout apply.rs and is not a defect, but the lowercase
category name stands out against the title-case section headers above and
below it.

Suggested: `ℹ   Agents: no provisioning configured (reserved for future use)`

### 9. Environmentally Friendly — PASS

The filter adds one boolean check (`should_apply`) per category before
entering each provisioning block. There are no additional subprocesses,
file reads, or network calls caused by filtering. Filtered categories exit
the predicate in a single branch comparison.

### 10. As Little Design as Possible — PASS (with advisory)

The filter banner is one line. The mutual-exclusion constraint is declared
at the flag definition, not enforced with a runtime error block. The empty-
category message is shown only when relevant. Nothing superfluous was added
to support this feature.

**Advisory A2 (Principle 10 — as little as possible): `--only` and `--skip`
help descriptions repeat the category list.**

Both flags have the description:
```
Only apply these categories (tools, mcp, agents, secrets).
```

The possible-values enumeration immediately below lists the same four
categories with their descriptions. The parenthetical `(tools, mcp, agents,
secrets)` in the description line is redundant with the `Possible values`
block clap already renders. Removing it shortens the help without losing
any information:

```
Only apply these categories. Repeatable. Mutually exclusive with --skip
```

This is minor and does not affect runtime behavior.

---

## Summary of Findings

| # | Principle | Status | Issue |
|---|-----------|--------|-------|
| 1 | Innovative | PASS | — |
| 2 | Useful | PASS | — |
| 3 | Aesthetic | PASS | — |
| 4 | Understandable | PASS | — |
| 5 | Unobtrusive | PASS | — |
| 6 | Honest | PASS | — |
| 7 | Long-lasting | PASS | — |
| 8 | Thorough | PASS | Advisory A1: `agents` should be `Agents` in no-op message |
| 9 | Environmentally friendly | PASS | — |
| 10 | As little as possible | PASS | Advisory A2: redundant category list in flag description |

---

## Advisories (non-blocking)

**A1 (Principle 8):** The no-op message for the agents category uses lowercase
`agents` where every section header in the same output uses title case. The
inconsistency is visible when filtering to agents.
File: `/home/isaac/src/sh.great/src/cli/apply.rs`, line 960.
Change: `"  agents: no provisioning configured (reserved for future use)"`
to: `"  Agents: no provisioning configured (reserved for future use)"`

**A2 (Principle 10):** Both `--only` and `--skip` help strings repeat the
category list already shown by clap's `Possible values` block. Remove the
parenthetical `(tools, mcp, agents, secrets)` from the description strings.
File: `/home/isaac/src/sh.great/src/cli/apply.rs`, lines 392 and 398 (arg
`doc` comments).

---

*"Less, but better."*
