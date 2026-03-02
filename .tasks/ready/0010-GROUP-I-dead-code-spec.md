# 0010 GROUP I: Dead Code and Safety Cleanup -- Technical Specification

**Task:** 0010 GROUP I
**Priority:** P1
**Size:** S (estimated 1-2 hours)
**Type:** Pure refactor -- no behavior changes, no new features
**Status:** ready

---

## 1. Summary

Audit and resolve all dead code annotations (`#[allow(dead_code)]`,
`#[allow(unused_imports)]`), remove the unused `thiserror` crate dependency,
and confirm zero `.unwrap()` calls exist in non-test production code.

This spec is based on a full audit of the codebase as of 2026-02-25 (commit
`544e795`). The original backlog description from project inception is
partially stale: `src/error.rs` no longer exists, clippy is already warning-
free at default lint level, and all `.unwrap()` calls in `src/` are already
confined to `#[cfg(test)]` blocks.

---

## 2. Current State (Audit Results)

### 2.1 Clippy Status

```
cargo clippy                          -> 0 warnings
cargo clippy -- -W dead_code          -> 0 warnings (suppressed by #[allow])
cargo clippy -- -W unused-crate-deps  -> 1 warning: thiserror unused
```

### 2.2 Inventory of `#[allow(dead_code)]` Annotations

Every annotation below suppresses a dead-code warning for an item that is NOT
called from production code. Some are called from test code only; others are
forward-compatibility stubs.

| # | File | Line(s) | Item | Called in production? | Called in tests? |
|---|------|---------|------|---------------------|-----------------|
| 1 | `src/config/mod.rs` | 61-62 | `pub fn data_dir()` | No | Yes |
| 2 | `src/config/mod.rs` | 69-70 | `pub fn config_dir()` | No | Yes |
| 3 | `src/sync/mod.rs` | 6 | `pub enum SyncStatus` | No | No |
| 4 | `src/sync/mod.rs` | 17 | `pub struct SyncBlob` | No | No |
| 5 | `src/sync/mod.rs` | 39 | `pub fn import_config()` | No | Yes |
| 6 | `src/mcp/mod.rs` | 45 | `McpJsonConfig::save()` | No | Yes |
| 7 | `src/mcp/mod.rs` | 53 | `McpJsonConfig::add_server()` | No | Yes |
| 8 | `src/mcp/mod.rs` | 69 | `McpJsonConfig::server_names()` | No | Yes |
| 9 | `src/mcp/mod.rs` | 98 | `pub fn user_mcp_path()` | No | Yes |
| 10 | `src/platform/package_manager.rs` | 17 | `PackageManager::installed_version()` trait method | No | No |
| 11 | `src/platform/package_manager.rs` | 24 | `PackageManager::update()` trait method | No | No |
| 12 | `src/platform/runtime.rs` | 35 | `MiseManager::version()` | No | Yes |
| 13 | `src/cli/statusline.rs` | 19 | `SessionInfo::model` field | No (deserialized) | Yes |
| 14 | `src/cli/statusline.rs` | 24 | `SessionInfo::workspace` field | No (deserialized) | No |
| 15 | `src/cli/statusline.rs` | 32 | `LoopState::loop_id` field | No (deserialized) | No |
| 16 | `src/cli/statusline.rs` | 42 | `AgentState::name` field | No (deserialized) | Yes |

### 2.3 Inventory of `#[allow(unused_imports)]` Annotations

| # | File | Line(s) | Items | Reason |
|---|------|---------|-------|--------|
| 1 | `src/config/mod.rs` | 8 | `AgentConfig, ConfigMessage, GreatConfig, McpConfig, PlatformConfig, PlatformOverride, ProjectConfig, SecretsConfig, ToolsConfig` | All are consumed downstream via `use crate::config::*` |
| 2 | `src/platform/mod.rs` | 5 | `command_exists, detect_architecture, detect_platform, detect_platform_info, Architecture, LinuxDistro, Platform, PlatformCapabilities, PlatformInfo` | `detect_architecture` and `PlatformCapabilities` have zero external callers |
| 3 | `src/platform/mod.rs` | 11 | `MiseManager, ProvisionAction, ProvisionResult` | `ProvisionResult` has zero external callers |

