# 0052 — Vault providers mask errors as "not found"

| Field | Value |
|---|---|
| Priority | P2 |
| Type | bug |
| Module | `src/vault/mod.rs` |
| Status | backlog |
| Estimated Complexity | M |

## Problem

`SecretProvider::get` for OnePassword, Bitwarden, and Keychain returns
`Ok(None)` on any non-zero exit from the underlying CLI. An auth failure,
locked vault, or network error is indistinguishable from a genuinely absent
key, so callers (e.g. `great apply` credential injection) silently proceed
without the secret instead of surfacing the broken provider.

## Proposed Fix

Give `get` a tri-state result (found / absent / provider error) — e.g.
`Result<Option<String>, ProviderError>` where known "not found" exit
codes/stderr patterns map to `Ok(None)` and everything else is an error with
the CLI's stderr attached.

## Acceptance Criteria

- Locked/unauthenticated provider produces an actionable error, not a silent miss
- Genuinely missing keys still return None without error
- Unit tests using a fake provider binary cover both paths
