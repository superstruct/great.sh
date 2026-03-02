# Socrates Review -- Task 0020: Docker Cross-Compilation UX Improvements

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Iteration:** 023
**Date:** 2026-02-27
**Spec:** `.tasks/ready/0020-docker-ux-spec.md`
**Round:** 1

---

## VERDICT: APPROVED

The specification is thorough, well-evidenced, and directly implementable. Every "current behavior" claim was verified against the actual files. The proposed changes are minimal, isolated, and address all four backlog issues. The scope contradiction between Nightingale and the compose mount change is correctly identified and resolved. No BLOCKING concerns were found.

---

## Per-Issue Analysis

### Issue 1: Windows cross Dockerfile CMD skips validation

**Current behavior correctly identified?** Yes. `docker/cross-windows.Dockerfile` line 18 is indeed `CMD ["cargo", "build", "--release", "--target", "x86_64-pc-windows-gnu"]`. The compose override at `docker-compose.yml` line 66 is confirmed as `command: ["bash", "/workspace/docker/cross-test-windows.sh"]`. The `cross-linux-aarch64.Dockerfile` line 20 does already use `CMD ["bash", "/workspace/docker/cross-test-linux-aarch64.sh"]` as claimed.

**Does the proposed change fix the problem?** Yes. Changing CMD to invoke the validation script ensures both `docker run` and `docker compose run` go through the same validation path. Adding `WORKDIR /build` aligns with `cross-linux-aarch64.Dockerfile` line 18 and is consistent with compose `working_dir: /build` at line 65.

**Edge cases?** The spec correctly notes the redundancy between Dockerfile `WORKDIR /build` and compose `working_dir: /build`, and explains why both are needed (compose takes precedence, but direct `docker run` needs the Dockerfile value).

**The resulting file is complete and self-consistent.** The spec provides the full target Dockerfile content (7 lines), which I verified against the current file. The transformation is clean: remove line 6 (bare cargo example), change WORKDIR from /workspace to /build, change CMD.

**Assessment: Sound.**

### Issue 2: test.sh silently swallows great doctor failures

**Current behavior correctly identified?** Yes. `docker/test.sh` line 41 is `${BIN} doctor 2>&1 || true`.

**Does the proposed change fix the problem?** Yes. The `doctor_rc=0; cmd || doctor_rc=$?` pattern is a standard POSIX idiom that is safe under `set -e`. The warning message `[WARN] great doctor exited non-zero (exit N)` matches the Nightingale acceptance criteria exactly.

**Edge cases?** The spec correctly identifies the `set -e` interaction risk and explains why the pattern is safe. The mitigation is accurate: the `||` clause prevents `set -e` from firing.

**Assessment: Sound.**

### Issue 3: No toolchain version printed at container startup

**Current behavior correctly identified?** Yes. None of the four scripts print toolchain information.

**Does the proposed change fix the problem?** Yes. A `rustc --version` line near the top of each script provides the traceability requested.

