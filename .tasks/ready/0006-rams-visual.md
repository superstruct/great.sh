# Visual Design Review: `great diff`

**Author:** Dieter Rams (Visual Reviewer)
**Date:** 2026-02-25
**Files reviewed:**
- `/home/isaac/src/sh.great/src/cli/diff.rs`
- `/home/isaac/src/sh.great/src/cli/output.rs`
**Command tested:** `great diff --config /tmp/great-diff-test/great.toml`

---

## Verdict: REJECTED

Two blocking defects (Principles 2 and 8). Five advisory concerns. Detailed
findings below.

---

## Test Corpus

### Config used

```toml
[project]
name = "diff-test"

[tools]
node = "22"
python = "3.12"
ruby = "3.3"

[tools.cli]
ripgrep = "latest"
fzf_nonexistent_tool = "latest"
gh = "latest"

[agents.claude]
provider = "anthropic"
api_key = "${ANTHROPIC_API_KEY}"

[mcp.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem"]

[mcp.nonexistent_mcp]
command = "nonexistent-mcp-command"

[secrets]
provider = "env"
required = ["ANTHROPIC_API_KEY", "MISSING_SECRET_FOR_TEST"]
```

### Full rendered output (colors stripped)

```
great diff
ℹ Comparing /tmp/great-diff-test/great.toml against system state

Tools
  + python (need 3.12)
  + ruby (need 3.3)
  + ripgrep (need latest)
  + fzf_nonexistent_tool (need latest)

MCP Servers — need configuration:
  + nonexistent_mcp (nonexistent-mcp-command — not found)
  ~ filesystem (command available, needs .mcp.json config)

Secrets — need to set:
  - ANTHROPIC_API_KEY (not set in environment)
  - MISSING_SECRET_FOR_TEST (not set in environment)

Secret References — unresolved:
  - ANTHROPIC_API_KEY (referenced in MCP env, not set)

ℹ 4 to install, 2 to configure, 3 secrets to resolve — run `great apply` to reconcile.
```

### Clean-state output (no diff)

```
great diff
ℹ Comparing /tmp/great-diff-test/great.toml against system state

✓ Environment matches configuration — nothing to do.
```

### Color assignments confirmed (CLICOLOR_FORCE=1)

| Element | ANSI code | Result |
|---------|-----------|--------|
| `+` marker | `\e[32m` | green |
| `~` marker | `\e[33m` | yellow |
| `-` marker | `\e[31m` | red |
| tool/key name | `\e[1m` | bold |
| parenthetical hint | `\e[2m` | dimmed |
| section headers | `\e[1m` | bold |
| `ℹ` prefix | `\e[34m` | blue |
| `✓` prefix | `\e[32m` | green |

---

## Principle-by-Principle Evaluation

### 1. Innovative — PASS

The three-tier marker system (`+` install, `~` configure, `-` blocked) maps
cleanly onto the three actionability categories a developer needs to
distinguish. Borrowed from `git diff` convention, but applied to a new domain
(environment state vs. committed config) in a coherent way. The separation of
"needs action" by effort level (automatic vs. manual) is a genuine contribution.
No gratuitous novelty.

### 2. Useful — FAIL (BLOCKING)

**Defect: stdout/stderr channel split severs headers from their content.**

Section headers (`Tools`, `MCP Servers — need configuration:`, etc.) are
written to stderr via `output::header` -> `eprintln!`. The diff lines
(`  + python ...`) are written to stdout via `println!`. When either channel
is redirected separately — a routine CI operation — the output fragments:

```
# stderr only (e.g., great diff 1>/dev/null):
great diff
ℹ Comparing ...
Tools
MCP Servers — need configuration:
Secrets — need to set:
Secret References — unresolved:
ℹ 4 to install, 2 to configure, 3 secrets to resolve — run `great apply` ...

# stdout only (e.g., great diff 2>/dev/null):

  + python (need 3.12)
  + ruby (need 3.3)
  ...
  - ANTHROPIC_API_KEY (referenced in MCP env, not set)
```

Headers without entries are meaningless. Entries without headers have no
grouping. The summary counter line and the header lines are critical context for
the diff lines — all should travel on the same channel.

The idiomatic Unix pattern for diagnostic commands: diff lines and their
structure both go to **stdout**; only unrecoverable errors and process-level
messages (spinners, warnings unrelated to the diff content itself) go to
stderr. The `output::success` / `output::info` / `output::header` functions
unconditionally use `eprintln!` — this is correct for spinners and brief status
messages, but not for the structured body of a diff report.

**Required fix:** Move all diff output — headers, diff lines, and summary — to
stdout. Keep only the "no great.toml found" error on stderr.

