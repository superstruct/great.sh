# Review: Spec 0011 — great statusline (Multi-Agent Statusline)

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-22
**Spec:** `/home/isaac/src/sh.great/.tasks/ready/0011-statusline-multi-agent.md`
**Backlog:** `/home/isaac/src/sh.great/.tasks/backlog/0011-statusline-multi-agent.md`

---

## VERDICT: APPROVED (with advisory notes)

---

## Concerns

### 1. BLOCKING (resolved by spec text, but ambiguously)

```
{
  "gap": "The `segments` config field is defined in StatuslineConfig (section 4.3)
         but never consumed by the rendering logic (sections 8.2-8.4). The render
         function's width-mode descriptions hard-code which segments appear and in
         what order. The config field exists as dead configuration.",
  "question": "Should the `render()` function consult `config.segments` to decide
               which segments to include and in what order, or is this field
               reserved for a future iteration? If the latter, should it be
               removed from the config struct to avoid confusing users who set it
               and see no effect?",
  "severity": "ADVISORY",
  "recommendation": "Either (a) describe how render() iterates config.segments to
                     assemble the output line, or (b) remove the segments field
                     from StatuslineConfig and note it as a future enhancement.
                     The builder should not ship dead config keys."
}
```

### 2. Settings injection scope vs. backlog requirement 7

```
{
  "gap": "Backlog requirement 7 states: 'great init and a new great configure
         subcommand (or addition to existing loop install) must write the
         statusLine key.' The spec only modifies `loop install`. The `great init`
         path is not addressed. A user who runs `great init` but not
         `great loop install` will never get the statusLine key.",
  "question": "Is it acceptable to limit settings injection to `great loop install`
               only? The statusline is only useful when the loop is installed, so
               this may be intentionally scoped down. If so, should the backlog be
               updated to reflect this decision?",
  "severity": "ADVISORY",
  "recommendation": "Add a brief rationale note in the spec explaining why
                     `great init` is excluded (e.g., 'statusline is only useful
                     with the loop installed, so injection belongs in loop install
                     exclusively'). This prevents the builder from wondering
                     whether they missed a requirement."
}
```

### 3. Integration test for state file is a no-op

```
{
  "gap": "The integration test `statusline_with_state_file` (section 20.2)
         creates a temp config file and state file, but the statusline command
         will call `dirs::config_dir()` which returns the real system config
         directory, not the temp dir. The spec acknowledges this in a comment
         ('Since we can't easily override dirs::config_dir()...') but the test
         still creates config files that will never be read. The test only
         verifies exit 0 with arbitrary stdin -- the same thing
         `statusline_empty_stdin_exits_zero` already tests.",
  "question": "Should the statusline module accept a config path override (e.g.,
               via an env var like GREAT_STATUSLINE_CONFIG or a hidden --config
               flag) to make integration tests meaningful? Without this, there is
               no integration-level test that exercises the state file rendering
               path.",
  "severity": "ADVISORY",
  "recommendation": "Either (a) add a hidden `--config` arg or env var for test
                     injection, or (b) remove the misleading test and rely on
                     unit tests for state file rendering. The current test creates
                     false confidence."
}
```

### 4. `env::remove_var` in unit test is unsound in Rust 2024

```
{
  "gap": "The test `test_resolve_width_fallback` calls
         `std::env::remove_var('COLUMNS')`. As of Rust 1.83 (2024-11),
         `env::remove_var` is deprecated because it is unsound in
         multi-threaded test execution (other tests may be reading env vars
         concurrently). The project uses edition 2021 so it compiles, but
         `cargo test` runs tests in parallel by default.",
  "question": "Is the builder aware that this test is inherently racy? If another
               test reads COLUMNS at the same time, either test may produce
               incorrect results. Should this test be marked #[ignore] or
               #[serial] (with the serial_test crate), or restructured to avoid
               mutating the environment?",
  "severity": "ADVISORY",
  "recommendation": "Mark the test with a comment acknowledging the race, or
                     restructure resolve_width() to accept an env-reading
                     closure/trait for testability. At minimum, note that `cargo
                     test -- --test-threads=1` is needed for reliable results."
}
```

