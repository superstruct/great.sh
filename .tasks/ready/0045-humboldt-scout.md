# 0045 -- Humboldt Scout Report: `--only` and `--skip` flags for `great apply`

| Field | Value |
|---|---|
| Task ID | 0045 |
| Scout | Humboldt |
| Date | 2026-03-05 |
| Spec | `0045-apply-only-skip-spec.md` (APPROVED by Socrates) |

---

## 1. File Map

| File | Lines | Role |
|---|---|---|
| `/home/isaac/src/sh.great/src/cli/apply.rs` | 1060 | Primary: add enum, fields, helper, gates |
| `/home/isaac/src/sh.great/tests/cli_smoke.rs` | 2325 | Add 6 integration tests after line 853 |
| `/home/isaac/src/sh.great/src/main.rs` | 52 | Read-only — no changes needed |
| `/home/isaac/src/sh.great/src/cli/mod.rs` | 84 | Read-only — no changes needed |
| `/home/isaac/src/sh.great/Cargo.toml` | 44 | Read-only — clap already has `derive` feature |

**No new files.** Two files modified.

---

## 2. apply.rs Structure

### `Args` struct — lines 354-372

```rust
#[derive(ClapArgs)]          // line 354
pub struct Args {            // line 355
    config: Option<String>,  // line 357-359
    dry_run: bool,           // line 361-363
    yes: bool,               // line 365-367
    // ← INSERT only + skip fields here (after line 367, before line 368)
    non_interactive: bool,   // line 369-372 (#[arg(skip)] field)
}                            // line 372
```

### `ApplyCategory` enum insertion point

Insert the new enum **before line 354** (the `#[derive(ClapArgs)]` of `Args`). There is a blank line at 353 and a doc-comment block starts at 352. The correct insertion is between the `tool_install_spec` / `install_with_spec` helper functions (which end around line 350) and the `Args` derive block.

### `run()` function — lines 380-907

Sectioned precisely:

| # | Header comment | Lines | Category |
|---|---|---|---|
| 1 | "Load config" | 385-397 | **unconditional** |
| 2 | "Detect platform" | 394-402 | **unconditional** |
| 2a | "Pre-cache sudo credentials" | 408-431 | `tools` |
| 2b | `bootstrap::ensure_prerequisites()` | 434 | `tools` |
| 2c | "Ensure Homebrew is available" | 440-492 | `tools` |
| 3 | "Install runtimes via mise" | 495-570 | `tools` (inside `if let Some(tools)`) |
| 4 | "Install CLI tools" | 572-646 | `tools` (inside same `if let Some(tools)`) |
| 5 | "Configure MCP servers" | 649-724 | `mcp` |
| 5a | "Register MCP bridge" | 726-778 | `mcp` |
| 5b | "Install bitwarden-cli" | 780-803 | `tools` (but see risk note below) |
| 5c | "Starship + Nerd Font" | 805-818 | `tools` |
| 6 | "Check secrets" | 820-838 | `secrets` |
| 7 | "Platform-specific tools" | 840-886 | `tools` |
| 8 | "Docker" | 888-889 | `tools` |
| 9 | "Claude Code" | 891-894 | `tools` |
| 10 | "System tuning" | 896-897 | `tools` |
| — | Summary | 899-906 | **unconditional** |

### Critical nesting observation (Socrates advisory #6)

Sections 3 and 4 are both **inside** `if let Some(tools) = &cfg.tools { ... }` (line 495), which itself ends at line 647. The `should_apply(Tools)` check must go **inside** this existing guard, not outside it. The recommended approach (per spec section 3.1) is a single large `if should_apply(ApplyCategory::Tools, ...)` block that wraps lines 408-897 (sections 2a through 10), with the `if let Some(tools)` guard remaining inside it unchanged.

---

## 3. Existing Patterns to Follow

### No `ValueEnum` exists in the codebase yet

There are zero uses of `clap::ValueEnum` across all of `src/`. This is the first. The clap dependency already has the `"derive"` feature (`clap = { version = "4.5", features = ["derive", "env", "string"] }` in Cargo.toml line 11), which is what `ValueEnum` requires — no Cargo.toml change needed.

