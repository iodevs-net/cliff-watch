//! Tests for git operations in cliff-watch-core
//!
//! This module provides comprehensive test coverage for git-related functionality
//! including repository operations, trailer handling, hooks, and witness data.

use cliff_watch_core::git::{
    open_repository, get_latest_commit, create_signature, add_trailer, has_trailer,
    install_hooks, remove_hooks, get_staged_diff, register_public_key, get_trusted_keys,
    get_governance_history, HumanProbability, WitnessData, calculate_human_probability,
    generate_witness_trailer, inject_witness_trailer, extract_witness_data,
};
use cliff_watch_core::config::GovConfig;
use cliff_watch_core::focus_session::FocusMetrics;
use std::path::PathBuf;
use std::fs;

/// Helper function to create a temporary test repository with a unique path
fn create_test_repo() -> (git2::Repository, PathBuf) {
    let dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let random: u64 = rand::random();
    let repo_path = dir.join(format!("test_repo_{}_{}", timestamp, random));
    
    // Remove existing directory if it exists
    if repo_path.exists() {
        let _ = fs::remove_dir_all(&repo_path);
    }
    
    let repo = git2::Repository::init(&repo_path).expect("Failed to create test repository");
    
    // Configure git user
    let mut config = repo.config().expect("Failed to get config");
    config.set_str("user.name", "Test User").expect("Failed to set user name");
    config.set_str("user.email", "test@example.com").expect("Failed to set user email");
    
    (repo, repo_path)
}

/// Helper function to create a test commit
fn create_test_commit(repo: &git2::Repository, message: &str, file_content: &str) -> git2::Oid {
    let repo_path = repo.workdir().expect("No workdir");
    let file_path = repo_path.join("test.txt");
    
    fs::write(&file_path, file_content).expect("Failed to write test file");
    
    let mut index = repo.index().expect("Failed to get index");
    index.add_path(PathBuf::from("test.txt").as_path()).expect("Failed to add path");
    index.write().expect("Failed to write index");
    
    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    
    let signature = repo.signature().expect("Failed to create signature");
    
    // Get the parent commit if HEAD exists
    let parent: Option<git2::Commit> = if let Ok(head) = repo.head() {
        if let Ok(commit) = head.peel_to_commit() {
            Some(commit)
        } else {
            None
        }
    } else {
        None
    };
    
    let parents: Vec<&git2::Commit> = parent.as_ref().map_or(vec![], |c| vec![c]);
    
    repo.commit(Some("HEAD"), &signature, &signature, message, &tree, parents.as_slice())
        .expect("Failed to create commit")
}

#[test]
fn test_open_repository() {
    let (_repo, repo_path) = create_test_repo();
    
    let result = open_repository(&repo_path);
    assert!(result.is_ok(), "Failed to open repository");
    
    let opened_repo = result.unwrap();
    assert_eq!(opened_repo.path(), repo_path.join(".git"));
}

#[test]
fn test_open_repository_invalid_path() {
    let invalid_path = PathBuf::from("/nonexistent/path/to/repo");
    let result = open_repository(&invalid_path);
    assert!(result.is_err(), "Should fail for invalid path");
}

#[test]
fn test_get_latest_commit() {
    let (repo, _repo_path) = create_test_repo();
    
    // Create a commit
    create_test_commit(&repo, "Initial commit", "test content");
    
    let commit = get_latest_commit(&repo).expect("Failed to get latest commit");
    assert_eq!(commit.message().unwrap(), "Initial commit");
}

#[test]
fn test_create_signature() {
    let signature = create_signature("Test User", "test@example.com");
    assert!(signature.is_ok(), "Failed to create signature");
    
    let sig = signature.unwrap();
    assert_eq!(sig.name(), Some("Test User"));
    assert_eq!(sig.email(), Some("test@example.com"));
}

#[test]
fn test_add_trailer() {
    let message = "Initial commit";
    let new_message = add_trailer(message, "cliff-watch-score", "0.85");
    
    assert!(new_message.contains("cliff-watch-score: 0.85"));
    assert!(new_message.contains("Initial commit"));
}

#[test]
fn test_add_trailer_with_existing_content() {
    let message = "Initial commit\n\nThis is a test commit";
    let new_message = add_trailer(message, "key", "value");
    
    assert!(new_message.contains("key: value"));
    assert!(new_message.contains("Initial commit"));
}

