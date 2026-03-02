# Scout Report: 0001 Platform Detection Engine
**Scout**: Alexander von Humboldt
**Date**: 2026-02-20
**Spec**: `.tasks/specs/0001-platform-detection.md` (Revision 2, approved)

---

## 1. Current State of `src/platform/`

Four files exist. Two are in scope; two are not touched.

### `src/platform/detection.rs` (316 lines) — PRIMARY CHANGE TARGET

Types currently defined:
- `Architecture` — `{X86_64, Aarch64, Unknown}` — derives `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize` (already correct)
- `LinuxDistro` — `{Ubuntu, Debian, Fedora, Arch, Other(String)}` — derives `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize` (already correct)
- `Platform` — **missing `Serialize, Deserialize`** — currently only `Debug, Clone, PartialEq, Eq`
- `PlatformCapabilities` — **missing `Serialize, Deserialize`**, has `is_wsl: bool`, **missing `has_pacman`**
- `PlatformInfo` — **missing `Serialize, Deserialize`** — currently only `Debug, Clone`

Public functions (signatures stable, keep them):
- `detect_platform() -> Platform` (line 71)
- `detect_platform_info() -> PlatformInfo` (line 105)
- `command_exists(cmd: &str) -> bool` (line 123) — uses shell subprocess; spec replaces with `which::which()`
- `detect_architecture() -> Architecture` (line 147) — no change needed
- `detect_capabilities(platform: &Platform) -> PlatformCapabilities` (line 156) — needs `has_pacman`, `is_wsl2` rename

Private functions:
- `is_wsl() -> bool` (line 178) — needs WSLInterop check added
- `detect_linux_distro() -> LinuxDistro` (line 189) — unchanged logic
- `detect_linux_version() -> Option<String>` (line 212) — unchanged logic
- `detect_macos_version() -> Option<String>` (line 229) — unchanged logic
- `is_root() -> bool` (line 248) — currently checks `$USER == "root"`; spec replaces with `id -u` subprocess + `#[cfg(unix)]`
- `detect_shell() -> String` (line 253) — adds `$COMSPEC` fallback via `#[cfg]`

Existing unit tests (lines 262-315): 6 tests. All must continue to pass. One test (`test_detect_capabilities`) checks `caps.is_wsl` — must be updated to `caps.is_wsl2`.

### `src/platform/mod.rs` (65 lines) — MINOR CHANGE TARGET

Re-exports `Platform`, `PlatformCapabilities`, `PlatformInfo`, etc. via `#[allow(unused_imports)]`.
`Display` impl for `Platform` is already correct (all 5 variants covered).
`display_detailed()` and `arch()` methods are already correct.

The re-export line 8 currently includes `PlatformCapabilities` — no structural change needed here unless Da Vinci splits files (spec says no splitting). The `#[allow(unused_imports)]` is already in place.

### `src/platform/package_manager.rs` — DO NOT TOUCH

Uses `super::detection::command_exists` (line 3). After the spec change, `command_exists` will use `which::which()` instead of shell spawn. The call site is identical — no changes to this file.

### `src/platform/runtime.rs` — DO NOT TOUCH

Uses `super::detection::command_exists` (line 3 per grep). Same reasoning as above.

---

## 2. Patterns to Follow

### Error handling
- No `Result` returned from any detection function — all return safe defaults on failure
- Use `.ok()`, `.map()`, `.unwrap_or()`, `.unwrap_or_else()` chains — never `.unwrap()` or `.expect()` in production paths
- Example from existing code (line 229-244): `Command::new("sw_vers").output().ok()?` with explicit `None` fallback
- Example from `is_wsl()` (line 183-185): `std::fs::read_to_string().map(|v| ...).unwrap_or(false)`

### cfg guards
- Current code uses `cfg!(target_os = "macos")` etc. as runtime branches inside functions
- Spec requires `#[cfg(unix)]` / `#[cfg(not(unix))]` as compile-time attribute guards for `is_root()`
- Use `#[cfg(unix)]` (not `#[cfg(target_os = "linux")]`) to cover both Linux and macOS
- Existing pattern in `command_exists` (lines 124-143) uses `cfg!()` macro — the new `is_root` uses `#[cfg]` attributes on separate function bodies (different style, both valid Rust)

