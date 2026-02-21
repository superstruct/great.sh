use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

// Helper to get a Command for the `great` binary
fn great() -> Command {
    Command::cargo_bin("great").expect("binary exists")
}

// -----------------------------------------------------------------------
// Basic CLI
// -----------------------------------------------------------------------

#[test]
fn help_shows_description() {
    great()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("The managed AI dev environment"));
}

#[test]
fn version_shows_semver() {
    great()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn no_args_shows_usage() {
    great()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

// -----------------------------------------------------------------------
// Init
// -----------------------------------------------------------------------

#[test]
fn init_help_shows_initialize() {
    great()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"));
}

// -----------------------------------------------------------------------
// Status
// -----------------------------------------------------------------------

#[test]
fn status_shows_platform() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("Platform:"));
}

#[test]
fn status_warns_no_config() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("No great.toml found"));
}

#[test]
fn status_json_outputs_json() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("platform"));
}

// -----------------------------------------------------------------------
// Doctor
// -----------------------------------------------------------------------

#[test]
fn doctor_runs_diagnostics() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stderr(predicate::str::contains("Platform"))
        .stderr(predicate::str::contains("Essential Tools"))
        .stderr(predicate::str::contains("Summary"));
}

#[test]
#[ignore] // performs real package installs (e.g. Homebrew) — too slow for CI
fn doctor_fix_runs_without_crash() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["doctor", "--fix"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Auto-fix mode"));
}

// -----------------------------------------------------------------------
// Diff
// -----------------------------------------------------------------------

#[test]
fn diff_no_config_shows_error() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stderr(predicate::str::contains("great.toml"));
}

// -----------------------------------------------------------------------
// Template
// -----------------------------------------------------------------------

#[test]
fn template_list_shows_templates() {
    great()
        .args(["template", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("ai-fullstack-ts"))
        .stderr(predicate::str::contains("ai-minimal"));
}

#[test]
fn template_apply_creates_config() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["template", "apply", "ai-minimal"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Created great.toml"));

    assert!(dir.path().join("great.toml").exists());
}

#[test]
fn template_apply_unknown_shows_error() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["template", "apply", "nonexistent-template"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Unknown template"));
}

// -----------------------------------------------------------------------
// MCP
// -----------------------------------------------------------------------

#[test]
fn mcp_list_no_config() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["mcp", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No MCP servers configured"));
}

#[test]
fn mcp_add_no_config() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["mcp", "add", "filesystem"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No great.toml found"));
}

#[test]
fn mcp_add_creates_entry() {
    let dir = TempDir::new().unwrap();
    // First create a config
    std::fs::write(
        dir.path().join("great.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .args(["mcp", "add", "filesystem"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Added MCP server 'filesystem'"));

    // Verify the toml was updated
    let content = std::fs::read_to_string(dir.path().join("great.toml")).unwrap();
    assert!(content.contains("[mcp.filesystem]"));
    assert!(content.contains("@modelcontextprotocol/server-filesystem"));
}

// -----------------------------------------------------------------------
// Vault
// -----------------------------------------------------------------------

#[test]
fn vault_unlock_shows_status() {
    great()
        .args(["vault", "unlock"])
        .assert()
        .success()
        .stderr(predicate::str::contains("keychain"));
}

#[test]
fn vault_import_unknown_provider() {
    // "nonexistent" is not a provider name, so it's treated as a file path and fails
    great()
        .args(["vault", "import", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to open"));
}

#[test]
fn vault_import_dotenv_missing_file() {
    great()
        .args(["vault", "import", "/tmp/great_test_nonexistent_file.env"])
        .assert()
        .failure();
}

// -----------------------------------------------------------------------
// Sync
// -----------------------------------------------------------------------

#[test]
fn sync_push_no_config() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["sync", "push"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No great.toml found"));
}

#[test]
fn sync_pull_no_data() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["sync", "pull"])
        .assert()
        .success()
        .stderr(
            predicate::str::contains("No sync data found")
                .or(predicate::str::contains("Cloud sync")),
        );
}

// -----------------------------------------------------------------------
// Update
// -----------------------------------------------------------------------

#[test]
fn update_check_runs() {
    great()
        .args(["update", "--check"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Current version: 0.1.0"));
}

// -----------------------------------------------------------------------
// Doctor — new sections
// -----------------------------------------------------------------------

#[test]
fn doctor_checks_system_prerequisites() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stderr(predicate::str::contains("System Prerequisites"));
}

#[test]
fn doctor_checks_docker() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stderr(predicate::str::contains("Docker"));
}

// -----------------------------------------------------------------------
// Apply
// -----------------------------------------------------------------------

#[test]
fn apply_dry_run_with_config() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
ripgrep = "latest"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .args(["apply", "--dry-run"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Dry run mode"));
}

#[test]
fn apply_dry_run_shows_prerequisites() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .args(["apply", "--dry-run"])
        .assert()
        .success()
        .stderr(predicate::str::contains("System Prerequisites"));
}

#[test]
fn apply_no_config_fails() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("apply")
        .assert()
        .failure()
        .stderr(predicate::str::contains("great.toml"));
}