### 2.4 Unused Crate Dependency

| Crate | Declared in | Used in source? | Action |
|-------|-------------|----------------|--------|
| `thiserror = "2.0"` | `Cargo.toml` line 13 | No (`src/error.rs` was deleted; no `#[derive(thiserror::Error)]` exists) | Remove |

### 2.5 `.unwrap()` in Production Code

**Zero instances.** All `.unwrap()` calls are confined to `#[cfg(test)]` blocks.
The backlog description was written at project inception; the codebase has been
cleaned since then.

### 2.6 `.unwrap_or()` Patterns (Informational -- No Action Needed)

All `.unwrap_or()` / `.unwrap_or_default()` / `.unwrap_or(false)` calls in
production code are safe and idiomatic. They provide sensible fallback values
for `Option` types (e.g., `version.as_deref().unwrap_or("unknown")`). These
are NOT the `.unwrap()` anti-pattern the backlog targets.

---

## 3. Recommended Fix for Each Item

### 3.1 `#[allow(dead_code)]` -- Improve Annotations with Justifications

For items that are genuinely unused stubs awaiting future GROUPs, keep the
annotation but add a comment explaining why. For items that are part of a
public API consumed by tests, the annotation is correct and necessary.

**Decision criteria:**
- Item is a forward-compatibility deserialization field -> keep `#[allow(dead_code)]` with comment (already done in statusline.rs)
- Item is a public API that will be consumed by a future GROUP -> keep, add `// Used by GROUP X` comment
- Item has zero callers anywhere (not even tests) and no planned GROUP -> remove the item entirely
- Trait method has no production callers but is part of a complete interface -> keep `#[allow(dead_code)]` with comment

#### Item-by-item actions:

| # | Item | Action | Rationale |
|---|------|--------|-----------|
| 1 | `config::data_dir()` | Keep `#[allow(dead_code)]`, add comment `// Planned for GROUP F (vault) and GROUP H (template registry)` | Will be needed when vault/template features store local data |
| 2 | `config::config_dir()` | Keep `#[allow(dead_code)]`, add comment `// Planned for GROUP F (vault) and GROUP B (starship config)` | Will be needed for starship config and vault unlock features |
| 3 | `SyncStatus` enum | **Remove entirely.** | Zero callers, zero test callers. Not referenced by any GROUP. If needed later, it can be re-introduced with proper design. |
| 4 | `SyncBlob` struct | **Remove entirely.** | Zero callers, zero test callers. `save_local` / `load_local` use raw `&[u8]`. |
| 5 | `sync::import_config()` | Keep `#[allow(dead_code)]`, add comment `// Planned for GROUP G (sync pull --apply)` | Required by the sync pull --apply feature |
| 6 | `McpJsonConfig::save()` | Keep `#[allow(dead_code)]`, add comment `// Planned for GROUP C (mcp add command)` | Required when mcp add writes .mcp.json |
| 7 | `McpJsonConfig::add_server()` | Keep `#[allow(dead_code)]`, add comment `// Planned for GROUP C (mcp add command)` | Required when mcp add inserts a server |
| 8 | `McpJsonConfig::server_names()` | Keep `#[allow(dead_code)]`, add comment `// Planned for GROUP C (mcp add command)` | Useful for listing/dedup in mcp add |
| 9 | `user_mcp_path()` | Keep `#[allow(dead_code)]`, add comment `// Planned for user-level MCP config support` | User-level config is a known future feature |
| 10 | `PackageManager::installed_version()` | Keep `#[allow(dead_code)]`, add comment `// Part of complete PackageManager interface; planned for GROUP D (doctor --fix) and GROUP E (update)` | Trait completeness; will be called by doctor --fix |
| 11 | `PackageManager::update()` | Keep `#[allow(dead_code)]`, add comment `// Part of complete PackageManager interface; planned for GROUP E (update command)` | Trait completeness; will be called by self-update |
| 12 | `MiseManager::version()` | Keep `#[allow(dead_code)]`, add comment `// Planned for doctor version display and status --verbose` | Useful diagnostic |
| 13 | `SessionInfo::model` | Keep existing annotation and comment (already has good comment) | Forward-compatibility deserialization |
| 14 | `SessionInfo::workspace` | Keep existing annotation and comment (already has good comment) | Forward-compatibility deserialization |
| 15 | `LoopState::loop_id` | Keep existing annotation and comment (already has good comment) | Forward-compatibility deserialization |
| 16 | `AgentState::name` | Keep existing annotation and comment (already has good comment) | Forward-compatibility deserialization |

