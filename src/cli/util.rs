//! Shared CLI utility functions.
//!
//! Extracts helpers that are used by multiple subcommands to avoid duplication.

/// Try to get a command's version string.
///
/// Runs `<cmd> --version` and returns the first line of stdout, or `None`
/// if the command fails or produces no output.
pub fn get_command_version(cmd: &str) -> Option<String> {
    let output = std::process::Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .ok()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        let first_line = version.lines().next().unwrap_or("").trim();
        if first_line.is_empty() {
            None
        } else {
            Some(first_line.to_string())
        }
    } else {
        None
    }
}
