use colored::Colorize;

/// Print a success message to stderr with a green checkmark prefix.
pub fn success(msg: &str) {
    eprintln!("{} {}", "✓".green(), msg);
}

/// Print a warning message to stderr with a yellow warning prefix.
pub fn warning(msg: &str) {
    eprintln!("{} {}", "⚠".yellow(), msg);
}

/// Print an error message to stderr with a red cross prefix.
pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red(), msg);
}

/// Print an informational message to stderr with a blue info prefix.
pub fn info(msg: &str) {
    eprintln!("{} {}", "ℹ".blue(), msg);
}

/// Print a bold header/section title to stderr.
pub fn header(msg: &str) {
    eprintln!("{}", msg.bold());
}

/// Print a bold header/section title to stdout.
///
/// Use this in pipeline-oriented commands (e.g., `diff`) where all output
/// is data and belongs on stdout. Interactive commands should use `header()`.
pub fn header_stdout(msg: &str) {
    println!("{}", msg.bold());
}

/// Print an informational message to stdout with a blue info prefix.
///
/// Stdout variant of `info()` for pipeline-oriented commands.
pub fn info_stdout(msg: &str) {
    println!("{} {}", "ℹ".blue(), msg);
}

/// Print a success message to stdout with a green checkmark prefix.
///
/// Stdout variant of `success()` for pipeline-oriented commands.
pub fn success_stdout(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

/// Create a progress spinner with the given message.
///
/// The spinner ticks at 80ms intervals and uses braille-dot characters.
/// Call `.finish_with_message()` or `.finish_and_clear()` when done.
pub fn spinner(msg: &str) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::with_template("{spinner:.green} {msg}")
            .expect("valid spinner template")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
