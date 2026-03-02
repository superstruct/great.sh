# Spec 0016: `great loop install` -- Overwrite Safety and `--force` Flag

**Task:** `.tasks/backlog/0016-loop-install-overwrite-safety.md`
**Status:** ready
**Type:** enhancement
**Estimated Complexity:** M (single file, 4 logical changes + 6 new tests)

---

## Summary

`great loop install` unconditionally overwrites all 21 managed files (15 agent personas, 5 slash commands, 1 teams config) on every invocation. A user who has customized an agent persona or slash command loses those edits silently on re-install.

This spec adds overwrite protection: before writing, the command checks which managed files already exist on disk. If any exist and `--force` was not passed, the user is prompted for confirmation (interactive TTY) or the command aborts with an explanatory message (non-interactive). A fresh install where no managed files exist proceeds silently without prompting.

Files already guarded by their own safety logic (`settings.json`, `.gitignore`) are out of scope.

---

## Files to Modify

| File | Change |
|------|--------|
| `/home/isaac/src/sh.great/src/cli/loop_cmd.rs` | Add `--force` flag to `Install` variant, add `collect_existing_paths` helper, add `confirm_overwrite` helper, thread `force` through `run_install`, add 6 unit tests |

No new files are created. No other files are modified.

---

## Interfaces

### CLI Surface Change

The `Install` variant of `LoopCommand` gains a `--force` flag:

```rust
/// Subcommands for managing the great.sh Loop agent team.
#[derive(Subcommand)]
pub enum LoopCommand {
    /// Install the great.sh Loop agent team to ~/.claude/
    Install {
        /// Also set up .tasks/ working state in current directory
        #[arg(long)]
        project: bool,

        /// Overwrite existing files without prompting
        #[arg(long)]
        force: bool,
    },
    /// Show loop installation status
    Status,
    /// Remove loop agent files from ~/.claude/
    Uninstall,
}
```

### `run` Dispatch Change

```rust
pub fn run(args: Args) -> Result<()> {
    match args.command {
        LoopCommand::Install { project, force } => run_install(project, force),
        LoopCommand::Status => run_status(),
        LoopCommand::Uninstall => run_uninstall(),
    }
}
```

### `run_install` Signature Change

```rust
fn run_install(project: bool, force: bool) -> Result<()>
```

### New Helper: `collect_existing_paths`

```rust
/// Returns the subset of the 21 managed file paths that already exist on disk.
///
/// Each returned path is the full absolute path (e.g., `/home/user/.claude/agents/davinci.md`).
/// The caller uses this list to decide whether to prompt the user for confirmation.
fn collect_existing_paths(claude_dir: &std::path::Path) -> Vec<std::path::PathBuf>
```

This function does not perform I/O beyond `Path::exists()` checks. It does not create directories or write files. It returns an owned `Vec` so the caller can inspect `.is_empty()` and iterate for display.

### New Helper: `confirm_overwrite`

```rust
/// Prompts the user to confirm overwriting existing files.
///
/// - If stdin is a TTY: prints the list of conflicting paths and asks `Overwrite? [y/N]`.
///   Returns `Ok(true)` if the user answers `y` or `yes` (case-insensitive),
///   `Ok(false)` otherwise (including empty input / Enter).
/// - If stdin is NOT a TTY: prints an explanatory message directing the user to `--force`
///   and returns `Ok(false)`.
///
/// This function never returns `Err` under normal conditions. It returns `Err` only if
/// reading from stdin fails (broken pipe, I/O error).
fn confirm_overwrite(existing: &[std::path::PathBuf]) -> Result<bool>
```

---

## Implementation Approach

### Build Order

All changes are in `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`. Apply in this order:

1. **Step 1:** Add `use std::io::IsTerminal;` import at the top of the file.
2. **Step 2:** Add `force: bool` field to the `Install` variant in `LoopCommand`.
3. **Step 3:** Update the `run()` dispatch to destructure and pass `force`.
4. **Step 4:** Add `collect_existing_paths` helper function.
5. **Step 5:** Add `confirm_overwrite` helper function.
6. **Step 6:** Modify `run_install` to accept `force`, call the helpers, and gate file writes.
7. **Step 7:** Add 6 unit tests.

---

## Exact Code Changes

### Change 1: Add `IsTerminal` import

**Current code (line 1):**

