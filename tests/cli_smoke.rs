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
fn diff_no_config_exits_nonzero() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::contains("great.toml"));
}

#[test]
fn diff_satisfied_config_exits_zero() {
    let dir = TempDir::new().unwrap();
    // Declare only tools we know exist on any CI runner
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
git = "latest"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("nothing to do"));
}

#[test]
fn diff_missing_tool_shows_plus() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent_tool_xyz_88888 = "1.0.0"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("nonexistent_tool_xyz_88888"))
        .stdout(predicate::str::contains("great apply"));
}

#[test]
fn diff_disabled_mcp_skipped() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.disabled-server]
command = "nonexistent_cmd_xyz_77777"
enabled = false
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("disabled-server").not())
        .stderr(predicate::str::contains("disabled-server").not());
}

#[test]
fn diff_version_mismatch_shows_tilde() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
git = "99.99.99"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("git"))
        .stdout(predicate::str::contains("want 99.99.99"));
}

#[test]
fn diff_with_custom_config_path() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("custom.toml");
    std::fs::write(
        &config_path,
        r#"
[project]
name = "custom"

[tools.cli]
git = "latest"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .args(["diff", "--config", config_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("custom.toml"));
}

#[test]
fn diff_summary_shows_counts() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent_tool_xyz_99999 = "1.0.0"

[secrets]
required = ["NONEXISTENT_SECRET_XYZ_99999"]
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 to install"))
        .stdout(predicate::str::contains("1 secrets to resolve"))
        .stdout(predicate::str::contains("great apply"));
}

#[test]
fn diff_unresolved_secret_shows_red_minus() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[secrets]
required = ["NONEXISTENT_SECRET_XYZ_88888"]
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("NONEXISTENT_SECRET_XYZ_88888"))
        .stdout(predicate::str::contains("not set in environment"));
}

#[test]
fn diff_mcp_missing_command_counted_as_install() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.fake-server]
command = "nonexistent_mcp_cmd_xyz_77777"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 to install"))
        .stdout(predicate::str::contains("to configure").not())
        .stdout(predicate::str::contains("nonexistent_mcp_cmd_xyz_77777"));
}

#[test]
fn diff_mcp_missing_command_and_missing_tool_install_count() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent_tool_xyz_66666 = "1.0.0"

[mcp.fake-server]
command = "nonexistent_mcp_cmd_xyz_66666"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("2 to install"));
}

#[test]
fn diff_secret_dedup_required_and_ref() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[secrets]
required = ["DEDUP_TEST_SECRET_XYZ_55555"]

[mcp.test-server]
command = "echo"
env = { KEY = "${DEDUP_TEST_SECRET_XYZ_55555}" }
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 secrets to resolve"))
        .stdout(predicate::str::contains("2 secrets").not());
}

#[test]
fn diff_secret_ref_only_no_required_section() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.test-server]
command = "echo"
env = { KEY = "${REFONLY_SECRET_XYZ_44444}" }
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 secrets to resolve"))
        .stdout(predicate::str::contains("REFONLY_SECRET_XYZ_44444"));
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
        .stderr(predicate::str::contains("System Prerequisites"));
}

#[test]
fn doctor_checks_docker() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .stderr(predicate::str::contains("Docker"));
}

#[test]
fn doctor_with_valid_config() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test-project"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .stderr(predicate::str::contains("great.toml: found at"))
        .stderr(predicate::str::contains("great.toml: valid syntax"));
}

#[test]
fn doctor_with_mcp_config_checks_servers() {
    let dir = TempDir::new().unwrap();
    // Write a great.toml with an MCP server whose command exists (ls is universal)
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.test-server]
command = "ls"
args = ["--help"]
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .stderr(predicate::str::contains("MCP Servers"))
        .stderr(predicate::str::contains("test-server"));
}

#[test]
fn doctor_mcp_missing_command_fails() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.broken]
command = "nonexistent_command_xyz_99999"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .stderr(predicate::str::contains("not found on PATH"));
}

#[test]
fn doctor_exits_nonzero_on_failure() {
    let dir = TempDir::new().unwrap();
    // Write a config with an MCP server that has a nonexistent command.
    // This guarantees at least one check_failed.
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.broken]
command = "nonexistent_command_xyz_99999"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Summary"));
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

