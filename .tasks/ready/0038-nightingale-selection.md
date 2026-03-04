# Nightingale Selection Report — Task 0038
**Prepared by:** Florence Nightingale (Requirements Curator)
**Date:** 2026-03-04
**Iteration:** 035

---

## 1. Task Selected and Why

**Selected:** 0038 — CLI: Handle SIGPIPE / EPIPE Gracefully Instead of Panicking

**Rationale for selection over the remaining backlog:**

| Task | Priority | Complexity | Blocked? | Decision |
|------|----------|-----------|----------|----------|
| 0038 | P2 | S | No | **SELECTED** |
| 0039 | P2 | S | No | Next (same priority, slightly narrower impact) |
| 0040 | P3 | M | Yes — design decision pending | Deferred |
| 0041 | P3 | XS | No | Deferred (cosmetic only) |

0038 and 0039 share the same P2 priority. 0038 is selected first because its fix
is global (one attribute in `src/main.rs` covers every subcommand) while 0039
targets a single detection module. Fixing the broadest-impact issue first is the
correct triage order.

0040 is blocked: three exit-code design options have been tabled but no option
has been chosen. Sending it to Lovelace now would produce a spec with an
unresolved fork. It stays in backlog until the team makes a decision.

0041 is a cosmetic error-message improvement. Correct but not urgent.

---

## 2. Summary of Requirements

**Problem statement:** Any `great` subcommand that writes to stdout panics with
exit code 101 when the read end of a pipe closes before the write completes.
This is the standard POSIX SIGPIPE / EPIPE condition. Rust's stdlib does not
install a default SIGPIPE handler; instead, `println!` / `write!` propagate an
`io::Error` which anyhow surfaces as a backtrace and exit 101.

**Reproduction:**
```bash
great status --json | head -0   # exits 101, stderr: "broken pipe"
great status --json | head -1   # same
great doctor | head -0          # same
```

**Expected outcome:** exit 0 or 141 (SIGPIPE-terminated), stderr clean.

**Scope of impact:** Every subcommand that calls `println!` or `print!` is
affected. This includes `status`, `doctor`, `diff`, `template list`, and any
future output-producing command. The problem is at the binary entry point, not
in individual subcommands.

**Root cause in code:** `src/main.rs` line 13 — `fn main() -> Result<()>` —
receives the propagated `io::Error(BrokenPipe)` from any subcommand and hands
it to anyhow's error handler, which prints to stderr and exits 101. No SIGPIPE
disposition is set at process startup.

**Why it matters in practice:**
- CI pipelines using `great status --json | jq -e '.has_issues'` can panic if
  jq exits after finding its result.
- `great status --json | head -1` is documented to exit 0 in JSON mode; exiting
  101 on EPIPE is an undocumented and inconsistent exception.
- The stderr panic message causes user confusion and spurious bug reports.

**Acceptance criteria (5, verbatim from task file):**
1. `great status --json | head -0` exits 0 or 141 with stderr empty.
2. `great status --json | head -1` exits 0 or 141 with stderr empty.
3. At least two additional subcommands (`great doctor`, `great diff`, or
   `great template list`) do not panic on `| head -0`, confirming the fix is at
   the binary entry point.
4. Normal operation unaffected: `great status --json` with stdout open exits 0
   and produces valid JSON; `great status` with issues still exits 1.
5. `cargo test` passes with no regressions.

---

## 3. Recommended Approach — Option A

**Option A: `#[unix_sigpipe = "sig_dfl"]` attribute on `fn main()`**

This is the correct and preferred fix. Reasons:

- **One line, zero unsafe code.** The attribute tells the Rust runtime to
  restore the default POSIX SIGPIPE disposition before `main()` runs. The kernel
  then terminates the process cleanly (SIGPIPE / exit 141) rather than
  delivering the error as an `io::Error`.
- **Stable since Rust 1.73** (released 2023-09-19). The project uses edition
  2021 and has no pinned MSRV that would exclude 1.73+. The `libc` crate is
  already a transitive dependency (added for the MCP bridge in task 0029), so
  there is no new dependency introduced even if the compiler internally uses it.
- **Covers all subcommands automatically.** The fix is at the process entry
  point; every `println!` in every subcommand benefits without per-call-site
  changes.
- **No behavior change on Windows.** The attribute is documented to be a no-op
  on non-Unix targets, so cross-compilation and Windows builds are unaffected.
- **Exit code 141 is acceptable to downstream tooling.** Shells report SIGPIPE-
  terminated processes as exit 141. CI systems that check for non-zero exit
  codes should use `|| true` or check for 141 specifically; this is the
  universal convention for piped Unix tools.

**Exact change:**

File: `/home/isaac/src/sh.great/src/main.rs`

Before (line 13):
```rust
fn main() -> Result<()> {
```

After:
```rust
#[cfg_attr(unix, unix_sigpipe = "sig_dfl")]
fn main() -> Result<()> {
```

Using `cfg_attr(unix, ...)` is the idiomatic guard that makes the attribute
conditional on Unix targets while keeping the file compiling cleanly on Windows
without any warning about an unknown attribute. This is the pattern recommended
in the Rust tracking issue for `unix_sigpipe`.

**Options B and C are not recommended:**

- Option B requires `unsafe { libc::signal(...) }` in `main()` and a custom
  error handler that inspects `io::ErrorKind::BrokenPipe`. More code, more risk,
  same outcome.
- Option C requires touching every `println!` / `print!` call site across
  multiple files. It is the most invasive option and provides no benefit over A.

---

## 4. Files Requiring Change

| File | Change |
|------|--------|
| `src/main.rs` | Add `#[cfg_attr(unix, unix_sigpipe = "sig_dfl")]` to `fn main()` |

No other files require modification. The fix is self-contained.

**Verification that no `#[unix_sigpipe]` attribute already exists:**
`src/main.rs` was read in full (45 lines). The file contains only module
declarations, `use` imports, and the `fn main()` match dispatch. No SIGPIPE
handling of any kind is present.

---

## 5. Confirmation — Ready for Lovelace

Task 0038 is **ready for specification**. All information Lovelace requires is
present:

- Root cause: confirmed in source (main.rs line 13, no SIGPIPE disposition set)
- Fix location: `src/main.rs`, one attribute addition
- Recommended option: A (`#[cfg_attr(unix, unix_sigpipe = "sig_dfl")]`)
- Acceptance criteria: 5 testable criteria, no ambiguity
- Dependencies: none
- Out-of-scope: MCP bridge SIGPIPE (different semantics; long-lived process),
  Windows behavior (no-op by design), exit code policy for non-EPIPE paths

**No open questions.** Da Vinci can implement this without further clarification.
The spec Lovelace writes should be correspondingly short.

---

*Nightingale note: This is an XS-to-S implementation task with a single-file
change. If Lovelace's spec runs longer than one page, it is over-specified.
The acceptance criteria are the specification.*
