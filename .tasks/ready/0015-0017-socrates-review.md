# Socrates Review: Spec 0015+0017 (Fix Cross-Compilation Dockerfiles)

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-24
**Spec:** `/home/isaac/src/sh.great/.tasks/ready/0015-0017-cross-builds-spec.md`
**Tasks:** 0015 (macOS Dockerfile rewrite) + 0017 (Windows Dockerfile Rust bump)

---

## VERDICT: APPROVED

---

## Concerns

### 1. Missing `/osxsdk` directory in COPY

```
{
  "gap": "Task 0015 states the upstream osxcross image contains files at BOTH
         '/osxcross' AND '/osxsdk'. The spec only COPYs '/osxcross'. If the
         SDK files live separately at '/osxsdk', the cross-compiler may fail
         to find SDK headers/libraries at link time.",
  "question": "Does the upstream crazymax/osxcross:26.1-r0-ubuntu image place
               all needed SDK files under /osxcross, or does it also require
               /osxsdk? If /osxsdk is a separate directory, should the spec
               add 'COPY --from=osxcross /osxsdk /osxsdk'?",
  "severity": "ADVISORY",
  "recommendation": "The builder should inspect the upstream image layout with
                     'docker run --rm --entrypoint='' crazymax/osxcross:26.1-r0-ubuntu
                     ls /' (which will fail since it is FROM scratch -- use
                     'docker create' + 'docker export' instead) to confirm
                     whether /osxsdk exists as a separate tree. If it does,
                     add a second COPY line. The upstream README example only
                     mounts /osxcross, suggesting the SDK may be nested within
                     it, but this should be verified empirically."
}
```

### 2. osxcross tag `26.1-r0-ubuntu` existence unverified

```
{
  "gap": "The spec pins to 'crazymax/osxcross:26.1-r0-ubuntu' and states this
         is the latest upstream release (2026-01-30). However, upstream tag
         naming conventions are not guaranteed stable. The tag may actually be
         named differently (e.g., '26.1-ubuntu', '26.1-r0', etc.).",
  "question": "Has anyone verified that the exact tag
               'crazymax/osxcross:26.1-r0-ubuntu' exists on Docker Hub? If
               the tag does not exist, the build will fail at the first FROM
               line with no useful diagnostic beyond 'manifest unknown'.",
  "severity": "ADVISORY",
  "recommendation": "The builder should verify the tag exists before applying
                     the change: 'docker manifest inspect
                     crazymax/osxcross:26.1-r0-ubuntu' or check
                     hub.docker.com/r/crazymax/osxcross/tags. The spec
                     already notes this in the edge cases table, which is
                     good, but making it a mandatory pre-flight check would
                     be stronger."
}
```

### 3. `COPY --from` vs `RUN --mount=type=bind,from=` tradeoff well-reasoned but image size implication unstated

```
{
  "gap": "The spec correctly chooses COPY --from over RUN --mount because the
         osxcross binaries are needed at container runtime (the test script
         invokes the cross-compiler). However, the spec does not mention the
         image size impact. The osxcross toolchain + SDK can be 1-2 GB. This
         is not a blocker but an awareness gap.",
  "question": "Is the resulting image size acceptable for local development
               use? Is this image ever pushed to a registry where size matters?",
  "severity": "ADVISORY",
  "recommendation": "Add a brief note that the macOS cross image will be large
                     (~2-4 GB) due to the embedded osxcross toolchain, and that
                     this is expected for a local-only build tool. If the image
                     is ever pushed to a registry, a multi-stage slim variant
                     should be considered."
}
```

### 4. `LD_LIBRARY_PATH` is new and its necessity is not explained

```
{
  "gap": "The spec adds 'ENV LD_LIBRARY_PATH=/osxcross/lib:${LD_LIBRARY_PATH}'
         which does not exist in the current Dockerfile. The current file worked
         (or would have worked if the base image had a shell) without it. The
         spec lists this as 'upstream-recommended' but does not explain what
         fails without it.",
  "question": "What specific osxcross library under /osxcross/lib requires
               LD_LIBRARY_PATH to be set? Will the auto-detect RUN block or
               the cross-test-macos.sh script fail without it? Or is this
               purely defensive?",
  "severity": "ADVISORY",
  "recommendation": "This is likely correct (upstream documents it), but the
                     builder should verify by attempting a build without the
                     LD_LIBRARY_PATH line. If it fails, the error message will
                     confirm the need. If it succeeds, the line is defensive
                     but harmless."
}
```

### 5. `PATH` for osxcross binaries is new -- verify interaction with auto-detect

```
{
  "gap": "The spec adds 'ENV PATH=/osxcross/bin:${PATH}' which was not in the
         original Dockerfile. The original file relied on full paths found by
         the 'find /osxcross/bin' auto-detect block. Adding PATH means the
         osxcross binaries are now available by short name (e.g., 'o64-clang')
         throughout the container, not just through the auto-detected full
         paths written to cargo config.",
  "question": "Does adding osxcross to PATH create any name collisions with
               the system clang/lld installed by apt-get? The Dockerfile
               installs both 'clang' and 'lld' via apt-get AND puts
               /osxcross/bin on PATH. Could the osxcross 'clang' shadow the
               system 'clang' or vice versa?",
  "severity": "ADVISORY",
  "recommendation": "Verify that /osxcross/bin does not contain a bare 'clang'
                     or 'lld' binary that would shadow the system ones. The
                     osxcross convention is to prefix all binaries with the
                     target triple (e.g., x86_64-apple-darwin26.1-clang), so
                     collisions are unlikely, but this should be confirmed."
}
```