#[test]
fn apply_dry_run_no_sudo_prompt() {
    let dir = TempDir::new().unwrap();
    // Create a minimal great.toml with a runtime declaration
    std::fs::write(
        dir.path().join("great.toml"),
        "[project]\nname = \"test\"\n\n[tools]\nnode = \"22\"\n",
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

// -----------------------------------------------------------------------
// Statusline
// -----------------------------------------------------------------------

#[test]
fn statusline_empty_stdin_exits_zero() {
    great()
        .arg("statusline")
        .write_stdin("{}")
        .assert()
        .success();
}

#[test]
fn statusline_no_stdin_exits_zero() {
    great().arg("statusline").assert().success();
}

#[test]
fn statusline_prints_one_line() {
    let output = great()
        .arg("statusline")
        .write_stdin("{}")
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "statusline must print exactly one line, got: {:?}",
        lines
    );
}

#[test]
fn statusline_no_color_no_ansi() {
    let output = great()
        .arg("statusline")
        .arg("--no-color")
        .write_stdin("{}")
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains('\x1b'),
        "output must contain no ANSI escapes with --no-color: {:?}",
        stdout
    );
}

#[test]
fn statusline_no_unicode_ascii_only() {
    let output = great()
        .arg("statusline")
        .arg("--no-unicode")
        .arg("--no-color")
        .write_stdin("{}")
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.is_ascii(),
        "output must be ASCII-only with --no-unicode: {:?}",
        stdout
    );
}

#[test]
fn statusline_with_state_file() {
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");
    std::fs::write(
        &state_path,
        r#"{
            "loop_id": "test",
            "started_at": 1740134400,
            "agents": [
                {"id": 1, "name": "nightingale", "status": "done", "updated_at": 1740134450},
                {"id": 2, "name": "lovelace", "status": "running", "updated_at": 1740134480},
                {"id": 3, "name": "socrates", "status": "queued", "updated_at": 1740134400},
                {"id": 4, "name": "humboldt", "status": "error", "updated_at": 1740134400},
                {"id": 5, "name": "davinci", "status": "idle", "updated_at": 1740134400}
            ]
        }"#,
    )
    .unwrap();

    // Create a temporary config that points to our state file
    let config_dir = dir.path().join("config").join("great");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("statusline.toml");
    std::fs::write(
        &config_path,
        format!("state_file = {:?}\n", state_path.to_str().unwrap()),
    )
    .unwrap();

    // Use GREAT_STATUSLINE_CONFIG env var to point to our custom config
    great()
        .arg("statusline")
        .arg("--no-color")
        .env("GREAT_STATUSLINE_CONFIG", &config_path)
        .write_stdin(r#"{"cost_usd": 0.05, "context_tokens": 10000, "context_window": 200000}"#)
        .assert()
        .success();
}

#[test]
fn statusline_help_shows_description() {
    great()
        .args(["statusline", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("statusline"));
}

#[test]
fn statusline_width_override() {
    let output = great()
        .args(["statusline", "--width", "60", "--no-color", "--no-unicode"])
        .write_stdin("{}")
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}

// -----------------------------------------------------------------------
// Statusline — adversarial tests (Turing)
// -----------------------------------------------------------------------

#[test]
fn statusline_no_color_env_var_no_ansi() {
    // Spec: NO_COLOR=1 env var must suppress all ANSI escape sequences
    let output = great()
        .arg("statusline")
        .env("NO_COLOR", "1")
        .write_stdin("{}")
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains('\x1b'),
        "NO_COLOR=1 must suppress ANSI escapes: {:?}",
        stdout
    );
}

#[test]
fn statusline_malformed_json_stdin_exits_zero() {
    // Spec: malformed stdin must not crash; exit 0
    great()
        .arg("statusline")
        .write_stdin("this is not json at all {{{")
        .assert()
        .success();
}

#[test]
fn statusline_binary_garbage_stdin_exits_zero() {
    // Spec: binary garbage on stdin must not crash; exit 0
    let garbage: Vec<u8> = (0..256).map(|i| i as u8).collect();
    great()
        .arg("statusline")
        .write_stdin(garbage)
        .assert()
        .success();
}

