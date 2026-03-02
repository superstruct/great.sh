# Task 0031 — Release Notes: Loop and MCP-Bridge Smoke Tests

## Summary

Added 6 new integration tests for `great loop` and `great mcp-bridge` subcommands, doubling test coverage for these commands from 6 to 12 tests.

## What Changed

**tests/cli_smoke.rs** now includes smoke tests for:
- `great loop --help` — verifies help output contains "Install" and "Status"
- `great loop status` — confirms graceful exit on fresh HOME (no prior install)
- `great loop install --force` — validates agent file writes to temp HOME
- `great loop uninstall` — confirms no-op behavior when not installed
- `great mcp-bridge --help` — verifies help output
- `great mcp-bridge --preset <invalid>` — confirms error path and exit nonzero

## Coverage

Tests are fully isolated using TempDir for HOME, ensuring CI safety with no network or keychain dependencies. All tests pass on ubuntu-latest and macos-latest.

## Why

`loop` and `mcp-bridge` shipped without integration test coverage. These tests catch regressions in file write paths, argument parsing, and error handling before release.

## Files Modified

- `tests/cli_smoke.rs` — 6 new test functions

No breaking changes. This is a test-only addition.
