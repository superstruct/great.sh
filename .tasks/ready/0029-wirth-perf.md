# 0029: Inbuilt MCP Bridge Server — Wirth Performance Report

**Sentinel:** Niklaus Wirth (Performance Sentinel)
**Task:** 0029 — Inbuilt MCP Bridge Server
**Date:** 2026-02-27
**Status:** Pre-completion measurement (Da Vinci in progress — `server.rs` missing, build blocked)

---

## VERDICT: WARN — BINARY SIZE INCREASE EXPECTED TO EXCEED 10%

The dependency additions are sound and the schemars version conflict (Socrates concern #11)
does NOT materialise. However, the projected binary size increase from `rmcp` + its
dependency tree is substantial and must be measured after Da Vinci completes the build.
Do NOT merge until the post-build binary size is confirmed.

---

## Measurements

### Binary Size

| Metric | Value |
|--------|-------|
| Baseline binary | 10,871,632 bytes (10.368 MiB) |
| Post-0029 binary | NOT YET BUILT — compilation blocked by 2 errors |
| Estimated increase (conservative) | +3.0 MB (+29%) |
| Estimated increase (high-end) | +5.5 MB (+53%) |
| Current binary timestamp | 2026-02-27 03:29 (unchanged) |

The binary cannot be measured yet. Two compilation errors block the final link:

1. `E0583`: `src/mcp/bridge/server.rs` declared in `mod.rs` but file does not exist.
2. `E0063`: `GreatConfig` struct initialisers in `src/cli/template.rs` (line 284 and
   ~15 other sites) are missing the new `mcp_bridge` field. These require either
   `mcp_bridge: None,` on each site or a `..Default::default()` tail.

**Estimated binary size basis (from compiled rlib sizes in `target/release/deps/`):**

| Crate | rlib size | Estimated binary contribution |
|-------|-----------|-------------------------------|
| `rmcp` v0.16.0 | 6.4 MB | 1.5 – 2.5 MB |
| `schemars` v1.2.1 | 1.1 MB | 200 – 400 KB |
| `tracing-subscriber` v0.3.22 | 1.7 MB | 300 – 600 KB |
| `chrono` v0.4.44 | 1.6 MB | 200 – 400 KB |
| `futures-util` v0.3.32 | 3.0 MB | 500 – 1000 KB |
| `tokio-util` v0.7.18 | 429 KB | 100 – 200 KB |
| `uuid` v1.21.0 | 349 KB | 50 – 100 KB |
| `sharded-slab` v0.1.7 | 601 KB | 100 – 200 KB |
| `nu-ansi-term`, `smallvec`, etc. | ~500 KB combined | 50 – 150 KB |
| **Total estimated** | | **3.0 – 5.5 MB** |

Note: rlib → binary ratio is 20–40% for generics-heavy async crates after LTO dead-code
elimination. The estimate is intentionally wide. Actual measurement is mandatory.

### Compile Time

| Phase | Time |
|-------|------|
| Baseline no-op incremental | ~0.3s |
| First build with new deps (all 38 new packages) | 18.4s (real) |
| Subsequent incremental (source errors, no dep recompile) | 1.1s |
| Estimated post-completion incremental (new source files) | ~5 – 10s |

The 18.4s first-build cost is a one-time developer experience cost. CI is unaffected
(Cargo caches deps between runs via `actions/cache`).

### Dependency Count

| Metric | Baseline | Post-0029 | Delta |
|--------|----------|-----------|-------|
| Direct Cargo.toml deps | 15 | 20 + 1 target-specific | +6 |
| Cargo.lock packages | 278 | 316 | +38 |
| `cargo tree` lines | 418 | 522 | +104 |

**New direct dependencies (5 + 1 platform):**

| Dep | Version | Status | Note |
|-----|---------|--------|------|
| `rmcp` | 0.16.0 | NEW | MCP SDK; includes `macros` via default features |
| `schemars` | 1.0 (resolves to 1.2.1) | NEW | JSON Schema gen; aligned with rmcp |
| `uuid` | 1.21.0 | NEW | Task ID generation |
| `tracing-subscriber` | 0.3.22 | NEW | Stderr logging for bridge |
| `tracing` | 0.1.44 | Promoted (was transitive) | No new package |
| `libc` | 0.2.182 | Promoted (was transitive) | Unix-only; no new package |

**38 net new packages in Cargo.lock** — all attributable to `rmcp` and its tree
(`chrono`, `futures-*`, `tokio-util`, `schemars`, `async-trait`, `darling-*`,
`pastey`, `num-traits`, `iana-time-zone`, `nu-ansi-term`, `sharded-slab`, `smallvec`,
`thread_local`, `tracing-log`, `matchers`, `windows-*` platform crates).

### Duplicate Dependency Versions

```
getrandom v0.2.17  — pre-existing (via ring <- rustls <- reqwest)
getrandom v0.3.4   — pre-existing (via zip v2.4.2)
getrandom v0.4.1   — pre-existing (via tempfile, confirmed in committed Cargo.lock)
```

**No new version conflicts introduced by task 0029.** The triple-`getrandom` situation
existed before this task and was recorded in the committed `Cargo.lock` at `HEAD`.

`uuid` 1.21.0 uses `getrandom 0.4.1` — which was already present. No additional
version conflict.

`schemars` is unified at **1.2.1** — the Socrates/Humboldt concern about a 0.8/1.0
split does NOT materialise. `rmcp` 0.16 requires `schemars = "1.0"` and resolves to
1.2.1. The explicit `schemars = "1.0"` in `Cargo.toml` adds no conflict. PASS.

---

## Regressions

### WARN — Binary size increase projected > 10% (measurement pending)

The rlib evidence strongly indicates the binary will grow by 3.0 – 5.5 MB
(+29% to +53%). This exceeds the BLOCK threshold of >10%.

**However:** this is a measurement pending verdict, not a confirmed regression.
The verdict will be finalised after Da Vinci completes the build and the binary is
re-measured. The new functionality (inbuilt MCP bridge, no Node.js dependency) is
a meaningful capability addition. Socrates concern #14 specifically flags this and
recommends measurement rather than blocking.

**Mitigation options for Da Vinci to evaluate:**
1. Replace `tracing-subscriber` with `eprintln!` macro stderr logging — removes
   `tracing-subscriber` (~300–600 KB) + `nu-ansi-term`, `sharded-slab`, `smallvec`,
   `thread_local`, `tracing-log`, `matchers` (~600 KB combined). Saves ~0.9–1.2 MB.
   The codebase already uses `eprintln!` throughout; `tracing-subscriber` adds
   format/filtering infrastructure that is useful but not essential for a bridge server.
2. Add `default-features = false` to the `rmcp` dep to suppress `base64` and `macros`
   features if the `#[tool]` proc-macros are not used in the implementation:
   `rmcp = { version = "0.16", features = ["server", "transport-io"], default-features = false }`
   Note: if `rmcp-macros` `#[tool]` derive macros ARE used in `server.rs`, this option
   is not available.
3. Accept the size increase with written justification — `rmcp` replaces an entire
   Node.js runtime + npm package chain. The binary cost is justified if compared against
   the alternative (requiring users to install Node.js + run `npm install`).

### WARN — Three tokio runtime creation sites

`mcp_bridge.rs::run()` will be the third site creating a `tokio::runtime::Runtime`
(after `update.rs:26` and `template.rs:187`). This is pre-flagged in Humboldt's
technical debt section. No performance impact, but noted per Socrates concern #8:
plan `#[tokio::main]` migration before a fourth site appears.

---

## Compilation Errors Blocking Build

Da Vinci must resolve these before a binary can be measured:

1. **`src/mcp/bridge/server.rs` missing** — `mod.rs` declares `pub mod server;` but
   the file does not exist. This is the core server loop (Phase 2 in Humboldt's build
   order). All Phase 2–5 files are absent (`server.rs`, `src/cli/mcp_bridge.rs`).

2. **`GreatConfig` struct field `mcp_bridge` missing** — `src/config/schema.rs` adds
   `pub mcp_bridge: Option<McpBridgeConfig>` to `GreatConfig` (line 26), but
   `src/cli/template.rs` constructs `GreatConfig` using named fields at ~17 call sites
   (line 284, 344, 352, 369, 384, 391, 409, 416, 434, 450, 464, 475, 493, 504, 524,
   538, 573, 580, 594, 604) without the new field. Each needs either:
   - `mcp_bridge: None,` added explicitly, or
   - A `..Default::default()` tail on the struct literal (requires `GreatConfig: Default`).
   `McpBridgeConfig` is `#[derive(Default)]` so `GreatConfig` can derive it too.

---

## Functional Assessment (from spec/scout analysis)

**What the new deps enable:**
- `rmcp` — replaces hand-rolled JSON-RPC 2.0 protocol code; eliminates `protocol.rs`
  and `handlers.rs` as separate files; `#[tool]` proc-macro generates JSON Schema automatically.
- `schemars` — unified with `rmcp`'s requirement; no separate schema crate needed.
- `uuid` — task ID generation in `registry.rs`; adds `getrandom` but 0.4 was pre-existing.
- `tracing-subscriber` — stderr-only logging for the bridge daemon; does not affect
  CLI commands that don't invoke the bridge.
- `libc` (explicit) — formalises the dependency used for `setpgid`/`killpg` in
  `registry.rs`; was already transitive via `tokio`.

**Functionality avoided by NOT using `process-wrap`** (as the spec explicitly chose):
- `process-wrap` was removed from the spec to reduce complexity. This saves ~200KB and
  avoids a new framework dep. The `libc` unsafe blocks are the cost of this trade-off.

**No duplicate functionality:** none of the 5 new direct deps duplicate existing
crate capabilities:
- `rmcp` — no existing MCP SDK in the codebase.
- `schemars` — no existing JSON Schema generation.
- `uuid` — no existing UUID generation (`rand` is not in deps).
- `tracing-subscriber` — `tracing` was transitive but subscriber was absent.
- `libc` — was transitive; now explicit. Correct hygiene.

---

## Required Actions Before Merge

1. **[BLOCK until measured]** Da Vinci must complete the build and re-run:
   ```
   cargo build --release 2>&1 | tail -3
   ls -la target/release/great
   ```
   Report the actual binary size and delta. If increase > 10%, evaluate mitigation
   options (tracing-subscriber replacement, rmcp default-features = false).

2. **[BLOCK]** Fix 2 compilation errors:
   - Create `src/mcp/bridge/server.rs` (Phase 2 of build plan).
   - Add `mcp_bridge: None,` to all `GreatConfig { ... }` literals in
     `src/cli/template.rs` (17 sites), or add `Default` derive to `GreatConfig`
     and use `..Default::default()` tail.

3. **[WARN]** Evaluate replacing `tracing-subscriber` with `eprintln!` stderr output.
   The bridge's `--log-level` flag can be implemented trivially with an `AtomicBool`
   or a simple env-var check. Removes ~0.9–1.2 MB from the binary at the cost of
   structured logging format.

---

## Summary

Dependencies are added correctly and the schemars version conflict does not occur.
38 new packages enter the lockfile, no new version conflicts are introduced. Binary
size impact is projected at +29% to +53% — a significant but potentially justifiable
cost for eliminating a Node.js runtime requirement — and must be confirmed by
measurement after Da Vinci resolves the 2 compilation errors blocking the final link.
