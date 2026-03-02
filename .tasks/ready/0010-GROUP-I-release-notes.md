# Release Notes — Task 0010 GROUP I: Dead Code and Safety Cleanup

**Date:** 2026-02-26
**Scope:** `Cargo.toml`, `src/config/mod.rs`, `src/sync/mod.rs`, `src/mcp/mod.rs`,
           `src/platform/mod.rs`, `src/platform/package_manager.rs`,
           `src/platform/runtime.rs`, `src/cli/doctor.rs`
**Cargo version:** 0.1.0 (no version bump; pure internal refactor)

---

## One-line summary

Remove unused `thiserror` dependency, delete two dead types, trim seven
over-broad re-exports, replace a lint-suppression workaround with `#[derive(Default)]`,
and annotate every retained `#[allow(dead_code)]` item with a justification comment.

---

## What changed

### `Cargo.toml`

The `thiserror = "2.0"` direct dependency has been removed.
`src/error.rs` was deleted in an earlier iteration; no source file contains a
`use thiserror` import or a `#[derive(thiserror::Error)]` attribute.
`thiserror` remains in `Cargo.lock` as a transitive dependency of `reqwest` and
`zip`, but the project no longer declares it directly.

```
cargo clippy -- -W unused-crate-dependencies
```
now produces zero warnings.

### `src/sync/mod.rs` — two dead types deleted

`SyncStatus` (a five-variant enum) and `SyncBlob` (a two-field struct) had zero
callers in production code and zero callers in tests. Neither type was
referenced by any planned GROUP. Both are removed entirely.

The `import_config` function, which is exercised by tests and required by
GROUP G (sync pull `--apply`), is kept. Its `#[allow(dead_code)]` annotation now
reads:

```rust
#[allow(dead_code)] // Planned for GROUP G (sync pull --apply).
pub fn import_config(data: &[u8], config_path: &Path) -> Result<()> {
```

### `src/config/mod.rs` — re-export trimmed and annotation removed

The `pub use schema::{...}` list previously re-exported nine schema types behind
an `#[allow(unused_imports)]` suppression. Only `GreatConfig` and `ConfigMessage`
are imported via the `config::` re-export path by downstream CLI modules; the
other seven are accessed directly through `crate::config::schema::*`. The
re-export is trimmed to the two symbols that are actually used via this path, and
the now-unnecessary `#[allow(unused_imports)]` annotation is removed.

Before:
```rust
#[allow(unused_imports)]
pub use schema::{
    AgentConfig, ConfigMessage, GreatConfig, McpConfig, PlatformConfig,
    PlatformOverride, ProjectConfig, SecretsConfig, ToolsConfig,
};
```

After:
```rust
// Re-exported for downstream consumption by CLI subcommands.
pub use schema::{ConfigMessage, GreatConfig};
```

The `data_dir()` and `config_dir()` functions retain their `#[allow(dead_code)]`
annotations, now with explicit justification comments:

```rust
#[allow(dead_code)] // Planned for GROUP F (vault) and GROUP H (template registry).
pub fn data_dir() -> Result<PathBuf> { ... }

#[allow(dead_code)] // Planned for GROUP F (vault) and GROUP B (starship config).
pub fn config_dir() -> Result<PathBuf> { ... }
```

### `src/platform/mod.rs` — re-exports trimmed and annotations removed

Two `#[allow(unused_imports)]` annotations are removed. Three symbols are
removed from the `pub use` lists because they have zero external callers:

- `detect_architecture` — used only inside `detection.rs`; callers outside
  the module access it indirectly through `detect_platform_info`.
- `PlatformCapabilities` — accessed only as the `.capabilities` field of
  `PlatformInfo`; never imported by name outside `detection.rs`.
- `ProvisionResult` — returned by `provision_from_config` but consumed via
  type inference; never imported by name outside `runtime.rs`.
- `detect_platform` — superseded for external use by `detect_platform_info`.

Before:
```rust
#[allow(unused_imports)]
pub use detection::{
    command_exists, detect_architecture, detect_platform, detect_platform_info,
    Architecture, LinuxDistro, Platform, PlatformCapabilities, PlatformInfo,
};

#[allow(unused_imports)]
pub use runtime::{MiseManager, ProvisionAction, ProvisionResult};
```

After:
```rust
pub use detection::{
    command_exists, detect_platform_info, Architecture, LinuxDistro, Platform,
    PlatformInfo,
};

pub use runtime::{MiseManager, ProvisionAction};
```