#[test]
fn test_has_trailer() {
    let (repo, _repo_path) = create_test_repo();
    
    // Create a commit with a trailer
    create_test_commit(&repo, "Initial commit\ncliff-watch-score: 0.85", "test content");
    
    let commit = get_latest_commit(&repo).expect("Failed to get latest commit");
    let has_trailer = has_trailer(&commit, "cliff-watch-score").expect("Failed to check trailer");
    
    assert!(has_trailer);
}

#[test]
fn test_has_trailer_not_found() {
    let (repo, _repo_path) = create_test_repo();
    
    // Create a commit without the trailer
    create_test_commit(&repo, "Initial commit", "test content");
    
    let commit = get_latest_commit(&repo).expect("Failed to get latest commit");
    let has_trailer = has_trailer(&commit, "cliff-watch-score").expect("Failed to check trailer");
    
    assert!(!has_trailer);
}

#[test]
fn test_install_hooks() {
    let (repo, _repo_path) = create_test_repo();
    let config = GovConfig::default();
    
    let result = install_hooks(&repo, &config);
    assert!(result.is_ok(), "Failed to install hooks");
    
    // Check that hooks were created
    let hooks_dir = repo.path().join("hooks");
    assert!(hooks_dir.join("prepare-commit-msg").exists());
    assert!(hooks_dir.join("pre-commit").exists());
}

#[test]
fn test_remove_hooks() {
    let (repo, _repo_path) = create_test_repo();
    let config = GovConfig::default();
    
    // Install hooks first
    install_hooks(&repo, &config).expect("Failed to install hooks");
    
    // Remove hooks
    let result = remove_hooks(&repo);
    assert!(result.is_ok(), "Failed to remove hooks");
    
    // Check that hooks were removed
    let hooks_dir = repo.path().join("hooks");
    assert!(!hooks_dir.join("prepare-commit-msg").exists());
    assert!(!hooks_dir.join("pre-commit").exists());
}

#[test]
fn test_get_staged_diff() {
    let (repo, _repo_path) = create_test_repo();
    
    // Create initial commit
    create_test_commit(&repo, "Initial commit", "initial content");
    
    // Modify file and stage it
    let repo_path = repo.workdir().expect("No workdir");
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "modified content").expect("Failed to modify file");
    
    let mut index = repo.index().expect("Failed to get index");
    index.add_path(PathBuf::from("test.txt").as_path()).expect("Failed to add path");
    index.write().expect("Failed to write index");
    
    let diff = get_staged_diff(&repo).expect("Failed to get staged diff");
    assert!(!diff.is_empty(), "Diff should not be empty");
    assert!(diff.contains("modified content") || diff.contains("initial content"));
}

#[test]
fn test_get_staged_diff_empty() {
    let (repo, _repo_path) = create_test_repo();
    
    // Create initial commit
    create_test_commit(&repo, "Initial commit", "initial content");
    
    // No changes staged
    let diff = get_staged_diff(&repo).expect("Failed to get staged diff");
    assert!(diff.is_empty(), "Diff should be empty when no changes are staged");
}

#[test]
fn test_register_public_key() {
    let (repo, _repo_path) = create_test_repo();
    
    let key_hex = "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899";
    let alias = "test_user";
    
    let result = register_public_key(&repo, key_hex, alias);
    assert!(result.is_ok(), "Failed to register public key");
    
    // Verify key was registered
    let keys = get_trusted_keys(&repo).expect("Failed to get trusted keys");
    assert_eq!(keys.get(alias), Some(&key_hex.to_string()));
}

#[test]
fn test_register_duplicate_key() {
    let (repo, _repo_path) = create_test_repo();
    
    let key_hex = "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899";
    let alias = "test_user";
    
    // Register key first time
    register_public_key(&repo, key_hex, alias).expect("Failed to register public key");
    
    // Try to register same alias again
    let result = register_public_key(&repo, key_hex, alias);
    assert!(result.is_err(), "Should fail to register duplicate key");
}

#[test]
fn test_get_trusted_keys() {
    let (repo, _repo_path) = create_test_repo();
    
    // Register multiple keys
    register_public_key(&repo, "key1_hex", "user1").expect("Failed to register key1");
    register_public_key(&repo, "key2_hex", "user2").expect("Failed to register key2");
    
    let keys = get_trusted_keys(&repo).expect("Failed to get trusted keys");
    assert_eq!(keys.len(), 2);
    assert_eq!(keys.get("user1"), Some(&"key1_hex".to_string()));
    assert_eq!(keys.get("user2"), Some(&"key2_hex".to_string()));
}