#[test]
fn statusline_null_bytes_stdin_exits_zero() {
    // Edge: null bytes embedded in stdin
    great()
        .arg("statusline")
        .write_stdin(b"\x00\x00\x00\x00" as &[u8])
        .assert()
        .success();
}

#[test]
fn statusline_14_agents_medium_width_one_indicator_each() {
    // Spec acceptance criterion: 14-agent state, medium width,
    // one indicator per agent
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 14 agents in mixed statuses
    let agents_json: Vec<String> = [
        ("nightingale", "done"),
        ("lovelace", "done"),
        ("socrates", "done"),
        ("humboldt", "done"),
        ("davinci", "running"),
        ("vonbraun", "running"),
        ("turing", "queued"),
        ("kerckhoffs", "queued"),
        ("rams", "idle"),
        ("nielsen", "idle"),
        ("knuth", "error"),
        ("gutenberg", "idle"),
        ("hopper", "idle"),
        ("deming", "done"),
    ]
    .iter()
    .enumerate()
    .map(|(i, (name, status))| {
        format!(
            r#"{{"id": {}, "name": "{}", "status": "{}", "updated_at": {}}}"#,
            i + 1,
            name,
            status,
            now - 2
        )
    })
    .collect();

    let state_json = format!(
        r#"{{"loop_id": "test", "started_at": {}, "agents": [{}]}}"#,
        now - 120,
        agents_json.join(", ")
    );

    std::fs::write(&state_path, state_json).unwrap();

    let config_dir = dir.path().join("config").join("great");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("statusline.toml");
    std::fs::write(
        &config_path,
        format!("state_file = {:?}\n", state_path.to_str().unwrap()),
    )
    .unwrap();

    // Medium mode: width 100 (>= 80, <= 120)
    let output = great()
        .args(["statusline", "--width", "100", "--no-color", "--no-unicode"])
        .env("GREAT_STATUSLINE_CONFIG", &config_path)
        .write_stdin("{}")
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.trim();

    // In medium mode with ASCII, each agent gets one symbol: *, v, ., X, or -
    // Count status indicator characters in the agent segment
    // The medium renderer produces symbols without spaces between them
    // We expect exactly 14 indicator symbols in sequence
    let agent_symbols: usize = line
        .chars()
        .filter(|c| matches!(c, '*' | 'v' | '.' | 'X' | '-'))
        .count();
    assert!(
        agent_symbols >= 14,
        "expected at least 14 agent indicator symbols in medium mode, got {}: {}",
        agent_symbols,
        line
    );
}

#[test]
fn statusline_zero_agents_valid_state_exits_zero() {
    // Edge: valid state file with zero agents
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    std::fs::write(
        &state_path,
        format!(
            r#"{{"loop_id": "test", "started_at": {}, "agents": []}}"#,
            now - 60
        ),
    )
    .unwrap();

    let config_dir = dir.path().join("config").join("great");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("statusline.toml");
    std::fs::write(
        &config_path,
        format!("state_file = {:?}\n", state_path.to_str().unwrap()),
    )
    .unwrap();

    let output = great()
        .args(["statusline", "--no-color", "--no-unicode"])
        .env("GREAT_STATUSLINE_CONFIG", &config_path)
        .write_stdin("{}")
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("idle"),
        "zero agents should render 'idle': {}",
        stdout
    );
}

#[test]
fn statusline_100_agents_exits_zero() {
    // Edge: 100+ agents should not crash, should truncate at 30
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let agents_json: Vec<String> = (1..=100)
        .map(|i| {
            format!(
                r#"{{"id": {}, "name": "agent{}", "status": "running", "updated_at": {}}}"#,
                i,
                i,
                now - 2
            )
        })
        .collect();

    let state_json = format!(
        r#"{{"loop_id": "test", "started_at": {}, "agents": [{}]}}"#,
        now - 60,
        agents_json.join(", ")
    );

    std::fs::write(&state_path, state_json).unwrap();

    let config_dir = dir.path().join("config").join("great");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("statusline.toml");
    std::fs::write(
        &config_path,
        format!("state_file = {:?}\n", state_path.to_str().unwrap()),
    )
    .unwrap();

    // Wide mode
    let output = great()
        .args(["statusline", "--width", "200", "--no-color", "--no-unicode"])
        .env("GREAT_STATUSLINE_CONFIG", &config_path)
        .write_stdin("{}")
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain truncation indicator
    assert!(
        stdout.contains("..."),
        "100 agents should trigger truncation ellipsis: {}",
        stdout
    );
    // Should still be exactly one line
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "must print exactly one line, got: {:?}",
        lines
    );
}