### Derive macros
- All types in `detection.rs` must use `serde::{Deserialize, Serialize}` from the `serde` crate already in `Cargo.toml`
- Pattern: `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` — see `Architecture` at line 4 for the exact order used in this file
- Do NOT add `Hash` to `Platform` (spec explicitly documents the rationale: `Other(String)` fragility)

### Module structure
- Each subcommand: `Args` struct with clap derive, `pub fn run(args: Args) -> Result<()>`
- Internal helpers are private (`fn`, not `pub fn`)
- Tests in `#[cfg(test)] mod tests { use super::*; }` at bottom of the file

### `anyhow` usage
- `anyhow::Result` for all CLI-facing code
- Detection layer uses no `Result` at all — pure value returns
- `context()` calls use `.context("human description")` from `anyhow::Context` trait

---

## 3. Files Da Vinci Must Modify

| File | Change |
|---|---|
| `Cargo.toml` | Add `which = "7"` to `[dependencies]` |
| `src/platform/detection.rs` | All spec changes (see build order below) |
| `src/platform/mod.rs` | None anticipated, but verify re-exports still compile after field rename `is_wsl` -> `is_wsl2` |

---

## 4. Files NOT to Modify

| File | Reason |
|---|---|
| `src/platform/package_manager.rs` | Uses `command_exists` by name only — signature unchanged |
| `src/platform/runtime.rs` | Same — uses `command_exists` by name only |
| `src/cli/status.rs` | Does not reference `caps.is_wsl` — accesses `info.capabilities` fields by name but only `has_homebrew`, `has_apt`, `has_dnf`, `has_snap`, `has_systemd`, `has_docker` — all preserved |
| `src/cli/doctor.rs` | Uses `command_exists` and `detect_platform_info()` — no field references to `is_wsl` |
| `src/cli/apply.rs` | Same pattern — `command_exists` and `detect_platform_info()` |
| `src/cli/diff.rs` | Uses `command_exists` only |
| `src/cli/mcp.rs` | Uses `command_exists` only |
| `src/vault/mod.rs` | Uses `crate::platform::command_exists` only |
| `src/cli/init.rs` | Uses `detect_platform_info()` only |
| `src/cli/update.rs` | Uses `detect_platform_info()` only |
| `tests/cli_smoke.rs` | Integration tests test CLI output strings, not struct fields |
| `src/main.rs` | No platform imports |

---

## 5. Dependencies to Add

In `Cargo.toml` under `[dependencies]`:

```toml
which = "7"
```

Current `[dependencies]` block ends at line 24 (`semver = "1.0"`). Insert after `semver`.
No other new dependencies. `serde` and `serde_json` are already present.

---

## 6. Legacy Code Worth Consulting

From `/home/isaac/src/great-sh/src/platform/detection.rs` (lines 1-100):
- The old repo's `Platform` enum embeds `is_root` directly into the Linux variant and uses `wsl_available` on Windows — this is a different design. Do NOT copy it.
- The old `LinuxDistribution` enum includes `CentOS`, `RHEL`, `OpenSUSE` — new spec intentionally omits these; stay with the current `LinuxDistro` definition.

From `/home/isaac/src/great-sh/src/core/utils.rs` (lines 1-100):
- `command_exists` (line 28-44) uses `which` binary on Unix and `where` on Windows — shell-spawning approach. The new spec replaces this with `which::which()` crate call: `which::which(cmd).is_ok()`. Do NOT copy the legacy implementation; use the crate.
- `is_ubuntu()` (line 79-93) shows the correct `/etc/os-release` pattern with fallback to `/etc/lsb-release` — the current codebase already handles this correctly via `ID=` parsing; no need to adopt the legacy fallback.
- The legacy repo's `#[cfg(target_os = "windows")]` gating on admin detection functions is a useful pattern reference for how to gate `is_root()` with `#[cfg(unix)]`.