```rust
use anyhow::{Context, Result};
```

**Replace with:**

```rust
use anyhow::{bail, Context, Result};
```

The `bail!` macro is used by the abort path when the user declines overwrite or stdin is not a TTY.

No additional imports are needed. `std::io::IsTerminal` is used via its fully qualified path `std::io::stdin().is_terminal()` to avoid polluting the namespace, or alternatively add:

```rust
use std::io::IsTerminal;
```

at the top of the file alongside the other imports. The spec uses the trait import approach for clarity.

### Change 2: Add `--force` flag to `Install` variant

**Current code (lines 15-21):**

```rust
    /// Install the great.sh Loop agent team to ~/.claude/
    Install {
        /// Also set up .tasks/ working state in current directory
        #[arg(long)]
        project: bool,
    },
```

**Replace with:**

```rust
    /// Install the great.sh Loop agent team to ~/.claude/
    Install {
        /// Also set up .tasks/ working state in current directory
        #[arg(long)]
        project: bool,

        /// Overwrite existing files without prompting
        #[arg(long)]
        force: bool,
    },
```

### Change 3: Update `run()` dispatch

**Current code (lines 135-141):**

```rust
pub fn run(args: Args) -> Result<()> {
    match args.command {
        LoopCommand::Install { project } => run_install(project),
        LoopCommand::Status => run_status(),
        LoopCommand::Uninstall => run_uninstall(),
    }
}
```

**Replace with:**

```rust
pub fn run(args: Args) -> Result<()> {
    match args.command {
        LoopCommand::Install { project, force } => run_install(project, force),
        LoopCommand::Status => run_status(),
        LoopCommand::Uninstall => run_uninstall(),
    }
}
```

### Change 4: Add `collect_existing_paths` helper

**Insert after the `statusline_value()` function (after line 152) and before `run_install`:**

```rust
/// Returns the subset of the 21 managed file paths that already exist on disk.
///
/// Each returned path is the full absolute path. The caller uses this list to
/// decide whether to prompt the user for confirmation before overwriting.
fn collect_existing_paths(claude_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let agents_dir = claude_dir.join("agents");
    let commands_dir = claude_dir.join("commands");
    let teams_dir = claude_dir.join("teams").join("loop");

    let mut existing = Vec::new();

    for agent in AGENTS {
        let path = agents_dir.join(format!("{}.md", agent.name));
        if path.exists() {
            existing.push(path);
        }
    }

    for cmd in COMMANDS {
        let path = commands_dir.join(format!("{}.md", cmd.name));
        if path.exists() {
            existing.push(path);
        }
    }

    let config_path = teams_dir.join("config.json");
    if config_path.exists() {
        existing.push(config_path);
    }

    existing
}
```

### Change 5: Add `confirm_overwrite` helper

**Insert immediately after `collect_existing_paths`:**

```rust
/// Prompts the user to confirm overwriting existing files, or aborts in non-TTY contexts.
///
/// Returns `Ok(true)` if the user confirms, `Ok(false)` if they decline or stdin is not a TTY.
fn confirm_overwrite(existing: &[std::path::PathBuf]) -> Result<bool> {
    use std::io::IsTerminal;

    eprintln!();
    output::warning("The following files already exist and will be overwritten:");
    for path in existing {
        // Display with ~ for home directory paths for readability
        let display = if let Some(home) = dirs::home_dir() {
            if let Ok(relative) = path.strip_prefix(&home) {
                format!("~/{}", relative.display())
            } else {
                path.display().to_string()
            }
        } else {
            path.display().to_string()
        };
        eprintln!("  {}", display);
    }

    if !std::io::stdin().is_terminal() {
        eprintln!();
        output::error("stdin is not a terminal -- cannot prompt for confirmation.");
        output::info("Re-run with --force to overwrite: great loop install --force");
        return Ok(false);
    }

    eprint!("Overwrite? [y/N] ");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("failed to read from stdin")?;

    let answer = input.trim().to_lowercase();
    Ok(answer == "y" || answer == "yes")
}
```

### Change 6: Modify `run_install` to use the overwrite check

**Current code (line 155):**

```rust
fn run_install(project: bool) -> Result<()> {
```

**Replace with:**

```rust
fn run_install(project: bool, force: bool) -> Result<()> {
```

