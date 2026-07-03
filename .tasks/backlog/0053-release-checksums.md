# 0053 — Release pipeline publishes no checksums; install.sh verifies nothing

| Field | Value |
|---|---|
| Priority | P2 |
| Type | security |
| Module | `.github/workflows/release.yml`, `install.sh`, `site/public/install.sh` |
| Status | backlog |
| Estimated Complexity | M |

## Problem

`curl | sh` installation has zero integrity checking: the release workflow
publishes no SHA256SUMS and install.sh verifies nothing. A compromised or
truncated download installs silently. (install.sh now at least fails on HTTP
errors via `curl -f`.)

## Proposed Fix

1. release.yml: generate `SHA256SUMS` over the four binaries and attach it to
   the GitHub Release.
2. install.sh: download `SHA256SUMS`, verify the fetched binary with
   `sha256sum -c --ignore-missing` (or `shasum -a 256` on macOS) before
   `install`. Skip with a loud warning only if no checksum tool exists.
3. Keep `install.sh` and `site/public/install.sh` in sync (consider a CI check
   that diffs them).

## Acceptance Criteria

- Tampered binary fails installation with a clear error
- Fresh release installs cleanly on macOS and Linux
- Both install.sh copies stay identical
