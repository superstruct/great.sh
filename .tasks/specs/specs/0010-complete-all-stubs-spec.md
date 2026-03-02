# Spec 0010: Complete All Stubs, TODOs, and Gaps

**Source task:** `.tasks/backlog/0010-complete-all-stubs.md`
**Author:** Lovelace (Spec Writer)
**Date:** 2026-02-20
**Status:** ready

---

## Summary

Eleven groups of incomplete work remain in the `great` CLI. This spec provides
code-level implementation details for each group -- function signatures, data
structures, algorithms, error handling, and tests -- so a builder can implement
from this document alone. Every group targets local-only operation; no cloud
backend is required.

---

## New Crate Dependencies

Add to `Cargo.toml` `[dependencies]`:

```toml
toml_edit = "0.22"
semver = "1.0"
```

`toml_edit` is used by GROUP C (format-preserving TOML modification).
`semver` is used by GROUP E (version comparison).

---

## GROUP A: Tool Install Mapping Table

### Files to Modify

- `src/cli/apply.rs`

### Data Structures

Add a new struct and a static lookup function inside `src/cli/apply.rs`:

```rust
/// Describes how to install a CLI tool, possibly differing from a naive
/// `brew install <name>`.
struct ToolInstallSpec {
    /// The command name the user types (e.g. "cdk", "az").
    cli_name: &'static str,
    /// Method used when Homebrew is the primary package manager.
    brew: Option<BrewInstall>,
    /// Method used when npm is available and preferred.
    npm_global: Option<&'static str>,
    /// Curl/shell fallback installer script URL.
    curl_installer: Option<&'static str>,
    /// Snap package name (Linux without Homebrew).
    snap: Option<&'static str>,
    /// pip package name (fallback).
    pip: Option<&'static str>,
}

enum BrewInstall {
    /// Formula name equals the cli_name.
    Default,
    /// Formula name differs (e.g. "azure-cli" for "az").
    Named(&'static str),
}
```

### Mapping Table

```rust
fn special_install_specs() -> &'static [ToolInstallSpec] {
    &[
        ToolInstallSpec {
            cli_name: "cdk",
            brew: None,
            npm_global: Some("aws-cdk"),
            curl_installer: None,
            snap: None,
            pip: None,
        },
        ToolInstallSpec {
            cli_name: "az",
            brew: Some(BrewInstall::Named("azure-cli")),
            npm_global: None,
            curl_installer: Some("https://aka.ms/InstallAzureCLIDeb"),
            snap: None,
            pip: None,
        },
        ToolInstallSpec {
            cli_name: "gcloud",
            brew: Some(BrewInstall::Named("google-cloud-sdk")),
            npm_global: None,
            curl_installer: Some("https://sdk.cloud.google.com"),
            snap: Some("google-cloud-cli"),
            pip: None,
        },
        ToolInstallSpec {
            cli_name: "aws",
            brew: Some(BrewInstall::Named("awscli")),
            npm_global: None,
            curl_installer: Some("https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip"),
            snap: None,
            pip: None,
        },
        ToolInstallSpec {
            cli_name: "pnpm",
            brew: Some(BrewInstall::Default),
            npm_global: Some("pnpm"),
            curl_installer: None,
            snap: None,
            pip: None,
        },
        ToolInstallSpec {
            cli_name: "uv",
            brew: Some(BrewInstall::Default),
            npm_global: None,
            curl_installer: Some("https://astral.sh/uv/install.sh"),
            snap: None,
            pip: Some("uv"),
        },
        ToolInstallSpec {
            cli_name: "starship",
            brew: Some(BrewInstall::Default),
            npm_global: None,
            curl_installer: Some("https://starship.rs/install.sh"),
            snap: None,
            pip: None,
        },
        ToolInstallSpec {
            cli_name: "bw",
            brew: None,
            npm_global: Some("@bitwarden/cli"),
            curl_installer: None,
            snap: Some("bw"),
            pip: None,
        },
    ]
}
```

### Algorithm

Add a function:

```rust
/// Look up the special install spec for a given CLI tool name.
/// Returns None for tools that can be installed via the default
/// `brew install <name>` / `apt install <name>` path.
fn find_install_spec(cli_name: &str) -> Option<&'static ToolInstallSpec> {
    special_install_specs().iter().find(|s| s.cli_name == cli_name)
}
```

Add a function that performs a single tool installation using the spec:

```rust
/// Install a CLI tool using its special install spec.
/// Tries methods in priority order: npm (if npm_global set) -> brew (if
/// available and brew set) -> curl -> snap -> pip -> generic fallback.
///
/// If `dry_run` is true, returns a string describing what *would* happen.
fn install_with_spec(
    spec: &ToolInstallSpec,
    version: Option<&str>,
    dry_run: bool,
    managers: &[Box<dyn PackageManager>],
) -> Result<String>
```

Logic:
1. If `spec.npm_global` is set and `npm` is available: try `npm install -g <package>`.
2. If `spec.brew` is set and Homebrew is in `managers`: try `brew install <formula>`.
3. If `spec.curl_installer` is set: run `curl -fsSL <url> | bash` (with `sh -s -- -y` for starship, `| sudo bash` for az).
4. If `spec.snap` is set and `snap` is available: `sudo snap install <name> --classic`.
5. If `spec.pip` is set and `pip3`/`pip` is available: `pip install <name>`.
6. If none work, return an error with the message: "Could not install {cli_name}. Try manually: {first available method instruction}".

### Changes to Existing Code

**CLI tool install loop** (apply.rs, around line 216): Before calling the
generic `mgr.install(name, version_opt)` loop, check `find_install_spec(name)`.
If found, call `install_with_spec()` instead.

```rust
// In the for (name, version) in cli_tools loop:
if let Some(spec) = find_install_spec(name) {
    if args.dry_run {
        let method = describe_install_method(spec);
        output::info(&format!("  {} {} -- would install via {}", name, version, method));
        continue;
    }
    match install_with_spec(spec, version_opt, false, &managers) {
        Ok(method) => output::success(&format!("  {} -- installed via {}", name, method)),
        Err(e) => output::error(&format!("  {} -- failed: {}", name, e)),
    }
    continue;
}
// ...existing generic loop follows
```

**Bitwarden CLI install** (apply.rs, around line 350): Replace the current
block with a call to `find_install_spec("bw")` and `install_with_spec()`. This
ensures npm is tried first (since `npm_global` is set in the spec for `bw`).

```rust
if let Some(spec) = find_install_spec("bw") {
    match install_with_spec(spec, None, args.dry_run, &managers) {
        Ok(method) => output::success(&format!("  bw -- installed via {}", method)),
        Err(e) => output::error(&format!("  bw -- could not install: {}", e)),
    }
}
```

Add a helper for dry-run descriptions:

```rust
fn describe_install_method(spec: &ToolInstallSpec) -> &'static str {
    if spec.npm_global.is_some() { return "npm (global)"; }
    if spec.brew.is_some() { return "homebrew"; }
    if spec.curl_installer.is_some() { return "curl installer"; }
    if spec.snap.is_some() { return "snap"; }
    if spec.pip.is_some() { return "pip"; }
    "package manager"
}
```