### `src/platform/package_manager.rs` — trait method annotations justified

Two `PackageManager` trait methods carry `#[allow(dead_code)]` because no
production call site exists yet. The annotations are retained (these methods
are part of the complete public interface) and each now names the GROUP that
will consume it:

```rust
#[allow(dead_code)] // Part of complete PackageManager interface; planned for GROUP D (doctor --fix).
fn installed_version(&self, package: &str) -> Option<String>;

#[allow(dead_code)] // Part of complete PackageManager interface; planned for GROUP E (update command).
fn update(&self, package: &str) -> Result<()>;
```

### `src/platform/runtime.rs` — annotation justified

```rust
#[allow(dead_code)] // Planned for doctor version display and status --verbose.
pub fn version() -> Option<String> { ... }
```

### `src/mcp/mod.rs` — four annotations justified

Each dead-code annotation on `McpJsonConfig` methods and `user_mcp_path`
now carries a comment identifying its consumer GROUP:

```rust
#[allow(dead_code)] // Planned for GROUP C (mcp add command).
pub fn save(&self, path: &Path) -> Result<()> { ... }

#[allow(dead_code)] // Planned for GROUP C (mcp add command).
pub fn add_server(&mut self, name: &str, config: &McpConfig) { ... }

#[allow(dead_code)] // Planned for GROUP C (mcp add command).
pub fn server_names(&self) -> Vec<&String> { ... }

#[allow(dead_code)] // Planned for user-level MCP config support.
pub fn user_mcp_path() -> Option<PathBuf> { ... }
```

### `src/cli/doctor.rs` — derive fix for `DiagnosticResult`

A manual `impl Default` block for `DiagnosticResult` was byte-for-byte
equivalent to what `#[derive(Default)]` produces. The implementation had
suppressed the resulting `clippy::derivable_impls` lint rather than fixing it.
The suppression and the manual `impl` block are replaced with a single derive:

Before:
```rust
struct DiagnosticResult {
    checks_passed: usize,
    checks_warned: usize,
    checks_failed: usize,
    fixable: Vec<FixableIssue>,
}

impl Default for DiagnosticResult {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        Self {
            checks_passed: 0,
            checks_warned: 0,
            checks_failed: 0,
            fixable: Vec::new(),
        }
    }
}
```

After:
```rust
#[derive(Default)]
struct DiagnosticResult {
    checks_passed: usize,
    checks_warned: usize,
    checks_failed: usize,
    fixable: Vec<FixableIssue>,
}
```

---

## Why

The codebase had accumulated three categories of low-grade noise:

1. **Lint suppressions without explanations.** `#[allow(dead_code)]` without
   a comment leaves no record of intent. A reader cannot distinguish "we will
   need this in two weeks" from "we forgot to delete this two years ago."
   Every remaining annotation now states the GROUP or feature it awaits.

2. **Over-broad re-exports silenced by blanket suppressions.** Both
   `#[allow(unused_imports)]` sites hid the fact that the re-export lists had
   grown stale. Trimming them means the compiler will immediately flag any
   future unused re-export.

3. **Suppressing a lint instead of fixing the underlying code.** The
   `#[allow(clippy::derivable_impls)]` in `doctor.rs` was the wrong response
   to clippy's correct diagnosis. Fixing the root cause (`#[derive(Default)]`)
   eliminates the suppression and the 13-line manual implementation.

The goal is a zero-warning baseline at all three lint levels so that new
regressions are immediately detectable on the next commit.

---

## Impact

This is a pure refactor. No behavioral changes exist anywhere in this diff.

- The `great` binary produces identical output for every subcommand before and
  after this change.
- No public CLI flags, exit codes, config keys, or output formats are affected.
- No new crate dependencies are introduced.
- No test logic is modified. The existing test suite (`cargo test`) serves as
  the regression gate and passes without change.

### Lint gate (verified)

```
cargo clippy                              -- 0 warnings
cargo clippy -- -W dead_code             -- 0 warnings
cargo clippy -- -W unused-crate-dependencies -- 0 warnings
cargo test                               -- 0 failures
```

---

## Migration notes

No migration is required. Nothing in this change affects the `great.toml`
schema, the CLI interface, or the runtime behavior of the `great` binary.

If any downstream code outside this repository imported `ProvisionResult`,
`detect_architecture`, or `PlatformCapabilities` from the `great-sh` crate
by name: add the direct `crate::platform::{detection,runtime}::` path.
In practice there are no external consumers — this is a single-crate binary.
