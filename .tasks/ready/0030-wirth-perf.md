# 0030 Wirth Performance Report — MCP Bridge Hardening

**Agent:** Niklaus Wirth (Performance Sentinel)
**Task:** 0030 — MCP Bridge Hardening (Item E: Binary Size Mitigation)
**Date:** 2026-02-28
**Baseline file:** `.tasks/baselines/wirth-baseline.json`

---

## VERDICT: PASS

The 12.5 MB size target is met after applying LTO + strip + codegen-units=1.
Binary reduced from 13.61 MiB to 8.54 MiB — a 37.2% reduction.

---

## Measurements

### Binary Size

| Checkpoint | Size (bytes) | Size (MiB) | Delta from prev |
|---|---|---|---|
| Baseline (task 0028 pre-impl) | 10,871,632 | 10.368 | — |
| Post-task-0029 (pre-0030) | 14,269,080 | 13.608 | +3,397,448 (+31.3%) |
| Post-0030 LTO+strip (this run) | 8,957,728 | 8.543 | -5,311,352 (-37.2%) |

- **Before (post-0029, no profile settings):** 14,269,080 bytes (13.608 MiB)
- **After (LTO + strip + codegen-units=1):** 8,957,728 bytes (8.543 MiB)
- **LTO+strip savings:** 5,311,352 bytes (-37.2%)
- **vs 0028 baseline:** -1,913,904 bytes (-17.6%) — the rmcp/uuid/tracing-subscriber deps from task 0029 are now net negative relative to the old unoptimized baseline
- **Target:** < 12.5 MiB (13,107,200 bytes) — **MET with 3.96 MiB headroom**

### Build Time

| Build type | Time |
|---|---|
| Full release (LTO, cold — all deps recompiled) | 1m 20.7s |
| Incremental release (no-op, after this build) | ~1-2s expected |

The 80-second cold build is expected with `lto = true` — LTO requires a single
link pass over all codegen units. This only affects `cargo build --release` and
CI release builds. Debug builds (`cargo build`) are completely unaffected.

### New Dependencies (task 0030)

No new crate dependencies. The spec explicitly states: "no new crate dependencies
are required". Item E only modifies `Cargo.toml` profile settings.

**Task 0030 changes detected in bridge files** (from stat timestamps, not yet
committed):
- `src/mcp/bridge/backends.rs` — modified 2026-02-27 22:11 (Da Vinci added
  `display_name`, `all_backend_specs()` — Items B/C pre-work)
- `src/mcp/bridge/registry.rs` — modified 2026-02-27 22:11
- `src/mcp/bridge/server.rs` — modified 2026-02-27 22:03
- `src/mcp/bridge/tools.rs` — modified 2026-02-27 21:46

These changes were already present before this measurement run. The profile
optimization is orthogonal to all items A-D.

### Duplicate Dependencies

`cargo tree -d` reports the following duplicates (pre-existing, unchanged from
task 0029):

| Crate | Versions | Root cause |
|---|---|---|
| `getrandom` | 0.2.17, 0.3.4, 0.4.1 | ring→0.2, zip→0.3, uuid+tempfile→0.4 |
| `memchr` | 2.8.0 (×2 paths) | Different dependency paths converge; same version |
| `serde_core` | 1.0.228 (×2 paths) | Alternate derivation paths; same version |
| `serde_json` | 1.0.149 (×2 paths) | Same as above |

**Assessment:** The getrandom triple was flagged in the task 0029 Wirth report
as pre-existing. No new duplicates introduced. memchr/serde_core/serde_json
are the same version appearing via multiple paths — not a functional duplicate,
just cargo tree reporting the diamond. No bloat concern.

---

## Regressions

None. This run resolves the regression introduced by task 0029.

The task 0029 binary size increase of +31.3% (from rmcp, uuid, tracing-subscriber,
schemars) exceeded the 10% BLOCK threshold. That regression is now remediated
by the profile optimizations — the final binary is **17.6% smaller** than the
pre-0029 baseline.

---

## Compiler Warnings (release build)

Two warnings observed from Da Vinci's partial task 0030 implementation:

```
warning: fields `display_name` and `api_key_env` are never read
  --> src/mcp/bridge/backends.rs:7:9

warning: function `all_backend_specs` is never used
   --> src/mcp/bridge/backends.rs:117:8
```

**Assessment:** These are expected intermediate-state warnings. Da Vinci added
`display_name`, `all_backend_specs()` (Items B/C) to `backends.rs`, but the
consumer (`doctor.rs` Item C refactor) has not yet been applied. These warnings
will resolve when Items C and D are committed. They are not regressions — the
previous release build was warning-clean on main source.

No action required from Wirth; Da Vinci should ensure the warnings are gone
when the full task 0030 commit lands.

---

## Profile Settings Applied

Added to `/home/isaac/src/sh.great/Cargo.toml`:

```toml
[profile.release]
lto = true
strip = true
codegen-units = 1
```

- `lto = true` — full link-time optimization; cross-crate dead-code elimination
  and monomorphization deduplication. Primary contributor to size reduction.
- `strip = true` — removes debug symbols and DWARF info from the final binary.
  Acceptable for a CLI tool (users report errors via exit codes, not backtraces).
- `codegen-units = 1` — single codegen unit enables maximum inlining and
  optimization across the entire crate. Complements LTO.

**Option 4 (tracing-subscriber removal) is NOT needed.** The 12.5 MiB target
is met with 3.96 MiB headroom using only options 1-3.

---

## Resource / Allocation Patterns (task 0030 items A-D)

Scanning new bridge code in `src/mcp/bridge/`:

- `validate_path()` (Item A spec): calls `std::fs::canonicalize` once per file
  path per tool invocation. Single syscall, bounded. No allocation concern.
- `all_backend_specs()` (Item C spec): allocates a `Vec` of 5 static tuples.
  Called once per `great doctor` invocation. Trivial.
- `GreatBridge` new fields (`allowed_dirs`, `auto_approve`): one `Option<Vec<PathBuf>>`
  per bridge instance. Allocated at startup, held for process lifetime. Bounded.
- No new loops over unbounded collections. No O(n²) patterns introduced.

---

## Summary

LTO + strip + codegen-units=1 reduced the release binary from 13.61 MiB to
8.54 MiB (-37.2%), meeting the 12.5 MiB target with 3.96 MiB headroom; no
new dependencies, no new allocation concerns, and the task 0029 size regression
is fully remediated.
