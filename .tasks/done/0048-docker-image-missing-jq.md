# 0048 — Docker Ubuntu image missing `jq` — hook_state test fails

| Field | Value |
|---|---|
| Priority | P3 |
| Type | bug |
| Module | `docker/ubuntu.Dockerfile`, `tests/hook_state.rs` |
| Status | done |
| Estimated Complexity | XS |

## Problem

The `test_hook_writes_state_and_statusline_reads_it` integration test requires `jq` to be installed, but the Ubuntu Docker image doesn't include it:

```
jq is required for this test and must be installed (apt install jq / brew install jq):
Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

## Fix

Add `jq` to the `apt-get install` line in `docker/ubuntu.Dockerfile`:

```dockerfile
RUN apt-get update && apt-get install -y \
    curl git build-essential pkg-config libssl-dev libdbus-1-dev libsecret-1-dev jq \
    && rm -rf /var/lib/apt/lists/*
```

## Evidence

Docker test run 2026-03-04. `cargo test --release` — hook_state test panics with "No such file or directory" for jq.