### 5. `colored::control::set_override` is process-global state

```
{
  "gap": "The spec calls `colored::control::set_override(true)` in run() to force
         colors on in a piped context. This is a process-global static
         (LazyLock<ShouldColorize>). If the statusline subcommand is ever
         composed with other subcommands (e.g., a future batch mode), or if unit
         tests call render functions without resetting the override, test results
         will leak between tests.",
  "question": "Are unit tests that call `set_override(false)` safe from
               interference by other tests running in parallel? The spec's unit
               tests (e.g., test_render_wide_mode) call set_override(false) but
               other tests don't, creating potential ordering dependencies.",
  "severity": "ADVISORY",
  "recommendation": "Note in the spec that all render-testing unit tests MUST call
                     set_override(false) at the start, and that these tests may
                     interfere with each other under parallel execution. Consider
                     having the render function accept a `use_color: bool` param
                     instead of relying on global state for the rendering path."
}
```

### 6. No test for the truncation/ellipsis behavior at >30 agents

```
{
  "gap": "Section 16.6 specifies that agent counts exceeding 30 trigger
         truncation with an ellipsis indicator. No unit test covers this edge
         case. The threshold of 30 is also not derived from any calculation --
         it's unclear how 30 was chosen relative to the width modes.",
  "question": "How was the threshold of 30 agents chosen? In wide mode at 121
               columns, with overhead of ~40 chars for prefix/separators/summary,
               each wide agent indicator ('10*' = 3 chars + space) would need
               ~4 chars * 30 = 120 chars -- already exceeding the available
               space. Should the truncation be dynamic based on available width
               rather than a fixed count?",
  "severity": "ADVISORY",
  "recommendation": "Add a unit test for >30 agents. Consider making the
                     truncation threshold dynamic: calculate available width
                     minus overhead, divide by per-agent character cost, and
                     truncate at that point. At minimum, document how 30 was
                     derived."
}
```

### 7. Missing `agent_names` usage in rendering

```
{
  "gap": "StatuslineConfig defines `agent_names: HashMap<String, String>` for
         display label overrides (section 4.3, example: '1' = 'NI'). However,
         none of the render function signatures accept or use agent_names. The
         wide mode renders agent IDs numerically ('1*', '2*') with no mechanism
         to substitute the configured labels.",
  "question": "Where and how should agent_names be applied? Should
               render_agents_wide show '1*' or 'NI*' when agent_names contains
               '1' = 'NI'? The config field exists but the rendering functions
               have no access to it.",
  "severity": "ADVISORY",
  "recommendation": "Either describe how render_agents_wide/medium should use
                     agent_names from the config, or remove the field. Dead
                     config that silently does nothing is a user experience
                     problem."
}
```

### 8. `run()` returns `Result<()>` but should never fail -- contract tension

```
{
  "gap": "Section 17 states 'the implementation of run() should never return Err'
         and wraps everything in catch_unwind. But the function signature is
         `pub fn run(args: Args) -> Result<()>` per project convention. In
         main.rs, the match arm propagates via `?`. If run() truly never returns
         Err, the Result return type is misleading. If the catch_unwind wrapper
         is the 'real' run(), the inner errors are swallowed silently.",
  "question": "Is it acceptable that errors inside run_inner() are silently
               swallowed (printing only an empty line)? For debugging, should
               there be a --verbose or --debug mode that lets errors propagate?
               Currently, a bug in the rendering logic would be invisible.",
  "severity": "ADVISORY",
  "recommendation": "Consider logging to stderr when run_inner fails (only if
                     --verbose is set, or to a log file). An empty statusline
                     with no indication of why is difficult to debug. At minimum,
                     the error variant should write something like '> err' so the
                     user knows the statusline is broken."
}
```

