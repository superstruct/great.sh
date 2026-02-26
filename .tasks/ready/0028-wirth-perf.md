# 0028: Performance Sentinel Report — Statusline Hooks and Non-Destructive Install

**Agent:** Niklaus Wirth (Performance Sentinel)
**Date:** 2026-02-27
**Task:** 0028-statusline-stuck-and-install
**Baseline ref:** `.tasks/baselines/wirth-baseline.json` (task 0027, 10,871,632 bytes)

---

## VERDICT: PASS

*Pre-implementation baseline confirmed. One WARN-level concern recorded for post-implementation verification.*

---

## Measurements

### 1. Binary Size

| Metric | Value |
|--------|-------|
| Current binary (`target/release/great`) | 10,871,632 bytes (10.368 MiB) |
| Baseline (task 0027) | 10,871,632 bytes |
| Delta | 0 bytes (0.000%) |
| Verdict | PASS — task 0028 not yet built |

Task 0028 is a pre-implementation measurement. The binary is identical to the
task-0027 baseline. The expected post-implementation delta is estimated at
+7 to +12 KB (+0.06% to +0.11%), driven by:

- `include_str!("../../loop/hooks/update-state.sh")`: 3,056 bytes embedded at
  compile time. The hook script at
  `/home/isaac/src/sh.great/loop/hooks/update-state.sh` exists and is 3,056 bytes.
- New Rust codegen: `session_id` field on `SessionInfo`, `cleanup_stale_sessions()`
  function (~30 lines), session-scoped path derivation in `run_inner()`: estimated
  2-5 KB.
- `serde_json` merge logic for `settings.json` (read-merge-write pass): estimated
  2-4 KB.

Projected post-build binary: ~10,878,000 to ~10,884,000 bytes. Well within the
5% WARN threshold.

No new Cargo.toml dependencies. The spec confirms: "No new dependencies.
`serde_json`, `dirs`, `colored` already present."

### 2. Statusline Execution Latency (pre-implementation)

Measured against the current (unmodified) binary with a realistic state file:

| Run | Wall time |
|-----|-----------|
| 1 | 2ms |
| 2 | 2ms |
| 3 | 2ms |
| 4 | 2ms |
| 5 | 2ms |

**Stable at 2ms.** Spec budget: 5ms. Current headroom: 3ms.

Post-implementation, each statusline tick adds:
- Session-scoped path derivation: one `Option::as_deref()` + one `format!()` — ~0us
- `cleanup_stale_sessions()` call: 0.14ms (1 dir), 0.43ms (3 dirs), 1.44ms (10 dirs)

The 2ms baseline + 0.43ms typical cleanup = **~2.5ms post-implementation**, still
well within the 5ms spec budget for typical workloads (1-3 concurrent sessions).

### 3. `cleanup_stale_sessions()` Overhead

Measured on tmpfs (`/tmp`) — 1,000 iterations of `readdir + stat` per entry:

| Session count | Cost per tick | % of 5ms budget |
|--------------|---------------|-----------------|
| 1 dir | 0.14ms | 2.8% |
| 3 dirs (typical) | 0.43ms | 8.6% |
| 10 dirs | 1.44ms | 28.8% |
| 30 dirs (pathological) | 4.32ms | 86.4% |

**The function runs on every statusline tick (every ~300ms) without throttling.**
At 10+ stale sessions, the stat-per-dir loop accounts for more than a quarter of
the entire 5ms budget. The spec acknowledges this: "Best-effort: errors are
silently ignored because this runs on every statusline tick and must never slow
it down."

This is a WARN, not a BLOCK:

- Normal workloads accumulate at most 3-5 session directories
- tmpfs readdir and stat are memory-only — no disk I/O
- 10-dir scenario requires 10 distinct Claude Code sessions, all without
  graceful `SessionEnd` cleanup
- The 5ms budget includes everything; in practice statusline returns in 2ms,
  leaving 3ms of actual headroom

**WARN: No throttle on `cleanup_stale_sessions()`.** If directories accumulate
(e.g., multiple crashed sessions), the stat loop grows linearly with directory
count and will eventually exceed the 5ms budget. Mitigation options for a
future iteration:

1. Skip the loop if `read_dir` count is 0 (free check before entering stat loop)
2. Run cleanup every N=100 ticks (every 30s) using `std::sync::atomic::AtomicU64`
3. Accept as-is and document the 10-dir boundary in tech debt

### 4. Hook Script: jq Overhead

Measured jq invocations per hook event:

