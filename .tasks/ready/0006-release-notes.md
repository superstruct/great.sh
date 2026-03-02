# Release Notes — Task 0006: `great diff` Gap Completion

**Date:** 2026-02-25
**Scope:** `src/cli/diff.rs`, `tests/cli_smoke.rs`
**Cargo version:** 0.1.0 (no version bump; pre-release feature completion)

---

## Summary

Five correctness gaps in `great diff` are now closed. The command accurately
reports version mismatches, exits non-zero when no configuration is found,
silently skips disabled MCP servers, shows a numeric summary by category, and
uses the correct red `-` marker for unresolved secrets.

---

## User-facing changes

### Version mismatch detection (`~` yellow marker)

Previously, tools that were already installed were silently omitted from the
diff output regardless of whether the installed version matched the declared
version. The command now checks the installed version against the declared
version and prints a `~` (yellow) line when they differ.

```
  ~ node (want 20.11.0, have node v22.14.0)
```

Versions declared as `"latest"` or `"stable"` always match; no `~` line is
shown for them.

### Exit code 1 when no `great.toml` found

`great diff` previously exited 0 (success) when no `great.toml` was present.
It now exits 1. Scripts and CI pipelines that test for a non-zero exit code to
detect a missing configuration will now work correctly.

```sh
great diff || echo "No great.toml — run great init first"
```

### Disabled MCP servers are silently skipped

MCP servers declared with `enabled = false` in `great.toml` were previously
still checked and could appear in the diff output. They are now skipped
entirely. No output line is printed; silence means no action is needed.

### Numeric summary line

The final output line now reports a count per category instead of a generic
message. Zero-count categories are omitted.

```
2 to install, 1 to configure, 1 secrets to resolve — run `great apply` to reconcile.
```

When the environment fully matches the configuration:

```
Environment matches configuration — nothing to do.
```

### Unresolved secrets use `-` (red) instead of `+` (green)

Missing secrets in both `secrets.required` and MCP `env` references now show
with a red `-` marker. The `+` (green) marker is reserved for items that
`great apply` can install automatically; secrets require the user to supply a
value manually (e.g., via `great vault set`). The visual distinction makes this
clear at a glance.

```
  - MY_API_KEY (not set in environment)
```

---

## Technical changes

- `src/cli/diff.rs`: added `use crate::cli::util` import to reach
  `util::get_command_version`.
- Three counters (`install_count`, `configure_count`, `secrets_count`)
  introduced alongside `has_diff`. Each diff push site increments the
  appropriate counter.
- `std::process::exit(1)` replaces `return Ok(())` in the missing-config error
  path, matching the pattern in `doctor.rs` and `status.rs`.
- `mcp.enabled == Some(false)` guard added at the top of the MCP loop body,
  matching the pattern in `doctor.rs`.
- Summary block replaced: `parts.join(", ")` builds the category string;
  zero-count categories are excluded.
- Docstring for `run()` updated to document the `-` (red) marker.
- Section header changed from `"Tools — need to install:"` to `"Tools"` because
  the section now includes both installation (`+`) and version-mismatch (`~`)
  lines.

No new dependencies. No new files. No public API surface changes.

---

## Test coverage

8 integration tests added to `tests/cli_smoke.rs` (85 tests total in that
file):

| Test | What it covers |
|------|----------------|
| `diff_no_config_exits_nonzero` | Replaces the old `diff_no_config_shows_error`; asserts `.failure()` |
| `diff_satisfied_config_exits_zero` | Satisfied config exits 0, prints "nothing to do" |
| `diff_missing_tool_shows_plus` | Missing tool produces `+` marker and summary on stderr |
| `diff_disabled_mcp_skipped` | Disabled MCP server absent from all output |
| `diff_version_mismatch_shows_tilde` | `git = "99.99.99"` produces `~` and "want 99.99.99" |
| `diff_with_custom_config_path` | `--config` flag exercises the non-discovery code path |
| `diff_summary_shows_counts` | Summary line contains "1 to install" and "1 secrets to resolve" |
| `diff_unresolved_secret_shows_red_minus` | Secret name on stdout; "not set in environment" text present |

---

## Migration notes

No configuration changes are required. The only breaking change is the exit
code correction (exit 1 on missing config). Any script that previously relied
on `great diff` always exiting 0 will now receive exit 1 when no `great.toml`
is present; update those scripts to run `great init` before `great diff`, or
check for the config file explicitly.
