# Nightingale Selection: Task 0008

**Selected Task:** 0008 — Runtime Version Manager Integration (mise)
**Priority:** P1
**Type:** feature
**Selected:** 2026-02-25

---

## Selection Rationale

### Why 0008 over all other candidates

The backlog contains three classes of unblocked work:

**Class 1 — Unblocks the P0 core command (0009):**
- 0008 (P1) — Runtime manager. The only remaining dependency blocking `great apply`.

**Class 2 — Standalone P1 bugfix with no downstream unblocking:**
- 0021 (loop/ dir fix) — Three one-line shell additions. Correct and necessary, but
  unblocks no further work. Its impact ceiling is: cross-compilation tests stop
  failing. It can be done in 10 minutes at any time without coordination.

**Class 3 — P0 sub-items inside an umbrella task:**
- 0010 GROUP J (integration tests) — P0 CI hygiene. No deps. But tests cannot
  exercise `great apply` until apply actually works. Completing this before 0008
  and 0009 means writing placeholder tests against a stub command.
- 0010 GROUP A (tool install mapping) — P0. No deps. But GROUP A's work lives inside
  `apply.rs` and is most efficiently done when apply itself is being built. It is
  also listed as a sub-item of 0010 (an umbrella), not a standalone task, so
  starting it as a fragment of a larger task is not the right framing.

**Decision:** 0008 is the sole task whose completion directly unblocks the next P0
task (0009). The path from current state to a working `great apply` runs through
0008. Every iteration spent on 0010 groups or 0021 before 0008 is complete delays
the product's core value delivery by one full iteration.

The 0021 shell fix should be carried as a same-iteration quickfix by Da Vinci
alongside the 0008 implementation — it is too trivial to warrant its own loop
iteration and should not be elevated to selection rank.

---

## Task Summary

**Module:** `src/platform/runtime.rs` (new file)

**What it delivers:**
A `MiseManager` struct that integrates the `mise` runtime version manager into
the great.sh CLI. This provides the runtime provisioning backbone that `great apply`
calls to install declared language runtimes (Node.js, Python, Go, Rust, etc.).

**Five requirements from the backlog task:**

1. **Detect mise** — `MiseManager::is_available() -> bool` via `command_exists("mise")`.
   `version() -> Option<String>` via `mise --version`. All other methods return clear
   errors if mise is absent.

2. **Install mise if absent** — `ensure_installed(pkg_manager: &dyn PackageManager)`
   uses Homebrew on macOS, curl installer on Linux/WSL2. Verifies post-install.

3. **Install a runtime** — `install_runtime(name, version)` runs
   `mise install <name>@<version>` then `mise use --global <name>@<version>`.
   Supports: node, python, go, rust, java, ruby.

4. **Check installed versions** — `installed_version(name) -> Option<String>` parses
   `mise current <name>`. Returns `None` if not installed. Used by `great diff` and
   `great status`.

5. **Batch provision from config** — `provision_from_config(tools: &ToolsConfig)`
   iterates `tools.runtimes`, skips the `"cli"` key, checks each against installed
   version, installs or updates as needed. Returns `Vec<ProvisionResult>` with
   structured feedback for `great apply` to display.

**Acceptance criteria (verbatim from backlog):**
- [ ] `cargo build` succeeds and `cargo clippy` produces zero warnings for `src/platform/runtime.rs`
- [ ] `MiseManager::is_available()` correctly returns `true` with mise installed, `false` otherwise
- [ ] `MiseManager::installed_version("node")` returns `Some(version_string)` when Node.js is active via mise, `None` otherwise
- [ ] `provision_from_config()` skips the `"cli"` key and processes only runtime entries
- [ ] No `.unwrap()` or `.expect()` calls exist in `src/platform/runtime.rs`

---

## Dependencies (all confirmed in done/)

| Task | Description | Status |
|------|-------------|--------|
| 0001 | Platform detection — `command_exists()` available | done |
| 0002 | Config schema — `ToolsConfig` with `runtimes: HashMap<String, String>` available | done |
| 0007 | Package manager — `PackageManager` trait and implementations available | done |

---

## Risks and Concerns

**Risk 1 — mise availability in CI**
The acceptance criteria require testing `is_available()` on a machine with and
without mise. The CI environment (ubuntu-latest) does not have mise pre-installed.
Mitigation: unit tests should use a mock or inject a `command_exists` function pointer;
the criterion can be validated in a dev environment and documented as such.

**Risk 2 — Rust runtime via mise vs rustup conflict**
The notes in the backlog flag that `mise use rust@stable` wraps rustup internally.
If the developer machine already has rustup managing Rust, `mise install rust@stable`
may conflict or produce confusing output. Lovelace should spec whether Rust is
treated as a special case (delegate to rustup check) or passed through to mise
unconditionally.

**Risk 3 — Linux curl install path**
The `curl https://mise.jdx.dev/install.sh | sh` path requires internet access and
trust in an external script. In air-gapped or locked-down environments this fails
silently. The implementation should check for the mise binary again after the curl
command and emit a clear error if it still absent, rather than assuming success.

**Risk 4 — Module registration**
The backlog explicitly notes the module must be re-exported from `src/platform/mod.rs`
as `pub mod runtime;`. This is easy to miss and causes a compile error in apply.rs
when it tries to import `MiseManager`. Lovelace should include this in the spec's
implementation checklist.

---

## Carry-along recommendation

The 0021 loop/ copy fix (three one-line changes across three shell scripts) should
be implemented in the same iteration as a zero-cost addition. It does not require
its own spec or scout phase. Da Vinci can apply it from the backlog task directly.
