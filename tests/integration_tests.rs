//! Integration tests for git-nexus CLI.
//!
//! These tests verify the command-line interface behavior by running
//! the binary with various arguments and checking the output.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to get the path to the compiled binary
fn git_nexus_bin() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove 'deps' directory
    path.push("git-nexus");
    path
}

/// Helper to create a test git repository
fn create_test_repo(dir: &std::path::Path, name: &str) -> PathBuf {
    let repo_path = dir.join(name);
    fs::create_dir(&repo_path).unwrap();
    
    Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");
    
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    // Create an initial commit
    fs::write(repo_path.join("README.md"), "# Test").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    repo_path
}

#[test]
fn test_help_flag() {
    let output = Command::new(git_nexus_bin())
        .arg("--help")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("git-nexus"));
    assert!(stdout.contains("blazing fast multi-repository scanner"));
}

#[test]
fn test_version_flag() {
    let output = Command::new(git_nexus_bin())
        .arg("--version")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("git-nexus"));
}

#[test]
fn test_scan_single_repo() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "test_repo");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute git-nexus");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("repositories found") || stdout.contains("test_repo"));
    assert!(output.status.success());
}

#[test]
fn test_scan_with_depth() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "repo1");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("--depth")
        .arg("2")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
}

#[test]
fn test_json_output() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "test_repo");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("--json")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");
}

#[test]
fn test_verbose_output() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "test_repo");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("--verbose")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verbose mode should show commit info
    assert!(stdout.contains("Initial commit") || stdout.contains("└─"));
}

#[test]
fn test_filter_clean() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "clean_repo");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("--filter")
        .arg("clean")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
}

#[test]
fn test_filter_dirty() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = create_test_repo(temp_dir.path(), "dirty_repo");
    
    // Make the repo dirty
    fs::write(repo_path.join("new_file.txt"), "content").unwrap();
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("--filter")
        .arg("dirty")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("repositories found") || stdout.contains("dirty_repo"));
}

#[test]
fn test_sort_by_path() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "repo_a");
    create_test_repo(temp_dir.path(), "repo_b");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("--sort")
        .arg("path")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
}

#[test]
fn test_sort_by_status() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "repo1");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("--sort")
        .arg("status")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
}

#[test]
fn test_sort_by_branch() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "repo1");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("--sort")
        .arg("branch")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
}

#[test]
fn test_export_csv() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "test_repo");
    
    let output_file = temp_dir.path().join("output.csv");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("export")
        .arg("csv")
        .arg("--output")
        .arg(&output_file)
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    assert!(output_file.exists());
    
    let contents = fs::read_to_string(&output_file).unwrap();
    assert!(contents.contains("Path"));
    assert!(contents.contains("Branch"));
    assert!(contents.contains("Status"));
}

#[test]
fn test_export_html() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "test_repo");
    
    let output_file = temp_dir.path().join("output.html");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("export")
        .arg("html")
        .arg("--output")
        .arg(&output_file)
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    assert!(output_file.exists());
    
    let contents = fs::read_to_string(&output_file).unwrap();
    assert!(contents.contains("<!DOCTYPE html>"));
    assert!(contents.contains("Git Nexus Report"));
}

#[test]
fn test_config_command() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("test-config.toml");
    
    let output = Command::new(git_nexus_bin())
        .arg("config")
        .arg("--output")
        .arg(&config_file)
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    assert!(config_file.exists());
    
    let contents = fs::read_to_string(&config_file).unwrap();
    assert!(contents.contains("scan_depth"));
    assert!(contents.contains("ignore_dirs"));
}

#[test]
fn test_no_repos_found() {
    let temp_dir = TempDir::new().unwrap();
    // Empty directory with no repos
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No git repositories found") || stdout.contains("0 repositories"));
}

#[test]
fn test_multiple_repos() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "repo1");
    create_test_repo(temp_dir.path(), "repo2");
    create_test_repo(temp_dir.path(), "repo3");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3") || stdout.contains("repo1"));
}

#[test]
fn test_json_with_verbose() {
    let temp_dir = TempDir::new().unwrap();
    create_test_repo(temp_dir.path(), "test_repo");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("--json")
        .arg("--verbose")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");
    
    // Verbose mode should include additional fields
    if let Some(arr) = json.as_array() {
        if !arr.is_empty() {
            let first_repo = &arr[0];
            assert!(first_repo.get("last_commit").is_some() || first_repo.get("stash_count").is_some());
        }
    }
}

#[test]
fn test_nested_repos() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create nested structure
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    
    create_test_repo(temp_dir.path(), "root_repo");
    create_test_repo(&sub_dir, "nested_repo");
    
    let output = Command::new(git_nexus_bin())
        .arg(temp_dir.path())
        .arg("--depth")
        .arg("5")
        .output()
        .expect("Failed to execute git-nexus");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2") || (stdout.contains("root_repo") && stdout.contains("nested_repo")));
}
