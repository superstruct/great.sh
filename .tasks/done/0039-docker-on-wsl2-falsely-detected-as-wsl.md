# 0039 — Docker-on-WSL2 container falsely detected as WSL

| Field                | Value                              |
|----------------------|------------------------------------|
| Priority             | P2                                 |
| Type                 | bugfix                             |
| Module               | `src/platform/detection.rs`        |
| Status               | backlog                            |
| Estimated Complexity | Small                              |

## Context

`great status` (and any command that branches on platform) reports `WSL Ubuntu 24.04` when run inside a Docker container on a WSL2 host. The container should be identified as plain `Linux Ubuntu 24.04`.

**Reproduction**

```bash
docker run --rm \
  -v /path/to/great:/usr/local/bin/great:ro \
  ubuntu:24.04 \
  great status 2>&1 | grep Platform
# Actual:   ℹ Platform: WSL Ubuntu 24.04 (x86_64)
# Expected: ℹ Platform: Linux Ubuntu 24.04 (x86_64)
```

**Root cause — three detection tiers, all reachable inside Docker-on-WSL2**

`is_wsl()` in `src/platform/detection.rs` (lines 169–181) runs three checks in order:

1. `WSL_DISTRO_NAME` env var — normally not set inside Docker, so this tier is safe.
2. `/proc/sys/fs/binfmt_misc/WSLInterop` path check — Docker containers on WSL2 share the host kernel's binfmt_misc namespace; this file **is present** inside containers, causing a false positive.
3. `/proc/version` contains "microsoft" — inherited from the WSL2 kernel (`6.6.87.2-microsoft-standard-WSL2`), also a false positive.

Both tiers 2 and 3 are hit. `is_wsl2()` (lines 187–189) has the same WSLInterop false positive, so `PlatformCapabilities.is_wsl2` is also wrong.

**Impact of false positive**

- `great apply` may attempt WSL-specific actions: copying fonts to `C:\Users\...\AppData\Local\Microsoft\Windows\Fonts`, invoking `cmd.exe` (fails with "No such file or directory").
- `great doctor` may suggest WSL-specific tooling (`wslu`, etc.) to a container.
- Install paths and generated scripts may be incorrect for container environments.

**Container indicators to check (before concluding WSL)**

| Indicator | Path / mechanism | Notes |
|-----------|-----------------|-------|
| `/.dockerenv` | File present in all Docker containers | Most reliable; Docker always creates it |
| `DOCKER_CONTAINER` env var | Set by some base images / compose setups | Secondary |
| `container` env var | Set by some OCI runtimes (podman, etc.) | Catches non-Docker containers too |
| cgroup v1 `/proc/1/cgroup` | Contains "docker" in container | Fallback; cgroup v2 may not have it |

The recommended fix: before returning `true` from `is_wsl()`, check for any container indicator and return `false` if one is found. Same guard needed in `is_wsl2()`.

## Acceptance Criteria

1. `is_wsl_with_probe()` returns `false` when `/.dockerenv` exists, even if `/proc/sys/fs/binfmt_misc/WSLInterop` exists and `/proc/version` contains "microsoft".
2. `is_wsl_with_probe()` returns `false` when the `container` or `DOCKER_CONTAINER` env var is set, regardless of other WSL indicators.
3. `is_wsl_with_probe()` continues to return `true` on a genuine WSL2 environment (WSLInterop present, no container indicators).
4. `is_wsl2_with_probe()` is subject to the same container-exclusion guard and returns `false` inside a detected container.
5. `great status` run inside an `ubuntu:24.04` Docker container on a WSL2 host prints `Platform: Linux Ubuntu 24.04 (x86_64)`, not `WSL Ubuntu 24.04`.

## Files That Need to Change

- `src/platform/detection.rs` — `is_wsl()`, `is_wsl2()`, `is_wsl_with_probe()`, `is_wsl2_with_probe()`, `MockProbe`-based tests (add container scenarios).

## Dependencies

None — self-contained platform module change.

## Out of Scope

- Detecting other container runtimes (LXC, systemd-nspawn) beyond the indicators listed.
- Changing the `Platform` enum to add a `Container` variant (separate task if needed).
- Any changes to `great apply` WSL-specific logic (that is downstream; fixing detection unblocks it).
