# Review 0038 -- SIGPIPE/EPIPE Graceful Handling

**Reviewer:** Socrates | **Date:** 2026-03-04 | **Round:** 1

---

VERDICT: APPROVED

## Elenchus

### 1. Does the fix solve the problem?

Yes. `#[cfg_attr(unix, unix_sigpipe = "sig_dfl")]` restores the POSIX default SIGPIPE disposition. When a pipe reader closes, the kernel delivers SIGPIPE and terminates the process (exit 141) before `println!` ever returns an `io::Error`. The anyhow error path is never reached. This addresses every `println!` call site in the codebase (90+ occurrences across doctor.rs, status.rs, diff.rs, vault.rs, mcp.rs, apply.rs, loop_cmd.rs, statusline.rs, template.rs, sync.rs).

### 2. MCP bridge server (long-lived process on stdio)

The backlog explicitly scopes this out: "Handling SIGPIPE in the MCP bridge server ... track separately if needed." I verified the concern is benign:

- The bridge server uses `rmcp::transport::io::stdio()` (server.rs line 620) for its own stdout transport.
- If the MCP client disconnects, SIGPIPE termination is the correct behavior -- an orphaned stdio server with no client should exit.
- The bridge's stderr logging (`tracing_subscriber` with `std::io::stderr` writer) and child-process stdout capture (`Stdio::piped()` on spawned backends) are unaffected by the SIGPIPE disposition on the bridge's own stdout fd.
- No bridge code catches `BrokenPipe` to perform cleanup; there is no cleanup logic that would be skipped.

No concern here.

### 3. Platform guard correctness

`#[cfg_attr(unix, unix_sigpipe = "sig_dfl")]` compiles to nothing on Windows. Windows has no SIGPIPE concept. The attribute is correctly scoped. The backlog confirms this is the expected approach.

### 4. Stability claim

The spec says "stabilized in Rust 1.75.0"; the backlog says "Rust 1.73+". The spec is correct -- `unix_sigpipe` was stabilized in Rust 1.75.0 (December 2023). The backlog's "1.73+" is imprecise but not contradictory (1.75 >= 1.73). No `rust-version` field in Cargo.toml, and transitive deps (rmcp 0.16, schemars 1.0) require toolchains well past 1.75. No compatibility concern.

### 5. Test plan sufficiency

The spec correctly notes that SIGPIPE tests cannot be automated in `cargo test` / `assert_cmd` because they require real pipe + signal delivery. The manual verification commands are sufficient and match the backlog's acceptance criteria exactly. `cargo test` for regression coverage is included.

### 6. Could any subcommand rely on catching BrokenPipe?

Verified: zero matches for `SIGPIPE`, `sigpipe`, `BrokenPipe`, or `broken_pipe` anywhere in `src/`. No code catches or handles this error. The fix breaks nothing.

## Concerns

```
{
  "gap": "None identified",
  "question": "N/A",
  "severity": "N/A",
  "recommendation": "N/A"
}
```

No BLOCKING or ADVISORY concerns. The spec is minimal, correct, and complete for a one-line fix. The backlog and spec are aligned on approach, scope, and acceptance criteria.

## Summary

A clean, one-line fix that restores standard POSIX behavior; the spec is precise, correctly scoped, and ready for Da Vinci to implement.
