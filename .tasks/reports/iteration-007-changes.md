## Changelog — Task 0004: `great status` Command Completion

**Date:** 2026-02-24
**Scope:** `src/cli/status.rs`, `tests/cli_smoke.rs`

### What changed

- `great status --json` now emits complete status: tools (installed/missing + detected versions), MCP servers (command availability), agents, and secrets (set/missing). Previously only reported platform, arch, and shell.
- `great status --verbose` now shows full version strings for tools and full command/args/transport for MCP servers.
- Replaced `.unwrap_or_default()` on config path UTF-8 conversion with proper error propagation and a user-visible warning on non-UTF-8 paths.
- Exit code semantics added: exits 1 when missing tools or required secrets are detected. `--json` mode always exits 0 and encodes issues in the payload.
- 12 new integration tests added (no-config, valid-config, `--json` valid JSON, `--verbose` accepted, exit-code-on-missing).

### Migration

None required. The new `--json` fields are additive. Existing scripts that parse the old minimal JSON output will receive more fields; no existing fields were removed or renamed.