### 6. No `--platform` flag on the osxcross FROM line

```
{
  "gap": "The upstream README example uses
         'FROM --platform=$BUILDPLATFORM crazymax/osxcross:...' but the spec
         omits the --platform flag. The spec explains this in the edge cases
         table ('the osxcross stage is architecture-independent'), but the
         upstream example explicitly uses it.",
  "question": "If the builder runs on an ARM64 host (e.g., Apple Silicon with
               Docker Desktop), will Docker pull the correct osxcross image
               variant without --platform? Or does the upstream image only
               publish linux/amd64 manifests, making the flag unnecessary?",
  "severity": "ADVISORY",
  "recommendation": "Check whether crazymax/osxcross publishes multi-arch
                     manifests. If it does, omitting --platform could pull the
                     wrong arch on ARM64 hosts. If it only publishes amd64,
                     the omission is fine. Either way, document the reasoning."
}
```

### 7. Windows Dockerfile `WORKDIR /workspace` but test script assumes `/build` exists

```
{
  "gap": "The Windows Dockerfile sets 'WORKDIR /workspace' and has no /build
         directory. The docker-compose overrides working_dir to /build, which
         Docker creates automatically. The cross-test-windows.sh script then
         copies files to /build/src, /build/tests, etc. This works because
         docker-compose creates /build via working_dir. However, if someone
         runs the image directly with 'docker run' (as shown in the Dockerfile
         usage comment), the CMD runs 'cargo build' in /workspace, which is
         fine -- but the test script path is never invoked.",
  "question": "Is it intentional that the Dockerfile CMD and docker-compose
               command follow completely different code paths? The Dockerfile
               CMD does a direct 'cargo build' in /workspace, while
               docker-compose runs cross-test-windows.sh which copies to
               /build first. This is a pre-existing concern, not introduced
               by this spec, but worth noting.",
  "severity": "ADVISORY",
  "recommendation": "No change needed for this spec. This is pre-existing
                     behavior. But consider aligning the Dockerfile CMD with
                     the docker-compose command in a future cleanup task."
}
```

### 8. `rust:1.85-slim` availability assumption

```
{
  "gap": "The spec states 'rust:1.85-slim' with certainty but does not verify
         the tag exists on Docker Hub. The Rust Docker images typically follow
         the pattern rust:X.Y-slim, and 1.85 was released 2025-02-20, so the
         tag should exist. But assumptions are not evidence.",
  "question": "Has 'docker pull rust:1.85-slim' been verified to succeed? If
               the Rust Docker team uses a different naming convention for this
               release, the Windows build will fail at step 1.",
  "severity": "ADVISORY",
  "recommendation": "The builder should verify with 'docker manifest inspect
                     rust:1.85-slim' before applying the change. This is a
                     low-risk concern since the naming convention is well
                     established."
}
```

---

## Lines of Questioning -- Summary of Findings

**Completeness:** The spec addresses all requirements from both task 0015 and task 0017. Both acceptance criteria lists are fully covered. The orphan container warning is correctly scoped out.

**Correctness:** The multi-stage build pattern is the correct approach for the osxcross FROM-scratch issue. The `COPY --from` choice over `RUN --mount` is well-justified (runtime access needed). The Rust version bump to 1.85 is the minimum correct fix for `edition2024`.

**Consistency:** Both Dockerfiles pin to specific versions (no floating tags). The macOS file pins Rust to `1.85.0` via rustup while Windows uses `rust:1.85-slim` -- different mechanisms but both achieve the same pinned result. Acceptable.

**Reproducibility:** All image tags are pinned (`rust:1.85-slim`, `crazymax/osxcross:26.1-r0-ubuntu`, `ubuntu:24.04`). No `latest` or `stable` floating tags. The spec correctly notes SHA256 digest pinning is out of scope.

**CI Impact:** Grep confirms no GitHub Actions workflows reference these Dockerfiles. No CI breakage risk.

**Security:** Appropriate for the scope. No secrets, HTTPS-only rustup, read-only workspace mounts, pinned tags.

**Edge Cases:** The edge cases table is thorough (14 scenarios). The `/osxsdk` directory question (concern #1) is the most significant gap but is unlikely to be a real issue since the upstream README example only references `/osxcross`.

---

## Summary

This is a well-structured, narrowly scoped infrastructure bugfix spec with thorough edge case analysis. All 8 concerns are ADVISORY -- the spec is implementable as-written with high confidence. The builder should verify the two Docker image tags exist before starting (concerns #2 and #8) and inspect whether `/osxsdk` is a separate directory in the upstream image (concern #1). No BLOCKING issues found.
