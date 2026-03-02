# Nielsen UX Review — Platform Detection Engine
**Task:** #4
**Inspector:** Jakob Nielsen
**Date:** 2026-02-20
**Verdict:** NO BLOCKERS — Task #4 COMPLETE

---

## Methodology

Four user journeys were walked against the live binary (`cargo run --` → `target/debug/great`). All commands were run from both a directory without `great.toml` (WSL home, repo root) and a synthetic `/tmp/great-test-config2/` with a fully populated config. Output was captured on stdout and stderr separately to verify stream correctness.

---

## Journey 1: `cargo run -- status`

**Command run:** `great status` (no config present), then `great status` (with great.toml in cwd)

**Output (no config):**
```
great status

ℹ Platform: WSL Ubuntu 24.04 (X86_64)
⚠ No great.toml found. Run `great init` to create one.
```

**Output (with config):**
```
great status

ℹ Platform: WSL Ubuntu 24.04 (X86_64)
ℹ Config: /tmp/great-test-config2/great.toml

Tools
✓   node 22 (v22.22.0)
✗   python 3.12 — not installed
✓   gh latest (gh version 2.85.0 (2026-01-14))
✓   pnpm latest (10.26.0)

Agents
ℹ   claude (anthropic/claude-opus-4-5)

MCP Servers
✓   filesystem (npx)

Secrets
✗   ANTHROPIC_API_KEY — missing
✗   GITHUB_TOKEN — missing
```

**Heuristic analysis:**

| # | Heuristic | Finding | Severity |
|---|-----------|---------|----------|
| 1 | Visibility of system status | Platform detected and shown: "WSL Ubuntu 24.04 (X86_64)". Clear, immediately visible. | PASS |
| 2 | Match between system and real world | "WSL Ubuntu 24.04" is human-readable. Arch shown as "X86_64" — Rust Debug format, not conventional "x86_64". | IMPROVEMENT |
| 4 | Recognition over recall | Tool status lines (`node 22 (v22.22.0)`) are self-explanatory. Version comparison between declared and detected is implicit — no explicit match/mismatch indicator. | IMPROVEMENT |
| 5 | Aesthetic and minimalist design | Output is clean. Sections are clearly delineated with blank lines and bold headers. | PASS |

**Issue UX-01 (P3 — Aesthetic):** Architecture displayed as `X86_64` (Rust enum Debug format) rather than conventional `x86_64`. Affects both `status` and `doctor` output. Users familiar with hardware terminology expect lowercase with underscore: `x86_64`, `aarch64`. The `{:?}` Debug formatter is leaking into user-facing strings.

Affected code — `/home/isaac/src/sh.great/src/platform/mod.rs`, line 221 (`doctor.rs`):
```rust
pass(result, &format!("Architecture: {:?}", info.platform.arch()));
```
And `status.rs` line 166 (JSON output):
```rust
r#"{{"platform": "{}", "arch": "{:?}", "shell": "{}"}}"#,
```
And `mod.rs` lines 45–51 (`display_detailed`):
```rust
format!("macOS {} ({:?})", ver, arch)
// ... all variants use {:?} for arch
```

**Issue UX-02 (P3 — Recognition):** In the Tools section, when a tool is installed, `great status` shows declared version alongside detected version (e.g., `node 22 (v22.22.0)`). There is no visual indicator of whether these match. A user cannot immediately tell if the installed version satisfies the declared constraint. For example, `cargo 1.89` declared with `cargo 1.89.0` detected: are these compatible? No feedback is given. Low priority but worth tracking.

---

## Journey 2: `cargo run -- status --json`

**Command run:** `great status --json`

**Output (to stdout):**
```json
{"platform": "wsl", "arch": "X86_64", "shell": "/bin/bash"}
```

**Parsed successfully** with `python3 -m json.tool`. Valid JSON.

**Heuristic analysis:**

| # | Heuristic | Finding | Severity |
|---|-----------|---------|----------|
| 1 | Visibility of system status | Platform present. | PASS |
| 5 | Minimalist design | JSON is compact — appropriate for machine consumption. | PASS |