### 3. Aesthetic — PASS (with advisory)

The color choices are semantically loaded and consistent: green = additive
action, yellow = corrective action, red = blocker. This maps to traffic-light
cognition without being decorative. The dimmed hint text creates a clean
foreground/background hierarchy without a second font. Marker characters (`+`,
`~`, `-`) are single bytes — readable in any encoding, with or without color.

Advisory: the `ℹ` (U+2139) prefix on summary lines is blue. Blue carries no
semantic meaning in this system's established vocabulary (green = ok, yellow =
warning, red = error). The ℹ is a repetition of the icon from `output::info`,
which was designed for incidental notes. The final summary line is primary
output, not incidental. Its current visual weight (blue, dimmed icon, plain
text) undersells its importance. The summary should have equivalent visual
weight to a section header. Minor.

### 4. Understandable — PASS

The marker legend is documented in the `run` function docstring and deducible
from context. `+` is universal for "add", `~` for "partial", `-` for
"blocked". The parenthetical hint text is unambiguous:
- `(need 3.12)` — what is required
- `(want 3.12, have 3.11.0)` — comparison in plain language
- `(not set in environment)` — states the precise condition
- `(command available, needs .mcp.json config)` — tells you exactly what is
  missing without requiring knowledge of internals

The "silence means agreement" design (no `=` lines for matching items) is
correct and is documented in the spec. The clean-state message is explicit:
"nothing to do."

### 5. Unobtrusive — PASS

The command is silent when nothing needs doing, beyond the mandatory header
line. The intro line (`ℹ Comparing ...`) adds one line of provenance. Sections
appear only when they have entries. No progress animation for a read-only
inspection. No color when the output is not a TTY (the `colored` crate respects
this by default).

One observation: the header line `great diff` is always printed even on
clean state. For a command run frequently in watch mode or CI, this adds a
constant line of noise. It is not blocking, but worth noting under thoroughness.

### 6. Honest — PASS (with advisory)

The MCP section carries a mismatch: a server with a missing command receives a
`+` (green, meaning "install") marker, while a server with an available command
but missing `.mcp.json` receives a `~` (yellow, meaning "configure"). The `+`
marker on a missing MCP command is semantically imprecise — "install" implies
`great apply` can handle it automatically, but the user must install
`nonexistent-mcp-command` manually just as they would resolve a missing secret.
The red `-` marker would be more honest here.

This is an advisory, not a block. The system's docstring does not promise that
`+` only appears for auto-installable items, and `great apply` may ultimately
handle MCP command installation. If it does not, the marker should be changed to
`-`. The implementer should clarify before shipping.

Advisory also: the `ANTHROPIC_API_KEY` key appears twice in the test output —
once under "Secrets — need to set:" and once under "Secret References —
unresolved:". This is the correct behavior (it is both explicitly required and
referenced in an MCP env value), but it doubles the visual noise and could
mislead a user into thinking they must act twice. A deduplicated display, or a
merged section, would be more honest about the actual number of distinct
unresolved items. The summary counter correctly counts it twice (incrementing
`secrets_count` twice), which means the numeric total is inflated relative to
the actual number of distinct secrets to resolve.

### 7. Long-lasting — PASS

The design avoids:
- Emoji (would require unicode-capable terminals and dating conventions)
- Framework-specific terminology
- Tabular layout that breaks on narrow terminals
- Progress bars or animations for a synchronous read-only operation

The three-marker vocabulary (`+`, `~`, `-`) is minimal and extensible. New
categories of diff (e.g., `agents`, `platform.extra_tools`) can be added with
no change to the rendering contract.

### 8. Thorough — FAIL (BLOCKING)

**Defect: MCP section counter mismatch.**

The MCP section header reads "MCP Servers — need configuration:" but items in
that section increment `configure_count` for both the `+` (missing command) and
`~` (needs `.mcp.json`) entries. In the rendered output:

```
MCP Servers — need configuration:
  + nonexistent_mcp (nonexistent-mcp-command — not found)
  ~ filesystem (command available, needs .mcp.json config)
```

The `+` item on `nonexistent_mcp` is classified by the summary as "to
configure" (counter: `configure_count += 1`). But the `+` marker signals
"install", not "configure" — creating a contradiction between the marker
semantics and the counter bucket.

The `install_count` and `configure_count` are not named correctly relative to
their population. `install_count` is only incremented for tools. MCP missing
commands go to `configure_count`. This means the numeric summary "4 to install,
2 to configure" does not reflect a consistent classification.

