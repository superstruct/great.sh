# Humboldt Scout Report — 0010 GROUP I: Dead Code and Safety Cleanup

**Date:** 2026-02-25
**Commit:** 544e795
**Spec:** `.tasks/ready/0010-GROUP-I-dead-code-spec.md`

---

## 1. Clippy Status (Verified)

```
cargo clippy                              -> 0 warnings (clean)
```

The spec's three-lint table is already documented; I re-ran default clippy to
confirm the baseline. No surprises.

---

## 2. Complete Annotation Inventory (Exact Lines)

### `#[allow(dead_code)]` — 15 sites

| # | File | Line | Item | Tests? | Production? |
|---|------|------|------|--------|-------------|
| 1 | `src/config/mod.rs` | 61 | `pub fn data_dir()` | Yes (line 105) | No |
| 2 | `src/config/mod.rs` | 69 | `pub fn config_dir()` | Yes (line 113) | No |
| 3 | `src/sync/mod.rs` | 6 | `pub enum SyncStatus` | No | No |
| 4 | `src/sync/mod.rs` | 17 | `pub struct SyncBlob` | No | No |
| 5 | `src/sync/mod.rs` | 39 | `pub fn import_config()` | Yes (line 138) | No |
| 6 | `src/mcp/mod.rs` | 45 | `McpJsonConfig::save()` | Yes (line 262) | No |
| 7 | `src/mcp/mod.rs` | 53 | `McpJsonConfig::add_server()` | Yes (line 150) | No |
| 8 | `src/mcp/mod.rs` | 69 | `McpJsonConfig::server_names()` | Yes (line 233) | No |
| 9 | `src/mcp/mod.rs` | 98 | `pub fn user_mcp_path()` | Yes (line 278) | No |
| 10 | `src/platform/package_manager.rs` | 17 | `PackageManager::installed_version()` | No | No |
| 11 | `src/platform/package_manager.rs` | 24 | `PackageManager::update()` | No | No |
| 12 | `src/platform/runtime.rs` | 35 | `MiseManager::version()` | Yes (line 336) | No |
| 13 | `src/cli/statusline.rs` | 19 | `SessionInfo::model` | — | Deserialized |
| 14 | `src/cli/statusline.rs` | 24 | `SessionInfo::workspace` | — | Deserialized |
| 15 | `src/cli/statusline.rs` | 32 | `LoopState::loop_id` | — | Deserialized |
| 16 | `src/cli/statusline.rs` | 42 | `AgentState::name` | — | Deserialized |

**Items 3 and 4** (`SyncStatus`, `SyncBlob`): confirmed zero callers anywhere
in the codebase. These are the only items to DELETE.

### `#[allow(unused_imports)]` — 3 sites

| # | File | Line | Items in re-export |
|---|------|------|--------------------|
| 1 | `src/config/mod.rs` | 8 | `AgentConfig, ConfigMessage, GreatConfig, McpConfig, PlatformConfig, PlatformOverride, ProjectConfig, SecretsConfig, ToolsConfig` |
| 2 | `src/platform/mod.rs` | 5 | `command_exists, detect_architecture, detect_platform, detect_platform_info, Architecture, LinuxDistro, Platform, PlatformCapabilities, PlatformInfo` |
| 3 | `src/platform/mod.rs` | 11 | `MiseManager, ProvisionAction, ProvisionResult` |

---

## 3. External Usage Audit (Socrates Advisory Response)

### 3.1 `src/config/mod.rs` — `pub use schema::{...}`

All 9 re-exported symbols from `config/mod.rs` line 9-12 are consumed
externally via `config::` or `crate::config::schema::` paths:

| Symbol | External caller(s) |
|--------|--------------------|
| `GreatConfig` | `src/cli/status.rs:301`, `src/cli/doctor.rs:472,557` |
| `McpConfig` | `src/mcp/mod.rs:6` (direct schema path), `src/cli/doctor.rs:500,503` |
| `ConfigMessage` | `src/cli/doctor.rs:500,503` |
| `AgentConfig` | `src/cli/init.rs:9` (`use crate::config::schema::*`) |
| `PlatformConfig` | `src/cli/init.rs:9` (via `schema::*`) |
| `PlatformOverride` | `src/cli/init.rs:9` (via `schema::*`) |
| `ProjectConfig` | `src/cli/init.rs:9` (via `schema::*`) |
| `SecretsConfig` | `src/cli/init.rs:9` (via `schema::*`) |
| `ToolsConfig` | `src/cli/init.rs:9`, `src/platform/runtime.rs:188,362,379` |

**Verdict:** `#[allow(unused_imports)]` on line 8 of `src/config/mod.rs` is
unnecessary — all 9 symbols ARE used. Removing the annotation is safe. Rustc
will correctly warn in future if any become unused.

### 3.2 `src/platform/mod.rs` — detection re-exports (lines 5-9)