#[test]
fn statusline_absent_state_file_renders_idle() {
    // Spec: absent state file -> exits 0, renders idle summary
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join("config").join("great");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("statusline.toml");
    // Point state_file to a path that does not exist
    std::fs::write(
        &config_path,
        format!(
            "state_file = {:?}\n",
            dir.path().join("nonexistent-state.json").to_str().unwrap()
        ),
    )
    .unwrap();

    let output = great()
        .args(["statusline", "--no-color", "--no-unicode"])
        .env("GREAT_STATUSLINE_CONFIG", &config_path)
        .write_stdin("{}")
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("idle"),
        "absent state file should render 'idle': {}",
        stdout
    );
}

#[test]
fn statusline_malformed_state_file_renders_err() {
    // Spec: malformed state file -> exits 0, renders "ERR:state"
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");
    std::fs::write(&state_path, "this is not json!!!").unwrap();

    let config_dir = dir.path().join("config").join("great");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("statusline.toml");
    std::fs::write(
        &config_path,
        format!("state_file = {:?}\n", state_path.to_str().unwrap()),
    )
    .unwrap();

    let output = great()
        .args(["statusline", "--no-color", "--no-unicode"])
        .env("GREAT_STATUSLINE_CONFIG", &config_path)
        .write_stdin("{}")
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ERR:state"),
        "malformed state file should render 'ERR:state': {}",
        stdout
    );
}

#[test]
fn statusline_malformed_config_uses_defaults() {
    // Spec: malformed config TOML -> use defaults silently, exit 0
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("statusline.toml");
    std::fs::write(&config_path, "this = [[[is not valid toml!!!").unwrap();

    great()
        .args(["statusline", "--no-color"])
        .env("GREAT_STATUSLINE_CONFIG", &config_path)
        .write_stdin("{}")
        .assert()
        .success();
}

#[test]
fn statusline_unknown_agent_status_exits_zero() {
    // Forward-compatibility: unknown status values should not crash
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    std::fs::write(
        &state_path,
        format!(
            r#"{{"loop_id": "test", "started_at": {}, "agents": [
                {{"id": 1, "name": "future", "status": "hyperspacing", "updated_at": {}}},
                {{"id": 2, "name": "time_traveler", "status": "quantum_superposition", "updated_at": {}}}
            ]}}"#,
            now - 60,
            now - 2,
            now - 2
        ),
    )
    .unwrap();

    let config_dir = dir.path().join("config").join("great");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("statusline.toml");
    std::fs::write(
        &config_path,
        format!("state_file = {:?}\n", state_path.to_str().unwrap()),
    )
    .unwrap();

    great()
        .args(["statusline", "--no-color", "--no-unicode", "--width", "150"])
        .env("GREAT_STATUSLINE_CONFIG", &config_path)
        .write_stdin("{}")
        .assert()
        .success();
}

#[test]
fn statusline_unicode_agent_names_exits_zero() {
    // Edge: agent names with unicode characters should not crash
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    std::fs::write(
        &state_path,
        format!(
            r#"{{"loop_id": "test", "agents": [
                {{"id": 1, "name": "\u00e9l\u00e8ve", "status": "done", "updated_at": {}}},
                {{"id": 2, "name": "\u5f00\u53d1\u8005", "status": "running", "updated_at": {}}}
            ]}}"#,
            now - 2,
            now - 2
        ),
    )
    .unwrap();

    let config_dir = dir.path().join("config").join("great");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("statusline.toml");
    std::fs::write(
        &config_path,
        format!("state_file = {:?}\n", state_path.to_str().unwrap()),
    )
    .unwrap();

    great()
        .args(["statusline", "--no-color"])
        .env("GREAT_STATUSLINE_CONFIG", &config_path)
        .write_stdin("{}")
        .assert()
        .success();
}

