# 0046 — `doctor` exits non-zero in minimal environments (Docker)

| Field | Value |
|---|---|
| Priority | P3 |
| Type | bug |
| Module | `src/cli/doctor.rs`, `test-in-docker.sh` |
| Status | backlog |
| Estimated Complexity | S |

## Problem

`great doctor` exits with code 1 in Docker containers because required tools (gh, claude, etc.) are missing. The test suite expects exit code 0.

This is a test-vs-reality mismatch — doctor *correctly* reports failures, but the test incorrectly expects it to always exit 0.

## Failing Tests

- `doctor exits 0` (exit=1)
- `doctor --fix exits 0` (exit=1)
- `doctor --non-interactive exits 0` (exit=1)
- `doctor with config exits 0` (exit=1)
- `doctor --fix --non-interactive` (exit=1)

## Proposed Fix

Update `test-in-docker.sh` to expect non-zero exit from doctor in Docker:
- Change `ok` to `contains` assertions that verify doctor *runs* and *shows output*, without requiring exit 0
- Or add a `--no-fail` flag to doctor that reports issues but exits 0

## Additional Finding

The test `doctor notes root/sudo status` also fails — doctor output doesn't mention root/sudo/user/permission when running as root. Consider adding a root-user notice to doctor output.

## Evidence

Docker test run 2026-03-04. Doctor correctly detects missing tools and exits 1.
