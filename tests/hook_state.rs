/// Integration test: simulate SubagentStart event via update-state.sh,
/// verify state file is written, then verify statusline reads it.
#[test]
fn test_hook_writes_state_and_statusline_reads_it() {
    use assert_cmd::Command;
    use std::io::Write;

    let session_id = format!("test-{}", std::process::id());
    let state_dir = format!("/tmp/great-loop/{}", session_id);

    // Clean up from any prior run
    let _ = std::fs::remove_dir_all(&state_dir);

    // Find the hook script (built from source)
    let hook_script =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("loop/hooks/update-state.sh");

    if !hook_script.exists() {
        eprintln!(
            "Skipping: hook script not found at {}",
            hook_script.display()
        );
        return;
    }

    // Check jq availability -- jq is a required dependency, fail explicitly
    let jq_output = std::process::Command::new("jq")
        .arg("--version")
        .output()
        .expect(
            "jq is required for this test and must be installed (apt install jq / brew install jq)",
        );
    assert!(
        jq_output.status.success(),
        "jq --version exited with non-zero status"
    );

    // Simulate SubagentStart
    let input = serde_json::json!({
        "session_id": session_id,
        "hook_event_name": "SubagentStart",
        "agent_id": "agent-test-1",
        "agent_type": "Explore",
        "cwd": "/tmp",
        "permission_mode": "default",
        "transcript_path": "/tmp/fake.jsonl"
    });

    let mut child = std::process::Command::new("bash")
        .arg(&hook_script)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn hook script");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .expect("failed to write to hook stdin");

    let status = child.wait().expect("failed to wait for hook");
    assert!(status.success(), "hook script should exit 0");

    // Verify state file
    let state_file = format!("{}/state.json", state_dir);
    let contents =
        std::fs::read_to_string(&state_file).expect("state file should exist after SubagentStart");
    let state: serde_json::Value =
        serde_json::from_str(&contents).expect("state file should be valid JSON");

    assert_eq!(state["loop_id"].as_str(), Some(&*session_id));
    assert_eq!(state["agents"][0]["status"].as_str(), Some("running"));
    assert_eq!(state["agents"][0]["name"].as_str(), Some("agent-test-1"));

    // Verify statusline reads it (not "idle")
    let statusline_input = serde_json::json!({ "session_id": session_id });
    let output = Command::cargo_bin("great")
        .expect("binary exists")
        .arg("statusline")
        .arg("--no-color")
        .write_stdin(statusline_input.to_string())
        .output()
        .expect("statusline should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("idle"),
        "statusline should not show 'idle' when agents are running, got: {}",
        stdout
    );

    // Simulate SessionEnd (cleanup)
    let end_input = serde_json::json!({
        "session_id": session_id,
        "hook_event_name": "SessionEnd",
        "reason": "other",
        "cwd": "/tmp",
        "permission_mode": "default",
        "transcript_path": "/tmp/fake.jsonl"
    });
    let mut child2 = std::process::Command::new("bash")
        .arg(&hook_script)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn hook script");

    child2
        .stdin
        .take()
        .unwrap()
        .write_all(end_input.to_string().as_bytes())
        .expect("failed to write to hook stdin");
    child2.wait().expect("failed to wait for hook");

    // Verify cleanup
    assert!(
        !std::path::Path::new(&state_dir).exists(),
        "SessionEnd should remove the session directory"
    );
}
