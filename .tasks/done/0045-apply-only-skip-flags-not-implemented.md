# 0045 — `apply --only` and `--skip` flags not implemented

| Field | Value |
|---|---|
| Priority | P2 |
| Type | feature |
| Module | `src/cli/apply.rs` |
| Status | backlog |
| Estimated Complexity | M |

## Problem

The comprehensive test suite expects `great apply` to support `--only <category>` and `--skip <category>` flags for selective provisioning, but these flags don't exist:

```
error: unexpected argument '--only' found
error: unexpected argument '--skip' found
```

## Failing Tests

- `apply --only tools --dry-run`
- `apply --only mcp --dry-run`
- `apply --only agents --dry-run`
- `apply --skip tools --dry-run`

## Decision Required

Either:
1. **Implement the flags** — Add `--only` and `--skip` args to the `apply` subcommand, accepting categories: `tools`, `mcp`, `agents`, `secrets`
2. **Update the test suite** — Remove these tests if selective apply isn't planned

## Evidence

Docker test run 2026-03-04. All 4 tests exit with code 2 (clap argument error).