**Issue UX-03 (P2 — Consistency):** The JSON output omits capabilities, distro detail, and config status that the human-readable output provides. A script consuming `--json` to determine platform capabilities (e.g., "does this machine have homebrew?") cannot do so. The `PlatformInfo` struct has a `capabilities` field with exactly this data, but `run_json()` in `status.rs` only serializes `platform`, `arch`, and `shell`.

Affected code — `/home/isaac/src/sh.great/src/cli/status.rs`, lines 164–172:
```rust
fn run_json(info: &platform::PlatformInfo) -> Result<()> {
    println!(
        r#"{{"platform": "{}", "arch": "{:?}", "shell": "{}"}}"#,
        info.platform,
        info.platform.arch(),
        info.shell
    );
    Ok(())
}
```

`PlatformInfo` already derives `Serialize` (via `detection.rs`). The fix is a one-liner: `serde_json::to_string(info)`. This would expose capabilities, `is_root`, distro, and version in structured form. Recommend filing as P2 since `--json` is the scripting interface and its current anemic output limits automation use cases.

**Issue UX-04 (P3 — Aesthetic):** The `arch` field in JSON output is `"X86_64"` (Rust enum name) rather than the conventional `"x86_64"`. This breaks the principle that JSON API field values should be stable, lowercase, and conventional. A CI script checking `arch == "x86_64"` would fail.

---

## Journey 3: `cargo run -- doctor`

**Output:**
```
great doctor

Platform
✓ Platform detected: WSL Ubuntu 24.04 (X86_64)
✓ Architecture: X86_64
✓ Running as regular user
✓ Homebrew (Linuxbrew): installed (primary package manager)
✓ apt: available (fallback for system packages)

Essential Tools
✓ Git version control: git (git version 2.52.0)
✓ curl HTTP client: curl (curl 8.18.0 (x86_64-pc-linux-gnu) ...)
✓ GitHub CLI: gh (gh version 2.85.0 (2026-01-14))
✓ Node.js runtime: node (v22.22.0)
✓ npm package manager: npm (11.10.0)
✓ pnpm package manager: pnpm (10.26.0)
✓ Rust toolchain: cargo (cargo 1.89.0 (c24e10642 2025-06-23))
⚠ mise: not installed — recommended for managing tool versions. Install: https://mise.jdx.dev
✓ bat (cat with syntax highlighting): installed
✓ uv (fast Python package manager): installed
✓ Deno runtime: installed
✓ Starship prompt: installed

AI Agents
✓ Claude Code: installed
✓ OpenAI Codex CLI: installed
⚠ Anthropic API key: not set (ANTHROPIC_API_KEY)
✓ OpenAI API key: set

Configuration
⚠ great.toml: not found — run `great init` to create one
✓ ~/.claude/ directory: exists

Shell
✓ Shell: /bin/bash
✓ ~/.local/bin in PATH

Summary
ℹ   22 passed, 3 warnings, 0 errors

✓ No critical issues found.
```

**Heuristic analysis:**

| # | Heuristic | Finding | Severity |
|---|-----------|---------|----------|
| 1 | Visibility of system status | All sections display, counts summarized. | PASS |
| 2 | Match between system and real world | Platform names readable. `X86_64` arch leaks again. | IMPROVEMENT |
| 3 | Error prevention | Platform detection never crashes; fallback to `Platform::Unknown`. | PASS |
| 4 | Recognition over recall | Tool names include human display name ("Git version control", "curl HTTP client") alongside binary name. Excellent pattern. | PASS |
| 5 | Minimalist design | Version strings for curl are very long (entire build info line). Clutters the output. | IMPROVEMENT |

**Issue UX-05 (P3 — Minimalist Design):** The curl version line is excessively long:
```
✓ curl HTTP client: curl (curl 8.18.0 (x86_64-pc-linux-gnu) libcurl/8.18.0 OpenSSL/3.6.0 zlib/1.3.1 brotli/1.2.0 ...)
```
The `get_command_version()` function takes only the first line of `--version` output, which for curl includes the full build configuration string. This is too verbose for a status display. The version token (e.g., `8.18.0`) should be extracted, not the full first line. Pattern: most tools follow `<name> <version>` in their first line; a simple split and take-second-token would work for the common case.

Affected code — `/home/isaac/src/sh.great/src/cli/doctor.rs`, `get_command_version()`. Same logic in `status.rs`.