### Error Handling

Each install method returns `Result<()>`. On failure, the next method is tried.
If all methods fail, `install_with_spec` returns `anyhow::bail!("no install
method succeeded for {}", spec.cli_name)`.

### Tests

Add unit tests in `src/cli/apply.rs` `#[cfg(test)]`:

```rust
#[test]
fn test_find_install_spec_known() {
    assert!(find_install_spec("cdk").is_some());
    assert!(find_install_spec("az").is_some());
    assert!(find_install_spec("bw").is_some());
}

#[test]
fn test_find_install_spec_unknown() {
    assert!(find_install_spec("ripgrep").is_none());
    assert!(find_install_spec("bat").is_none());
}

#[test]
fn test_describe_install_method_npm_first() {
    let spec = find_install_spec("cdk").unwrap();
    assert_eq!(describe_install_method(spec), "npm (global)");
}

#[test]
fn test_describe_install_method_brew_for_az() {
    let spec = find_install_spec("az").unwrap();
    assert_eq!(describe_install_method(spec), "homebrew");
}

#[test]
fn test_all_specs_have_at_least_one_method() {
    for spec in special_install_specs() {
        assert!(
            spec.brew.is_some()
                || spec.npm_global.is_some()
                || spec.curl_installer.is_some()
                || spec.snap.is_some()
                || spec.pip.is_some(),
            "{} has no install method",
            spec.cli_name
        );
    }
}
```

---

## GROUP B: Starship Configuration

### Files to Modify

- `src/cli/apply.rs` (the TODO block at lines 368-375)

### Data Structures

None new. Uses existing `platform::PlatformInfo` for shell detection.

### Algorithm

Add a function in `src/cli/apply.rs`:

```rust
/// Configure Starship prompt after installation.
///
/// 1. Generate `~/.config/starship.toml` with a great.sh preset if absent.
/// 2. Detect the user's shell from `$SHELL` and append the init line to the
///    appropriate profile file, guarding against duplicates.
fn configure_starship(dry_run: bool) -> Result<()>
```

Steps:

1. **Generate starship.toml**:
   - `let config_path = dirs::config_dir().map(|d| d.join("starship.toml"))`.
   - If `config_path` exists, skip (print "starship.toml already exists").
   - Otherwise, write a minimal great.sh-branded preset:
     ```toml
     # great.sh Starship preset
     format = "$all"

     [character]
     success_symbol = "[>](bold green)"
     error_symbol = "[>](bold red)"
     ```
   - If `dry_run`, print what would be written instead.

2. **Detect shell and profile file**:
   - Read `$SHELL` env var. Extract the shell name (last path component).
   - Map to profile path and init line:
     - `bash` -> `~/.bashrc`, `eval "$(starship init bash)"`
     - `zsh` -> `~/.zshrc`, `eval "$(starship init zsh)"`
     - `fish` -> `~/.config/fish/config.fish`, `starship init fish | source`
   - If unknown shell, print a warning with manual instructions and return Ok.

3. **Append init line**:
   - Read the profile file contents (or empty string if file does not exist).
   - If the file already contains the init line substring (`starship init`), skip.
   - Otherwise append `\n# Added by great.sh\n{init_line}\n`.
   - If `dry_run`, print what would be appended.

### Integration Point

In `apply.rs`, replace the TODO block at lines 368-375 with:

```rust
// 5c. Configure Starship prompt if starship was just installed or is in CLI tools
if let Some(tools) = &cfg.tools {
    if let Some(cli) = &tools.cli {
        if cli.contains_key("starship") && command_exists("starship") {
            output::header("Starship Configuration");
            if let Err(e) = configure_starship(args.dry_run) {
                output::warning(&format!("Starship configuration failed: {}", e));
            }
            println!();
        }
    }
}
```

### Error Handling

- `dirs::config_dir()` returns `None` -> print warning, skip config generation.
- Profile file write fails -> propagate with context message.
- Shell detection fails -> print manual instructions, do not error.

### Tests

```rust
#[test]
fn test_starship_init_line_detection() {
    // Verify the duplicate guard logic
    let profile_with_init = "some stuff\neval \"$(starship init bash)\"\nmore stuff";
    assert!(profile_with_init.contains("starship init"));

    let profile_without = "some stuff\nexport PATH=foo\n";
    assert!(!profile_without.contains("starship init"));
}
```

Integration test in `tests/cli_smoke.rs`:
- `great apply --dry-run` with a fixture containing `starship = "latest"` in
  `[tools.cli]` should mention "Starship Configuration" in output (if starship
  binary happens to be present on the test machine; otherwise it is skipped).

---

## GROUP C: MCP Add Command

### Files to Modify

- `src/cli/mcp.rs` -- `run_add()` function (lines 110-145)

### New Dependency

`toml_edit = "0.22"` in `Cargo.toml` (already listed above).

### Algorithm

Replace the body of `run_add()`:

```rust
fn run_add(name: &str) -> Result<()> {
    output::header(&format!("Adding MCP server: {}", name));

    // 1. Find great.toml
    let config_path = match config::discover_config() {
        Ok(p) => p,
        Err(_) => {
            output::error("No great.toml found. Run `great init` first.");
            return Ok(());
        }
    };

    // 2. Read raw text for format-preserving edit
    let raw = std::fs::read_to_string(&config_path)
        .context("failed to read great.toml")?;

    // 3. Parse with toml_edit for format preservation
    let mut doc = raw.parse::<toml_edit::DocumentMut>()
        .context("failed to parse great.toml")?;

    // 4. Check for duplicates
    if let Some(mcp_table) = doc.get("mcp") {
        if let Some(table) = mcp_table.as_table() {
            if table.contains_key(name) {
                output::warning(&format!(
                    "MCP server '{}' is already in great.toml", name
                ));
                return Ok(());
            }
        }
    }

    // 5. Build the new entry
    let mut entry = toml_edit::Table::new();
    entry.insert("command", toml_edit::value("npx"));
    let mut args_array = toml_edit::Array::new();
    args_array.push("-y");
    args_array.push(format!("@modelcontextprotocol/server-{}", name));
    entry.insert("args", toml_edit::value(args_array));

    // 6. Ensure [mcp] table exists
    if !doc.contains_key("mcp") {
        doc.insert("mcp", toml_edit::Item::Table(toml_edit::Table::new()));
    }
    let mcp_table = doc["mcp"].as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("'mcp' key in great.toml is not a table"))?;

    mcp_table.insert(name, toml_edit::Item::Table(entry));

    // 7. Write back
    std::fs::write(&config_path, doc.to_string())
        .context("failed to write great.toml")?;

    output::success(&format!("Added [mcp.{}] to {}", name, config_path.display()));
    output::info("Run `great apply` to configure it.");
    Ok(())
}
```

### Error Handling

- File not found: handled by `discover_config()`.
- TOML parse error: propagated with context.
- Duplicate entry: warn and return Ok (not an error).
- Write failure: propagated with context.

### Tests

Integration test in `tests/`:

```rust
#[test]
fn mcp_add_creates_entry() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("great.toml");
    std::fs::write(&config_path, "[project]\nname = \"test\"\n").unwrap();

    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .args(["mcp", "add", "postgres"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Added [mcp.postgres]"));

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[mcp.postgres]"));
    assert!(content.contains("@modelcontextprotocol/server-postgres"));

    // Verify the file still parses
    let _: toml::Value = toml::from_str(&content).unwrap();
}

#[test]
fn mcp_add_duplicate_warns() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("great.toml");
    std::fs::write(&config_path, "[project]\nname = \"test\"\n\n[mcp.postgres]\ncommand = \"npx\"\n").unwrap();

    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .args(["mcp", "add", "postgres"])
        .assert()
        .success()
        .stderr(predicate::str::contains("already in great.toml"));
}
```

---

## GROUP D: Doctor --fix

### Files to Modify

- `src/cli/doctor.rs`

### Data Structures

Extend `DiagnosticResult` with a fix log:

```rust
struct DiagnosticResult {
    checks_passed: usize,
    checks_warned: usize,
    checks_failed: usize,
    fixes_attempted: usize,
    fixes_succeeded: usize,
}
```

Add a struct to track fixable issues:

```rust
struct FixableIssue {
    description: String,
    fix: FixAction,
}

enum FixAction {
    /// Install a tool via available package managers.
    InstallTool { name: String, install_hint: String },
    /// Install Homebrew.
    InstallHomebrew,
    /// Create a directory.
    CreateDir { path: std::path::PathBuf },
    /// Manual action required -- print instructions only.
    Manual { instructions: String },
}
```

### Algorithm

Modify `run()`:

```rust
pub fn run(args: Args) -> Result<()> {
    // Run all checks first, collecting fixable issues
    let mut result = DiagnosticResult { /* ... */ };
    let mut fixable: Vec<FixableIssue> = Vec::new();

    check_platform(&mut result, &mut fixable);
    check_essential_tools(&mut result, &mut fixable);
    check_ai_agents(&mut result, &mut fixable);
    check_config(&mut result, &mut fixable);
    check_shell(&mut result, &mut fixable);

    // Print summary
    // ...existing summary code...

    // If --fix, attempt fixes
    if args.fix {
        if fixable.is_empty() {
            output::success("Nothing to fix.");
            return Ok(());
        }

        println!();
        output::header("Attempting fixes");

        for issue in &fixable {
            match &issue.fix {
                FixAction::InstallTool { name, .. } => {
                    output::info(&format!("Fixing: {}", issue.description));
                    let managers = package_manager::available_managers();
                    // Try special install spec first
                    if let Some(spec) = crate::cli::apply::find_install_spec(name) {
                        match crate::cli::apply::install_with_spec(spec, None, false, &managers) {
                            Ok(method) => {
                                result.fixes_succeeded += 1;
                                output::success(&format!("  Installed {} via {}", name, method));
                            }
                            Err(e) => {
                                output::error(&format!("  Failed to install {}: {}", name, e));
                            }
                        }
                    } else {
                        // Generic install
                        let mut installed = false;
                        for mgr in &managers {
                            if mgr.install(name, None).is_ok() {
                                result.fixes_succeeded += 1;
                                output::success(&format!("  Installed {} via {}", name, mgr.name()));
                                installed = true;
                                break;
                            }
                        }
                        if !installed {
                            output::error(&format!("  Could not install {}", name));
                        }
                    }
                    result.fixes_attempted += 1;
                }
                FixAction::InstallHomebrew => {
                    output::info("Fixing: Installing Homebrew...");
                    // Same Homebrew install logic as apply.rs step 2b
                    let status = std::process::Command::new("bash")
                        .args(["-c", "NONINTERACTIVE=1 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""])
                        .status();
                    result.fixes_attempted += 1;
                    match status {
                        Ok(s) if s.success() => {
                            result.fixes_succeeded += 1;
                            output::success("  Homebrew installed");
                        }
                        _ => output::error("  Homebrew installation failed"),
                    }
                }
                FixAction::CreateDir { path } => {
                    output::info(&format!("Fixing: Creating {}", path.display()));
                    result.fixes_attempted += 1;
                    match std::fs::create_dir_all(path) {
                        Ok(()) => {
                            result.fixes_succeeded += 1;
                            output::success(&format!("  Created {}", path.display()));
                        }
                        Err(e) => output::error(&format!("  Failed: {}", e)),
                    }
                }
                FixAction::Manual { instructions } => {
                    output::warning(&format!("Manual fix required: {}", instructions));
                }
            }
        }

        // Print fix summary
        println!();
        output::info(&format!(
            "Fixes: {} attempted, {} succeeded",
            result.fixes_attempted, result.fixes_succeeded
        ));

        // Re-run checks after fixes
        if result.fixes_succeeded > 0 {
            println!();
            output::header("Re-checking...");
            let mut recheck = DiagnosticResult::default();
            check_platform(&mut recheck, &mut Vec::new());
            check_essential_tools(&mut recheck, &mut Vec::new());
            check_ai_agents(&mut recheck, &mut Vec::new());
            check_config(&mut recheck, &mut Vec::new());
            check_shell(&mut recheck, &mut Vec::new());
            println!();
            output::info(&format!(
                "After fixes: {} passed, {} warnings, {} errors",
                recheck.checks_passed, recheck.checks_warned, recheck.checks_failed
            ));
        }
    }
    Ok(())
}
```

### Changes to Check Functions

Each `check_*` function gains a second parameter `fixable: &mut Vec<FixableIssue>`.

Example for `check_essential_tools`:

When a tool is missing, push a `FixableIssue`:

```rust
fn check_essential_tools(result: &mut DiagnosticResult, fixable: &mut Vec<FixableIssue>) {
    // ...existing check logic...
    // When a tool is not found:
    fail(result, &format!("{}: not found -- install: {}", name, install_hint));
    fixable.push(FixableIssue {
        description: format!("{} not installed", name),
        fix: FixAction::InstallTool {
            name: cmd.to_string(),
            install_hint: install_hint.to_string(),
        },
    });
}
```

For API keys (in `check_ai_agents`), push `FixAction::Manual`:

```rust
fixable.push(FixableIssue {
    description: format!("{} not set", name),
    fix: FixAction::Manual {
        instructions: format!("export {}=<your-key>", key),
    },
});
```

For missing Homebrew (in `check_platform`), push `FixAction::InstallHomebrew`.

### Visibility

To allow `doctor.rs` to call `find_install_spec` and `install_with_spec`, make
those two functions `pub(crate)` in `apply.rs` and change the `ToolInstallSpec`
and `BrewInstall` types to `pub(crate)` as well.

### Error Handling

Each fix attempt is wrapped in its own error handling. A failed fix does not
abort the remaining fixes. The function always returns `Ok(())`.

### Tests

Integration test:

```rust
#[test]
fn doctor_fix_nothing_to_fix() {
    Command::cargo_bin("great").unwrap()
        .args(["doctor", "--fix"])
        .assert()
        .success();
    // Output depends on system state. At minimum, should not panic.
}
```

---

## GROUP E: Update Command

### Files to Modify

- `src/cli/update.rs` (full rewrite)

### Data Structures