**Insert the overwrite check after the directory creation block and before the file-writing block.** Specifically, insert after line 171 (the `create_dir_all(&teams_dir)` call) and before line 173 (the `// Write agent files` comment):

```rust
    // Check for existing files before writing
    let existing = collect_existing_paths(&claude_dir);
    if !existing.is_empty() && !force {
        let confirmed = confirm_overwrite(&existing)?;
        if !confirmed {
            bail!("aborted: no files were modified");
        }
    }

    if force && !existing.is_empty() {
        output::info("(--force: overwriting existing files)");
    }
```

The full sequence in `run_install` after the directory creation block becomes:

```rust
    std::fs::create_dir_all(&agents_dir).context("failed to create ~/.claude/agents/ directory")?;
    std::fs::create_dir_all(&commands_dir)
        .context("failed to create ~/.claude/commands/ directory")?;
    std::fs::create_dir_all(&teams_dir)
        .context("failed to create ~/.claude/teams/loop/ directory")?;

    // Check for existing files before writing
    let existing = collect_existing_paths(&claude_dir);
    if !existing.is_empty() && !force {
        let confirmed = confirm_overwrite(&existing)?;
        if !confirmed {
            bail!("aborted: no files were modified");
        }
    }

    if force && !existing.is_empty() {
        output::info("(--force: overwriting existing files)");
    }

    // Write agent files
    for agent in AGENTS {
        // ... existing code unchanged ...
```

### Change 7: Add unit tests

Append these 6 tests inside the `mod tests` block (before the final closing `}`):

```rust
    #[test]
    fn test_collect_existing_paths_empty_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        // Don't create any subdirectories or files
        let existing = super::collect_existing_paths(&claude_dir);
        assert!(
            existing.is_empty(),
            "fresh directory should have no existing managed files"
        );
    }

    #[test]
    fn test_collect_existing_paths_partial_install() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        let agents_dir = claude_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        // Create only one agent file
        std::fs::write(agents_dir.join("davinci.md"), "test").unwrap();

        let existing = super::collect_existing_paths(&claude_dir);
        assert_eq!(existing.len(), 1);
        assert!(existing[0].ends_with("davinci.md"));
    }

    #[test]
    fn test_collect_existing_paths_full_install() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        let agents_dir = claude_dir.join("agents");
        let commands_dir = claude_dir.join("commands");
        let teams_dir = claude_dir.join("teams").join("loop");
        std::fs::create_dir_all(&agents_dir).unwrap();
        std::fs::create_dir_all(&commands_dir).unwrap();
        std::fs::create_dir_all(&teams_dir).unwrap();

        // Create all agent files
        for agent in super::AGENTS {
            std::fs::write(agents_dir.join(format!("{}.md", agent.name)), "test").unwrap();
        }
        // Create all command files
        for cmd in super::COMMANDS {
            std::fs::write(commands_dir.join(format!("{}.md", cmd.name)), "test").unwrap();
        }
        // Create teams config
        std::fs::write(teams_dir.join("config.json"), "{}").unwrap();

        let existing = super::collect_existing_paths(&claude_dir);
        // 15 agents + 5 commands + 1 teams config = 21
        assert_eq!(
            existing.len(),
            21,
            "full install should detect all 21 managed files, got {}",
            existing.len()
        );
    }

    #[test]
    fn test_collect_existing_paths_only_commands() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        let commands_dir = claude_dir.join("commands");
        std::fs::create_dir_all(&commands_dir).unwrap();

        std::fs::write(commands_dir.join("loop.md"), "test").unwrap();
        std::fs::write(commands_dir.join("bugfix.md"), "test").unwrap();

        let existing = super::collect_existing_paths(&claude_dir);
        assert_eq!(existing.len(), 2);
    }

    #[test]
    fn test_collect_existing_paths_ignores_unknown_files() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        let agents_dir = claude_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        // Create a file that is NOT one of the managed agents
        std::fs::write(agents_dir.join("custom-agent.md"), "user content").unwrap();

        let existing = super::collect_existing_paths(&claude_dir);
        assert!(
            existing.is_empty(),
            "non-managed files should not be detected"
        );
    }

    #[test]
    fn test_install_variant_has_force_flag() {
        // Verify that clap parses --force correctly by constructing the enum
        use clap::Parser;

        #[derive(clap::Parser)]
        struct TestCli {
            #[command(subcommand)]
            cmd: super::LoopCommand,
        }

        let cli = TestCli::parse_from(["test", "install", "--force"]);
        match cli.cmd {
            super::LoopCommand::Install { force, project } => {
                assert!(force, "--force should be true");
                assert!(!project, "--project should default to false");
            }
            _ => panic!("expected Install variant"),
        }

        let cli2 = TestCli::parse_from(["test", "install"]);
        match cli2.cmd {
            super::LoopCommand::Install { force, .. } => {
                assert!(!force, "--force should default to false");
            }
            _ => panic!("expected Install variant"),
        }
    }
```