### 9. Spec uses `println!` but `output::*` uses `eprintln!`

```
{
  "gap": "The statusline is one of the few commands that writes to stdout (via
         println!), because Claude Code reads stdout. Meanwhile, the settings
         injection code uses `output::success()` which writes to stderr. This
         dual-channel behavior is correct but should be explicitly called out to
         prevent the builder from accidentally using output::* in the statusline
         rendering path.",
  "question": "N/A -- this is correctly handled but worth a note.",
  "severity": "ADVISORY",
  "recommendation": "Add a comment in the spec's section 13 (entry point):
                     'The statusline MUST write to stdout (println!), not
                     stderr. Do NOT use output::* helpers for the statusline
                     output. The settings injection code in loop_cmd.rs may
                     use output::* normally since it writes to stderr.'"
}
```

### 10. Cost formatting precision not specified

```
{
  "gap": "Section 8.2 shows '$0.14' for cost rendering, but the spec never
         specifies the formatting precision. Should $1.5 display as '$1.50' or
         '$1.5'? What about $10.142857 -- is it '$10.14' (2 decimal places)?
         The render_cost function signature returns Option<String> but the
         formatting rules are absent.",
  "question": "What precision should be used for cost_usd display? Two decimal
               places always? Adaptive precision (e.g., $0.001 shows as
               '$0.001')? This affects user perception of cost accuracy.",
  "severity": "ADVISORY",
  "recommendation": "Specify: 'Format cost_usd to 2 decimal places
                     (format!('${:.2}', cost))'. Add a unit test for values
                     like 0.0, 0.001, 1.5, and 99.99."
}
```

---

## Completeness Check: Backlog Requirements

| # | Backlog Requirement | Spec Coverage | Status |
|---|---------------------|---------------|--------|
| 1 | Subcommand registration | Section 5 (mod.rs + main.rs) | COVERED |
| 2 | Stdin JSON parsing (<10ms cap) | Section 11 (64KB cap, graceful fallback) | COVERED (spec uses 64KB cap not 10ms timeout, but achieves same goal) |
| 3 | Agent state file reading | Sections 4.2, 12 | COVERED |
| 4 | Adaptive width rendering | Section 8 (wide/medium/narrow) | COVERED |
| 5 | Semantic color mapping | Sections 7, 9 | COVERED |
| 6 | TOML configuration | Sections 4.3, 10 | COVERED (with advisory notes on unused fields) |
| 7 | Settings injection | Section 15 (loop install only, not great init) | PARTIALLY COVERED (see concern #2) |

---

## Consistency Check

| Check | Result |
|-------|--------|
| SessionInfo fields match backlog JSON schema | PASS |
| AgentState fields match backlog state file schema | PASS |
| Rendering examples match width mode descriptions | PASS |
| `colored` crate version (3.0/3.1.1) and API (`set_override`) | PASS -- verified against crate source |
| Dependencies all in Cargo.toml | PASS -- no new deps required |
| Subcommand pattern matches existing mod.rs/main.rs | PASS |
| `libc` as transitive dependency claim | PASS -- verified in Cargo.lock (0.2.182) |

---

## Feasibility Check

| Check | Result |
|-------|--------|
| 5ms performance budget | REALISTIC -- two small file reads + string formatting, no async overhead |
| No tokio runtime needed | PASS -- command is synchronous |
| `catch_unwind` works (no `panic = "abort"`) | PASS -- no abort profile in Cargo.toml |
| `colored` force-enable in pipe context | PASS -- `set_override(true)` overrides TTY check |

---

## Summary

The spec is thorough, well-structured, and implementable as written. All seven backlog requirements are addressed, data structures match the schemas, and the error handling strategy is comprehensive. The advisory concerns are quality-of-life improvements around dead configuration fields (`segments`, `agent_names`), test reliability (`env::remove_var` race, global color state), and minor specification gaps (cost formatting precision, settings injection scope). None of these block implementation -- the builder can resolve them during development. **Approved for implementation.**