```rust
/// A GitHub release as returned by the Releases API.
#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(serde::Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}
```

### Algorithm

```rust
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASES_URL: &str = "https://api.github.com/repos/great-sh/great/releases/latest";

pub fn run(args: Args) -> Result<()> {
    output::header("great update");
    println!();
    output::info(&format!("Current version: {}", CURRENT_VERSION));

    if args.check {
        return check_for_update().await;  // Note: see runtime section below
    }

    perform_update().await
}
```

**Note on async:** The project already depends on `tokio` with `features = ["full"]`.
However, `main()` is currently synchronous. Two options:

**Option A (recommended):** Keep `main()` sync; use `tokio::runtime::Runtime`
inside `update.rs` only:

```rust
fn check_for_update() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .context("failed to create async runtime")?;
    rt.block_on(async { check_for_update_async().await })
}

async fn check_for_update_async() -> Result<()> {
    output::info("Checking for updates...");

    let client = reqwest::Client::builder()
        .user_agent("great-sh-cli")
        .build()
        .context("failed to create HTTP client")?;

    let release: GithubRelease = client
        .get(RELEASES_URL)
        .send()
        .await
        .context("failed to reach GitHub -- check your internet connection")?
        .error_for_status()
        .context("GitHub API returned an error (possible rate limit)")?
        .json()
        .await
        .context("failed to parse GitHub response")?;

    let remote_tag = release.tag_name.trim_start_matches('v');
    let remote_ver = semver::Version::parse(remote_tag)
        .context(format!("invalid remote version: {}", release.tag_name))?;
    let local_ver = semver::Version::parse(CURRENT_VERSION)
        .context("invalid local version")?;

    if remote_ver > local_ver {
        output::warning(&format!(
            "New version available: v{} (current: v{})",
            remote_ver, local_ver
        ));
        output::info(&format!("Release: {}", release.html_url));
        output::info("Run `great update` to install.");
    } else {
        output::success(&format!("Up to date (v{})", CURRENT_VERSION));
    }

    Ok(())
}
```

**Self-update logic** (`perform_update`):

```rust
async fn perform_update_async() -> Result<()> {
    // 1. Fetch latest release (same as check)
    // 2. Compare versions; if up to date, print and return
    // 3. Determine asset name from platform:
    //    - macOS aarch64: "great-macos-arm64"
    //    - macOS x86_64: "great-macos-intel"
    //    - Linux x86_64: "great-linux-x86_64"
    //    - Linux aarch64: "great-linux-arm64"
    let asset_name = platform_asset_name()?;

    // 4. Find the matching asset in release.assets
    let asset = release.assets.iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| anyhow::anyhow!(
            "no release binary found for this platform ({})", asset_name
        ))?;

    // 5. Download to a temp file
    output::info(&format!("Downloading {}...", asset.name));
    let bytes = client.get(&asset.browser_download_url)
        .send().await?
        .error_for_status()?
        .bytes().await?;

    // 6. Get current executable path
    let current_exe = std::env::current_exe()
        .context("could not determine current executable path")?;

    // 7. Atomic replacement:
    //    a. Write new binary to a temp file in the same directory
    //    b. Set executable permissions (chmod +x)
    //    c. Rename current to current.bak
    //    d. Rename temp to current
    //    e. Remove .bak
    let tmp_path = current_exe.with_extension("new");
    std::fs::write(&tmp_path, &bytes)
        .context("failed to write new binary")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))
            .context("failed to set executable permission")?;
    }

    let bak_path = current_exe.with_extension("bak");
    std::fs::rename(&current_exe, &bak_path)
        .context("failed to back up current binary (do you have write permission?)")?;
    std::fs::rename(&tmp_path, &current_exe)
        .context("failed to replace binary")?;
    let _ = std::fs::remove_file(&bak_path); // best-effort cleanup

    output::success(&format!("Updated to v{}", remote_ver));
    Ok(())
}
```

Helper:

```rust
fn platform_asset_name() -> Result<String> {
    let info = platform::detect_platform_info();
    let name = match (&info.platform, info.platform.arch()) {
        (platform::Platform::MacOS { .. }, platform::Architecture::Aarch64) => "great-macos-arm64",
        (platform::Platform::MacOS { .. }, platform::Architecture::X86_64) => "great-macos-intel",
        (platform::Platform::Linux { .. } | platform::Platform::Wsl { .. }, platform::Architecture::X86_64) => "great-linux-x86_64",
        (platform::Platform::Linux { .. } | platform::Platform::Wsl { .. }, platform::Architecture::Aarch64) => "great-linux-arm64",
        _ => anyhow::bail!("unsupported platform for self-update"),
    };
    Ok(name.to_string())
}
```

### Error Handling

- Network failure: "failed to reach GitHub -- check your internet connection"
- Rate limit: "GitHub API returned an error (possible rate limit)"
- No matching asset: "no release binary found for this platform"
- Permission error on binary replacement: "failed to back up current binary (do you have write permission?)"
- Invalid semver: "invalid remote version: {tag}"

### Tests

Unit tests in `src/cli/update.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version_is_valid_semver() {
        let ver = semver::Version::parse(CURRENT_VERSION);
        assert!(ver.is_ok(), "CARGO_PKG_VERSION must be valid semver");
    }

    #[test]
    fn test_platform_asset_name_not_empty() {
        // Should succeed on any supported dev machine
        let name = platform_asset_name();
        assert!(name.is_ok());
        assert!(!name.unwrap().is_empty());
    }
}
```

Integration test: Since we cannot actually hit GitHub in CI, test only the
`--check` path with a mock or test that it fails gracefully:

```rust
#[test]
fn update_check_does_not_panic() {
    // This may fail with a network error in CI, but should not panic
    let result = Command::cargo_bin("great").unwrap()
        .args(["update", "--check"])
        .assert();
    // Accept either success or failure, but stderr should not be empty
    // and should contain version info
    result.stderr(predicate::str::contains("Current version"));
}
```

---

## GROUP F: Vault Completion (Local-Only)

### Files to Modify

- `src/cli/vault.rs` -- `run_login()`, `run_unlock()`, `run_import()`

### Algorithm

**`run_login()` -- Verify keychain access:**

```rust
fn run_login() -> Result<()> {
    output::header("Vault Login");
    println!();

    // Try to access the keychain by storing and retrieving a test value
    let providers = vault::available_providers();
    let keychain = providers.iter().find(|p| p.name() == "keychain");

    match keychain {
        Some(provider) => {
            // Test write
            let test_key = "great-sh-login-test";
            let test_val = "ok";
            match provider.set(test_key, test_val) {
                Ok(()) => {
                    // Verify read-back
                    match provider.get(test_key) {
                        Ok(Some(v)) if v == test_val => {
                            output::success("System keychain is accessible and working.");
                            output::info("Secrets can be stored with: great vault set <KEY> <VALUE>");
                        }
                        Ok(_) => {
                            output::warning("Keychain write succeeded but read-back failed.");
                        }
                        Err(e) => {
                            output::error(&format!("Keychain read failed: {}", e));
                        }
                    }
                    // Clean up test key (best effort)
                    // Note: KeychainProvider doesn't have delete, so leave it.
                    // The key is harmless.
                }
                Err(e) => {
                    output::error(&format!("Cannot write to system keychain: {}", e));
                    output::info("On macOS: ensure Keychain Access is unlocked.");
                    output::info("On Linux: ensure secret-tool is installed (libsecret).");
                }
            }
        }
        None => {
            output::error("No system keychain available.");
            output::info("On macOS: Keychain Access should be available by default.");
            output::info("On Linux: install libsecret-tools (apt install libsecret-tools).");
        }
    }

    Ok(())
}
```

