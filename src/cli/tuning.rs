use crate::cli::output;
use crate::platform::PlatformInfo;

use super::bootstrap;

/// Minimum recommended value for inotify max_user_watches.
const MIN_INOTIFY_WATCHES: u64 = 524_288;

/// Apply system-level kernel tuning. Only runs on Linux or WSL.
pub fn apply_system_tuning(dry_run: bool, info: &PlatformInfo) {
    if !bootstrap::is_linux_like(&info.platform) {
        return;
    }

    output::header("System Tuning");
    tune_inotify_watches(dry_run);
    println!();
}

/// Check the current inotify max_user_watches value.
///
/// Returns `(current_value, is_sufficient)`. `current_value` is `None` if the
/// sysctl file cannot be read (e.g. on macOS).
pub fn check_inotify_watches() -> (Option<u64>, bool) {
    let content = match std::fs::read_to_string("/proc/sys/fs/inotify/max_user_watches") {
        Ok(c) => c,
        Err(_) => return (None, true), // Not Linux — skip
    };

    let current = content.trim().parse::<u64>().unwrap_or(0);
    (Some(current), current >= MIN_INOTIFY_WATCHES)
}

/// Tune inotify max_user_watches if below the recommended threshold.
fn tune_inotify_watches(dry_run: bool) {
    let (current, sufficient) = check_inotify_watches();

    let Some(current) = current else {
        // Not on Linux — nothing to do
        return;
    };

    if sufficient {
        output::success(&format!(
            "  inotify max_user_watches: {} (>= {})",
            current, MIN_INOTIFY_WATCHES
        ));
        return;
    }

    output::warning(&format!(
        "  inotify max_user_watches: {} (below {} recommended)",
        current, MIN_INOTIFY_WATCHES
    ));

    if dry_run {
        output::info(&format!(
            "  Would set fs.inotify.max_user_watches = {}",
            MIN_INOTIFY_WATCHES
        ));
        return;
    }

    // Apply immediately
    let sysctl_status = std::process::Command::new("sudo")
        .args([
            "sysctl",
            "-w",
            &format!("fs.inotify.max_user_watches={}", MIN_INOTIFY_WATCHES),
        ])
        .status();

    match sysctl_status {
        Ok(s) if s.success() => {
            output::success(&format!(
                "  Set fs.inotify.max_user_watches = {} (temporary)",
                MIN_INOTIFY_WATCHES
            ));
        }
        _ => {
            output::error("  Failed to set inotify watches via sysctl");
            return;
        }
    }

    // Persist via sysctl.d config
    let conf_line = format!("fs.inotify.max_user_watches = {}\n", MIN_INOTIFY_WATCHES);
    let tee_status = std::process::Command::new("bash")
        .args([
            "-c",
            &format!(
                "echo '{}' | sudo tee /etc/sysctl.d/99-great.conf > /dev/null",
                conf_line.trim()
            ),
        ])
        .status();

    match tee_status {
        Ok(s) if s.success() => {
            output::success("  Persisted to /etc/sysctl.d/99-great.conf");
        }
        _ => {
            output::warning(
                "  Could not persist to /etc/sysctl.d/99-great.conf (change is temporary)",
            );
        }
    }
}