### Clap derive pattern (from `cli/apply.rs` line 6)

```rust
use clap::Args as ClapArgs;
```

For the new enum, the import needed is `clap::ValueEnum`. Add it to the existing use line or add a new one:

```rust
use clap::{Args as ClapArgs, ValueEnum};
```

### Nested subcommand enum pattern (from `cli/template.rs` lines 13-24)

`template.rs` uses `#[derive(Subcommand)]` on `TemplateCommand`. `ValueEnum` is structurally similar but for flat flag values, not subcommands. The derive macro pattern is the same.

### `#[arg(skip)]` hidden field (apply.rs line 369-371)

```rust
#[arg(skip)]
pub non_interactive: bool,
```

The new `only` and `skip` fields are normal `#[arg(long, ...)]` fields — NOT skip fields. Insert them after the `yes` field (line 367) and before the `non_interactive` skip field.

### `conflicts_with` pattern

This codebase has not used `conflicts_with` before. The clap attribute form is:

```rust
#[arg(long, value_delimiter = ',', conflicts_with = "skip")]
pub only: Vec<ApplyCategory>,
```

The string `"skip"` must match the field name exactly (Rust field name, not the `--skip` flag name — they happen to be the same here).

---

## 4. Test Patterns

### Apply test section in `cli_smoke.rs`

Location: lines 781-853. Section delimiter at line 781:

```
// -----------------------------------------------------------------------
// Apply
// -----------------------------------------------------------------------
```

The section ends at line 853 (`}`), followed by a blank line at 854, then the Statusline delimiter at 855.

**Insert new apply tests at line 854** (the blank line after the last apply test closes), before the Statusline section delimiter.

### Existing test structure to clone

```rust
#[test]
fn apply_dry_run_with_config() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"[project]\nname = "test"\n"#,
    ).unwrap();

    great()
        .current_dir(dir.path())
        .args(["apply", "--dry-run"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Dry run mode"));
}
```

All tests use:
- `TempDir::new().unwrap()` for isolation
- `great().current_dir(dir.path()).args([...])` invocation
- `.assert().success()` or `.assert().failure()`
- `predicate::str::contains(...)` for output assertions
- `predicate::str::contains(...)` is also negatable: `.stdout(predicate::str::contains("CLI Tools").not())`

### Note on stdout vs stderr

Current apply tests check `.stderr(...)` for "Dry run mode" (because `output::warning` writes to stderr). The section headers like "CLI Tools" and "MCP Servers" are printed by `output::header`, which writes to stderr as well (confirmed by existing test `apply_dry_run_shows_prerequisites` checking `.stderr(predicate::str::contains("System Prerequisites"))`).

**New tests asserting absence of section headers must use `.stderr(...).not()`**, not `.stdout(...).not()`.

### Exit code 2 pattern for clap errors

Clap argument errors (conflicts, invalid values) produce exit code 2. `assert_cmd` uses `.assert().failure()` which accepts any non-zero exit. For precise exit code 2, use:

```rust
.assert()
.code(2)
.stderr(predicate::str::contains("cannot be used with"));
```

The current codebase uses `.failure()` without `.code()` specificity (e.g., `apply_no_config_fails` at line 826). Following spec test 5 and 6, using `.failure()` with the stderr content check is sufficient.

---

## 5. Dependencies

| Crate | In Cargo.toml | Needed for |
|---|---|---|
| `clap` 4.5 with `derive` feature | Yes (line 11) | `ValueEnum` derive — already available |
| `anyhow` | Yes (line 12) | Existing error handling — unchanged |

No new crate dependencies. `ValueEnum` is part of the `clap::derive` feature already present.

---

## 6. Risk Areas

### Risk 1: Section 5b (bitwarden-cli) category boundary

Section 5b (lines 780-803) installs bitwarden-cli when `cfg.secrets.provider == "bitwarden"`. It is triggered by the secrets config but spec gates it under `tools`. If a user runs `--only secrets`, bitwarden-cli will not be auto-installed. Socrates flagged this as advisory. The spec decision is to gate it under `tools` — Da Vinci should implement as specced and add a comment noting the caveat.