**`run_unlock()` -- No-op for local keychain:**

```rust
fn run_unlock() -> Result<()> {
    output::header("Vault Unlock");
    println!();
    output::success("Local keychain vault is always available -- no unlock needed.");
    output::info("Secrets are stored in the system keychain (macOS Keychain / Linux Secret Service).");
    output::info("Use `great vault set <KEY> <VALUE>` to store secrets.");
    Ok(())
}
```

**`run_import()` -- Actually import secrets:**

```rust
fn run_import(path: &str) -> Result<()> {
    // Check if path is a provider name
    if let Some(provider) = vault::get_provider(path) {
        return import_from_provider(provider.as_ref());
    }

    // Otherwise treat as a file path (.env format)
    import_from_file(path)
}

fn import_from_provider(source: &dyn vault::SecretProvider) -> Result<()> {
    output::header(&format!("Importing from {}", source.name()));

    if !source.is_available() {
        output::error(&format!("{} is not available on this system", source.name()));
        return Ok(());
    }

    let keys = match source.list(None) {
        Ok(k) => k,
        Err(e) => {
            output::error(&format!("Failed to list secrets: {}", e));
            return Ok(());
        }
    };

    if keys.is_empty() {
        output::warning("No secrets found to import.");
        return Ok(());
    }

    output::info(&format!("Found {} secrets to import", keys.len()));

    // Get a writable provider (keychain preferred)
    let providers = vault::available_providers();
    let target = providers.iter()
        .find(|p| p.name() != "env" && p.name() != source.name())
        .or_else(|| providers.iter().find(|p| p.name() != "env"));

    let target = match target {
        Some(t) => t,
        None => {
            output::error("No writable vault provider available.");
            return Ok(());
        }
    };

    let mut imported = 0;
    let mut failed = 0;

    for key in &keys {
        match source.get(key) {
            Ok(Some(value)) => {
                match target.set(key, &value) {
                    Ok(()) => {
                        output::success(&format!("  {} -- imported to {}", key, target.name()));
                        imported += 1;
                    }
                    Err(e) => {
                        output::error(&format!("  {} -- failed to store: {}", key, e));
                        failed += 1;
                    }
                }
            }
            Ok(None) => {
                output::warning(&format!("  {} -- no value (skipped)", key));
            }
            Err(e) => {
                output::error(&format!("  {} -- failed to read: {}", key, e));
                failed += 1;
            }
        }
    }

    println!();
    output::info(&format!("Imported: {}, Failed: {}", imported, failed));
    Ok(())
}

fn import_from_file(path: &str) -> Result<()> {
    output::header(&format!("Importing from file: {}", path));

    let file_path = std::path::Path::new(path);
    if !file_path.exists() {
        output::error(&format!("File not found: {}", path));
        output::info("Supported import sources: .env files, provider names (env, keychain, 1password, bitwarden)");
        return Ok(());
    }

    let content = std::fs::read_to_string(file_path)
        .context(format!("failed to read {}", path))?;

    // Parse KEY=VALUE pairs (dotenv format)
    let entries: Vec<(String, String)> = content.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .filter_map(|line| {
            let trimmed = line.trim();
            // Handle export KEY=VALUE
            let trimmed = trimmed.strip_prefix("export ").unwrap_or(trimmed);
            let (key, value) = trimmed.split_once('=')?;
            let key = key.trim().to_string();
            let value = value.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            if key.is_empty() { return None; }
            Some((key, value))
        })
        .collect();

    if entries.is_empty() {
        output::warning("No KEY=VALUE pairs found in file.");
        return Ok(());
    }

    output::info(&format!("Found {} entries", entries.len()));

    // Get a writable provider
    let providers = vault::available_providers();
    let target = providers.iter().find(|p| p.name() != "env");

    let target = match target {
        Some(t) => t,
        None => {
            output::error("No writable vault provider available.");
            output::info("Install libsecret-tools (Linux) or use macOS Keychain.");
            return Ok(());
        }
    };

    let mut imported = 0;
    let mut failed = 0;

    for (key, value) in &entries {
        match target.set(key, value) {
            Ok(()) => {
                output::success(&format!("  {} -- stored in {}", key, target.name()));
                imported += 1;
            }
            Err(e) => {
                output::error(&format!("  {} -- failed: {}", key, e));
                failed += 1;
            }
        }
    }

    println!();
    output::info(&format!("Imported: {}, Failed: {}", imported, failed));
    Ok(())
}
```

### Edge Cases

- Empty `.env` file: "No KEY=VALUE pairs found in file."
- `.env` with comments and blank lines: Filtered out.
- Quoted values (`KEY="value"` or `KEY='value'`): Quotes stripped.
- `export KEY=VALUE`: `export ` prefix stripped.
- Provider lists zero secrets: "No secrets found to import."
- No writable provider: Error with instructions.

### Tests

