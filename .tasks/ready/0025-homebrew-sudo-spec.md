# Spec 0025: Pre-cache sudo credentials before Homebrew install

**Size:** S (one new helper function, three call sites, no new dependencies)

## Summary

`great apply` and `great doctor --fix` install Homebrew with `NONINTERACTIVE=1`, which suppresses the sudo password prompt. When sudo credentials are not already cached, the installer fails immediately with "Need sudo access on macOS" even though the user has admin rights. Meanwhile, `bootstrap.rs` runs `sudo apt-get` interactively, creating an inconsistency.

This spec adds an `ensure_sudo_cached()` helper in `src/cli/sudo.rs` that runs `sudo -v` once at the start of the apply/doctor flow -- prompting the user for their password while the terminal is free -- and optionally keeps the credential cache alive with a background thread. All subsequent `NONINTERACTIVE=1` and `sudo apt-get` calls then succeed without re-prompting.

## Interfaces

### New module: `src/cli/sudo.rs`

```rust
use std::io::IsTerminal;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::cli::output;

/// Handle to a background thread that periodically refreshes the sudo
/// credential cache. Dropping this handle signals the thread to stop.
pub struct SudoKeepalive {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Drop for SudoKeepalive {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Result of attempting to cache sudo credentials.
pub enum SudoCacheResult {
    /// Credentials were successfully cached (or were already cached).
    /// The `SudoKeepalive` handle keeps them alive until dropped.
    Cached(SudoKeepalive),
    /// Sudo prompt was skipped because the session is non-interactive
    /// (piped stdin, CI, etc.).
    NonInteractive,
    /// Skipped because the process is already running as root.
    AlreadyRoot,
    /// Skipped because `sudo` is not available on PATH.
    NoSudoBinary,
    /// The user cancelled the sudo prompt (Ctrl-C at password prompt,
    /// or entered wrong password 3 times). The caller should warn but
    /// can continue -- individual sudo calls will fail on their own.
    PromptFailed,
}

/// Pre-cache sudo credentials by running `sudo -v`.
///
/// Call this once, early in the apply/doctor flow, before any commands
/// that require sudo. The returned `SudoCacheResult` tells the caller
/// what happened. If `Cached`, the embedded `SudoKeepalive` refreshes
/// the cache every 60 seconds until dropped.
///
/// # When this is a no-op
///
/// - Non-interactive terminal (piped stdin, `--non-interactive` flag, CI)
/// - Already running as root (`is_root == true`)
/// - `sudo` binary not found on PATH
///
/// # Arguments
///
/// * `is_root` - Whether the current user is UID 0 (from `PlatformInfo.is_root`).
pub fn ensure_sudo_cached(is_root: bool) -> SudoCacheResult {
    // Already root -- no sudo needed.
    if is_root {
        return SudoCacheResult::AlreadyRoot;
    }

    // Non-interactive -- do not prompt; let individual commands fail fast.
    if !std::io::stdin().is_terminal() {
        return SudoCacheResult::NonInteractive;
    }

    // Check that the sudo binary exists.
    if which::which("sudo").is_err() {
        return SudoCacheResult::NoSudoBinary;
    }

    // Inform the user why they are being prompted.
    output::info(
        "Some operations require administrator access. \
         You may be prompted for your password.",
    );

    // Run `sudo -v` to prompt for the password and cache credentials.
    let status = Command::new("sudo")
        .arg("-v")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => {
            // Start keepalive thread.
            let stop = Arc::new(AtomicBool::new(false));
            let stop_clone = Arc::clone(&stop);
            let handle = thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_secs(60));
                    if stop_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    let refresh = Command::new("sudo")
                        .args(["-vn"])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                    if !matches!(refresh, Ok(s) if s.success()) {
                        // Cache expired or sudo revoked -- stop trying.
                        break;
                    }
                }
            });
            SudoCacheResult::Cached(SudoKeepalive {
                stop,
                handle: Some(handle),
            })
        }
        _ => {
            output::warning(
                "sudo authentication failed or was cancelled. \
                 Some operations may fail.",
            );
            SudoCacheResult::PromptFailed
        }
    }
}
```

### Visibility and re-export

`src/cli/mod.rs` gains:

```rust
pub mod sudo;
```

The function is `pub` within the crate. Call sites use `crate::cli::sudo::ensure_sudo_cached`.

## Implementation approach and build order

### Step 1: Create `src/cli/sudo.rs` (new file)

The complete module as specified in the Interfaces section above. Contains:

- `SudoKeepalive` struct with `Drop` implementation
- `SudoCacheResult` enum (five variants)
- `ensure_sudo_cached(is_root: bool) -> SudoCacheResult` function

No external crate dependencies. Uses `which` (already a dependency via `platform/detection.rs`), `std::io::IsTerminal` (stabilized Rust 1.70, already used in `loop_cmd.rs`), and `std::sync::atomic`.

### Step 2: Register module in `src/cli/mod.rs`

Add `pub mod sudo;` to the module list (alphabetically between `status` on line 9 and `statusline` on line 10).

```diff
 pub mod status;
+pub mod sudo;
 pub mod statusline;
```

### Step 3: Call `ensure_sudo_cached()` in `src/cli/apply.rs`

Insert the call after platform detection (line 390, after `let info = platform::detect_platform_info();`) and before `bootstrap::ensure_prerequisites()` (line 400). The call must happen before any sudo-requiring operation. The best insertion point is after the dry-run check block (after line 397), just before the comment on line 399.

The `_sudo_keepalive` binding keeps the `SudoKeepalive` handle alive for the duration of `run()`. It is dropped automatically when `run()` returns, which stops the background thread.

```rust
// After line 397 (end of dry-run check block)
// Before line 399 (`// 2b. System prerequisites...`)

// 2a. Pre-cache sudo credentials before any installs that need root.
let needs_sudo = !args.dry_run && {
    let needs_homebrew = match &info.platform {
        platform::Platform::MacOS { .. } => true,
        platform::Platform::Linux { distro, .. }
        | platform::Platform::Wsl { distro, .. } => {
            matches!(
                distro,
                platform::LinuxDistro::Ubuntu | platform::LinuxDistro::Debian
            )
        }
        _ => false,
    };
    (needs_homebrew && !info.capabilities.has_homebrew)
        || bootstrap::is_apt_distro(&info.platform)
};

let _sudo_keepalive = if needs_sudo {
    use crate::cli::sudo::{ensure_sudo_cached, SudoCacheResult};
    match ensure_sudo_cached(info.is_root) {
        SudoCacheResult::Cached(keepalive) => Some(keepalive),
        _ => None,
    }
} else {
    None
};
```

Note: The `needs_sudo` check is intentionally conservative. It returns `true` when Homebrew is absent (macOS/Ubuntu/Debian) or when the platform is apt-based (bootstrap uses `sudo apt-get`). It does NOT prompt for sudo in dry-run mode.

### Step 4: Call `ensure_sudo_cached()` in `src/cli/doctor.rs`

Insert the call inside the `if args.fix && !result.fixable.is_empty()` block (line 94), before the fix loop. The doctor only needs sudo when `--fix` is passed and there are fixable issues.

```rust
// After line 97 (`let managers = package_manager::available_managers(false);`)
// Before line 98 (`let mut fixed = 0;`)

// Pre-cache sudo if any fix might need it.
let has_sudo_fix = result.fixable.iter().any(|issue| {
    matches!(
        issue.action,
        FixAction::InstallHomebrew
            | FixAction::InstallSystemPrerequisite { .. }
            | FixAction::InstallDocker
            | FixAction::FixInotifyWatches
    )
});

