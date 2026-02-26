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
            h.thread().unpark(); // wake the parked thread immediately
            let _ = h.join(); // returns in microseconds
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
/// - Non-interactive terminal (piped stdin, CI)
/// - Already running as root (`is_root == true`)
/// - `sudo` binary not found on PATH
///
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
                // Use park_timeout instead of sleep so Drop can unpark us
                // for sub-millisecond shutdown latency.
                loop {
                    thread::park_timeout(Duration::from_secs(60));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_root_returns_immediately() {
        let result = ensure_sudo_cached(true, false);
        assert!(matches!(result, SudoCacheResult::AlreadyRoot));
    }

    #[test]
    fn non_interactive_flag_returns_non_interactive() {
        let result = ensure_sudo_cached(false, true);
        assert!(matches!(result, SudoCacheResult::NonInteractive));
    }

    #[test]
    fn keepalive_drop_signals_stop() {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            // Simulate the real keepalive: park until unparked or timeout.
            while !stop_clone.load(Ordering::Relaxed) {
                thread::park_timeout(Duration::from_secs(60));
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