Unit tests for dotenv parsing (in `src/cli/vault.rs`):

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_dotenv_line() {
        let line = "API_KEY=hunter2";
        let (key, value) = line.split_once('=').unwrap();
        assert_eq!(key.trim(), "API_KEY");
        assert_eq!(value.trim(), "hunter2");
    }

    #[test]
    fn test_parse_dotenv_quoted() {
        let line = "API_KEY=\"hunter2\"";
        let (key, value) = line.split_once('=').unwrap();
        assert_eq!(key.trim(), "API_KEY");
        assert_eq!(value.trim().trim_matches('"'), "hunter2");
    }

    #[test]
    fn test_parse_dotenv_export() {
        let line = "export API_KEY=hunter2";
        let trimmed = line.strip_prefix("export ").unwrap_or(line);
        let (key, value) = trimmed.split_once('=').unwrap();
        assert_eq!(key.trim(), "API_KEY");
        assert_eq!(value.trim(), "hunter2");
    }
}
```

Integration test:

```rust
#[test]
fn vault_import_missing_file() {
    Command::cargo_bin("great").unwrap()
        .args(["vault", "import", "/tmp/nonexistent_great_test_env_12345"])
        .assert()
        .success()
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn vault_login_runs() {
    Command::cargo_bin("great").unwrap()
        .args(["vault", "login"])
        .assert()
        .success();
}

#[test]
fn vault_unlock_runs() {
    Command::cargo_bin("great").unwrap()
        .args(["vault", "unlock"])
        .assert()
        .success()
        .stderr(predicate::str::contains("always available"));
}
```

---

## GROUP G: Sync Pull --apply

### Files to Modify

- `src/cli/sync.rs`

### Data Structures

Add `--apply` flag to `Pull` variant:

```rust
#[derive(Subcommand)]
pub enum SyncCommand {
    /// Push local configuration to sync storage
    Push,
    /// Pull configuration from sync storage
    Pull {
        /// Actually apply the pulled config (overwrites great.toml, creates .bak)
        #[arg(long)]
        apply: bool,
    },
}
```

Update dispatch:

```rust
pub fn run(args: Args) -> Result<()> {
    match args.command {
        SyncCommand::Push => run_push(),
        SyncCommand::Pull { apply } => run_pull(apply),
    }
}
```

### Algorithm

```rust
fn run_pull(apply: bool) -> Result<()> {
    output::header("great sync pull");
    println!();

    output::warning("Cloud sync is not yet available. Loading from local storage.");

    match sync::load_local()? {
        Some(data) => {
            output::info(&format!("Found sync blob: {} bytes", data.len()));

            if apply {
                // Find or default the config path
                let config_path = config::discover_config()
                    .unwrap_or_else(|_| std::path::PathBuf::from("great.toml"));

                // Create backup if file exists
                if config_path.exists() {
                    let bak_path = config_path.with_extension("toml.bak");
                    std::fs::copy(&config_path, &bak_path)
                        .context("failed to create backup of great.toml")?;
                    output::info(&format!("Backed up to {}", bak_path.display()));
                }

                // Write the pulled data
                sync::import_config(&data, &config_path)?;

                // Verify the written file parses
                match config::load(Some(config_path.to_str().unwrap_or("great.toml"))) {
                    Ok(_) => output::success(&format!(
                        "Restored great.toml from sync ({} bytes)",
                        data.len()
                    )),
                    Err(e) => {
                        output::error(&format!("Restored file has parse errors: {}", e));
                        output::info("The backup is at great.toml.bak if you need to revert.");
                    }
                }
            } else {
                // Preview mode: show what would be restored
                output::info("Preview of pulled config:");
                println!();
                match std::str::from_utf8(&data) {
                    Ok(text) => {
                        // Show first 50 lines
                        for (i, line) in text.lines().take(50).enumerate() {
                            println!("  {}", line);
                            if i == 49 {
                                println!("  ... (truncated)");
                            }
                        }
                    }
                    Err(_) => {
                        output::warning("Sync blob is not valid UTF-8 (may be encrypted).");
                    }
                }
                println!();
                output::info("Run `great sync pull --apply` to overwrite great.toml with this config.");
            }
        }
        None => {
            output::warning("No sync data found. Run `great sync push` first.");
        }
    }

    Ok(())
}
```

### Error Handling

- No sync data: Warning with instructions.
- Backup copy fails: Propagate with "failed to create backup of great.toml".
- Write fails: Propagate with import_config's context.
- Restored file has parse errors: Warn but do not fail; point to backup.

### Tests

Integration tests:

```rust
#[test]
fn sync_pull_no_data() {
    // Use a temp HOME so no real sync data is found
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("great").unwrap()
        .env("XDG_DATA_HOME", dir.path().to_str().unwrap())
        .args(["sync", "pull"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No sync data found"));
}

#[test]
fn sync_push_no_config() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .args(["sync", "push"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No great.toml found"));
}
```

---

## GROUP H: Template Update from Registry

### Files to Modify

- `src/cli/template.rs` -- `run_update()` function

### Data Structures

```rust
/// Metadata for a downloaded template from the registry.
#[derive(serde::Deserialize)]
struct RegistryTemplate {
    name: String,
    description: String,
    download_url: String,
}
```

### Algorithm

```rust
const TEMPLATES_RELEASES_URL: &str =
    "https://api.github.com/repos/great-sh/great/contents/templates";

fn run_update() -> Result<()> {
    output::header("Template Update");
    println!();

    let rt = tokio::runtime::Runtime::new()
        .context("failed to create async runtime")?;

    match rt.block_on(fetch_remote_templates()) {
        Ok(templates) => {
            if templates.is_empty() {
                output::info("No new templates found in registry.");
                return Ok(());
            }

            let data_dir = config::data_dir()?;
            let templates_dir = data_dir.join("templates");
            std::fs::create_dir_all(&templates_dir)
                .context("failed to create templates directory")?;

            let mut updated = 0;
            for tmpl in &templates {
                let dest = templates_dir.join(&tmpl.name);
                // Download and save
                match rt.block_on(download_template(&tmpl.download_url)) {
                    Ok(content) => {
                        std::fs::write(&dest, &content)
                            .context(format!("failed to write template {}", tmpl.name))?;
                        output::success(&format!("  {} -- updated", tmpl.name));
                        updated += 1;
                    }
                    Err(e) => {
                        output::error(&format!("  {} -- failed: {}", tmpl.name, e));
                    }
                }
            }

            println!();
            output::info(&format!("{} templates updated.", updated));
        }
        Err(e) => {
            output::error(&format!("Failed to check template registry: {}", e));
            output::info("Check your internet connection and try again.");
        }
    }

    Ok(())
}

async fn fetch_remote_templates() -> Result<Vec<RegistryTemplate>> {
    let client = reqwest::Client::builder()
        .user_agent("great-sh-cli")
        .build()?;

    let entries: Vec<serde_json::Value> = client
        .get(TEMPLATES_RELEASES_URL)
        .send().await?
        .error_for_status()?
        .json().await?;

    let templates = entries.iter()
        .filter(|e| {
            e.get("name")
                .and_then(|n| n.as_str())
                .is_some_and(|n| n.ends_with(".toml"))
        })
        .filter_map(|e| {
            let name = e.get("name")?.as_str()?.to_string();
            let download_url = e.get("download_url")?.as_str()?.to_string();
            Some(RegistryTemplate {
                name: name.clone(),
                description: name.trim_end_matches(".toml").to_string(),
                download_url,
            })
        })
        .collect();

    Ok(templates)
}

async fn download_template(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("great-sh-cli")
        .build()?;
    let text = client.get(url).send().await?.error_for_status()?.text().await?;
    Ok(text)
}
```

Update `run_list()` to include downloaded templates:

```rust
fn run_list() -> Result<()> {
    output::header("Available Templates");
    println!();

    // Built-in
    output::info("Built-in:");
    for tmpl in builtin_templates() {
        output::info(&format!("  {} -- {}", tmpl.name, tmpl.description));
    }

    // Downloaded
    if let Ok(data_dir) = config::data_dir() {
        let templates_dir = data_dir.join("templates");
        if templates_dir.exists() {
            let mut downloaded: Vec<String> = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&templates_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".toml") {
                            downloaded.push(name.to_string());
                        }
                    }
                }
            }
            if !downloaded.is_empty() {
                println!();
                output::info("Downloaded:");
                downloaded.sort();
                for name in &downloaded {
                    output::info(&format!("  {}", name.trim_end_matches(".toml")));
                }
            }
        }
    }

    println!();
    output::info("Apply with: great template apply <name>");
    Ok(())
}
```

Update `run_apply()` to search downloaded templates as a fallback after
built-in templates:

```rust
fn run_apply(name: &str) -> Result<()> {
    let templates = builtin_templates();
    let tmpl_content = match templates.iter().find(|t| t.name == name) {
        Some(t) => t.content.to_string(),
        None => {
            // Check downloaded templates
            match load_downloaded_template(name) {
                Some(content) => content,
                None => {
                    output::error(&format!("Unknown template: {}", name));
                    // ...existing error message...
                    return Ok(());
                }
            }
        }
    };
    // ...rest of apply logic using tmpl_content...
}

