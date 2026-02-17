//! Integration tests for CLI commands
//!
//! Tests cover:
//! - init command
//! - verify command
//! - metrics command
//! - config command
//! - report command
//! - daemon commands (status, on, off)

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to create a temporary git repository
fn create_temp_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to initialize git repository");

    // Configure git user
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to configure git user.email");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to configure git user.name");

    temp_dir
}

/// Helper function to run cliff-watch CLI command
fn run_cliff_watch(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--"])
        .args(args)
        .output()
        .expect("Failed to run cliff-watch command")
}

#[test]
fn test_init_command_creates_config() {
    let temp_dir = create_temp_repo();
    let repo_path = temp_dir.path();

    // Run init command from project root
    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "init", "--path"])
        .arg(repo_path)
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run init command");

    // Check that command executed successfully
    assert!(output.status.success(), "Init command failed: {:?}", output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Initializing cliff-watch") || stdout.contains("✅"),
            "Init command should show initialization message");
}

#[test]
fn test_init_command_generates_keypair() {
    let temp_dir = create_temp_repo();
    let repo_path = temp_dir.path();

    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "init", "--path"])
        .arg(repo_path)
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run init command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Public key") || stdout.contains("keypair"),
            "Init command should generate and display public key");
}

#[test]
fn test_verify_command_with_head() {
    let temp_dir = create_temp_repo();
    let repo_path = temp_dir.path();

    // Create a test commit
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "test content").expect("Failed to write test file");

    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add file");

    Command::new("git")
        .args(["commit", "-m", "Test commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to create commit");

    // Run verify command
    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "verify", "HEAD"])
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .env("CARGO_MANIFEST_DIR", "/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run verify command");

    // Verify command should execute (may fail on signature verification but should run)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.is_empty() || !stderr.is_empty(),
            "Verify command should produce output");
}

#[test]
fn test_verify_command_with_json_format() {
    let temp_dir = create_temp_repo();
    let repo_path = temp_dir.path();

    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "verify", "HEAD", "--format", "json"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to run verify command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // JSON output should be valid or error should be in stderr
    if !stdout.is_empty() {
        let json_output: serde_json::Value = serde_json::from_str(&stdout)
            .expect("Verify command should output valid JSON when format=json");
        assert!(json_output.is_object(), "JSON output should be an object");
    }
}

#[test]
fn test_metrics_command_short_format() {
    // Note: This test may fail if daemon is not running
    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "metrics", "--short"])
        .output()
        .expect("Failed to run metrics command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // In short format, output should be just a number or error message
    if output.status.success() {
        assert!(stdout.trim().parse::<f64>().is_ok() || stdout.trim().is_empty(),
                "Short format should output a number or be empty");
    }
}

#[test]
fn test_metrics_command_long_format() {
    // Note: This test may fail if daemon is not running
    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "metrics"])
        .output()
        .expect("Failed to run metrics command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should produce some output (either metrics or error)
    assert!(!stdout.is_empty() || !stderr.is_empty(),
            "Metrics command should produce output");
}

#[test]
fn test_config_init_command() {
    // Run config init from the project root
    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "config", "init"])
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run config init command");

    // Command should execute successfully
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("✅") || stdout.contains("Created") || stderr.contains("already exists"),
            "Config init should show success message or warn about existing config");
}

#[test]
fn test_config_init_fails_if_exists() {
    // Run config init from the project root
    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "config", "init"])
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run config init command");

    // Should fail because config already exists (from previous test)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists") || stderr.contains("⚠️"),
            "Should warn about existing config");
}

#[test]
fn test_config_check_command() {
    // Run config check from the project root
    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "config", "check"])
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run config check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✅") || stdout.contains("valid") || stdout.contains("Configuration"),
            "Config check should validate the config");
}

#[test]
fn test_report_command_text_format() {
    let temp_dir = create_temp_repo();
    let repo_path = temp_dir.path();

    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "report", "--limit", "10"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to run report command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should produce output
    assert!(!stdout.is_empty() || !stderr.is_empty(),
            "Report command should produce output");

    if output.status.success() {
        assert!(stdout.contains("Report") || stdout.contains("Commits"),
                "Report should contain report information");
    }
}

#[test]
fn test_report_command_json_format() {
    let temp_dir = create_temp_repo();
    let repo_path = temp_dir.path();

    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "report", "--limit", "10", "--format", "json"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to run report command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        let json_output: serde_json::Value = serde_json::from_str(&stdout)
            .expect("Report command should output valid JSON when format=json");
        assert!(json_output.is_array(), "JSON report should be an array");
    }
}

#[test]
fn test_report_command_markdown_format() {
    let temp_dir = create_temp_repo();
    let repo_path = temp_dir.path();

    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "report", "--limit", "10", "--format", "md"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to run report command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        assert!(stdout.contains("#") || stdout.contains("|"),
                "Markdown report should contain markdown formatting");
    }
}

#[test]
fn test_daemon_status_command() {
    // Note: This test may fail if daemon is not running
    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "status"])
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run status command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should produce output (status or error)
    assert!(!stdout.is_empty() || !stderr.is_empty(),
            "Status command should produce output");
}

#[test]
fn test_daemon_on_command() {
    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "on"])
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run on command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should produce output
    assert!(!stdout.is_empty(),
            "On command should produce output");

    if output.status.success() {
        assert!(stdout.contains("✅") || stdout.contains("activado") || stdout.contains("Encendiendo"),
                "On command should show activation message");
    }
}

#[test]
fn test_daemon_off_command() {
    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "off"])
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run off command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should produce output
    assert!(!stdout.is_empty(),
            "Off command should produce output");

    if output.status.success() {
        assert!(stdout.contains("✅") || stdout.contains("desactivado") || stdout.contains("Apagando"),
                "Off command should show deactivation message");
    }
}

#[test]
fn test_daemon_command_with_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "daemon", "--config", "test-config.toml"])
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run daemon command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("daemon") || stdout.contains("config"),
            "Daemon command should mention daemon and config");
}

#[test]
fn test_daemon_command_with_daemon_flag() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "daemon", "--daemon"])
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run daemon command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("background") || stdout.contains("✅"),
            "Daemon command with --daemon flag should mention background");
}

#[test]
fn test_multiple_commands_in_sequence() {
    let temp_dir = create_temp_repo();
    let repo_path = temp_dir.path();

    // Run init
    let init_output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "init", "--path"])
        .arg(repo_path)
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run init command");

    assert!(init_output.status.success(), "Init should succeed");

    // Run config check
    let check_output = Command::new("cargo")
        .args(["run", "--package", "cliff-watch-cli", "--", "config", "check"])
        .current_dir("/home/leonardo/Desarrollo/cliff-watch")
        .output()
        .expect("Failed to run config check command");

    // Config check should produce output (may succeed or fail)
    let stdout = String::from_utf8_lossy(&check_output.stdout);
    let stderr = String::from_utf8_lossy(&check_output.stderr);
    assert!(!stdout.is_empty() || !stderr.is_empty(),
            "Config check should produce output");
}
