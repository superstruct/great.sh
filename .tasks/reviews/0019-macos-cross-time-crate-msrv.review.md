# Review: 0019 -- Bump Rust Toolchain in Cross-Compilation Dockerfiles

**Reviewer:** Socrates
**Date:** 2026-02-25
**Spec:** `.tasks/ready/0019-macos-cross-time-crate-msrv.spec.md`
**Round:** 1 of 3

---

## VERDICT: APPROVED

---

## Concerns

### 1. Task 0018 Supersession Not Documented

```
{
  "gap": "Spec 0019 subsumes the entirety of backlog task 0018 (bump aarch64 from 1.83
          to 1.85), but goes further (1.83 -> 1.88). Neither the spec nor the backlog
          task 0019 mentions closing, superseding, or cancelling task 0018.",
  "question": "After 0019 lands, what happens to task 0018? Will someone attempt it
               independently, creating a conflict or a downgrade?",
  "severity": "ADVISORY",
  "recommendation": "Add a note to the spec's Out of Scope section stating that task
                     0018 is superseded by this change and should be closed upon
                     completion of 0019. The builder should mark 0018 as done/superseded
                     after landing the change."
}
```

### 2. Backlog Scope Mismatch (Spec Exceeds Backlog)

```
{
  "gap": "Backlog task 0019 explicitly scopes to macOS and Windows Dockerfiles. The spec
          adds the Linux aarch64 Dockerfile. The backlog acceptance criteria say 'Both
          Dockerfiles' (2 files), but the spec changes 3 files.",
  "question": "Is expanding scope beyond the backlog task acceptable without updating
               the backlog task first?",
  "severity": "ADVISORY",
  "recommendation": "This is the correct engineering decision -- leaving aarch64 at 1.83
                     when the MSRV is 1.88 would just create another P0. No action
                     required beyond noting the scope expansion is intentional. The spec
                     already provides rationale in section 3."
}
```

## Verified Claims

All factual claims in the spec were independently verified against the codebase:

| Claim | Verified | Evidence |
|-------|----------|----------|
| `time 0.3.47` at Cargo.lock line 1720 | YES | Confirmed exact line |
| `time-core 0.1.8` at Cargo.lock line 1733 | YES | Confirmed exact line |
| `zip 2.4.2` depends on `time` | YES | Cargo.lock line 2577 |
| `zip = "2"` in Cargo.toml | YES | Cargo.toml line 19 |
| macOS Dockerfile line 42: `--default-toolchain 1.85.0` | YES | Exact match |
| Windows Dockerfile line 7: `FROM rust:1.85-slim` | YES | Exact match |
| aarch64 Dockerfile line 6: `FROM rust:1.83-slim` | YES | Exact match |
| Zero `unsafe` blocks in `src/` | YES | Grep returned zero matches |
| Zero `#[bench]` attributes | YES | Grep returned zero matches |
| CI uses `dtolnay/rust-toolchain@stable` (not pinned) | YES | All 6 references in release.yml and ci.yml |
| Ubuntu/Fedora Dockerfiles unaffected | YES | No `--default-toolchain` pin in either |
| No `rust-toolchain.toml` in repo | YES | Glob returned no results |
| aarch64 Dockerfile not in docker-compose.yml | YES | Only mentioned in a comment about macOS targets |

## Assessment of Elenchus Questions

**Why 1.88.0 and not latest stable (1.93.1)?** The spec provides explicit rationale: minimum required version for reproducibility and conservatism. Eight releases behind stable is well-tested. This is defensible for a P0 hotfix. A follow-on task to bump to a newer version is acknowledged as future work.

**Could bumping Rust introduce clippy failures?** No. CI already runs clippy on stable (1.93.1), which is newer than 1.88.0. The Docker containers only run `cargo build --release`, not clippy. Any lint issues between 1.85 and 1.88 would already manifest in CI. The spec's risk analysis of specific 1.88.0 lints is thorough and correct.

**osxcross compatibility with Rust 1.88.0?** osxcross provides the C toolchain (linker, ar, SDK headers). It is Rust-version-agnostic. The Dockerfile's `find`-based tool detection and cargo config generation do not depend on Rust version. No compatibility issue.

**Is the spec implementable without further questions?** Yes. The changes are three simple version string substitutions with exact file paths and line numbers, all independently verified. Verification commands are provided.

## Summary

A thorough, well-researched spec for a straightforward P0 fix. All factual claims verified against the codebase. Two ADVISORY concerns about task management (0018 supersession and scope expansion), neither of which block implementation. The dependency chain is confirmed, the line numbers are exact, the risk analysis is sound, and the verification steps are concrete and actionable.
