# Release Notes — Task 0040

## Changed
- `great status` now always exits 0, matching `great status --json` behavior
- Previously, `great status` exited 1 when tools or secrets were missing; now issues are reported via colored output only
- This aligns with `git status` convention: exit 0 means the command ran successfully, output carries diagnostic info

## Migration
- Scripts that relied on `great status` exit code to detect missing tools should use `great status --json` and check the `has_issues` field instead
- `great doctor` remains available for exit-code-based health checks
