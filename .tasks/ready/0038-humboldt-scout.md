# Scout Report 0038 -- SIGPIPE Handling

**Scout:** Humboldt | **Date:** 2026-03-04 | **Scope:** Minimal (one-line change)

---

## 1. Exact Change Location

**File:** `src/main.rs`

Insert one line immediately before `fn main()` at line 13:

```rust
// Line 12 (new):
#[cfg_attr(unix, unix_sigpipe = "sig_dfl")]
// Line 13 (was 13, now 14):
fn main() -> Result<()> {
```

The file is 45 lines total. No other line in `src/main.rs` is touched.

---

## 2. Existing SIGPIPE / BrokenPipe Handling

Grep across all `*.rs` files finds **zero** existing SIGPIPE or BrokenPipe handling anywhere in the codebase. The word "signal" appears only in:

- `src/mcp/bridge/registry.rs:143` -- `setpgid` / `killpg` (process group management for child MCP servers, unrelated)
- `src/cli/statusline.rs:258` -- "signals renderer" (English prose, unrelated)
- `src/cli/sudo.rs:11,148` -- drop-signals-stop pattern for a keepalive thread (unrelated)

Nothing to reconcile. This is a net-new behavior.

---

## 3. Test Infrastructure

Tests live in `tests/cli_smoke.rs` using `assert_cmd` + `predicates`. The pattern is:

```rust
Command::cargo_bin("great").expect("binary exists")
    .arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::contains("..."));
```

**SIGPIPE tests cannot be automated here.** The spec correctly notes this: signal delivery requires a real OS pipe between two processes, which `assert_cmd` does not model. `assert_cmd` captures stdout internally with no downstream reader that closes early.

Verification is manual shell commands as listed in the spec. No new test file is needed and none should be added -- a test that shells out `great status --json | head -0` and checks exit code would be fragile and platform-dependent.

---

## 4. Cargo.toml / Rust Version

`Cargo.toml` has `edition = "2021"` and **no `rust-version` field**. The `unix_sigpipe` attribute was stabilized in Rust 1.75.0 (2023-12-28). The project's heavy transitive dependencies (rmcp 0.16, schemars 1.0) already require a toolchain well past 1.75. No compatibility risk.

---

## 5. Risk Assessment

| Risk | Severity | Notes |
|---|---|---|
| Attribute not recognized | None | Stabilized 1.75; project is well past that |
| Windows breakage | None | `cfg_attr(unix, ...)` is a no-op on Windows |
| MCP bridge regression | None | Spec explicitly scopes it out; bridge is a long-lived server |
| Behavior change for non-pipe runs | None | `sig_dfl` only fires when a pipe reader closes; normal invocations unaffected |

Zero risk. This is a one-line, well-understood Rust idiom.

---

## Recommended Build Order

1. Edit `src/main.rs` -- insert the attribute (one line)
2. `cargo check` -- confirm no compile error
3. `cargo test` -- confirm no regressions
4. Manual pipe smoke tests from spec section "Test Plan"

No dependency changes. No other files change.