#[test]
fn statusline_negative_cost_exits_zero() {
    // Edge: negative cost_usd should not crash (spec says display as-is)
    let output = great()
        .args(["statusline", "--no-color", "--no-unicode", "--width", "150"])
        .write_stdin(r#"{"cost_usd": -0.01, "context_tokens": 1000, "context_window": 200000}"#)
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("$-0.01"),
        "negative cost should display as '$-0.01': {}",
        stdout
    );
}

#[test]
fn statusline_context_window_zero_no_crash() {
    // Edge: context_window = 0 should not cause division by zero
    great()
        .args(["statusline", "--no-color", "--width", "150"])
        .write_stdin(r#"{"context_tokens": 1000, "context_window": 0}"#)
        .assert()
        .success();
}

#[test]
fn statusline_very_large_cost_exits_zero() {
    // Edge: very large cost value
    let output = great()
        .args(["statusline", "--no-color", "--no-unicode", "--width", "150"])
        .write_stdin(r#"{"cost_usd": 999999.99}"#)
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("$999999.99"),
        "large cost should render: {}",
        stdout
    );
}

#[test]
fn statusline_extra_json_fields_stdin_ignored() {
    // Forward-compat: extra fields in stdin JSON should be silently ignored
    great()
        .args(["statusline", "--no-color"])
        .write_stdin(r#"{"cost_usd": 0.05, "unknown_field": true, "nested": {"a": 1}}"#)
        .assert()
        .success();
}

#[test]
fn statusline_state_file_path_with_spaces() {
    // Edge: state file path containing spaces
    let dir = TempDir::new().unwrap();
    let spaced_dir = dir.path().join("path with spaces");
    std::fs::create_dir_all(&spaced_dir).unwrap();
    let state_path = spaced_dir.join("state.json");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    std::fs::write(
        &state_path,
        format!(
            r#"{{"loop_id": "test", "agents": [
                {{"id": 1, "name": "test", "status": "done", "updated_at": {}}}
            ]}}"#,
            now - 2
        ),
    )
    .unwrap();

    let config_dir = dir.path().join("config").join("great");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("statusline.toml");
    std::fs::write(
        &config_path,
        format!("state_file = {:?}\n", state_path.to_str().unwrap()),
    )
    .unwrap();

    great()
        .args(["statusline", "--no-color"])
        .env("GREAT_STATUSLINE_CONFIG", &config_path)
        .write_stdin("{}")
        .assert()
        .success();
}

#[test]
fn statusline_width_zero_exits_zero() {
    // Edge: width=0 should not panic or crash
    great()
        .args(["statusline", "--width", "0", "--no-color"])
        .write_stdin("{}")
        .assert()
        .success();
}

#[test]
fn statusline_width_one_exits_zero() {
    // Edge: width=1 (extreme narrow) should not panic
    great()
        .args(["statusline", "--width", "1", "--no-color"])
        .write_stdin("{}")
        .assert()
        .success();
}

#[test]
fn statusline_width_max_exits_zero() {
    // Edge: max u16 width should not panic
    great()
        .args(["statusline", "--width", "65535", "--no-color"])
        .write_stdin("{}")
        .assert()
        .success();
}

#[test]
fn statusline_all_modes_print_exactly_one_line() {
    // Verify one-line invariant across all three width modes
    for width in &["60", "100", "150"] {
        let output = great()
            .args(["statusline", "--width", width, "--no-color"])
            .write_stdin("{}")
            .output()
            .expect("failed to run");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(
            lines.len(),
            1,
            "width={}: must print exactly one line, got {:?}",
            width,
            lines
        );
    }
}

#[test]
fn statusline_no_color_flag_and_env_combined() {
    // Both --no-color and NO_COLOR=1 set simultaneously
    let output = great()
        .arg("statusline")
        .arg("--no-color")
        .env("NO_COLOR", "1")
        .write_stdin("{}")
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains('\x1b'),
        "combined --no-color + NO_COLOR=1 must suppress ANSI: {:?}",
        stdout
    );
}