#[test]
fn test_get_trusted_keys_empty() {
    let (repo, _repo_path) = create_test_repo();
    
    let keys = get_trusted_keys(&repo).expect("Failed to get trusted keys");
    // Keys should be empty when none are registered (or only have default entries)
    assert!(keys.is_empty() || keys.len() <= 1, "Keys should be empty or minimal when none are registered");
}

#[test]
fn test_get_governance_history() {
    let (repo, _repo_path) = create_test_repo();
    
    // Create commits with cliff-watch-score trailers
    create_test_commit(&repo, "Commit 1\ncliff-watch-score: score=0.85:sig=abc123", "content1");
    create_test_commit(&repo, "Commit 2\ncliff-watch-score: score=0.90:sig=def456", "content2");
    
    let history = get_governance_history(&repo, 10).expect("Failed to get governance history");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].score, 0.90); // Most recent first
    assert_eq!(history[1].score, 0.85);
}

#[test]
fn test_get_governance_history_limit() {
    let (repo, _repo_path) = create_test_repo();
    
    // Create 5 commits
    for i in 0..5 {
        create_test_commit(
            &repo,
            &format!("Commit {}\ncliff-watch-score: score=0.{}:sig=sig{}", i, i, i),
            &format!("content{}", i),
        );
    }
    
    let history = get_governance_history(&repo, 3).expect("Failed to get governance history");
    assert_eq!(history.len(), 3, "Should respect limit parameter");
}

#[test]
fn test_calculate_human_probability_high() {
    let metrics = FocusMetrics {
        total_focus_mins: 10.0,
        edit_burst_count: 15,
        unique_files: 5,
        navigation_events: 10,
        chars_edited_net: 100,
        is_synthetic: false,
    };
    
    let probability = calculate_human_probability(&metrics);
    assert_eq!(probability, HumanProbability::High);
}

#[test]
fn test_calculate_human_probability_medium() {
    let metrics = FocusMetrics {
        total_focus_mins: 1.0,
        edit_burst_count: 2,
        unique_files: 1,
        navigation_events: 1,
        chars_edited_net: 50,
        is_synthetic: false,
    };
    
    // Score = 1.0 * 10.0 + 2 * 5.0 + 1 * 3.0 + 1 * 1.0 = 10 + 10 + 3 + 1 = 24 (Medium range: 15-49)
    let probability = calculate_human_probability(&metrics);
    assert_eq!(probability, HumanProbability::Medium);
}

#[test]
fn test_calculate_human_probability_low() {
    let metrics = FocusMetrics {
        total_focus_mins: 0.5,
        edit_burst_count: 1,
        unique_files: 1,
        navigation_events: 0,
        chars_edited_net: 10,
        is_synthetic: false,
    };
    
    // Score = 0.5 * 10.0 + 1 * 5.0 + 1 * 3.0 + 0 * 1.0 = 5 + 5 + 3 = 13 (Low range: 1-14)
    let probability = calculate_human_probability(&metrics);
    assert_eq!(probability, HumanProbability::Low);
}

#[test]
fn test_calculate_human_probability_unknown() {
    let metrics = FocusMetrics {
        total_focus_mins: 0.0,
        edit_burst_count: 0,
        unique_files: 0,
        navigation_events: 0,
        chars_edited_net: 0,
        is_synthetic: false,
    };
    
    let probability = calculate_human_probability(&metrics);
    assert_eq!(probability, HumanProbability::Unknown);
}

#[test]
fn test_witness_data_from_metrics() {
    let metrics = FocusMetrics {
        total_focus_mins: 5.25,
        edit_burst_count: 12,
        unique_files: 3,
        navigation_events: 8,
        chars_edited_net: 100,
        is_synthetic: false,
    };
    
    let witness = WitnessData::from_metrics(&metrics);
    assert_eq!(witness.focus_time_mins, 5.25);
    assert_eq!(witness.edit_bursts, 12);
    assert_eq!(witness.files_touched, 3);
    assert_eq!(witness.version, "2.0");
    assert_eq!(witness.human_probability, HumanProbability::High);
}