fn load_downloaded_template(name: &str) -> Option<String> {
    let data_dir = config::data_dir().ok()?;
    let path = data_dir.join("templates").join(format!("{}.toml", name));
    std::fs::read_to_string(&path).ok()
}
```

### Error Handling

- Network failure: "Failed to check template registry: {error}"
- Rate limit: Handled by `.error_for_status()`.
- No new templates: Informational message.
- Write failure: Propagate with context.

### Tests

```rust
#[test]
fn template_list_shows_builtins() {
    Command::cargo_bin("great").unwrap()
        .args(["template", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("ai-minimal"))
        .stderr(predicate::str::contains("ai-fullstack-ts"));
}

#[test]
fn template_update_handles_network_error_gracefully() {
    // Without network, should print error, not panic
    Command::cargo_bin("great").unwrap()
        .args(["template", "update"])
        .assert()
        .success(); // may succeed with error message
}
```

---

## GROUP I: Dead Code and Safety Cleanup

### Files to Modify

- `src/config/mod.rs`
- `src/error.rs`
- `src/mcp/mod.rs`
- `src/platform/package_manager.rs`
- `src/platform/runtime.rs`
- `src/platform/mod.rs`
- `src/sync/mod.rs`
- `src/vault/mod.rs`
- `src/cli/status.rs`
- `src/cli/doctor.rs`

### Algorithm

Run `cargo clippy -- -W dead_code -W unused_imports` and address each warning.

**Specific changes:**

1. **`src/error.rs`**: `GreatError::Network` uses `reqwest::Error` via `#[from]`.
   Once GROUP E lands, it will be actively used. Until then, add:
   ```rust
   #[allow(dead_code)] // Used by update command (GROUP E)
   Network(#[from] reqwest::Error),
   ```

2. **`src/config/mod.rs`**: The `#[allow(unused_imports)]` on the re-exports
   block (line 8) should be removed once groups consume these types. For items
   still unused after all groups land, remove them.

3. **`src/platform/mod.rs`**: The `#[allow(unused_imports)]` on line 5 should
   be narrowed to specific unused items, or removed if all are now used.

4. **`src/sync/mod.rs`**: `SyncBlob` and `SyncStatus` are defined but never
   constructed. Since GROUP G does not use them either (it uses raw `Vec<u8>`),
   either:
   - Remove them if no future use is planned, or
   - Add `#[allow(dead_code)] // Reserved for cloud sync` to each.

5. **`.unwrap()` audit**: Search all `src/cli/*.rs` for `.unwrap()` outside
   `#[cfg(test)]`:
   - `src/cli/status.rs:191`: `version.lines().next().unwrap_or("")` -- already
     safe, but change to `.unwrap_or_default()` for consistency.
   - `src/cli/doctor.rs:377`: Same pattern -- change to `.unwrap_or_default()`.
   - `src/cli/mcp.rs:49,123,153`: `.unwrap_or_default()` on `to_str()` -- these
     are safe but the pattern `path.to_str().unwrap_or_default()` should be kept
     consistent. No change needed.
   - `src/cli/apply.rs:271`: `serde_json::from_str(&content).unwrap_or_default()`
     -- already handled, but change to use `?` with a context message for better
     error reporting.
   - `src/cli/diff.rs:43`: `.unwrap_or_default()` on `to_str()` -- consistent.
   - `src/cli/apply.rs:460`: `.expect("valid regex")` -- this is acceptable for
     compile-time-constant regexes. No change.
   - `src/mcp/mod.rs:78`: Same `.expect("valid regex")` -- acceptable.

6. **Remaining dead code items**: After all groups are implemented, run
   `cargo clippy` and remove any remaining `#[allow(dead_code)]` annotations
   that are no longer needed. Dead code that is genuinely unused should be
   deleted.

### Acceptance Criteria

- `cargo clippy` produces zero warnings at default lint level.
- Zero `.unwrap()` calls in `src/cli/*.rs` outside `#[cfg(test)]` blocks.
- All `#[allow(dead_code)]` annotations have a comment explaining why.

---

## GROUP J: Integration Test Coverage

### Files to Create / Modify

- `tests/cli_smoke.rs` -- add tests to existing file
- `tests/cli_integration.rs` -- new file for fixture-based tests

### Test List (minimum 12)

**In `tests/cli_smoke.rs` (no filesystem fixtures needed):**

```rust
// Already existing: help, version, init_help, no_args (4 tests)

#[test]
fn status_no_config_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("No great.toml"));
}

#[test]
fn status_json_outputs_json() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("platform"));
}

#[test]
fn doctor_runs_successfully() {
    Command::cargo_bin("great").unwrap()
        .arg("doctor")
        .assert()
        .success()
        .stderr(predicate::str::contains("Summary"));
}

#[test]
fn diff_no_config_shows_error() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stderr(predicate::str::contains("No great.toml"));
}

#[test]
fn template_list_shows_templates() {
    Command::cargo_bin("great").unwrap()
        .args(["template", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("ai-minimal"))
        .stderr(predicate::str::contains("ai-fullstack-ts"));
}

#[test]
fn mcp_list_no_config() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .args(["mcp", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No MCP servers"));
}

#[test]
fn vault_help() {
    Command::cargo_bin("great").unwrap()
        .args(["vault", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage"));
}

#[test]
fn sync_push_no_config() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .args(["sync", "push"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No great.toml"));
}
```

**In `tests/cli_integration.rs` (fixture-based):**

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

/// Create a temp directory with a minimal great.toml fixture.
fn setup_fixture() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let config = r#"
[project]
name = "test-project"

[tools]
node = "22"

[tools.cli]
ripgrep = "latest"
bat = "latest"

[agents.claude]
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[mcp.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "."]

[secrets]
provider = "env"
required = ["ANTHROPIC_API_KEY"]
"#;
    fs::write(dir.path().join("great.toml"), config).unwrap();
    dir
}

