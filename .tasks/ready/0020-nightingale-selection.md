# Nightingale Selection — Task 0020
# Docker Cross-Compilation UX Improvements

**Selected:** 2026-02-27
**Iteration:** 023
**Priority:** P2
**Type:** bugfix / UX
**Module:** `docker/`
**Estimated Complexity:** S

---

## Priority Justification

P2 is correct. Two of the four issues (issues 1 and 2) are silent failure modes:
the Windows Dockerfile CMD exits zero after a bare build regardless of output
correctness, and `test.sh` line 41 swallows `great doctor` failures with `||
true`. Both will mask regressions in CI. The remaining two (issues 3 and 4) are
lower-impact but issue 4 — the fragile `mkdir` against a read-only mount — will
cause a hard failure in the macOS cross-compilation path whenever the host bind
mount is actually read-only, making it a latent P2 as well.

No higher-priority open tasks exist. This is the only genuinely open backlog
item.

---

## Scope Summary

**In scope:**

1. Replace the bare `CMD ["cargo", "build", ...]` in
   `docker/cross-windows.Dockerfile` with a wrapper script that validates the
   output binary (confirms `.exe` and PE format) and exits non-zero on failure.

2. Promote `great doctor` exit-code handling in `docker/test.sh` line 41 from
   silent `|| true` to a logged warning that still does not abort the run
   (doctor checks may legitimately fail inside a minimal container), but prints
   a visible `[WARN] great doctor exited non-zero` line so failures are
   observable.

3. Add a `rustc --version && rustup show active-toolchain` line near the top of
   each cross-compilation container entrypoint (before step 1 output) so the
   active toolchain is visible in build logs.

4. Move the `mkdir -p /workspace/test-files` in
   `docker/cross-test-macos.sh` line 66 to write into `/build/test-files/`
   (the writable build dir already used by that script), and update the
   subsequent `cp` destination paths to match. Add a comment explaining that
   `/workspace` is a read-only mount and output staging uses `/build`.

**Out of scope:**

- Changing the compose setup or mount configuration
- Adding new cross-compilation targets
- Any changes to `cross-test-linux-aarch64.sh` or the Linux Dockerfiles unless
  they share the same `mkdir` pattern (they do not — verified)

---

## Refined Acceptance Criteria

The original task had a single vague checkbox. Replacing with five testable
criteria:

- [ ] `docker/cross-windows.Dockerfile` CMD invokes a validation script that
  runs `file` on the output binary and exits non-zero if the binary is absent
  or not a PE32+ executable.

- [ ] `docker/test.sh` line 41 no longer uses `|| true`; a `great doctor`
  non-zero exit prints `[WARN] great doctor exited non-zero (exit $?)` to
  stdout and the script continues.

- [ ] Each cross-compilation entrypoint script (`cross-test-macos.sh`,
  `cross-test-windows.sh`, `cross-test-linux-aarch64.sh`, `test.sh`) prints
  `rustc --version` output before step 1.

- [ ] `docker/cross-test-macos.sh` writes exported binaries to `/build/test-files/`
  rather than `/workspace/test-files/`, with an explanatory comment that
  `/workspace` is read-only.

- [ ] All four Docker scripts pass `shellcheck` with no errors.

---

## Key Risks and Blockers

- **Windows script wrapper**: a new `cross-test-windows.sh` will be needed (none
  exists yet; only `cross-test-windows.sh` is present in `docker/`). The
  Windows Dockerfile CMD must be updated to invoke it. Da Vinci should confirm
  whether `cross-test-windows.sh` already exists before creating it.
  (Checked: `docker/cross-test-windows.sh` does exist — read it before writing
  to avoid duplication.)

- **Read-only mount reality**: the issue description says the macOS workspace
  mount is "partially read-only." The scripts confirm the intent (comment at
  line 19 of `cross-test-macos.sh`) but the Dockerfile does not enforce it via
  `:ro`. If the mount is not actually `:ro` in compose, the `mkdir` works by
  accident. The fix should be to move the export target to `/build` regardless,
  and the compose file should be updated to document or enforce `:ro` on the
  workspace volume. This is a small compose change but should be bundled with
  the fix to make the intent explicit.

- **No blockers**: all four files are shell scripts and one Dockerfile; no
  Rust compilation required.

---

## Recommended Approach

1. Read `docker/cross-test-windows.sh` to confirm its current state before
   any edits.
2. Fix issues in dependency order: issue 4 first (data correctness — wrong
   output path), then issue 2 (silent failure masking), then issue 1 (Windows
   CMD wrapper), then issue 3 (toolchain version — purely additive, lowest
   risk of regression).
3. Run `shellcheck` on all modified scripts as a quality gate before committing.
4. A single commit per issue keeps the diff reviewable and makes bisection
   straightforward, but all four may be batched into one PR given the S
   complexity estimate.