#[test]
fn test_witness_data_to_json() {
    let witness = WitnessData {
        focus_time_mins: 5.25,
        edit_bursts: 12,
        files_touched: 3,
        human_probability: HumanProbability::High,
        version: "2.0".to_string(),
    };
    
    let json = witness.to_json();
    assert!(json.contains("focus_time_mins"));
    assert!(json.contains("edit_bursts"));
    assert!(json.contains("files_touched"));
    assert!(json.contains("human_probability"));
    assert!(json.contains("version"));
}

#[test]
fn test_generate_witness_trailer() {
    let metrics = FocusMetrics {
        total_focus_mins: 5.25,
        edit_burst_count: 12,
        unique_files: 3,
        navigation_events: 8,
        chars_edited_net: 100,
        is_synthetic: false,
    };
    
    let trailer = generate_witness_trailer(&metrics);
    assert!(trailer.starts_with("Cliff-Watch-Witness: "));
    assert!(trailer.contains("focus_time_mins"));
    assert!(trailer.contains("edit_bursts"));
}

#[test]
fn test_inject_witness_trailer() {
    let metrics = FocusMetrics {
        total_focus_mins: 5.25,
        edit_burst_count: 12,
        unique_files: 3,
        navigation_events: 8,
        chars_edited_net: 100,
        is_synthetic: false,
    };
    
    let message = "Initial commit";
    let new_message = inject_witness_trailer(message, &metrics);
    
    assert!(new_message.contains("Initial commit"));
    assert!(new_message.contains("Cliff-Watch-Witness:"));
}

#[test]
fn test_inject_witness_trailer_no_duplicate() {
    let metrics = FocusMetrics {
        total_focus_mins: 5.25,
        edit_burst_count: 12,
        unique_files: 3,
        navigation_events: 8,
        chars_edited_net: 100,
        is_synthetic: false,
    };
    
    let message = "Initial commit\nCliff-Watch-Witness: existing data";
    let new_message = inject_witness_trailer(message, &metrics);
    
    // Should not duplicate the trailer
    assert_eq!(new_message, message);
}

#[test]
fn test_extract_witness_data() {
    let message = "Initial commit\nCliff-Watch-Witness: {\"focus_time_mins\":5.25,\"edit_bursts\":12,\"files_touched\":3,\"human_probability\":\"high\",\"version\":\"2.0\"}";
    
    let witness = extract_witness_data(message);
    assert!(witness.is_some(), "Should extract witness data");
    
    let data = witness.unwrap();
    assert_eq!(data.focus_time_mins, 5.25);
    assert_eq!(data.edit_bursts, 12);
    assert_eq!(data.files_touched, 3);
    assert_eq!(data.human_probability, HumanProbability::High);
    assert_eq!(data.version, "2.0");
}

#[test]
fn test_extract_witness_data_not_found() {
    let message = "Initial commit\nSome other trailer: value";
    
    let witness = extract_witness_data(message);
    assert!(witness.is_none(), "Should return None when witness data is not found");
}

#[test]
fn test_extract_witness_data_invalid_json() {
    let message = "Initial commit\nCliff-Watch-Witness: invalid json";
    
    let witness = extract_witness_data(message);
    assert!(witness.is_none(), "Should return None for invalid JSON");
}

#[test]
fn test_human_probability_display() {
    assert_eq!(format!("{}", HumanProbability::High), "High");
    assert_eq!(format!("{}", HumanProbability::Medium), "Medium");
    assert_eq!(format!("{}", HumanProbability::Low), "Low");
    assert_eq!(format!("{}", HumanProbability::Unknown), "Unknown");
}

#[test]
fn test_human_probability_serialization() {
    let high = HumanProbability::High;
    let json = serde_json::to_string(&high).expect("Failed to serialize");
    assert_eq!(json, "\"high\"");
    
    let deserialized: HumanProbability = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized, HumanProbability::High);
}

#[test]
fn test_witness_data_serialization_roundtrip() {
    let original = WitnessData {
        focus_time_mins: 5.25,
        edit_bursts: 12,
        files_touched: 3,
        human_probability: HumanProbability::High,
        version: "2.0".to_string(),
    };
    
    let json = serde_json::to_string(&original).expect("Failed to serialize");
    let deserialized: WitnessData = serde_json::from_str(&json).expect("Failed to deserialize");
    
    assert_eq!(deserialized.focus_time_mins, original.focus_time_mins);
    assert_eq!(deserialized.edit_bursts, original.edit_bursts);
    assert_eq!(deserialized.files_touched, original.files_touched);
    assert_eq!(deserialized.human_probability, original.human_probability);
    assert_eq!(deserialized.version, original.version);
}
