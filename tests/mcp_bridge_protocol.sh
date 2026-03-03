#!/usr/bin/env bash
set -euo pipefail

# Protocol smoke test for the MCP bridge server (positive path: with backends).
# Not run by `cargo test` -- use manually or in CI.
# Requires at least one AI CLI backend on PATH.
# For the no-backends (degraded mode) path, see tests/cli_smoke.rs:
#   mcp_bridge_starts_without_backends
#   mcp_bridge_no_backends_emits_warning

RESULT=$(printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}\n{"jsonrpc":"2.0","method":"notifications/initialized"}\n{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}\n' \
  | timeout 10 cargo run -- mcp-bridge --preset minimal --log-level off 2>/dev/null)

echo "$RESULT" | grep -q '"protocolVersion"' || { echo "FAIL: no protocolVersion"; exit 1; }
echo "$RESULT" | grep -q '"tools"' || { echo "FAIL: no tools"; exit 1; }
echo "PASS: MCP bridge protocol smoke test"