#[test]
fn apply_dry_run_with_fixture() {
    let dir = setup_fixture();
    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .args(["apply", "--dry-run"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Dry run mode"))
        .stderr(predicate::str::contains("ripgrep"));
}

#[test]
fn diff_with_fixture() {
    let dir = setup_fixture();
    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success();
    // Output depends on system state -- at minimum should not panic
}

#[test]
fn status_with_fixture() {
    let dir = setup_fixture();
    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("test-project").or(predicate::str::contains("Tools")));
}

#[test]
fn init_template_creates_config() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .args(["init", "--template", "ai-minimal"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Created"));

    let config_path = dir.path().join("great.toml");
    assert!(config_path.exists());
    let content = fs::read_to_string(config_path).unwrap();
    assert!(content.contains("[project]"));
}

#[test]
fn apply_dry_run_no_config_fails() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("great").unwrap()
        .current_dir(dir.path())
        .args(["apply", "--dry-run"])
        .assert()
        .failure(); // Should exit 1 because no great.toml
}
```

**Total:** 4 existing + 8 smoke + 5 fixture = 17 tests (exceeds minimum of 12).

### Test Conventions

- All fixture tests use `tempfile::TempDir` for isolation.
- Tests assert on stderr (since `output::*` prints to stderr).
- `--json` tests assert on stdout.
- Tests must not require network access or elevated privileges.
- All tests should complete in under 2 seconds individually.

---

## GROUP K: Docker Test Rigs

### Files to Create

- `docker-compose.yml` (repo root)
- `docker/test-ubuntu.sh`
- `docker/test-fedora.sh`
- `docker/Dockerfile.ubuntu`
- `docker/Dockerfile.fedora`

### docker-compose.yml

```yaml
# great.sh Docker test rigs
#
# Lightweight Linux containers for CI and local testing.
# These do NOT require KVM and work in GitHub Actions.
#
# Usage:
#   docker compose build                    # Build test images
#   docker compose run ubuntu-test          # Run tests on Ubuntu 22.04
#   docker compose run fedora-test          # Run tests on Fedora 39
#   docker compose up                       # Run all tests in parallel
#
# Heavyweight VM-based containers for full cross-platform testing are
# commented out at the bottom. They require KVM and a beefy host.

services:
  ubuntu-test:
    build:
      context: .
      dockerfile: docker/Dockerfile.ubuntu
    container_name: great-ubuntu-test
    volumes:
      - .:/workspace:ro
      - cargo-cache-ubuntu:/root/.cargo
    working_dir: /build
    command: /workspace/docker/test-ubuntu.sh

  fedora-test:
    build:
      context: .
      dockerfile: docker/Dockerfile.fedora
    container_name: great-fedora-test
    volumes:
      - .:/workspace:ro
      - cargo-cache-fedora:/root/.cargo
    working_dir: /build
    command: /workspace/docker/test-fedora.sh

  # ---------------------------------------------------------------
  # Heavyweight VM-based containers (require KVM, commented out)
  # ---------------------------------------------------------------
  #
  # windows-test:
  #   image: dockurr/windows
  #   container_name: great-windows-test
  #   environment:
  #     VERSION: "11"
  #     USERNAME: "developer"
  #     PASSWORD: "greatsh2025"
  #     DISK_SIZE: "128G"
  #     RAM_SIZE: "4G"
  #     CPU_CORES: "4"
  #   devices:
  #     - /dev/kvm
  #     - /dev/net/tun
  #   cap_add:
  #     - NET_ADMIN
  #   ports:
  #     - "8006:8006"
  #     - "3389:3389/tcp"
  #     - "3389:3389/udp"
  #   volumes:
  #     - windows-storage:/storage
  #     - ./docker:/data
  #
  # macos-test:
  #   image: dockurr/macos
  #   container_name: great-macos-test
  #   environment:
  #     VERSION: "15"
  #     DISK_SIZE: "128G"
  #     RAM_SIZE: "4G"
  #     CPU_CORES: "4"
  #   devices:
  #     - /dev/kvm
  #     - /dev/net/tun
  #   cap_add:
  #     - NET_ADMIN
  #   ports:
  #     - "8007:8006"
  #     - "5900:5900/tcp"
  #   volumes:
  #     - macos-storage:/storage
  #     - ./docker:/shared

volumes:
  cargo-cache-ubuntu:
  cargo-cache-fedora:
  # windows-storage:
  # macos-storage:
```

### docker/Dockerfile.ubuntu

```dockerfile
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build
```

### docker/Dockerfile.fedora

```dockerfile
FROM fedora:39

RUN dnf install -y \
    curl \
    git \
    gcc \
    openssl-devel \
    pkg-config \
    && dnf clean all

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build
```

### docker/test-ubuntu.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "=== great.sh test rig: Ubuntu 22.04 ==="
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"

# Copy source to build directory (avoid modifying the mounted source)
cp -r /workspace/. /build/
cd /build

# Build
echo ""
echo "=== Building ==="
cargo build 2>&1

# Run tests
echo ""
echo "=== Running tests ==="
cargo test 2>&1

# Run clippy
echo ""
echo "=== Running clippy ==="
cargo clippy --all-targets -- -D warnings 2>&1

echo ""
echo "=== All checks passed on Ubuntu 22.04 ==="
```

### docker/test-fedora.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "=== great.sh test rig: Fedora 39 ==="
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"

# Copy source to build directory
cp -r /workspace/. /build/
cd /build

# Build
echo ""
echo "=== Building ==="
cargo build 2>&1

# Run tests
echo ""
echo "=== Running tests ==="
cargo test 2>&1

# Run clippy
echo ""
echo "=== Running clippy ==="
cargo clippy --all-targets -- -D warnings 2>&1

echo ""
echo "=== All checks passed on Fedora 39 ==="
```

Both scripts must be executable: `chmod +x docker/test-ubuntu.sh docker/test-fedora.sh`.

### Cross-Compilation Targets (Documentation)

The existing CI (`.github/workflows/ci.yml` in the reference repo) already
covers cross-compilation. The Docker rigs are for _native_ testing inside each
distro, verifying:

- Build succeeds on Ubuntu 22.04 (glibc 2.35) and Fedora 39 (glibc 2.38).
- All integration tests pass.
- Clippy is clean.
- Platform detection (`detect_linux_distro`) correctly identifies the distro.

### Acceptance Criteria

- `docker compose build` completes without errors.
- `docker compose run ubuntu-test` builds and runs `cargo test` successfully.
- `docker compose run fedora-test` builds and runs `cargo test` successfully.
- No KVM requirement for the lightweight containers.
- Test scripts are idempotent (can be run multiple times).

---

## Build Order

```
Phase 1 (no dependencies):
  GROUP I  -- Dead code cleanup (quick, unblocks clean clippy)
  GROUP J  -- Integration tests (enables CI confidence)
  GROUP A  -- Tool install mapping (unblocks B, D, F-bitwarden)

Phase 2 (depends on A and/or J):
  GROUP C  -- MCP add (no deps, small)
  GROUP G  -- Sync pull --apply (no deps, small)
  GROUP F  -- Vault completion (no deps)
  GROUP E  -- Update command (no deps)
  GROUP B  -- Starship config (depends on A)
  GROUP D  -- Doctor --fix (depends on A)

Phase 3 (depends on E and J):
  GROUP H  -- Template update (depends on E for async/HTTP patterns)
  GROUP K  -- Docker test rigs (depends on J for tests to run)
```

Within each phase, groups are independent and can be implemented in parallel.

---

## Acceptance Criteria (Global)

1. `cargo build` succeeds with zero errors.
2. `cargo clippy --all-targets -- -D warnings` produces zero warnings.
3. `cargo test` passes all tests (at least 17 integration tests + existing unit tests).
4. No `.unwrap()` calls in `src/cli/*.rs` outside `#[cfg(test)]` blocks.
5. No stub messages remain ("not yet available", "not yet implemented") for
   implemented features.
6. `docker compose run ubuntu-test` and `docker compose run fedora-test` pass.
7. All commands handle missing config, missing network, and invalid input
   gracefully with actionable error messages.