**Issue UX-06 (P3 — Consistency):** The Summary line uses `ℹ` (info prefix) for counts: `ℹ 22 passed, 3 warnings, 0 errors`. Counts are not "informational" in the ℹ sense — they are a result. Using a neutral format or `✓` for the summary would better match user expectations. Minor tone issue.

---

## Journey 4: Error Resilience

**Scenario A: No great.toml present**

Command: `great status` from `/home/isaac/src/sh.great` (no great.toml in project root or ancestors up to home).

Output:
```
great status

ℹ Platform: WSL Ubuntu 24.04 (X86_64)
⚠ No great.toml found. Run `great init` to create one.
```

**Assessment:** Excellent degraded-mode behavior. Platform detection still runs. Warning is actionable (`Run \`great init\` to create one.`). Exit code is 0 (success). Heuristic #5: minimal output, no panic, no stack trace.

**Scenario B: Malformed great.toml**

Test: Created a `great.toml` with `[tools.runtimes]` as a table (violates the `flatten` schema that expects top-level string values).

Output:
```
great status

ℹ Platform: WSL Ubuntu 24.04 (X86_64)
ℹ Config: /tmp/great-test-config/great.toml
✗ Failed to parse config: TOML parse error at line 1, column 2
  |
1 | [tools.runtimes]
  |  ^^^^^
invalid type: map, expected a string
```

**Assessment:** Parse error is surfaced with file location and column pointer from the TOML parser — clear and actionable. Platform info still displayed. The TOML error message format (`invalid type: map, expected a string`) is technically accurate but assumes user knows the schema. Heuristic #9 (Help users recognize/diagnose/recover from errors) — partially satisfied. A hint like "Tip: `[tools]` uses flat keys like `node = \"22\"`, not subtables" would improve recovery. Filed as P3.

**Issue UX-07 (P3 — Error Recovery):** Config parse errors display raw TOML library messages without additional context guiding the user to the correct schema format. Users encountering `invalid type: map, expected a string` may not know that `[tools.runtimes]` should be `[tools]` with `node = "22"` style entries. A brief schema hint in the error path would close the loop.

---

## Stream Correctness (Bonus Check)

All human-readable output (`eprintln!`) correctly goes to **stderr**. JSON output (`println!`) correctly goes to **stdout**. This is the correct UNIX convention for scriptable CLIs — human output on stderr keeps stdout clean for pipes and redirects. The 8 blank bytes on stdout from `doctor` are harmless `println!()` calls used as spacers (they use `println!` not `eprintln!`, which is a minor inconsistency but not a UX issue for the user).

---

## Summary

| ID | Description | Severity | Heuristic |
|----|-------------|----------|-----------|
| UX-01 | `X86_64` / `Aarch64` enum names shown instead of conventional `x86_64` / `aarch64` | P3 | #2 Match with real world |
| UX-02 | No version constraint satisfaction feedback in Tools section | P3 | #4 Recognition over recall |
| UX-03 | `--json` omits capabilities, distro detail, config info | P2 | #1 Visibility of system status |
| UX-04 | `arch` field in JSON is `"X86_64"` not `"x86_64"` — breaks script consumers | P2 | #2 Match with real world |
| UX-05 | curl (and similar tools) full build string on first --version line, very long | P3 | #5 Minimalist design |
| UX-06 | Summary count uses `ℹ` prefix — tone mismatch | P3 | #5 Minimalist design |
| UX-07 | Config parse errors lack schema hints for recovery | P3 | #9 Help recover from errors |

**BLOCKERS:** None.

**Task #4 status: COMPLETE.**

P2 issues (UX-03, UX-04) filed for Nightingale backlog. P3 issues (UX-01, UX-02, UX-05, UX-06, UX-07) filed as P3 discoveries.

---

## Files Inspected

- `/home/isaac/src/sh.great/src/cli/status.rs`
- `/home/isaac/src/sh.great/src/cli/doctor.rs`
- `/home/isaac/src/sh.great/src/cli/output.rs`
- `/home/isaac/src/sh.great/src/platform/mod.rs`
- `/home/isaac/src/sh.great/src/platform/detection.rs`
- `/home/isaac/src/sh.great/src/config/schema.rs`
