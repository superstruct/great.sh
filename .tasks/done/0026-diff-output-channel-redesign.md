# 0026: `great diff` Output Channel Redesign

**Priority:** P2
**Type:** enhancement
**Module:** `src/cli/diff.rs`, `src/cli/output.rs`
**Status:** selected — iteration 026
**Estimated Complexity:** S
**Source:** Rams visual review, iteration 013 (originally filed as 0021)

## Context

`src/cli/output.rs` routes all helpers — `success`, `warning`, `error`, `info`,
`header` — to `eprintln!` (stderr). `src/cli/diff.rs` calls these helpers for
section headers and summary lines, while printing diff lines themselves via
`println!` (stdout).

The result: `great diff 2>/dev/null` produces orphaned diff lines with no section
context (headers are swallowed), and `great diff 1>/dev/null` produces headers
with no content (diff lines are swallowed). In CI pipelines where stderr is
discarded or redirected separately, the output is incoherent and unparseable.

`diff` is a read-only, pipeline-oriented command — its entire output is data,
not interactive status. It should behave like standard Unix diff tools and write
everything to stdout. Fatal errors only (missing config, parse failure) belong
on stderr.

`status`, `doctor`, `apply`, and other interactive commands deliberately keep
their output on stderr so that piped usage is opt-in. This task does not change
those commands.

## Requirements

1. Add stdout variants of the output helpers used in `diff.rs`: at minimum
   `output::header_stdout`, `output::info_stdout`, and `output::success_stdout`.
   These are identical to their stderr counterparts except they use `println!`
   instead of `eprintln!`. Do not remove or change the existing stderr helpers.

2. Update `src/cli/diff.rs` to use the new stdout variants everywhere: section
   headers ("Tools", "MCP Servers", "Secrets"), info lines, success messages,
   and the final summary line. The `output::error` call for missing config
   (before `std::process::exit(1)`) must remain on stderr.

3. The diff lines themselves (`println!("{}", diff)`) are already on stdout and
   need no change.

4. Update or add integration tests that assert on `great diff` output to expect
   the headers and summary on stdout rather than stderr.

## Acceptance Criteria

- [ ] `great diff 2>/dev/null` with a valid `great.toml` produces complete,
      coherent output on stdout — section headers, diff lines, and summary
      are all present.
- [ ] `great diff 1>/dev/null` with a valid `great.toml` produces no output
      on stdout; nothing appears on stderr (stderr is reserved for fatal errors).
- [ ] `great diff` run against a missing config file still prints the error
      message on stderr and exits with code 1.
- [ ] `cargo clippy` passes with zero new warnings after the change.
- [ ] Integration tests that capture `great diff` output pass with the updated
      channel routing (assert on stdout, not stderr, for headers and summary).

## Dependencies

None. `src/cli/diff.rs` and `src/cli/output.rs` are self-contained.

## Notes

- Keep the implementation additive: new `*_stdout` functions alongside the
  existing helpers, not a refactor that touches all callers. This limits blast
  radius to diff.rs only.
- Do not change `status`, `doctor`, `apply`, or any other command in this task.
  Those commands are interactive and their stderr-first design is intentional.
- A future global redesign (e.g., a `Writer` trait or `--output-channel` flag)
  can be a separate P3 task if it is ever warranted.
- This task was originally numbered 0021 in the backlog, but 0021 was already
  used by a completed fix (`loop/ dir missing from cross-build context`).
  Renumbered to 0026 to resolve the collision.
