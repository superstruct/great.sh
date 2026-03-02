# 0027: Wire `--non-interactive` Flag Through CLI -- Technical Specification

**Iteration:** 024
**Author:** Lovelace (Spec Writer)
**Date:** 2026-02-27
**Complexity:** S (4 files, ~30 lines changed)

---

## 1. Overview

The `--non-interactive` global flag is declared on `Cli` in `src/cli/mod.rs:35` and parsed
correctly by clap. However, `src/main.rs` dispatches subcommands by consuming `cli.command`
while silently discarding `cli.non_interactive` -- the value is never forwarded to any
subcommand handler. This means a user or CI job running `great apply --non-interactive`
or `great doctor --fix --non-interactive` will still receive a `sudo -v` password prompt on
platforms that require sudo (Ubuntu, Debian, WSL2 Homebrew installs, apt operations).

This specification describes the exact changes needed to wire the parsed flag through to the
two subcommands that perform interactive operations: `apply` and `doctor`.

---

## 2. Current State

### 2a. Flag declaration (no change needed)

**File:** `src/cli/mod.rs`, line 34-35

```rust
/// Disable interactive prompts (for CI/automation)
#[arg(long, global = true)]
pub non_interactive: bool,
```

This is correct and requires no modification.

### 2b. Dispatch in main.rs (flag discarded)

