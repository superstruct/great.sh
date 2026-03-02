# Security Audit: 0002 TOML Config Schema Enrichment

**Auditor:** Auguste Kerckhoffs
**Date:** 2026-02-24
**Verdict:** BLOCK (compilation failure in test code)

---

## Files Reviewed

| File | Lines | Role |
|------|-------|------|
| `src/config/schema.rs` | 803 | New fields, validation, secret scanning |
| `src/cli/init.rs` | 465 | Interactive wizard, config construction |
| `src/cli/template.rs` | 616 | Template merge, test construction sites |
| `src/mcp/mod.rs` | 283 | MCP server management, test construction |
| `src/config/mod.rs` | 119 | Config loader, validation runner |
| `templates/*.toml` | 4 files | Built-in templates |

---

## BLOCKING: Compilation Failure (HIGH)

### B1: `template.rs` test code does not compile

`ProjectConfig` gained a `version: Option<String>` field, but three test
construction sites in `src/cli/template.rs` explicitly list fields without
`..Default::default()`:

- Line 345: `test_merge_existing_takes_precedence_project`
- Line 352: same test, template side
- Line 368: `test_merge_template_fills_gap_project`

The compiler error is:
```
error[E0063]: missing field `version` in initializer of `schema::ProjectConfig`
```

**Fix:** Add `version: None,` to each `ProjectConfig { ... }` literal, or
(preferred) replace the explicit field list with `..Default::default()` to
avoid the same breakage when future fields are added.

**Impact:** `cargo test` fails. This blocks the commit.

---

## Security Findings

### S1: `api_key` field -- serialization leak risk (MEDIUM)

`AgentConfig` derives both `Serialize` and `Debug`. When `api_key` contains a
literal secret (not a `${...}` reference), it will be emitted:

1. **By `toml::to_string_pretty()`** in `init.rs:271` when writing `great.toml`.
   This is by design -- the file is the user's own config.

2. **By `Debug` derive** if any future code prints an `AgentConfig` via `{:?}`.
   Currently no production code does this, but the derive is a latent risk.

The `#[serde(skip_serializing_if = "Option::is_none")]` annotation prevents the
field from appearing in serialized output when `None`, which is correct.

**Recommendation (P2):** Add a custom `Debug` impl for `AgentConfig` that
redacts `api_key`, or document that `api_key` should always use `${...}`
references and add a validation warning when a literal key is detected.

### S2: `find_secret_refs()` does not scan `McpConfig.url` (LOW)

The `url` field in `McpConfig` could contain `${SECRET_NAME}` references (e.g.,
an authenticated endpoint URL). `find_secret_refs()` only scans:
- `AgentConfig.api_key`
- `McpConfig.env` values

It does not scan `McpConfig.url` or `McpConfig.args`.

**Current risk:** LOW. The `url` field is not yet consumed by any HTTP client
code. No actual secret leak today. But when HTTP transport is implemented, an
unresolved `${...}` reference in a URL would be sent as a literal string.

**Recommendation (P3):** Extend `find_secret_refs()` to also scan `url` and
`args` fields.

### S3: `enabled = false` bypass risk (LOW -- no risk today)

The `enabled` field on `AgentConfig` and `McpConfig` is not yet consumed by
any production code path. `apply.rs`, `status.rs`, and `doctor.rs` do not
filter on `enabled`. This means:

- Setting `enabled = false` on an MCP server does NOT prevent it from being
  written to `.mcp.json` by `great apply`.
- Setting `enabled = false` on an agent has no effect.

**Current risk:** LOW. The feature is inert. No bypass because there is no
gate to bypass.

**Recommendation (P3):** When the `enabled` field is wired into `apply.rs`,
ensure it is checked *before* any side effects (process spawning, file writes).
Add test coverage for `enabled = false` preventing MCP server configuration.

### S4: `.expect("valid regex")` in `find_secret_refs()` (LOW)

Line 248: `Regex::new(r"...").expect("valid regex")` is a compile-time-constant
regex pattern. This is safe -- the regex is hardcoded and known-valid. The
`.expect()` will never fire at runtime.

This does NOT violate the "no `.unwrap()` in production" rule because:
- The pattern is a string literal, not user input.
- `expect()` with a message is acceptable for invariants per Rust conventions.

**Status:** ACCEPTABLE.

### S5: No hardcoded secrets in templates (PASS)

All four templates (`ai-fullstack-ts.toml`, `ai-fullstack-py.toml`,
`ai-minimal.toml`, `saas-multi-tenant.toml`) reference secrets by name only
(e.g., `required = ["ANTHROPIC_API_KEY"]`). No literal keys or tokens found.

### S6: Secret ref names printed safely (PASS)

`diff.rs` and `doctor.rs` print secret *reference names* (e.g.,
`"ANTHROPIC_API_KEY"`) but never print the *resolved values*. The pattern:
```rust
if std::env::var(secret_ref).is_ok() { ... }
```
checks for existence without capturing or printing the value. This is correct.

### S7: No `.unwrap()` in production code (PASS)

All `.unwrap()` calls in `schema.rs` are within `#[cfg(test)]` blocks.
Production code uses `?` propagation, `anyhow::Context`, or `.expect()` with
invariant messages on compile-time constants.

### S8: Validation coverage (PASS)

The `validate()` function correctly flags:
- Empty MCP commands (Error)
- Unknown transport types (Warning)
- HTTP transport missing URL (Error)
- Unknown secrets providers (Warning)
- Unknown agent providers (Warning)
- Missing agent provider/model (Warning)
- Invalid secret names (Error)

The error/warning severity levels are appropriate.

---

## Summary

| ID | Severity | Status | Description |
|----|----------|--------|-------------|
| B1 | HIGH | BLOCK | `template.rs` tests do not compile (missing `version` field) |
| S1 | MEDIUM | P2 | `api_key` visible through `Debug` derive (latent risk) |
| S2 | LOW | P3 | `find_secret_refs()` does not scan `url`/`args` |
| S3 | LOW | P3 | `enabled` field is inert -- no production consumer yet |
| S4 | LOW | OK | `.expect()` on compile-time regex is acceptable |
| S5 | -- | PASS | No hardcoded secrets in templates |
| S6 | -- | PASS | Secret names printed, not values |
| S7 | -- | PASS | No `.unwrap()` in production code |
| S8 | -- | PASS | Validation coverage is comprehensive |

**Verdict: BLOCK** -- B1 must be fixed before commit. The code does not compile.
After B1 is resolved, re-run `cargo test` and this audit passes as CONDITIONAL PASS.