**Verdict on legacy repo**: The old repo is more complex than needed and uses different design. Consult only for the `#[cfg]` attribute guard pattern and the `which`/`where` binary approach (to confirm the new `which` crate approach supersedes it).

---

## 7. Potential Conflicts / Risks

### A. `is_wsl` -> `is_wsl2` rename — VERIFY scope is complete

The field `is_wsl: bool` in `PlatformCapabilities` is referenced in exactly these locations:
- `detection.rs` line 52 (declaration)
- `detection.rs` line 157 (let binding `let is_wsl = ...`)
- `detection.rs` line 165 (struct init field `is_wsl,`)
- `detection.rs` line 311-312 (test: `caps.is_wsl`)

No external callers access `caps.is_wsl` — confirmed by grep. `status.rs` prints `caps.has_homebrew`, `has_apt`, `has_dnf`, `has_snap`, `has_systemd`, `has_docker` but never `is_wsl`. Rename is safe.

### B. `PlatformCapabilities::is_wsl` semantic change

The old `is_wsl: bool` was set to `matches!(platform, Platform::Wsl { .. })` — derived from the platform variant.
The new `is_wsl2: bool` is set to `is_wsl2()` which checks `/proc/sys/fs/binfmt_misc/WSLInterop` — a real filesystem check.

These can diverge: a WSL1 machine would have `Platform::Wsl` but `is_wsl2 = false`. The existing test `test_detect_capabilities` asserts they match — this test must be rewritten. The new correct assertion is: if running on WSL2 (interop file exists), both `Platform::Wsl` and `is_wsl2 = true` should hold. On WSL1, `Platform::Wsl` is true but `is_wsl2 = false`.

### C. `OsProbe` trait is `#[cfg(test)]` only

The spec places `OsProbe` inside `#[cfg(test)]`. This is unusual but intentional — production code uses real filesystem directly, test helpers inject mocks. Da Vinci must ensure the `_with_probe` variants are also `#[cfg(test)]` or at minimum not exported. If any `_with_probe` function accidentally leaks to production code, the compiler will flag dead_code but it will still compile.

### D. `which = "7"` — verify crate version exists

The `which` crate at version 7 is a major version. Confirm `Cargo.lock` resolves correctly after `cargo add which@7` or manual edit. The crate's `which::which(cmd).is_ok()` API is stable across versions 4+.

### E. `detect_capabilities` no longer takes `&Platform` for WSL detection

Under the new design, `is_wsl2: is_wsl2()` is independent of the `platform` argument. The `platform: &Platform` parameter is now effectively unused inside `detect_capabilities`. This will trigger a `clippy` warning (`unused_variable`). Either add `let _ = platform;` / `#[allow(unused_variables)]`, or prefix the parameter with underscore: `_platform: &Platform`. Check if callers pass the platform — yes: `detect_platform_info()` calls `detect_capabilities(&platform)`. The parameter should be kept to preserve the public API signature (callers in `package_manager.rs` tests and elsewhere may call it), but prefixed with `_` to silence clippy.

---

## 8. Test Infrastructure

### Existing patterns in `detection.rs` (lines 262-315)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        // arrange
        // act
        // assert using assert!, assert_eq!, assert_ne!
    }
}
```

No external test helpers needed — all unit tests are inline.

### Integration tests in `tests/cli_smoke.rs`

Pattern: `assert_cmd::Command::cargo_bin("great")` + `.args([...])` + `.assert().success().stdout/stderr(predicate::str::contains(...))`.

The existing `status_shows_platform` test (line 58-66) checks that `stderr` contains `"Platform:"`. After the spec changes, `status --verbose` output format is unchanged — the test will continue to pass without modification.

### New tests to add (per spec — all in `detection.rs`)

24 tests total (10 machine-dependent + 14 mock-based). The `OsProbe` trait approach:

```rust
#[cfg(test)]
trait OsProbe {
    fn read_file(&self, path: &str) -> Option<String>;
    fn env_var(&self, name: &str) -> Option<String>;
    fn path_exists(&self, path: &str) -> bool;
    fn command_output(&self, cmd: &str, args: &[&str]) -> Option<String>;
}
```

Each `_with_probe` variant (e.g., `is_wsl_with_probe(probe: &dyn OsProbe) -> bool`) lives inside `#[cfg(test)]`.