| Symbol | External caller(s) |
|--------|--------------------|
| `command_exists` | `src/vault/mod.rs:89,130,172,173`, `src/cli/diff.rs:8`, `src/cli/status.rs:7`, `src/cli/mcp.rs:7`, `src/cli/bootstrap.rs:4` |
| `detect_platform_info` | `src/cli/status.rs:95`, `src/cli/init.rs:49`, `src/cli/apply.rs:390`, `src/cli/update.rs:195`, `src/cli/doctor.rs:73,269` |
| `detect_platform` | No direct external callers (called via `detect_platform_info`) |
| `Architecture` | `src/cli/bootstrap.rs:413`, `src/cli/update.rs:197,201`, `src/cli/doctor.rs:278` |
| `LinuxDistro` | `src/cli/bootstrap.rs:4`, `src/cli/apply.rs:411` |
| `Platform` | `src/cli/bootstrap.rs:4`, `src/cli/apply.rs:407-445`, `src/cli/init.rs:253,259`, `src/cli/update.rs:197-200` |
| `PlatformInfo` | `src/cli/bootstrap.rs:4`, `src/cli/apply.rs:13`, `src/cli/tuning.rs:2`, `src/cli/doctor.rs:7` |
| `detect_architecture` | **Zero external callers** — used only internally in `detection.rs` |
| `PlatformCapabilities` | **Zero external callers** — accessed as a field of `PlatformInfo` via `.capabilities`, never imported by name |

**Action:** Remove `detect_architecture` and `PlatformCapabilities` from the
`pub use` list on lines 6-8. Remove the `#[allow(unused_imports)]` annotation.

### 3.3 `src/platform/mod.rs` — runtime re-exports (lines 11-12)

| Symbol | External caller(s) |
|--------|--------------------|
| `MiseManager` | `src/cli/apply.rs:12` |
| `ProvisionAction` | `src/cli/apply.rs:12` |
| `ProvisionResult` | **Zero external callers** — `provision_from_config` returns `Vec<ProvisionResult>` and callers iterate via type inference; `ProvisionResult` is never imported by name outside `runtime.rs` |

**Action:** Remove `ProvisionResult` from the `pub use` list on line 12.
Remove the `#[allow(unused_imports)]` annotation.

---

## 4. Dependency Map

```
Cargo.toml
  thiserror = "2.0" ──> REMOVE (no source file uses it; src/error.rs deleted)

src/sync/mod.rs
  SyncStatus (lines 5-13) ──> REMOVE (zero callers, zero tests)
  SyncBlob (lines 15-21) ──> REMOVE (zero callers, zero tests)

src/platform/mod.rs
  pub use detection::{...} (line 6-8) ──> TRIM detect_architecture, PlatformCapabilities
  pub use runtime::{...} (line 12) ──> TRIM ProvisionResult

src/config/mod.rs
  #[allow(unused_imports)] (line 8) ──> REMOVE (all 9 symbols are used)
```

No file depends on `SyncStatus` or `SyncBlob`. Confirmed with grep: zero
matches in `src/` excluding the definition sites themselves.

---

## 5. Items Requiring Comment Addition Only (No Structural Change)

These `#[allow(dead_code)]` annotations are correct; they only need a
justification comment added inline:

| File | Line | Add comment |
|------|------|-------------|
| `src/config/mod.rs` | 61 | `// Planned for GROUP F (vault) and GROUP H (template registry).` |
| `src/config/mod.rs` | 69 | `// Planned for GROUP F (vault) and GROUP B (starship config).` |
| `src/sync/mod.rs` | 39 | `// Planned for GROUP G (sync pull --apply).` |
| `src/mcp/mod.rs` | 45 | `// Planned for GROUP C (mcp add command).` |
| `src/mcp/mod.rs` | 53 | `// Planned for GROUP C (mcp add command).` |
| `src/mcp/mod.rs` | 69 | `// Planned for GROUP C (mcp add command).` |
| `src/mcp/mod.rs` | 98 | `// Planned for user-level MCP config support.` |
| `src/platform/package_manager.rs` | 17 | `// Part of complete PackageManager interface; planned for GROUP D (doctor --fix).` |
| `src/platform/package_manager.rs` | 24 | `// Part of complete PackageManager interface; planned for GROUP E (update command).` |
| `src/platform/runtime.rs` | 35 | `// Planned for doctor version display and status --verbose.` |

`src/cli/statusline.rs` lines 19, 24, 32, 42 already have correct comments.
No changes needed there.

---

## 6. Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Removing `ProvisionResult` from re-exports breaks a caller | Low | Verified zero external callers. Run `cargo check` after this step. |
| Removing `detect_architecture` from re-exports breaks a caller | Low | Verified zero external callers via grep. |
| Removing `SyncStatus`/`SyncBlob` breaks a downstream usage | None | Single-crate binary, zero callers confirmed. |
| Removing `thiserror` exposes a transitive use | None | No `use thiserror` or `#[derive(thiserror::Error)]` anywhere in `src/`. |
| Comment-only changes introduce compilation errors | None | Attribute annotations remain structurally identical. |

---

## 7. Build Order

1. **`Cargo.toml` line 13** — remove `thiserror = "2.0"`. Run `cargo check`.
2. **`src/sync/mod.rs` lines 4-21** — delete `SyncStatus` and `SyncBlob`. Run `cargo test`.
3. **`src/platform/mod.rs` lines 5-12** — trim re-exports, remove both `#[allow(unused_imports)]`. Run `cargo check`.
4. **`src/config/mod.rs` line 8** — remove `#[allow(unused_imports)]`. Run `cargo check`.
5. **All remaining files** — add justification comments to `#[allow(dead_code)]` annotations. Run `cargo clippy`.
6. **Final gate** — run `cargo test` + all three clippy invocations from spec Section 8.

Total estimated lines changed: ~30 (deletions + annotation changes + comments).

---

## 8. Files NOT Touched

- `src/cli/statusline.rs` — annotations already have good comments; no action.
- `src/error.rs` — does not exist (deleted in prior cleanup).
- All test files — no production changes, test suite is regression gate only.
- `site/` — not in scope.