### Risk 2: `tools` gate around sudo block (lines 408-431)

The `_sudo_keepalive` variable is declared inside the sudo block. If the tools gate wraps lines 408-431, the `_sudo_keepalive` binding goes out of scope before it is needed. However, since the keepalive only matters during tool installation (which is also inside the tools gate), this is safe — scope the binding inside the gate block.

### Risk 3: Single large `if should_apply(Tools)` block nesting depth

Wrapping all of sections 2a through 10 in a single `if should_apply(ApplyCategory::Tools, &args.only, &args.skip) { ... }` creates a large block (~489 lines). The alternative is individual gates per section. The spec says "each gate is a single `if should_apply(...)` check at the outermost level of each section" — this implies per-section gates, not one big block. Either approach works but the per-section approach is cleaner for review. Per-section approach for tools:

- Gate 1: around lines 408-431 (sudo)
- Gate 2: around line 434 (prerequisites call)
- Gate 3: around lines 440-492 (Homebrew)
- Gate 4: inside `if let Some(tools)` at line 495, around lines 497-570 (runtimes)
- Gate 5: inside `if let Some(tools)` at line 495, around lines 573-646 (CLI tools)
- Gate 6: around lines 780-803 (bitwarden-cli)
- Gate 7: around lines 805-818 (starship)
- Gate 8: around lines 840-886 (platform tools)
- Gate 9: around line 889 (docker)
- Gate 10: around lines 892-894 (Claude Code)
- Gate 11: around line 897 (tuning)

**Recommendation:** A single wrapping `if should_apply(ApplyCategory::Tools, ...)` from line 408 to 897 (before the summary) is cleaner and less error-prone than 11 individual gates. The internal structure (config-presence guards, nesting) is unchanged. This is consistent with "do not nest gates inside helper functions."

### Risk 4: `Agents` category no-op output

Socrates advisory #7: `--only agents --dry-run` succeeds silently. The spec says insert an empty comment block. Consider emitting `output::info("  agents: no provisioning configured")` inside the agents gate to confirm the category was recognized. This prevents user confusion.

### Risk 5: `should_apply` placement

Spec says to place `should_apply` "after the `Args` struct, before `pub fn run()`." The `run()` function begins at line 380. The `Args` struct ends at line 372. There is a doc-comment block at lines 374-379. Insert `should_apply` between line 372 and 374 (i.e., after the closing `}` of `Args`, before the doc-comment for `run()`).

---

## 7. Recommended Build Order

1. Add `use clap::{Args as ClapArgs, ValueEnum};` import (update line 6)
2. Insert `ApplyCategory` enum before line 354 (before `#[derive(ClapArgs)]`)
3. Add `only` and `skip` fields to `Args` struct after the `yes` field (after line 367)
4. Insert `should_apply` free function between `Args` (line 372) and `run()` doc-comment (line 374)
5. Apply tools gate — single `if should_apply(ApplyCategory::Tools, &args.only, &args.skip)` wrapping lines 408-897 (the large tools block)
6. Apply mcp gate — `if should_apply(ApplyCategory::Mcp, ...)` wrapping lines 649-778 (inside the tools-gated region)
7. Apply agents gate — empty `if should_apply(ApplyCategory::Agents, ...)` block with comment, placed after MCP gate and before secrets gate
8. Apply secrets gate — `if should_apply(ApplyCategory::Secrets, ...)` wrapping lines 820-838
9. Add unit tests to `mod tests` block at bottom of `apply.rs`
10. Add 6 integration tests to `tests/cli_smoke.rs` after line 853
11. `cargo build` — compiler will report if any construction sites were broken
12. `cargo clippy` — confirm no new warnings
13. `cargo test` — confirm all 4 existing apply tests pass plus 6 new ones

---

## 8. Technical Debt Noted (Do Not Fix in This Task)

- `should_apply` adds a seventh location where `ApplyCategory` variants are referenced. If categories expand (sub-categories per Socrates advisory #1), the current flat enum approach will need revisiting.
- The `tools` category is a coarse umbrella (11+ sections). Sub-categories (`tools.runtimes`, `tools.docker`) are deferred post-V1 per Socrates advisory #1.