The live functions call the real implementations directly (no probe indirection in production).

### Recommended build order for Da Vinci

1. Add `which = "7"` to `Cargo.toml` (line 24, after `semver = "1.0"`)
2. Update `PlatformCapabilities` struct: add `has_pacman`, rename `is_wsl` -> `is_wsl2`, add `Serialize, Deserialize`
3. Add `Serialize, Deserialize` derives to `Platform` and `PlatformInfo`
4. Replace `command_exists()` body with `which::which(cmd).is_ok()`
5. Update `is_wsl()`: add `/proc/sys/fs/binfmt_misc/WSLInterop` check as second tier
6. Add `is_wsl2() -> bool` private function
7. Replace `is_root()` with `#[cfg(unix)]` / `#[cfg(not(unix))]` split using `id -u`
8. Update `detect_shell()` with `#[cfg(unix)]` / `#[cfg(windows)]` / `#[cfg(not(any(...)))]` blocks
9. Update `detect_capabilities()`: add `has_pacman: command_exists("pacman")`, set `is_wsl2: is_wsl2()`, prefix `_platform` parameter
10. Update existing test `test_detect_capabilities`: change `caps.is_wsl` -> `caps.is_wsl2`, revise assertion logic
11. Add `OsProbe` trait and `_with_probe` variants inside `#[cfg(test)]`
12. Add 10 machine-dependent tests + 14 mock-based tests
13. `cargo clippy` — expect zero warnings
14. `cargo test` — all tests pass

---

## Dependency Map (call graph for changed code)

```
detect_platform_info()
  -> detect_platform()
       -> is_wsl()              [MODIFIED: adds WSLInterop check]
       -> detect_linux_distro() [unchanged]
       -> detect_linux_version() [unchanged]
       -> detect_macos_version() [unchanged]
       -> detect_architecture()  [unchanged]
  -> detect_capabilities()      [MODIFIED: has_pacman, is_wsl2, _platform]
       -> command_exists()       [MODIFIED: which::which()]
       -> is_wsl2()              [NEW]
  -> is_root()                  [MODIFIED: id -u + cfg(unix)]
  -> detect_shell()             [MODIFIED: cfg blocks + $COMSPEC]

command_exists()               [MODIFIED]
  <- package_manager.rs (unchanged call sites)
  <- runtime.rs (unchanged call sites)
  <- cli/status.rs (unchanged call sites)
  <- cli/doctor.rs (unchanged call sites)
  <- cli/apply.rs (unchanged call sites)
  <- cli/diff.rs (unchanged call sites)
  <- cli/mcp.rs (unchanged call sites)
  <- vault/mod.rs (unchanged call sites)
```

---

## Technical Debt Observed

1. `is_root()` current implementation (line 248-250) uses `std::env::var("USER").is_ok_and(|u| u == "root")` — this silently fails in Docker containers running as root without `$USER` set. The spec fix (`id -u`) is correct and necessary.

2. `command_exists()` current implementation spawns `sh -c "command -v {}"` — this is a shell injection vector if `cmd` ever contained shell metacharacters. The `which` crate eliminates this surface entirely.

3. `#[allow(unused_imports)]` in `mod.rs` (lines 5 and 11) — the platform module re-exports symbols not all used by every consumer. This suppressor is acceptable but worth noting: if any symbol is removed from `detection.rs`, the re-export will fail to compile. Verify re-exports after struct changes.

4. `Platform` derives `PartialEq` but not `Serialize/Deserialize` — this is the gap the spec closes. JSON output in `status --json` (status.rs line 164-172) currently uses manual `format!()` with `{:?}` — after adding `Serialize`, future tasks can use `serde_json::to_string()` for proper serialization.
