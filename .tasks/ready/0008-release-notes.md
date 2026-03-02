# Release Notes: Task 0008 — Runtime Version Manager (mise)

**Module:** `src/platform/runtime.rs`
**Date:** 2026-02-25

---

## Bug Fixes

- **`version_matches`: prefix-boundary false positive eliminated.**
  Previously, a declared version of `"3.12"` would match an installed version of
  `"3.120.0"` because the check used a bare string prefix. The fix requires the
  character immediately following the declared prefix to be a dot (`'.'`), so
  `"3.12"` now correctly matches `"3.12.5"` and `"3.12.0"` but not `"3.120.0"`.
  The same boundary rule applies to major-only declarations: `"22"` matches
  `"22.11.0"` but not `"220.0.0"`. Exact matches and the keywords `"latest"` /
  `"stable"` are unaffected.

- **`installed_version`: handles both "No version" and "Not installed" output.**
  Different mise versions emit different strings when a runtime is absent
  (`"No version set for <name>"` vs `"not installed"`). Both are now detected
  case-insensitively and mapped to `None`, preventing a spurious `Some("not
  installed")` from propagating to callers such as `great diff` and `great
  status`.

---

## Improvements

- **`ensure_installed`: prefers Homebrew when available.**
  When `brew` is on `PATH` (macOS or Linuxbrew), `ensure_installed` now runs
  `brew install mise` instead of the curl pipe. The curl installer (`curl -fsSL
  https://mise.jdx.dev/install.sh | sh`) is retained as a fallback for systems
  without Homebrew. This avoids piping untrusted shell scripts on machines that
  already have a package manager capable of installing mise.

- **Actionable error messages throughout.**
  Every failure path now includes the process exit code and a concrete next step.
  Examples:
  - `brew install mise failed (exit code 1) — install manually: https://mise.jdx.dev`
  - `mise install node@22 failed (exit code 1) — check runtime name and version are valid`
  - `mise was installed but not found on PATH — you may need to restart your shell or add ~/.local/bin to PATH`
  - `mise is not installed — run \`great doctor\` for installation instructions`

---

## Testing

The test suite for `src/platform/runtime.rs` grows from 7 tests to 15 tests.

New tests added in this task:

| Test | What it covers |
|---|---|
| `test_version_matches_exact_only_declared` | Exact match when declared == installed (no trailing dot) |
| `test_version_matches_stable_keyword` | `"stable"` matches any version string, including pre-releases |
| `test_version_matches_no_false_longer_prefix` | Regression: `"3.12"` must not match `"3.120.0"`; `"1.7"` must not match `"1.78.0"` |
| `test_version_matches_partial_major` | `"3"` matches `"3.12.5"` but not `"30.0.0"` |
| `test_version_does_not_panic` | `MiseManager::version()` returns without panic on any host |
| `test_installed_version_nonexistent_runtime` | Returns `None` for an impossible runtime name |
| `test_installed_version_does_not_panic` | `installed_version("node")` safe on hosts without mise |
| `test_provision_skips_cli_key` | `provision_from_config` ignores the reserved `"cli"` key |
| `test_provision_empty_runtimes` | `provision_from_config` returns empty vec for empty config |

All tests run on every `cargo test` invocation without requiring mise to be
installed (network-free, no side effects).

---

## Migration Notes

No configuration changes required. The `ensure_installed` signature changed
internally (the `PackageManager` parameter from the original spec was removed in
favour of Homebrew auto-detection), but this function is not part of any public
CLI contract exposed to users or config files.

If you previously relied on `installed_version` returning a non-`None` string
like `"not installed"`, update callers to treat `None` as the canonical
"not present" signal.