### 3.2 `#[allow(unused_imports)]` -- Trim Unused Re-exports

| # | File | Action |
|---|------|--------|
| 1 | `src/config/mod.rs:8` | **Remove annotation.** All 9 symbols are consumed downstream. Without the annotation, clippy/rustc will correctly warn if any become unused in the future. |
| 2 | `src/platform/mod.rs:5` | **Remove `detect_architecture` and `PlatformCapabilities` from the `pub use` list.** They have zero external callers. Keep the annotation removal -- the remaining 7 symbols are all used. |
| 3 | `src/platform/mod.rs:11` | **Remove `ProvisionResult` from the `pub use` list.** It has zero external callers (all usage is internal to `runtime.rs`). Keep `MiseManager` and `ProvisionAction` which are used externally. |

### 3.3 Remove `thiserror` Dependency

Remove `thiserror = "2.0"` from `[dependencies]` in `Cargo.toml`. No source
file imports it. The project uses `anyhow` for all error handling.

---

## 4. Files to Modify

| File | Changes |
|------|---------|
| `Cargo.toml` | Remove `thiserror = "2.0"` from `[dependencies]` (line 13) |
| `src/config/mod.rs` | (1) Remove `#[allow(unused_imports)]` on line 8. (2) Add justification comment to `#[allow(dead_code)]` on lines 61 and 69. |
| `src/sync/mod.rs` | (1) Delete `SyncStatus` enum (lines 4-13). (2) Delete `SyncBlob` struct (lines 15-21). (3) Add justification comment to `#[allow(dead_code)]` on `import_config` (line 39). |
| `src/mcp/mod.rs` | Add justification comments to `#[allow(dead_code)]` on lines 45, 53, 69, 98. |
| `src/platform/mod.rs` | (1) Remove `#[allow(unused_imports)]` on lines 5 and 11. (2) Remove `detect_architecture` and `PlatformCapabilities` from the `pub use` on line 6-8. (3) Remove `ProvisionResult` from the `pub use` on line 12. |
| `src/platform/package_manager.rs` | Add justification comments to `#[allow(dead_code)]` on lines 17 and 24. |
| `src/platform/runtime.rs` | Add justification comment to `#[allow(dead_code)]` on line 35. |

**Files NOT modified (no changes needed):**
- `src/cli/statusline.rs` -- all 4 annotations already have good comments
- `src/cli/status.rs` -- no `.unwrap()` in production; `.unwrap_or()` patterns are correct
- `src/cli/doctor.rs` -- no `.unwrap()` in production; `.unwrap_or()` patterns are correct
- `src/error.rs` -- does not exist (already deleted in a prior cleanup)

---

## 5. Implementation Approach and Build Order

This is a single-pass refactor. Order does not matter since there are no
cross-file dependencies, but the recommended sequence minimizes risk:

1. **Remove `thiserror` from `Cargo.toml`.** Run `cargo check` to confirm no
   breakage.

2. **Remove `SyncStatus` and `SyncBlob` from `src/sync/mod.rs`.** These have
   zero callers anywhere. Run `cargo test` to confirm no breakage.

3. **Trim `pub use` re-exports in `src/platform/mod.rs`.** Remove
   `detect_architecture`, `PlatformCapabilities`, `ProvisionResult`. Remove
   both `#[allow(unused_imports)]` annotations. Run `cargo check` to confirm
   no downstream code imports these symbols.