**Line number accuracy:** The spec's insertion point descriptions have minor inaccuracies -- see Concern 1 below. The descriptions reference `echo "[1/N]"` at line numbers that are actually comment lines. However, the spec's intent is unambiguous (insert after the banner's trailing blank line, before the first step), and the provided code blocks are correct. Da Vinci will have no trouble locating the insertion point.

**Edge case -- macOS PATH:** The spec correctly notes that `rustc` is available via `ENV PATH="/opt/rust/bin:${PATH}"` in the Dockerfile (confirmed at line 38 of `cross-macos.Dockerfile`), and places the version line after `source /etc/profile.d/osxcross-env.sh` as a belt-and-suspenders measure.

**Assessment: Sound, with ADVISORY note on line numbers.**

### Issue 4: Fragile mkdir in cross-compilation export scripts

**Current behavior correctly identified?** Yes. All three cross scripts write to `/workspace/test-files/`:
- `cross-test-macos.sh` line 66: `mkdir -p /workspace/test-files` -- confirmed
- `cross-test-windows.sh` line 48: `mkdir -p /workspace/test-files` -- confirmed
- `cross-test-linux-aarch64.sh` line 48: `mkdir -p /workspace/test-files` -- confirmed

The compose bind-mount overlay at lines 52, 64, 76 is confirmed as `./test-files:/workspace/test-files`.

**Does the proposed change fix the problem?** Yes. Moving the export target to `/build/test-files/` (inside the writable build directory) and updating the compose bind-mount from `./test-files:/workspace/test-files` to `./test-files:/build/test-files` ensures:
1. Scripts write to a genuinely writable location.
2. The host-side `./test-files` directory still receives exported binaries via the bind-mount.
3. Direct `docker run` without the bind-mount writes to the ephemeral `/build/test-files/` (which is the correct behavior -- no silent filesystem error).

**Scope contradiction resolved correctly.** The spec's note at the end of Issue 4 explicitly calls out that Nightingale's "out of scope" declaration contradicts the required path change. The spec resolves this by including the minimal compose mount path edit. Without it, exported binaries would be lost. This is the right call.

**Line number accuracy for linux-aarch64:** The spec says "Replace lines 47-52" but the comment `# Export binary to shared volume` is at line 46, not 47. The old code block in the spec does start with the comment, so the match text is correct even though the line number is off by one. See Concern 1.

**Assessment: Sound.**

---

## Concerns

### Concern 1

```
{
  "gap": "Several line number references are off by one",
  "question": "For Issue 3, the spec says test.sh line 14 is 'echo [1/5] Copying source...'
    but actual line 14 is '# Copy source (workspace is read-only mounted)' (a comment),
    and line 15 is the echo. Similarly for cross-test-macos.sh: spec says line 19 is
    'echo [1/4]...' but actual line 19 is a comment, with the echo at line 20. For Issue 4
    linux-aarch64, spec says 'lines 47-52' but the comment starts at line 46. Do these
    inaccuracies risk confusing the implementer?",
  "severity": "ADVISORY",
  "recommendation": "Da Vinci should use the surrounding context (code text, not line
    numbers) to locate insertion/replacement points. The spec's code blocks are correct;
    only the numeric references are off. No spec revision needed -- this note serves as
    guidance for the implementer."
}
```

### Concern 2

```
{
  "gap": "cross-macos.Dockerfile also has a misleading bare-cargo usage comment",
  "question": "The spec removes the bare 'cargo build' usage comment from the Windows
    Dockerfile (line 6) because it bypasses validation. However, cross-macos.Dockerfile
    line 6 has the identical pattern: 'docker compose run macos-cross cargo build --release
    --target x86_64-apple-darwin'. Should this also be cleaned up for consistency, or is
    it deliberately left for a separate task?",
  "severity": "ADVISORY",
  "recommendation": "The macOS CMD already invokes the script, so compose users are
    protected. But the misleading comment remains. This is outside the backlog scope for
    0020 (which only calls out the Windows Dockerfile). If desired, file a follow-up
    backlog item. Not blocking."
}
```

### Concern 3

```
{
  "gap": "cross-linux-aarch64.Dockerfile usage comment suggests direct docker run without validation",
  "question": "cross-linux-aarch64.Dockerfile lines 4-5 show 'docker run --rm -v $(pwd):/workspace
    great-cross-aarch64' as usage, which mounts the workspace as read-write (no :ro). This
    contradicts the spec's premise that /workspace is read-only, and bypasses compose
    entirely. Is this a latent issue that should be flagged?",
  "severity": "ADVISORY",
  "recommendation": "Similar to Concern 2 -- outside the 0020 backlog scope. The Linux
    aarch64 CMD already invokes the script, so the binary gets validated. The missing :ro
    in the usage example is a documentation issue. Consider a follow-up cleanup task."
}
```

### Concern 4

```
{
  "gap": "No mention of rustup show active-toolchain",
  "question": "The Nightingale selection (scope item 3) specifies 'Add a rustc --version
    && rustup show active-toolchain line' but the spec only adds 'rustc --version' and
    omits 'rustup show active-toolchain'. Was this intentionally dropped? Is there a risk
    that rustup is not installed in the macOS cross container (custom Rust install)?",
  "severity": "ADVISORY",
  "recommendation": "The macOS cross container installs Rust via rustup.rs directly
    (cross-macos.Dockerfile line 40), so rustup IS available. However, the simpler
    'rustc --version' is sufficient to confirm the active compiler version, which is the
    actual requirement from the backlog ('Cannot confirm pinned Rust version is active').
    The spec's choice to simplify is reasonable. If the full toolchain name (e.g.,
    'stable-x86_64-unknown-linux-gnu') is desired, Da Vinci can add it, but it is not
    strictly required."
}
```

---

## Gaps Found

1. **No rollback procedure.** The spec does not describe how to revert if the compose mount path change breaks an active workflow. Given the S complexity and the fact that `git revert` handles this trivially, this is not blocking.

2. **No CI verification step.** The spec's Section 8 (Verification Procedure) is purely manual (`grep` commands). There is no mention of running the actual containers to verify the changes work end-to-end. This is reasonable given the S complexity -- building all cross containers is expensive -- but should be noted.

3. **`test.sh` does not get the `/build/test-files` change.** This is correct since ubuntu/fedora services do not export binaries, but the spec could explicitly state why `test.sh` is excluded from Issue 4. It is implicit but not explicit. ADVISORY.

---

## Contradictions

1. **Nightingale scope vs. compose change:** The spec correctly identifies and resolves this contradiction (Nightingale says "no compose changes" but requires `/build/test-files` output path). The resolution is sound -- the compose mount path change is the minimal necessary companion fix.

2. **No internal contradictions found in the spec itself.** The acceptance criteria (Section 4) are consistent with the changes described in Section 2. The files-modified table (Section 5) matches the changes. The fix order (Section 3) is logically sound.

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation in Spec |
|------|-----------|--------|-------------------|
| Compose mount path breaks existing workflows | Low | Medium | Yes -- Section 7.1 explains host-side path is unchanged |
| `set -e` interaction with doctor warning | Very Low | High | Yes -- Section 7.2 explains POSIX compliance |
| `rustc` not on PATH in macOS container | Very Low | Medium | Yes -- Section 7.3 confirms ENV in Dockerfile |
| WORKDIR/working_dir redundancy causes confusion | Very Low | Low | Yes -- Section 7.4 explains precedence |
| Off-by-one line numbers cause wrong edit location | Low | Low | Code context in spec is unambiguous |

**Overall risk: LOW.** All four changes are isolated, well-understood shell edits. The highest-risk change (compose mount path) is correctly identified and mitigated. The spec's risk section is comprehensive and honest.

---

## Summary

The spec is well-crafted, thoroughly evidenced against actual file contents, and directly implementable. All four backlog issues are addressed with minimal, correct changes. The Nightingale scope contradiction is transparently resolved. Four ADVISORY concerns are noted (off-by-one line numbers, two similar-but-out-of-scope comment cleanup opportunities, and the dropped `rustup show` from Nightingale's scope). None are blocking. Approved for implementation.

---

*"The unexamined spec leads to unexamined containers. This one, however, has been examined."*