**File:** `src/main.rs`, lines 14-29

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Apply(args) => cli::apply::run(args),
        Command::Doctor(args) => cli::doctor::run(args),
        // ... other arms ...
    }
}
```

`cli.non_interactive` is parsed into the `Cli` struct but never read. After `cli.command` is
consumed by the `match`, the `non_interactive` field is dropped.

### 2c. apply::run() -- no non_interactive parameter

**File:** `src/cli/apply.rs`, line 375

```rust
pub fn run(args: Args) -> Result<()> {
```

The `Args` struct (lines 354-367) has no `non_interactive` field. The function calls
`ensure_sudo_cached(info.is_root)` at line 421 with only one argument, and calls
`package_manager::available_managers(false)` at lines 572, 730, and 803 with a hardcoded
`false`.

### 2d. doctor::run() -- no non_interactive parameter

**File:** `src/cli/doctor.rs`, line 52

```rust
pub fn run(args: Args) -> Result<()> {
```

The `Args` struct (lines 10-15) has no `non_interactive` field. The function calls
`ensure_sudo_cached(info.is_root)` at line 112 with only one argument, and calls
`package_manager::available_managers(false)` at line 97 with a hardcoded `false`.

### 2e. ensure_sudo_cached() -- TODO comment

**File:** `src/cli/sudo.rs`, lines 61-63

```rust
// TODO: Also accept a `non_interactive` parameter from the --non-interactive
// global CLI flag once it is wired through to apply::run() / doctor::run().
pub fn ensure_sudo_cached(is_root: bool) -> SudoCacheResult {
```

The function only checks `std::io::stdin().is_terminal()` (line 70) to detect
non-interactive sessions. The explicit `--non-interactive` flag is not considered.

---

## 3. Change-by-Change

### Change 1: `src/cli/apply.rs` -- Add `non_interactive` field to `Args`

**Current code** (lines 354-367):

```rust
#[derive(ClapArgs)]
pub struct Args {
    /// Path to configuration file
    #[arg(long)]
    pub config: Option<String>,

    /// Preview changes without applying
    #[arg(long)]
    pub dry_run: bool,

    /// Skip confirmation prompts
    #[arg(long, short)]
    pub yes: bool,
}
```

**Desired code:**

```rust
#[derive(ClapArgs)]
pub struct Args {
    /// Path to configuration file
    #[arg(long)]
    pub config: Option<String>,

    /// Preview changes without applying
    #[arg(long)]
    pub dry_run: bool,

    /// Skip confirmation prompts
    #[arg(long, short)]
    pub yes: bool,

    /// Set by main.rs from the global --non-interactive flag.
    /// Not a CLI argument -- hidden from clap.
    #[arg(skip)]
    pub non_interactive: bool,
}
```

**Rationale:** The `#[arg(skip)]` attribute tells clap to exclude this field from CLI parsing.
It will default to `false` from clap's perspective, and `main.rs` will set it to the correct
value after parsing but before calling `run()`. This avoids creating a duplicate
`--non-interactive` flag on the subcommand (which would conflict with the global flag on `Cli`).

### Change 2: `src/cli/apply.rs` -- Pass `non_interactive` to `ensure_sudo_cached`

**Current code** (line 421):

```rust
match ensure_sudo_cached(info.is_root) {
```

**Desired code:**

```rust
match ensure_sudo_cached(info.is_root, args.non_interactive) {
```

### Change 3: `src/cli/apply.rs` -- Pass `non_interactive` to `available_managers` (3 call sites)

There are three calls to `package_manager::available_managers(false)` in apply.rs. All three
must be updated.

**Call site 1** (line 572 -- CLI tools section):

```rust
// Current:
let managers = package_manager::available_managers(false);
// Desired:
let managers = package_manager::available_managers(args.non_interactive);
```

**Call site 2** (line 730 -- bitwarden-cli install):

```rust
// Current:
let managers = package_manager::available_managers(false);
// Desired:
let managers = package_manager::available_managers(args.non_interactive);
```

**Call site 3** (line 803 -- platform-specific tools):

```rust
// Current:
let managers = package_manager::available_managers(false);
// Desired:
let managers = package_manager::available_managers(args.non_interactive);
```

### Change 4: `src/cli/doctor.rs` -- Add `non_interactive` field to `Args`

**Current code** (lines 10-15):

```rust
#[derive(ClapArgs)]
pub struct Args {
    /// Attempt to fix issues automatically
    #[arg(long)]
    pub fix: bool,
}
```

**Desired code:**

```rust
#[derive(ClapArgs)]
pub struct Args {
    /// Attempt to fix issues automatically
    #[arg(long)]
    pub fix: bool,

    /// Set by main.rs from the global --non-interactive flag.
    /// Not a CLI argument -- hidden from clap.
    #[arg(skip)]
    pub non_interactive: bool,
}
```

### Change 5: `src/cli/doctor.rs` -- Pass `non_interactive` to `available_managers`

**Current code** (line 97):

```rust
let managers = package_manager::available_managers(false);
```

**Desired code:**

```rust
let managers = package_manager::available_managers(args.non_interactive);
```

### Change 6: `src/cli/doctor.rs` -- Pass `non_interactive` to `ensure_sudo_cached`

**Current code** (line 112):

```rust
match ensure_sudo_cached(info.is_root) {
```

**Desired code:**

```rust
match ensure_sudo_cached(info.is_root, args.non_interactive) {
```

### Change 7: `src/cli/sudo.rs` -- Extend `ensure_sudo_cached` signature and remove TODO

**Current code** (lines 58-71):

```rust
/// # Arguments
///
/// * `is_root` - Whether the current user is UID 0 (from `PlatformInfo.is_root`).
// TODO: Also accept a `non_interactive` parameter from the --non-interactive
// global CLI flag once it is wired through to apply::run() / doctor::run().
pub fn ensure_sudo_cached(is_root: bool) -> SudoCacheResult {
    // Already root -- no sudo needed.
    if is_root {
        return SudoCacheResult::AlreadyRoot;
    }

    // Non-interactive -- do not prompt; let individual commands fail fast.
    if !std::io::stdin().is_terminal() {
        return SudoCacheResult::NonInteractive;
    }
```

**Desired code:**

```rust
/// # Arguments
///
/// * `is_root` - Whether the current user is UID 0 (from `PlatformInfo.is_root`).
/// * `non_interactive` - Whether the user passed `--non-interactive` on the CLI.
pub fn ensure_sudo_cached(is_root: bool, non_interactive: bool) -> SudoCacheResult {
    // Already root -- no sudo needed.
    if is_root {
        return SudoCacheResult::AlreadyRoot;
    }

    // Non-interactive -- do not prompt; let individual commands fail fast.
    if non_interactive || !std::io::stdin().is_terminal() {
        return SudoCacheResult::NonInteractive;
    }
```

**Key details:**
- The TODO comment on lines 61-62 is removed entirely.
- A new `non_interactive` parameter is added.
- A new doc comment line is added for the parameter.
- The `non_interactive` check comes first in the `if` condition (short-circuit), followed
  by the existing `!stdin().is_terminal()` check. Both paths return `NonInteractive`.
- When `non_interactive` is `true`, the function returns `NonInteractive` without ever
  calling `sudo -v`, meaning no password prompt will appear.

### Change 8: `src/cli/sudo.rs` -- Update existing unit test

**Current code** (line 138):

```rust
#[test]
fn already_root_returns_immediately() {
    let result = ensure_sudo_cached(true);
    assert!(matches!(result, SudoCacheResult::AlreadyRoot));
}
```

**Desired code:**

```rust
#[test]
fn already_root_returns_immediately() {
    let result = ensure_sudo_cached(true, false);
    assert!(matches!(result, SudoCacheResult::AlreadyRoot));
}
```

### Change 9: `src/cli/sudo.rs` -- Add new unit test for `non_interactive`

Add the following test after `already_root_returns_immediately`:

```rust
#[test]
fn non_interactive_flag_returns_non_interactive() {
    let result = ensure_sudo_cached(false, true);
    assert!(matches!(result, SudoCacheResult::NonInteractive));
}
```

This test verifies that when `non_interactive` is `true` and `is_root` is `false`, the
function returns `NonInteractive` without attempting any sudo operation.

### Change 10: `src/main.rs` -- Extract `non_interactive` and forward it

**Current code** (lines 13-29):

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init(args) => cli::init::run(args),
        Command::Apply(args) => cli::apply::run(args),
        Command::Status(args) => cli::status::run(args),
        Command::Sync(args) => cli::sync::run(args),
        Command::Vault(args) => cli::vault::run(args),
        Command::Mcp(args) => cli::mcp::run(args),
        Command::Doctor(args) => cli::doctor::run(args),
        Command::Update(args) => cli::update::run(args),
        Command::Diff(args) => cli::diff::run(args),
        Command::Template(args) => cli::template::run(args),
        Command::Loop(args) => cli::loop_cmd::run(args),
        Command::Statusline(args) => cli::statusline::run(args),
    }
}
```

**Desired code:**

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();
    let non_interactive = cli.non_interactive;

    match cli.command {
        Command::Init(args) => cli::init::run(args),
        Command::Apply(mut args) => {
            args.non_interactive = non_interactive;
            cli::apply::run(args)
        }
        Command::Status(args) => cli::status::run(args),
        Command::Sync(args) => cli::sync::run(args),
        Command::Vault(args) => cli::vault::run(args),
        Command::Mcp(args) => cli::mcp::run(args),
        Command::Doctor(mut args) => {
            args.non_interactive = non_interactive;
            cli::doctor::run(args)
        }
        Command::Update(args) => cli::update::run(args),
        Command::Diff(args) => cli::diff::run(args),
        Command::Template(args) => cli::template::run(args),
        Command::Loop(args) => cli::loop_cmd::run(args),
        Command::Statusline(args) => cli::statusline::run(args),
    }
}
```

**Key details:**
- `cli.non_interactive` is extracted into a local `let` binding **before** the `match`
  consumes `cli.command`. This is necessary because `match cli.command` moves `cli.command`
  out of the struct; we cannot access `cli.non_interactive` after that.
- Only the `Apply` and `Doctor` arms use `mut args` to set the field. All other arms are
  unchanged.
- The match arms for `Apply` and `Doctor` become block expressions `{ ... }` that set the
  field and then call `run()`.

---

## 4. Behavioral Changes

### When `--non-interactive` is NOT passed (default)

No change. The `non_interactive` field defaults to `false` via `#[arg(skip)]`. The
`ensure_sudo_cached` function behaves exactly as today: it checks `stdin().is_terminal()`
and prompts for sudo if the terminal is interactive.

### When `--non-interactive` IS passed

| Scenario | Before | After |
|----------|--------|-------|
| `great apply --non-interactive` on Ubuntu (needs Homebrew install + apt) | `sudo -v` prompt appears; blocks waiting for password | Returns `SudoCacheResult::NonInteractive` immediately; no prompt; apt commands using `sudo -n` will fail fast with actionable error |
| `great doctor --fix --non-interactive` with fixable sudo issues | `sudo -v` prompt appears | Returns `NonInteractive`; fixes requiring sudo are skipped |
| `great apply --non-interactive` on macOS (Homebrew already installed) | No sudo prompt (sudo not needed) | No change |
| `great apply --non-interactive` piped stdin (e.g., CI) | `stdin().is_terminal()` already returns `NonInteractive` | Same result via `non_interactive || !is_terminal()` |
| `available_managers()` call with `--non-interactive` | `Apt::new(false)` -- apt uses interactive sudo | `Apt::new(true)` -- apt uses `sudo -n` (non-interactive sudo), fails fast if no cached credentials |

---

## 5. Edge Cases

### 5a. Non-interactive flag AND piped stdin

Both conditions independently cause `NonInteractive` return. The `||` in the condition
means the flag check short-circuits, so `stdin().is_terminal()` is never called. This is
correct and produces the same result.

### 5b. Non-interactive flag with `--dry-run`

The `needs_sudo` computation in `apply.rs` line 403 already short-circuits on `args.dry_run`:
```rust
let needs_sudo = !args.dry_run && { ... };
```
When `--dry-run` is set, `needs_sudo` is `false` and `ensure_sudo_cached` is never called.
The `non_interactive` flag is irrelevant in this case. No special handling needed.

### 5c. Non-interactive flag when already root

The `is_root` check (line 65 of sudo.rs) runs before the `non_interactive` check. When
running as root, the function returns `AlreadyRoot` regardless of the flag. This is correct:
root does not need sudo.

### 5d. Non-interactive flag when sudo is not on PATH

The `is_root` check and `non_interactive` check both run before the `which::which("sudo")`
check. When `non_interactive` is `true`, the function returns `NonInteractive` before ever
checking for the sudo binary. This is correct: if the user asked for non-interactive mode,
we should not prompt regardless of whether sudo exists.

### 5e. Platform differences

| Platform | Sudo needed for | Effect of `--non-interactive` |
|----------|-----------------|-------------------------------|
| macOS ARM64/x86_64 | Homebrew install (only if Homebrew missing) | Skips sudo prompt; Homebrew install will fail or use `NONINTERACTIVE=1` env var (already set in the bash command at apply.rs line 466) |
| Ubuntu (bare metal) | apt-get operations, Homebrew install | Skips sudo prompt; apt uses `sudo -n` via `Apt::new(true)` |
| WSL2 (Ubuntu) | Same as Ubuntu | Same as Ubuntu |
| Fedora/other | No apt, no Homebrew auto-install | `non_interactive` still respected but rarely triggered since these platforms rarely need sudo in the great.sh flow |

### 5f. Future subcommands

Other subcommands (`init`, `status`, `sync`, `vault`, `mcp`, `update`, `diff`, `template`,
`loop`, `statusline`) do not call `ensure_sudo_cached` or `available_managers`. They are not
modified. If a future subcommand needs the flag, the same `#[arg(skip)]` + `main.rs` setter
pattern should be followed.

---

## 6. Error Handling

No new error paths are introduced. The `NonInteractive` return from `ensure_sudo_cached` is
already handled by all callers -- both `apply.rs` (line 422-424) and `doctor.rs` (line
112-115) match `SudoCacheResult::Cached(keepalive) => Some(keepalive)` and fall through to
`_ => None` for all other variants. When `NonInteractive` is returned:

1. No sudo keepalive thread is started.
2. Subsequent commands that need sudo (via `Apt::new(true)`) will use `sudo -n` and fail
   fast with an actionable error message if credentials are not cached.
3. The overall `apply` or `doctor --fix` run continues -- individual operations that need
   sudo will fail independently with their own error messages.

---

## 7. Security Considerations

- The `--non-interactive` flag is opt-in. It does not weaken security; it prevents sudo
  prompts from appearing in automated contexts where no human is present to enter a password.
- In CI, the combination of `--non-interactive` and piped stdin was already handled by the
  `stdin().is_terminal()` check. The flag adds an explicit, documented way to achieve the
  same behavior without relying on terminal detection heuristics.

---

## 8. Testing Strategy

### 8a. Unit tests (in `src/cli/sudo.rs`)

| Test | Status | Description |
|------|--------|-------------|
| `already_root_returns_immediately` | **Existing -- update signature** | `ensure_sudo_cached(true, false)` returns `AlreadyRoot` |
| `keepalive_drop_signals_stop` | **Existing -- no change** | Tests `SudoKeepalive` drop behavior (does not call `ensure_sudo_cached`) |
| `non_interactive_flag_returns_non_interactive` | **New** | `ensure_sudo_cached(false, true)` returns `NonInteractive` |

### 8b. Integration tests (in `tests/cli_smoke.rs`)

No new integration tests required. The existing smoke tests (`help_shows_description`,
`version_shows_semver`, `no_args_shows_usage`, `init_help_shows_initialize`) do not exercise
`apply` or `doctor` deeply. Testing the actual sudo behavior requires a real system with sudo
configured, which is not suitable for automated tests.

The flag's effect can be manually verified:

```bash
# Verify flag is accepted (no "unrecognized argument" error):
great apply --non-interactive --dry-run

# Verify on a platform that needs sudo (Ubuntu):
# With flag: no sudo prompt appears
great apply --non-interactive
# Without flag: sudo prompt appears (if terminal is interactive)
great apply
```

### 8c. Clippy

After this change, `cargo clippy` should no longer produce a warning about the unused
`non_interactive` field on `Cli`, because it is now read in `main.rs`.

---

## 9. Acceptance Criteria

- [ ] `great apply --non-interactive` on Ubuntu/Debian (where `ensure_sudo_cached` would
      normally trigger `sudo -v`) completes without any sudo password prompt.
- [ ] `great doctor --fix --non-interactive` similarly skips the sudo prompt.
- [ ] `great apply --non-interactive --dry-run` works without error (flag accepted, dry-run
      proceeds normally).
- [ ] `ensure_sudo_cached(false, true)` returns `SudoCacheResult::NonInteractive` (new unit
      test).
- [ ] `ensure_sudo_cached(true, false)` returns `SudoCacheResult::AlreadyRoot` (updated
      existing unit test).
- [ ] The TODO comment at `src/cli/sudo.rs` lines 61-62 is removed.
- [ ] `cargo clippy` produces zero warnings related to `non_interactive`.
- [ ] `cargo test` passes with no regressions (all existing tests plus new test pass).
- [ ] The `--non-interactive` flag does NOT appear as a subcommand-level argument in
      `great apply --help` or `great doctor --help` output (it remains global-only).

---

## 10. Files Modified

| File | Nature of Change |
|------|-----------------|
| `src/main.rs` | Extract `cli.non_interactive`, set it on `Apply` and `Doctor` args before dispatch |
| `src/cli/apply.rs` | Add `#[arg(skip)] pub non_interactive: bool` to `Args`; pass to `ensure_sudo_cached` and 3x `available_managers` calls |
| `src/cli/doctor.rs` | Add `#[arg(skip)] pub non_interactive: bool` to `Args`; pass to `ensure_sudo_cached` and 1x `available_managers` call |
| `src/cli/sudo.rs` | Add `non_interactive: bool` parameter to `ensure_sudo_cached`; add `non_interactive ||` to the terminal check; remove TODO; update 1 test; add 1 test |

Total: 4 files modified, 0 files created, ~30 lines changed.

---

## 11. Out of Scope

The following are explicitly NOT part of this task:

- **`src/cli/mod.rs`** -- The `Cli` struct and its `non_interactive` field are already
  correct. No changes.
- **`src/platform/package_manager.rs`** -- The `available_managers(non_interactive: bool)`
  signature and `Apt::new(non_interactive)` plumbing are already correct. The only issue is
  that callers pass `false` instead of the actual flag value. Those call sites are in
  `apply.rs` and `doctor.rs` (covered above).
- **Other subcommands** -- `init`, `status`, `sync`, `vault`, `mcp`, `update`, `diff`,
  `template`, `loop`, `statusline` do not call `ensure_sudo_cached` or `available_managers`.
  They do not need the `non_interactive` flag.
- **`verbose` and `quiet` flags** -- These global flags on `Cli` are also currently unused.
  Wiring them through is a separate task.
- **New CLI-level tests for the sudo prompt** -- Testing sudo prompts requires a real system
  with sudo configured and cannot be meaningfully automated in unit/integration tests.

---

## 12. Build Order

This task has a single natural build order because the signature change in `sudo.rs` breaks
the call sites in `apply.rs` and `doctor.rs`, and the `Args` struct changes break `main.rs`.

1. **`src/cli/sudo.rs`** -- Change the `ensure_sudo_cached` signature, remove TODO, update
   existing test, add new test. (Causes compile errors in apply.rs and doctor.rs.)
2. **`src/cli/apply.rs`** -- Add `non_interactive` to `Args`, update `ensure_sudo_cached`
   call, update 3x `available_managers` calls. (Causes compile error in main.rs because
   `Args` struct changed shape, though `#[arg(skip)]` defaults it.)
3. **`src/cli/doctor.rs`** -- Add `non_interactive` to `Args`, update `ensure_sudo_cached`
   call, update 1x `available_managers` call.
4. **`src/main.rs`** -- Extract `non_interactive` from `Cli`, set it on `Apply` and `Doctor`
   args.

After step 4, `cargo build` and `cargo test` should both succeed.

In practice, steps 1-4 can be done in any order and compiled together, since this is a single
commit. The build order above is the logical dependency order for understanding.
