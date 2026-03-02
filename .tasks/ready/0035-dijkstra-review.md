# Dijkstra Review — Task 0035: Init Wizard MCP Bridge UX Polish

Reviewer: Edsger Dijkstra (Code Reviewer)
Date: 2026-02-28
File reviewed: `src/cli/init.rs`

---

```
VERDICT: APPROVED

Issues:
- [WARN] src/cli/init.rs:237 — `config.agents.as_ref().map_or(0, |a| a.len()) > 1`
  is correct but verbose. The established idiom in this file for querying
  `config.agents` is `config.agents.as_ref().is_some_and(|a| ...)` (lines 258,
  265). A consistent alternative would be
  `config.agents.as_ref().map_or(false, |a| a.len() > 1)`. The current form
  is not wrong and compiles cleanly, but it diverges from the local pattern.
  Advisory only; no structural defect.

- [WARN] src/cli/init.rs:233-236 — The four-line block comment is well-reasoned,
  but it is addressed to future maintainers rather than to a reader trying to
  understand what the code does right now. A single sentence suffices:
  "Wizard uses agent-count heuristic; templates use complexity-based presets."
  The current comment is acceptable; the warning is a conciseness observation.

Summary: Both changes are correct, the abstraction boundary is respected, no
dead code or unused imports are introduced, and `&format!(...)` at line 246
follows the established pattern used at lines 50, 302, 319, and 329 throughout
this file.
```

---

## Detailed findings

### Line 15 — doc comment addition

```rust
/// Template to initialize from (ai-fullstack-ts, ai-fullstack-py, ai-minimal, saas-multi-tenant)
```

`saas-multi-tenant` is added to the exhaustive list. The template file
`templates/saas-multi-tenant.toml` exists and is matched in `init_from_template`
at line 317. The error message at line 320 also lists all four templates. The
doc comment, the match arm, and the error message are in sync. No issue.

### Lines 233-247 — Dynamic preset selection

```rust
// Preset heuristic is agent-count-based (wizard context).
// Templates use complexity-based presets (fullstack projects get "agent"
// even with a single agent). These semantics differ intentionally —
// do not "fix" one to match the other.
let preset = if config.agents.as_ref().map_or(0, |a| a.len()) > 1 {
    "agent"
} else {
    "minimal"
};
config.mcp_bridge = Some(McpBridgeConfig {
    preset: Some(preset.to_string()),
    ..Default::default()
});
output::success(&format!("MCP bridge enabled with {} preset", preset));
output::info("  Presets: minimal (1 tool) | agent (6 tools) | research (8 tools) | full (9 tools)");
```

**Correctness:** `config.agents` is always `Some(...)` at this point in the
function (it is unconditionally assigned at line 190). The `map_or(0, ...)` on
an always-`Some` value is safe; it will never return 0 via the default path.
The logic is correct.

**Idiom consistency (WARN):** The rest of the file uses `is_some_and` to query
`config.agents` (lines 258, 265). The `map_or(0, |a| a.len()) > 1` form works
but is the only site that extracts a count rather than checking for a key.
It is not an anti-pattern — just a minor local inconsistency.

**`&format!(...)` at line 246:** `output::success` takes `&str`. Passing
`&format!(...)` produces a temporary `String` whose reference is valid for the
call. This is the established pattern in the file (lines 50, 302, 319, 329) and
is idiomatic Rust. No issue.

**Comment quality (WARN):** The comment explicitly warns against a future
"fix." This is legitimate defensiveness when two parallel systems share
vocabulary but differ in semantics. The concern is that the comment is longer
than necessary, not that it is wrong.

### No dead code or unused imports

The diff introduces no new `use` items. No `#[allow(dead_code)]` annotations
are added without a GROUP annotation. All new identifiers (`preset`) are
consumed immediately within the same block.

### Error handling

No new error paths are introduced. The `McpBridgeConfig` construction and
`output::success` call cannot fail. The existing `?` propagation in the
surrounding `run` function is unaffected.