A thorough design would define the counter semantics explicitly and apply them
uniformly. Either:
- Rename/redefine: `install_count` = items `great apply` can auto-install,
  `configure_count` = items requiring user action; then classify MCP missing
  command as `install_count` if `apply` handles it, `secrets_count`-equivalent
  if it does not.
- Or: collapse to a single "action required" counter and let the marker colors
  carry the severity.

The current hybrid is internally inconsistent and will mislead CI consumers of
the numeric summary.

**Secondary thoroughness issue:** The `Secret References — unresolved:` section
uses `println!` (stdout) while the secrets required section uses a collected
`secret_diffs` vec. The code paths are structurally different for what is
semantically the same kind of item. This is a maintenance hazard.

```rust
// secrets.required uses a collected vec, then prints:
for diff in &secret_diffs {
    println!("{}", diff);
}

// secret refs use inline println!, skipping the vec:
for name in &unresolved_refs {
    secrets_count += 1;
    println!("  {} {} {}", "-".red(), name.bold(), "...".dimmed());
}
```

If the rendering contract ever changes (e.g., add a leading blank line), one
path would be updated and the other missed.

### 9. Environmentally Friendly — PASS

The command does not spawn subprocesses beyond `<tool> --version` checks for
installed tools. It makes no network requests. It reads one file. The output
lines scale with the number of diff items — there is no fixed overhead
proportional to config size. The `command_exists` call is a simple `which`-
equivalent with no disk writes.

### 10. As Little Design as Possible — PASS (with advisory)

The marker + bold-name + dimmed-hint triple is the minimum needed to convey
three pieces of information per item (severity, subject, reason). Nothing is
redundant in a single row.

Advisory: the section headers contain the action verb redundantly. "Secrets —
need to set:" repeats the information carried by the `-` (red) marker. "MCP
Servers — need configuration:" repeats what `~` conveys. A section header
need only name the category: `Secrets`, `MCP Servers`. The verb belongs to the
markers. Reducing the headers to single nouns would also eliminate the
inconsistency between `Tools` (no verb, current) and the other sections (verb
present).

---

## Summary of Findings

| # | Principle | Status | Issue |
|---|-----------|--------|-------|
| 1 | Innovative | PASS | — |
| 2 | Useful | **FAIL** | stdout/stderr split severs headers from content |
| 3 | Aesthetic | PASS | Advisory: blue ℹ undersells the summary line |
| 4 | Understandable | PASS | — |
| 5 | Unobtrusive | PASS | — |
| 6 | Honest | PASS | Advisory: `+` on missing MCP command may misrepresent auto-installability; duplicate secret key display inflates counter |
| 7 | Long-lasting | PASS | — |
| 8 | Thorough | **FAIL** | Counter bucket mismatch; divergent code paths for secret refs vs. required secrets |
| 9 | Environmentally friendly | PASS | — |
| 10 | As little as possible | PASS | Advisory: section header verbs are redundant with marker semantics |

---

## Required Changes Before Approval

### Blocking — must fix

**R1 (Principle 2): Unify output channel.**

All diff output (headers, diff lines, summary) must be written to stdout. Only
the "no great.toml found" error message belongs on stderr. This means changing
`output::header` calls within `diff.rs` to use `println!` rather than
`eprintln!`, or creating a new `output::section_header` variant that writes to
stdout.

The cleanest solution is a local `println!` for headers within `diff.rs`,
keeping `output::header` as-is for use by commands where stderr is appropriate.

**R2 (Principle 8): Resolve counter bucket definition.**

Define and document which items go to `install_count` vs. `configure_count`,
and apply the classification consistently to both tools and MCP entries. The
current split (tools-missing -> `install_count`, MCP-missing -> `configure_count`) is
arbitrary and unexplained. The summary line read by CI consumers depends on this
being correct.

### Advisory — should fix before shipping

**A1 (Principle 8): Unify secret rendering paths.**

The `secrets.required` block collects into `secret_diffs` then prints. The
`find_secret_refs` block prints inline. Make both use the same pattern (either
both collected, or both inline) to reduce maintenance divergence.

**A2 (Principle 6): Deduplicate secret display.**

When a key appears in both `secrets.required` and as a secret reference, emit
it once. The summary counter should reflect distinct secrets, not total
occurrences.

**A3 (Principle 10): Remove verbs from section headers.**

Change:
- `"MCP Servers — need configuration:"` -> `"MCP Servers"`
- `"Secrets — need to set:"` -> `"Secrets"`
- `"Secret References — unresolved:"` -> `"Secret References"`

The `Tools` header already follows this pattern. Consistency is thoroughness.

---

*"Less, but better."*