// -----------------------------------------------------------------------
// Loop install -- overwrite safety
// -----------------------------------------------------------------------

#[test]
fn loop_install_force_flag_accepted() {
    great()
        .args(["loop", "install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--force"));
}

#[test]
fn loop_install_force_fresh_succeeds() {
    let dir = TempDir::new().unwrap();
    great()
        .args(["loop", "install", "--force"])
        .env("HOME", dir.path())
        .assert()
        .success();

    assert!(dir.path().join(".claude/agents/nightingale.md").exists());
    assert!(dir.path().join(".claude/commands/loop.md").exists());
    assert!(dir.path().join(".claude/teams/loop/config.json").exists());
}

#[test]
fn loop_install_non_tty_existing_files_aborts() {
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
        .write_stdin("y\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn loop_install_force_overwrites_existing() {
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
        .stderr(predicate::str::contains(
            "--force: overwriting existing files",
        ));

    // Verify file was overwritten
    let content = std::fs::read_to_string(&agent_path).unwrap();
    assert!(
        !content.contains("user customization"),
        "file should have been overwritten"
    );
}

// -----------------------------------------------------------------------
// Loop -- help, status, uninstall, and install artefacts
// -----------------------------------------------------------------------

#[test]
fn loop_help_shows_subcommands() {
    great()
        .args(["loop", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("install").and(predicate::str::contains("status")));
}

#[test]
fn loop_status_fresh_home_reports_not_installed() {
    let dir = TempDir::new().unwrap();
    great()
        .args(["loop", "status"])
        .env("HOME", dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn loop_uninstall_fresh_home_is_noop() {
    let dir = TempDir::new().unwrap();
    great()
        .args(["loop", "uninstall"])
        .env("HOME", dir.path())
        .assert()
        .success();
}

#[test]
fn loop_install_force_writes_hook_script() {
    let dir = TempDir::new().unwrap();
    great()
        .args(["loop", "install", "--force"])
        .env("HOME", dir.path())
        .assert()
        .success();

    let hook = dir.path().join(".claude/hooks/great-loop/update-state.sh");
    assert!(hook.exists(), "hook script must be written");
}

#[test]
fn loop_install_force_writes_settings_json() {
    let dir = TempDir::new().unwrap();
    great()
        .args(["loop", "install", "--force"])
        .env("HOME", dir.path())
        .assert()
        .success();

    let settings = dir.path().join(".claude/settings.json");
    assert!(settings.exists(), "settings.json must be created");
    let content = std::fs::read_to_string(&settings).unwrap();
    assert!(
        content.contains("hooks"),
        "settings.json must contain hooks configuration"
    );
    assert!(
        content.contains("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS"),
        "settings.json must contain agent teams env"
    );
}

#[test]
fn statusline_with_state_file_renders_agents() {
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");

    // Use a recent timestamp so agents don't time out
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    std::fs::write(
        &state_path,
        format!(
            r#"{{
            "loop_id": "test",
            "started_at": {},
            "agents": [
                {{"id": 1, "name": "nightingale", "status": "done", "updated_at": {}}},
                {{"id": 2, "name": "lovelace", "status": "running", "updated_at": {}}},
                {{"id": 3, "name": "socrates", "status": "error", "updated_at": {}}}
            ]
        }}"#,
            now - 120,
            now - 5,
            now - 2,
            now - 3,
        ),
    )
    .unwrap();

    let config_dir = dir.path().join("config").join("great");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("statusline.toml");
    std::fs::write(
        &config_path,
        format!("state_file = {:?}\n", state_path.to_str().unwrap()),
    )
    .unwrap();

    // Wide mode (150 cols), no color for easy assertion
    let output = great()
        .args(["statusline", "--width", "150", "--no-color", "--no-unicode"])
        .env("GREAT_STATUSLINE_CONFIG", &config_path)
        .write_stdin(r#"{"cost_usd": 1.50, "context_tokens": 90000, "context_window": 200000}"#)
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain "loop", cost, context info, and agent data
    assert!(
        stdout.contains("loop"),
        "should contain 'loop' label: {}",
        stdout
    );
    assert!(stdout.contains("$1.50"), "should contain cost: {}", stdout);
    assert!(
        stdout.contains("90K/200K"),
        "should contain context: {}",
        stdout
    );
}

// -----------------------------------------------------------------------
// Status -- expanded (task 0004)
// -----------------------------------------------------------------------

#[test]
fn status_with_valid_config_exits_ok() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("Config:"));
}

#[test]
fn status_verbose_accepted() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["status", "--verbose"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Platform:"));
}

#[test]
fn status_verbose_short_flag_accepted() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["status", "-v"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Platform:"));
}

#[test]
fn status_json_valid_json() {
    let dir = TempDir::new().unwrap();
    let output = great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Must parse as valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    // Must contain required top-level keys
    assert!(parsed.get("platform").is_some());
    assert!(parsed.get("arch").is_some());
    assert!(parsed.get("shell").is_some());
    assert!(parsed.get("has_issues").is_some());
    assert!(parsed.get("issues").is_some());
}

#[test]
fn status_json_no_config_still_valid() {
    let dir = TempDir::new().unwrap();
    let output = great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    // config_path should be null
    assert!(parsed.get("config_path").unwrap().is_null());
    // tools/mcp/agents/secrets should be absent or null
    assert!(parsed.get("tools").is_none() || parsed.get("tools").unwrap().is_null());
}

#[test]
fn status_json_with_config_includes_tools() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent-tool-xyz = "latest"
"#,
    )
    .unwrap();

    let output = great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .output()
        .expect("failed to run");

    // --json always exits 0
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    // tools array should exist and contain our tool
    let tools = parsed.get("tools").unwrap().as_array().unwrap();
    assert!(tools.iter().any(|t| t["name"] == "nonexistent-tool-xyz"));
    assert!(tools.iter().any(|t| t["installed"] == false));
    // has_issues should be true (tool not installed)
    assert_eq!(parsed["has_issues"], true);
}