---

## Behavioral Specification

### Scenario: Fresh install (no existing files)

```
$ great loop install

great.sh Loop -- Installing agent team

[checkmark] 15 agent personas -> ~/.claude/agents/
[checkmark] 5 commands -> ~/.claude/commands/
[checkmark] Agent Teams config -> ~/.claude/teams/loop/
...
```

No prompt is shown. Exit code 0. Identical behavior regardless of `--force`.

### Scenario: Re-install with existing files, interactive TTY, user confirms

```
$ great loop install

great.sh Loop -- Installing agent team

[warning] The following files already exist and will be overwritten:
  ~/.claude/agents/nightingale.md
  ~/.claude/agents/lovelace.md
  ...
  ~/.claude/commands/loop.md
  ...
  ~/.claude/teams/loop/config.json
Overwrite? [y/N] y
[checkmark] 15 agent personas -> ~/.claude/agents/
...
```

All 21 files are written. Exit code 0.

### Scenario: Re-install with existing files, interactive TTY, user declines

```
$ great loop install

great.sh Loop -- Installing agent team

[warning] The following files already exist and will be overwritten:
  ~/.claude/agents/davinci.md
Overwrite? [y/N]
Error: aborted: no files were modified
```

No files are written. Exit code 1 (anyhow `bail!` exits non-zero).

### Scenario: Re-install with existing files, non-interactive (piped stdin)

```
$ echo "y" | great loop install

great.sh Loop -- Installing agent team

[warning] The following files already exist and will be overwritten:
  ~/.claude/agents/davinci.md
[error] stdin is not a terminal -- cannot prompt for confirmation.
[info] Re-run with --force to overwrite: great loop install --force
Error: aborted: no files were modified
```

No files are written. Exit code 1. The piped "y" is ignored because `is_terminal()` returns false before any read attempt.

### Scenario: Re-install with `--force`

```
$ great loop install --force

great.sh Loop -- Installing agent team

[info] (--force: overwriting existing files)
[checkmark] 15 agent personas -> ~/.claude/agents/
...
```

All 21 files are written without prompting. Exit code 0.

### Scenario: `--force` on fresh install

```
$ great loop install --force

great.sh Loop -- Installing agent team

[checkmark] 15 agent personas -> ~/.claude/agents/
...
```

No "(--force: overwriting existing files)" message is printed because no files existed. Exit code 0.

---

## Control Flow Pseudocode

```
run_install(project, force):
    claude_dir = ~/.claude
    create directories: agents/, commands/, teams/loop/

    existing = collect_existing_paths(claude_dir)

    if existing is NOT empty AND NOT force:
        confirmed = confirm_overwrite(existing)
        if NOT confirmed:
            bail!("aborted: no files were modified")

    if force AND existing is NOT empty:
        print "(--force: overwriting existing files)"

    // -- rest of install proceeds unchanged --
    write 15 agent files
    write 5 command files
    write teams config
    handle settings.json (existing non-destructive logic)
    handle --project if passed
    print summary
```

---

## Edge Cases

