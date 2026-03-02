# Release Notes: Task 0020 — Docker Cross-Compilation UX Improvements

**Date:** 2026-02-27
**Scope:** `docker/cross-windows.Dockerfile`, `docker/test.sh`, `docker/cross-test-macos.sh`, `docker/cross-test-windows.sh`, `docker/cross-test-linux-aarch64.sh`, `docker-compose.yml`

---

## What Changed

Four UX issues in the Docker cross-compilation infrastructure are now fixed.
No Rust source changes. No Cargo version bump.

### Windows Dockerfile now routes through the validation script

`docker/cross-windows.Dockerfile` previously set its default `CMD` to a bare
`cargo build` invocation, bypassing the validation and export logic in
`cross-test-windows.sh` when the container was run directly with `docker run`.
The macOS and Linux aarch64 Dockerfiles already invoked their respective
validation scripts. The Windows Dockerfile now matches that pattern:

```dockerfile
WORKDIR /build
CMD ["bash", "/workspace/docker/cross-test-windows.sh"]
```

The usage comment block has been updated to remove the bare `cargo build`
example, which was misleading. Direct `docker run` and `docker compose run`
now take the same code path.

### `great doctor` failures are now visible in test logs

`docker/test.sh` previously suppressed the `great doctor` exit code with
`|| true`. A real regression in `great doctor` would produce no signal in CI
logs. The line is now replaced with a capture-and-warn pattern that is safe
under `set -e`:

```bash
doctor_rc=0
${BIN} doctor 2>&1 || doctor_rc=$?
if [ "$doctor_rc" -ne 0 ]; then
    echo "[WARN] great doctor exited non-zero (exit ${doctor_rc})"
fi
```

The script continues after a non-zero exit (a minimal container may legitimately
lack optional tools), but the failure is now visible in the build log.

### All four entrypoint scripts print the Rust toolchain version at startup

Each of the four Docker entrypoint scripts now prints the active Rust toolchain
version before the first numbered step. Build logs can be traced back to an
exact toolchain without exec-ing into the container:

```
Toolchain: rustc 1.88.0 (07dca489a 2025-05-08)
```

The line is placed after the opening banner and before `[1/N]` in every script.
In the macOS cross script it appears after `source /etc/profile.d/osxcross-env.sh`
so the correct toolchain is active.

### Export paths moved from `/workspace/test-files/` to `/build/test-files/`

All three cross-compilation scripts documented `/workspace` as read-only but
wrote exported binaries to `/workspace/test-files/`. This worked in practice
only because `docker-compose.yml` placed a writable bind-mount overlay at that
exact path. Running the container directly with `docker run` — without the
overlay — caused `mkdir -p /workspace/test-files` to fail against the read-only
filesystem boundary.

The export target is now `/build/test-files/` in all three scripts, which is
inside the writable `/build` tree that the scripts already use for compilation.
The companion bind-mount in `docker-compose.yml` is updated for each cross
service:

```yaml
- ./test-files:/build/test-files
```

The host-side `./test-files/` directory continues to receive the exported
binaries unchanged.

---

## Migration Notes

If you run cross-compilation containers directly with `docker run` (not via
`docker compose`), the exported binaries are now written to `/build/test-files/`
inside the container instead of `/workspace/test-files/`. Update any host-side
scripts that exec into a running container and look for binaries at the old
path. Users who only use `docker compose run` will see no change: the
`./test-files/` directory on the host receives binaries as before.