#[test]
fn status_json_with_secrets() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[secrets]
provider = "env"
required = ["GREAT_TEST_SECRET_PRESENT", "GREAT_TEST_SECRET_MISSING"]
"#,
    )
    .unwrap();

    let output = great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .env("GREAT_TEST_SECRET_PRESENT", "value")
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    let secrets = parsed.get("secrets").unwrap().as_array().unwrap();
    let present = secrets
        .iter()
        .find(|s| s["name"] == "GREAT_TEST_SECRET_PRESENT")
        .unwrap();
    assert_eq!(present["is_set"], true);
    let missing = secrets
        .iter()
        .find(|s| s["name"] == "GREAT_TEST_SECRET_MISSING")
        .unwrap();
    assert_eq!(missing["is_set"], false);
}

#[test]
fn status_exit_code_nonzero_missing_tools() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent-tool-xyz-9999 = "latest"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn status_exit_code_nonzero_missing_secrets() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[secrets]
provider = "env"
required = ["GREAT_STATUS_TEST_NONEXISTENT_SECRET"]
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing"));
}

#[test]
fn status_json_always_exits_zero_even_with_issues() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent-tool-xyz-9999 = "latest"

[secrets]
provider = "env"
required = ["GREAT_STATUS_TEST_NONEXISTENT_SECRET"]
"#,
    )
    .unwrap();

    // --json must exit 0 even when there are issues
    great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .success();
}

#[test]
fn status_no_config_exits_zero() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success();
}

#[test]
fn status_verbose_with_config_shows_capabilities() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .args(["status", "--verbose"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Shell:"));
}

// -----------------------------------------------------------------------
// MCP Bridge
// -----------------------------------------------------------------------

#[test]
fn mcp_bridge_help_shows_description() {
    great()
        .args(["mcp-bridge", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MCP bridge server"));
}

#[test]
fn mcp_bridge_unknown_preset_fails() {
    great()
        .args(["mcp-bridge", "--preset", "invalid"])
        .assert()
        .failure();
}

#[test]
fn mcp_bridge_unknown_preset_shows_error_message() {
    great()
        .args(["mcp-bridge", "--preset", "invalid_preset_xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid preset"));
}