| Scenario | Handling |
|----------|----------|
| Fresh install (0 existing files) | `collect_existing_paths` returns empty vec. Prompt is skipped entirely. Files are written silently. Same behavior with or without `--force`. |
| Partial install (some files present) | Only the existing files are listed in the prompt. All 21 files are written if the user confirms (both new and existing). The command does not skip writing non-existing files -- it always writes all 21. |
| Full re-install (all 21 exist) | All 21 paths are listed. User must confirm or use `--force`. |
| `--force` with no existing files | No "(--force: overwriting)" message. Proceeds silently. |
| `--force` with existing files | Prints informational message. Overwrites all files. No prompt. |
| Non-TTY stdin, no existing files | Proceeds silently. No prompt needed. |
| Non-TTY stdin, existing files, no `--force` | Prints the list of conflicting files, prints error message with `--force` hint, returns `Ok(false)`, then `bail!`. Exit code 1. No files written. |
| `--force --project` | Both flags work together. `--force` governs the 21 managed files. `--project` writes `.tasks/` as before (no overwrite check on `.tasks/` -- out of scope per task definition). |
| User types "Y" (uppercase) | `to_lowercase()` normalizes to "y". Accepted. |
| User types "yes" | Accepted (case-insensitive). |
| User types "yep", "yeah", "ok" | NOT accepted. Only "y" and "yes" match. Treated as decline. |
| User types " y " (with whitespace) | `.trim()` strips whitespace. Accepted. |
| Empty input (just Enter) | `.trim()` produces "". Does not match "y" or "yes". Treated as decline (default N). |
| EOF on stdin (Ctrl+D) | `read_line` returns `Ok(0)`, `input` is empty. Treated as decline. |
| Broken pipe on stdin | `read_line` returns `Err`. Propagated via `?` with context message. |
| `$HOME` not set | `dirs::home_dir()` returns `None`. Existing `.context()` error fires before `collect_existing_paths` is reached. No change needed. |
| Read-only filesystem | `create_dir_all` fails before the overwrite check is reached. Existing error handling covers this. |
| Concurrent `great loop install` | Two processes both check, both see existing files. If both get confirmation, both write. Last writer wins. All writes produce the same content (compile-time embedded), so the result is correct regardless of ordering. |
| Symlinked managed files | `Path::exists()` follows symlinks. A symlinked agent file is treated as "existing" and listed in the prompt. This is correct -- overwriting a symlink target is destructive. |
| macOS ARM64 / x86_64 | `dirs::home_dir()` resolves to `/Users/<name>`. `IsTerminal` uses `libc::isatty`. No platform-specific behavior needed. |
| Ubuntu / WSL2 | `dirs::home_dir()` resolves to `/home/<name>`. `IsTerminal` uses `libc::isatty`. WSL2 terminals report `is_terminal() == true` correctly. |

---

## Error Handling

| Condition | Message | Exit Code |
|-----------|---------|-----------|
| User declines overwrite (types N / Enter) | `"aborted: no files were modified"` | 1 |
| Non-TTY stdin with existing files, no `--force` | stderr: `"stdin is not a terminal -- cannot prompt for confirmation."` + `"Re-run with --force to overwrite: great loop install --force"` then `"aborted: no files were modified"` | 1 |
| Failed to read from stdin | `"failed to read from stdin"` (anyhow context) | 1 |
| `$HOME` not set | `"could not determine home directory -- is $HOME set?"` (existing) | 1 |
| Filesystem write failure | Existing context messages (e.g., `"failed to write agent file: ..."`) | 1 |

All error messages are actionable. The non-TTY abort message explicitly tells the user what flag to use.

---

## Security Considerations

- **No new dependencies.** `std::io::IsTerminal` is in the standard library (stable since Rust 1.70; project uses rustc 1.89.0).
- **No user input incorporated into file content.** The prompt reads a single line from stdin for "y/N" comparison only. The line is never written to any file, passed to any command, or used in any format string.
- **No new filesystem paths.** The same 21 managed paths are checked and written as before.
- **No secrets or credentials involved.**
- **The `--force` flag is opt-in.** The default behavior is safer than before (prompts instead of silently overwriting).
- **TTY detection uses `std::io::IsTerminal`** (backed by `libc::isatty`), which is the standard Rust approach. No third-party crate needed.

---

## Testing Strategy

### Unit tests (in `src/cli/loop_cmd.rs::tests`)

6 new tests using `tempfile::TempDir` (already in `dev-dependencies`):

| Test | What it asserts |
|------|-----------------|
| `test_collect_existing_paths_empty_dir` | Returns empty vec when no managed files exist |
| `test_collect_existing_paths_partial_install` | Returns only the files that actually exist on disk |
| `test_collect_existing_paths_full_install` | Returns all 21 paths when all managed files exist |
| `test_collect_existing_paths_only_commands` | Detects command files independently of agent files |
| `test_collect_existing_paths_ignores_unknown_files` | Non-managed files in the agents directory are not reported |
| `test_install_variant_has_force_flag` | Clap correctly parses `--force` and defaults it to `false` |