let _sudo_keepalive = if has_sudo_fix {
    use crate::cli::sudo::{ensure_sudo_cached, SudoCacheResult};
    let info = platform::detect_platform_info();
    match ensure_sudo_cached(info.is_root) {
        SudoCacheResult::Cached(keepalive) => Some(keepalive),
        _ => None,
    }
} else {
    None
};
```

### Step 5: No changes to existing `NONINTERACTIVE=1` or `sudo apt-get` calls

The Homebrew install commands in `apply.rs:433-438` and `doctor.rs:122-124` remain unchanged. `NONINTERACTIVE=1` is correct -- it prevents the Homebrew *installer script* from prompting interactively (avoiding hanging in CI). With sudo credentials pre-cached, the installer's internal `sudo` calls succeed silently.

The `sudo apt-get` calls in `bootstrap.rs` also remain unchanged. They already inherit stdin, so they would prompt if needed -- but with pre-cached credentials, they just succeed immediately.

## Files to modify

| File | Action | Description |
|------|--------|-------------|
| `src/cli/sudo.rs` | **Create** | New module with `ensure_sudo_cached()` and `SudoKeepalive` |
| `src/cli/mod.rs` | Modify (line 9-10) | Add `pub mod sudo;` between `status` and `statusline` |
| `src/cli/apply.rs` | Modify (after line 397) | Insert `needs_sudo` check and `ensure_sudo_cached()` call |
| `src/cli/doctor.rs` | Modify (line 94-98) | Insert `ensure_sudo_cached()` call before fix loop |

No files deleted. No new crate dependencies.

## Edge cases

### Non-interactive / CI environment

When stdin is not a terminal (piped input, GitHub Actions, `great apply < /dev/null`), `ensure_sudo_cached()` returns `NonInteractive` immediately. No prompt is attempted. The existing `NONINTERACTIVE=1` fail-fast behavior is preserved. This is the correct behavior for CI -- sudo should either be pre-configured (passwordless) or the build should fail fast.

### Already running as root

When `PlatformInfo.is_root` is `true` (UID 0), `ensure_sudo_cached()` returns `AlreadyRoot`. No prompt, no keepalive thread. `sudo` commands then succeed without a password because the user is already root. This covers Docker containers and `sudo -i` sessions.

### User cancels sudo prompt

If the user presses Ctrl-C at the password prompt or enters the wrong password three times, `sudo -v` exits with a non-zero status. `ensure_sudo_cached()` returns `PromptFailed` and prints a warning. The apply flow continues -- individual sudo calls will fail on their own with specific error messages ("Failed to install Homebrew", etc.). This avoids a hard abort for a partial failure.

### sudo binary not found

On systems without `sudo` (some minimal containers, Windows), `which::which("sudo")` returns `Err` and `ensure_sudo_cached()` returns `NoSudoBinary`. The apply flow continues. This is safe because:
- On macOS, `sudo` is always present.
- On Linux containers without `sudo`, the user is typically root.
- On Windows (non-WSL), the Homebrew/apt code paths are not reached.

### sudo credential timeout

The default `timestamp_timeout` is 5 minutes on macOS and 15 minutes on most Linux distributions. The keepalive thread refreshes every 60 seconds with `sudo -vn` (non-interactive). If the cache expires anyway (user configured `timestamp_timeout=0` in `/etc/sudoers`), the refresh silently fails and the thread exits. Subsequent sudo calls may prompt or fail, which is the expected behavior for that sudoers configuration.

### Dry-run mode

In dry-run mode (`great apply --dry-run`), `needs_sudo` evaluates to `false` because of the `!args.dry_run &&` guard. No sudo prompt appears during dry runs.

### Concurrent keepalive thread and process exit

The `SudoKeepalive` struct implements `Drop`, which sets the atomic stop flag and joins the thread. This ensures clean shutdown even on early return or error propagation via `?`. The thread sleeps for at most 60 seconds, so the join blocks for at most 60 seconds. In practice, the thread checks the stop flag after each sleep, so shutdown is prompt.

If the process is killed (SIGKILL), the thread dies with it. The `sudo -vn` child process, if running, is also cleaned up by the OS. No orphaned processes.

### Platform matrix

| Platform | sudo exists | Homebrew path | apt path | Expected behavior |
|----------|-------------|---------------|----------|-------------------|
| macOS ARM64 | Yes | Brew install | N/A | Prompts once, brew succeeds |
| macOS x86_64 | Yes | Brew install | N/A | Same |
| Ubuntu 24.04 | Yes | Linuxbrew install | apt-get | Prompts once, both succeed |
| WSL2 Ubuntu | Yes | Linuxbrew install | apt-get | Prompts once, both succeed |
| Debian 12 | Yes | Linuxbrew install | apt-get | Same as Ubuntu |
| Fedora (no brew) | Yes | N/A | N/A | `needs_sudo` = false, no prompt |
| Docker root | No/Yes | Varies | Varies | `AlreadyRoot`, no prompt |
| GitHub Actions | Yes | Varies | Varies | `NonInteractive`, no prompt |

## Error handling

| Scenario | Error message (stderr) | Behavior |
|----------|----------------------|----------|
| sudo prompt failed/cancelled | "sudo authentication failed or was cancelled. Some operations may fail." | Warning via `output::warning`, flow continues |
| sudo binary missing | (none) | Silent skip, flow continues |
| Keepalive refresh fails | (none) | Thread exits silently, no user output |
| Individual sudo call fails later | Existing per-tool error messages | No change from current behavior |

All messages are actionable -- the user knows what happened and what to expect. No panics, no `.unwrap()` on fallible operations.

## Security considerations

- **No privilege escalation**: `sudo -v` only validates credentials; it does not execute anything as root. The actual root commands remain exactly as they are today.
- **No credential storage**: Credentials are cached by the system's `sudo` facility in its standard timestamp file (typically `/var/run/sudo/ts/` or `/var/db/sudo/`). `great` never sees or stores the password.
- **Keepalive thread**: Runs `sudo -vn` (non-interactive refresh) which extends the existing cache. This is the same mechanism used by Homebrew itself, Ansible's `become`, and macOS installer scripts. The `-n` flag ensures the refresh never prompts -- it either succeeds silently or fails silently.
- **No new attack surface**: The only new subprocess calls are `sudo -v` and `sudo -vn`, both of which are standard POSIX. No new network calls, no new file writes by `great` itself.

## Testing strategy

### Unit tests in `src/cli/sudo.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_root_returns_immediately() {
        let result = ensure_sudo_cached(true);
        assert!(matches!(result, SudoCacheResult::AlreadyRoot));
    }

    #[test]
    fn keepalive_drop_signals_stop() {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !stop_clone.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(10));
            }
        });
        let keepalive = SudoKeepalive {
            stop: Arc::clone(&stop),
            handle: Some(handle),
        };
        drop(keepalive);
        assert!(stop.load(Ordering::Relaxed));
    }
}
```

The `NonInteractive` path cannot be reliably unit-tested because CI runners have piped stdin (so it would always trigger). The `already_root` test is safe on all platforms.

### Integration test in `tests/cli_smoke.rs`

Add one test to confirm that `apply --dry-run` does not attempt a sudo prompt (regression guard):

```rust
#[test]
fn apply_dry_run_no_sudo_prompt() {
    let dir = TempDir::new().unwrap();
    // Create a minimal great.toml
    std::fs::write(
        dir.path().join("great.toml"),
        "[tools.runtimes]\nnode = \"22\"\n",
    )
    .unwrap();

    // Run with piped stdin (non-interactive) -- should not hang on sudo
    great()
        .current_dir(dir.path())
        .args(["apply", "--dry-run"])
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success();
}
```

### Manual test script (for macOS/Linux with sudo)

```bash
# Clear sudo cache
sudo -k

# Run apply -- expect a single password prompt, then Homebrew installs
cargo run -- apply

# Verify: only one password prompt appeared
# Verify: Homebrew installed successfully (if it was missing)
```

## Acceptance criteria

- [ ] When `great apply` runs interactively on macOS and Homebrew is absent, the user sees a single password prompt (`sudo -v`) before installation begins -- not a failure.
- [ ] After the `sudo -v` prompt, the Homebrew install with `NONINTERACTIVE=1` succeeds using the cached credentials.
- [ ] `great doctor --fix` also pre-caches sudo before attempting Homebrew install or system package fixes.
- [ ] In non-interactive contexts (piped stdin, CI), no sudo prompt is attempted -- existing fail-fast behavior is preserved.
- [ ] The sudo keepalive thread does not outlive the process or produce visible output.
- [ ] `bootstrap.rs` sudo calls (`apt-get`) also benefit from the cached credentials -- no double-prompting.
- [ ] No new crate dependencies added.
- [ ] `cargo clippy` passes with no new warnings.
- [ ] `cargo test` passes (existing tests unbroken, new tests pass).
- [ ] `great apply --dry-run` never prompts for sudo.