4. **Remove `#[allow(unused_imports)]` from `src/config/mod.rs`.** Run `cargo
   check`.

5. **Add justification comments to remaining `#[allow(dead_code)]`
   annotations** in `config/mod.rs`, `sync/mod.rs`, `mcp/mod.rs`,
   `platform/package_manager.rs`, `platform/runtime.rs`.

6. **Final verification** (see Section 8).

---

## 6. Exact Code Changes

### 6.1 `Cargo.toml`

Remove line 13:
```diff
-thiserror = "2.0"
```

### 6.2 `src/config/mod.rs`

Remove `#[allow(unused_imports)]` annotation on line 8:
```diff
 // Re-exported for downstream consumption by CLI subcommands.
-#[allow(unused_imports)]
 pub use schema::{
```

Add justification comments to dead_code annotations:
```diff
 /// Return the platform-specific data directory (~/.local/share/great on Linux).
-#[allow(dead_code)]
+#[allow(dead_code)] // Planned for GROUP F (vault) and GROUP H (template registry).
 pub fn data_dir() -> Result<PathBuf> {
```

```diff
 /// Return the platform-specific config directory (~/.config/great on Linux).
-#[allow(dead_code)]
+#[allow(dead_code)] // Planned for GROUP F (vault) and GROUP B (starship config).
 pub fn config_dir() -> Result<PathBuf> {
```

### 6.3 `src/sync/mod.rs`

Delete the `SyncStatus` enum and `SyncBlob` struct:
```diff
-/// Status of sync between local and remote.
-#[derive(Debug, Clone, PartialEq, Eq)]
-#[allow(dead_code)]
-pub enum SyncStatus {
-    InSync,
-    LocalNewer,
-    RemoteNewer,
-    Conflict,
-    NeverSynced,
-}
-
-/// Encrypted blob for sync transport.
-#[derive(Debug)]
-#[allow(dead_code)]
-pub struct SyncBlob {
-    pub data: Vec<u8>,
-    pub timestamp: u64,
-}
-
```

Add justification comment to `import_config`:
```diff
 /// Import config from bytes.
-#[allow(dead_code)]
+#[allow(dead_code)] // Planned for GROUP G (sync pull --apply).
 pub fn import_config(data: &[u8], config_path: &Path) -> Result<()> {
```

### 6.4 `src/mcp/mod.rs`

```diff
     /// Save this config as pretty-printed JSON to the given path.
-    #[allow(dead_code)]
+    #[allow(dead_code)] // Planned for GROUP C (mcp add command).
     pub fn save(&self, path: &Path) -> Result<()> {
```

```diff
     /// Add a server from a [`McpConfig`] entry parsed from `great.toml`.
-    #[allow(dead_code)]
+    #[allow(dead_code)] // Planned for GROUP C (mcp add command).
     pub fn add_server(&mut self, name: &str, config: &McpConfig) {
```

```diff
     /// List all configured server names.
-    #[allow(dead_code)]
+    #[allow(dead_code)] // Planned for GROUP C (mcp add command).
     pub fn server_names(&self) -> Vec<&String> {
```

```diff
 /// Return the user-level Claude config path (`~/.claude.json`).
-#[allow(dead_code)]
+#[allow(dead_code)] // Planned for user-level MCP config support.
 pub fn user_mcp_path() -> Option<PathBuf> {
```

### 6.5 `src/platform/mod.rs`

Replace lines 5-12 with trimmed re-exports and no `#[allow(unused_imports)]`:
```diff
-#[allow(unused_imports)]
-pub use detection::{
-    command_exists, detect_architecture, detect_platform, detect_platform_info, Architecture,
-    LinuxDistro, Platform, PlatformCapabilities, PlatformInfo,
-};
+pub use detection::{
+    command_exists, detect_platform, detect_platform_info, Architecture, LinuxDistro, Platform,
+    PlatformInfo,
+};

-#[allow(unused_imports)]
-pub use runtime::{MiseManager, ProvisionAction, ProvisionResult};
+pub use runtime::{MiseManager, ProvisionAction};
```

### 6.6 `src/platform/package_manager.rs`