### Integration tests (in `tests/cli_smoke.rs`)

The builder should add these integration tests:

```rust
// -----------------------------------------------------------------------
// Loop install -- overwrite safety
// -----------------------------------------------------------------------

#[test]
fn loop_install_force_flag_accepted() {
    // Verify --force is a valid flag (help output includes it)
    great()
        .args(["loop", "install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--force"));
}

#[test]
fn loop_install_force_fresh_succeeds() {
    // --force on a fresh install (no existing files) should succeed
    // We use a fake HOME to avoid touching the real ~/.claude/
    let dir = TempDir::new().unwrap();
    great()
        .args(["loop", "install", "--force"])
        .env("HOME", dir.path())
        .assert()
        .success();

    // Verify files were written
    assert!(dir.path().join(".claude/agents/nightingale.md").exists());
    assert!(dir.path().join(".claude/commands/loop.md").exists());
    assert!(dir.path().join(".claude/teams/loop/config.json").exists());
}

#[test]
fn loop_install_non_tty_existing_files_aborts() {
    // When stdin is not a TTY and files exist, should abort
    let dir = TempDir::new().unwrap();

    // First install (fresh, should succeed)
    great()
        .args(["loop", "install", "--force"])
        .env("HOME", dir.path())
        .assert()
        .success();

    // Second install without --force, piped stdin (not a TTY)
    great()
        .args(["loop", "install"])
        .env("HOME", dir.path())
        .write_stdin("y\n")  // piped stdin is not a TTY
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn loop_install_force_overwrites_existing() {
    // --force should overwrite existing files without error
    let dir = TempDir::new().unwrap();

    // First install
    great()
        .args(["loop", "install", "--force"])
        .env("HOME", dir.path())
        .assert()
        .success();

    // Modify a file to prove it gets overwritten
    let agent_path = dir.path().join(".claude/agents/nightingale.md");
    std::fs::write(&agent_path, "user customization").unwrap();

    // Second install with --force
    great()
        .args(["loop", "install", "--force"])
        .env("HOME", dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("--force: overwriting existing files"));

    // Verify file was overwritten (no longer contains user customization)
    let content = std::fs::read_to_string(&agent_path).unwrap();
    assert!(
        !content.contains("user customization"),
        "file should have been overwritten"
    );
}
```

### Build gate

```bash
cargo clippy -- -D warnings
cargo test -- loop_cmd
cargo test -- loop_install
```

All must exit 0 with zero warnings.

### Manual verification

1. **Fresh install:** Remove `~/.claude/agents/`, `~/.claude/commands/`, `~/.claude/teams/loop/`. Run `great loop install`. Confirm all files are written with no prompt.

2. **Re-install with decline:** Run `great loop install` again. Confirm the list of existing files is printed. Press Enter (default N). Confirm no files were modified (check mtimes).

3. **Re-install with confirm:** Run `great loop install` again. Type `y` at the prompt. Confirm all files are rewritten.

4. **Non-TTY abort:** Run `echo "y" | great loop install`. Confirm it aborts with the `--force` hint message.

5. **Force flag:** Run `great loop install --force`. Confirm it overwrites without prompting and prints the "(--force: overwriting existing files)" message.

---

## Verification Gate

The builder declares this task complete when:

- [ ] `LoopCommand::Install` has a `force: bool` field with `#[arg(long)]`
- [ ] `great loop install --help` shows the `--force` flag
- [ ] `collect_existing_paths` correctly identifies only the 21 managed files
- [ ] Fresh install (no existing files) proceeds silently without prompting, with or without `--force`
- [ ] Re-install with existing files and interactive TTY shows the file list and `Overwrite? [y/N]` prompt
- [ ] Default answer (Enter) aborts with exit code 1 and no files written
- [ ] Answer "y" or "yes" (case-insensitive) proceeds with full install
- [ ] Non-TTY stdin with existing files aborts with actionable `--force` hint message
- [ ] `--force` skips the prompt and prints "(--force: overwriting existing files)"
- [ ] `cargo clippy -- -D warnings` exits 0
- [ ] `cargo test -- loop_cmd` exits 0 (all existing + 6 new unit tests pass)
- [ ] Integration tests in `tests/cli_smoke.rs` pass
- [ ] `git diff` shows changes only in `src/cli/loop_cmd.rs` and `tests/cli_smoke.rs`