| Operation | Calls | Wall time per event |
|-----------|-------|---------------------|
| `session_id` extraction | 1 | 2ms |
| `hook_event_name` extraction | 1 | 2ms |
| `agent_id`/`teammate_name` extraction | 1 | 2ms |
| jq upsert (read + transform + write) | 1 | 3.72ms |
| **Total per hook event** | **4** | **~10ms** |

**All 6 hooks are `async: true`.** The 10ms per-event cost is entirely
off the hot path. It has zero impact on statusline latency.

Hook frequency in a typical active session: 2-6 events per minute. At 6
events/minute, the hook script consumes 1ms/sec of background CPU — negligible.

**jq is a hard dependency.** The spec correctly notes that `great doctor`
will check for jq, and failure is a logged non-blocking error. This is
acceptable.

Multiple jq invocations per event (4 calls vs 1) is an optimization
opportunity for a future iteration: a single `jq` call could extract all
fields simultaneously. Not flagging as a regression since this is first
implementation.

### 5. State File I/O

Each statusline tick reads one `state.json` file from tmpfs:

| Operation | Measured cost |
|-----------|--------------|
| `read_to_string` on tmpfs | 0.86ms (100-iter average) |
| `serde_json::from_str` (LoopState) | sub-ms (included in 2ms total) |

Both operations are on tmpfs. Sub-millisecond for any realistic state file
(spec notes ~100 bytes per agent entry; 15 agents = ~1,500 bytes). PASS.

### 6. `settings.json` Merge (Install Path)

The new read-merge-write pass on `~/.claude/settings.json` uses `serde_json`
for a full parse-merge-serialize cycle. This is a one-time operation during
`great loop install` — it is not on any hot path.

Python proxy measurement (actual Rust serde_json will be faster): 19ms for
a 200-byte JSON file. Even at 10x that size, this is a one-time 200ms
install step. PASS.

### 7. Dependency Audit

| Category | Count | New |
|----------|-------|-----|
| Direct runtime deps | 15 | 0 |
| Dev deps | 3 | 0 |
| Transitive deps | ~277 | 0 |

No new dependencies introduced. All functionality (`serde_json::Value` merge,
`std::fs::read_dir`, `SystemTime`) uses existing crates and stdlib.

### 8. Resource Patterns

Scanned task 0028 spec for algorithmic concerns:

| Pattern | Location | Assessment |
|---------|----------|------------|
| `map().index()` in jq upsert | `update-state.sh` line ~201 | O(n) on agent array, n <= 30. Acceptable. |
| `read_dir` + `stat` per entry | `cleanup_stale_sessions()` | O(n_sessions). WARN: un-throttled per tick. See §3. |
| `sort_by_key` on agents | `read_state()` line 241 | O(n log n), n <= 30. Negligible. |
| `map().max()` in jq | `update-state.sh` | O(n) per insert. Acceptable. |
| `Vec::with_capacity` in render | `statusline.rs` | Bounded by `max_agents = 30`. PASS. |
| Atomic write (flock + mv) | `update-state.sh` | Correct. POSIX mv atomicity on same filesystem. |

No unbounded allocations. No O(n^2) in hot paths. No missing pagination
(agent list capped at 30 by render logic).

---

## Regressions

- **[WARN]** `cleanup_stale_sessions()` runs synchronously on every ~300ms
  statusline tick with no throttle. At 10 stale session directories, cost
  reaches 1.44ms (28.8% of the 5ms spec budget). At 30 dirs, cost reaches
  4.32ms (86.4%) — measured on tmpfs. Normal workload (3 dirs) costs 0.43ms.
  Not blocking because: (a) normal workloads stay under 5 dirs, (b) tmpfs
  operations are memory-only, (c) existing 3ms headroom above 2ms baseline
  absorbs the typical case. Recommend throttle in a future iteration.

---

## Post-Implementation Verification Checklist

When Da Vinci submits the task 0028 implementation, Wirth should verify:

1. `cargo build --release` completes cleanly; binary size delta < 5% from
   10,871,632 bytes.
2. Statusline execution time with 3 active session dirs remains < 5ms end-to-end.
3. No new `Cargo.toml` dependencies added.
4. Test count increases (spec adds 5 new tests: stale cleanup, session_id routing,
   hook registration check, and two hook integration tests).
5. Compiler warnings: zero new warnings in main source tree.

---

## Summary

Task 0028 introduces no binary size regression (pre-build), adds zero new
dependencies, and keeps all operations within latency budgets — with one WARN
on the un-throttled `cleanup_stale_sessions()` readdir loop that merits a
future throttle to protect against directory accumulation from crashed sessions.