```diff
     /// Get the installed version of a package, if any.
-    #[allow(dead_code)]
+    #[allow(dead_code)] // Part of complete PackageManager interface; planned for GROUP D (doctor --fix).
     fn installed_version(&self, package: &str) -> Option<String>;

     /// Update a package to the latest version.
-    #[allow(dead_code)]
+    #[allow(dead_code)] // Part of complete PackageManager interface; planned for GROUP E (update command).
     fn update(&self, package: &str) -> Result<()>;
```

### 6.7 `src/platform/runtime.rs`

```diff
     /// Get the mise version string.
-    #[allow(dead_code)]
+    #[allow(dead_code)] // Planned for doctor version display and status --verbose.
     pub fn version() -> Option<String> {
```

---

## 7. Edge Cases

| Edge case | Handling |
|-----------|----------|
| Removing `SyncStatus`/`SyncBlob` breaks a downstream crate | Not possible -- this is a single-crate binary. No external consumers. |
| Removing `thiserror` breaks a transitive import | Verified: no `use thiserror` or `#[derive(thiserror::Error)]` in any source file. |
| Removing `detect_architecture` from `pub use` breaks callers | Verified: zero callers outside `src/platform/detection.rs`. Internal callers use the local path. |
| Removing `PlatformCapabilities` from `pub use` breaks callers | Verified: zero callers outside `src/platform/detection.rs`. The struct is a field of `PlatformInfo` (accessed via `.capabilities`) but never imported by name externally. |
| Removing `ProvisionResult` from `pub use` breaks callers | Verified: zero callers outside `src/platform/runtime.rs`. `provision_from_config` returns `Vec<ProvisionResult>` but callers access it via type inference, not by importing the name. **Builder must verify:** run `cargo check` after this change. If `apply.rs` or another caller imports `ProvisionResult` by name, add it back to the re-export list without the unused annotation. |
| Future GROUP implementer cannot find removed `SyncStatus`/`SyncBlob` | These types had no consumers and no tests. If GROUP G or a future sync feature needs a status enum, it should be designed fresh to match actual requirements. |

---

## 8. Acceptance Criteria

All criteria are machine-verifiable:

- [ ] `cargo clippy` produces zero warnings at default lint level.
- [ ] `cargo clippy -- -W dead_code` produces zero warnings.
- [ ] `cargo clippy -- -W unused-crate-dependencies` produces zero warnings.
- [ ] `cargo test` passes with zero failures.
- [ ] `cargo build` succeeds.
- [ ] `grep -rn 'thiserror' Cargo.toml` returns no matches.
- [ ] `grep -rn 'SyncStatus\|SyncBlob' src/` returns no matches.
- [ ] Every `#[allow(dead_code)]` annotation in `src/` has an adjacent comment
      explaining why the item is kept (either forward-compatibility
      deserialization or a planned GROUP reference).
- [ ] Zero `.unwrap()` calls exist in `src/**/*.rs` outside `#[cfg(test)]` blocks.
      (Already true -- this criterion confirms no regressions.)

---

## 9. Security Considerations

None. This is a pure refactor removing unused code and adding comments. No new
dependencies, no new I/O, no changed behavior.

---

## 10. Testing Strategy

No new tests are needed. This refactor only removes unused items and adds
comments. The existing test suite (`cargo test`) serves as a regression gate:

1. Run `cargo test` before starting (baseline).
2. Apply changes incrementally per Section 5 build order.
3. Run `cargo check` after each step to catch compilation errors early.
4. Run `cargo test` after all changes to confirm zero regressions.
5. Run the three `cargo clippy` invocations from Section 8 as the final gate.

---

## 11. Platform Considerations

This refactor is platform-independent. All changes are to Rust source
annotations, comments, re-export lists, and `Cargo.toml`. No platform-specific
code paths are affected. The changes apply identically on:

- macOS ARM64 (aarch64-apple-darwin)
- macOS x86_64 (x86_64-apple-darwin)
- Ubuntu (x86_64-unknown-linux-gnu)
- WSL2 Ubuntu (x86_64-unknown-linux-gnu)
