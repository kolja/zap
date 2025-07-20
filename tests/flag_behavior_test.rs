use std::fs::{self, File};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

fn get_file_times(path: &Path) -> (SystemTime, SystemTime) {
    let metadata = fs::metadata(path).expect("Failed to get file metadata");
    (
        metadata.accessed().expect("Failed to get access time"),
        metadata
            .modified()
            .expect("Failed to get modification time"),
    )
}

fn sleep_for_time_resolution() {
    // Sleep for a bit to ensure time difference is detectable
    std::thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_no_flags_updates_both_times() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file and get initial times
    File::create(&test_file).expect("Failed to create test file");
    let (initial_atime, initial_mtime) = get_file_times(&test_file);

    sleep_for_time_resolution();

    // Run zap without any flags (should update both times)
    let output = Command::new("cargo")
        .args(["run", "--", test_file.to_str().unwrap()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (new_atime, new_mtime) = get_file_times(&test_file);

    // Both times should have been updated
    assert!(
        new_atime > initial_atime,
        "Access time should have been updated"
    );
    assert!(
        new_mtime > initial_mtime,
        "Modification time should have been updated"
    );
}

#[test]
fn test_access_flag_only_updates_access_time() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file and get initial times
    File::create(&test_file).expect("Failed to create test file");
    let (initial_atime, initial_mtime) = get_file_times(&test_file);

    sleep_for_time_resolution();

    // Run zap with -a flag (should update only access time)
    let output = Command::new("cargo")
        .args(["run", "--", "-a", test_file.to_str().unwrap()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (new_atime, new_mtime) = get_file_times(&test_file);

    // Only access time should have been updated
    assert!(
        new_atime > initial_atime,
        "Access time should have been updated"
    );
    assert_eq!(
        new_mtime, initial_mtime,
        "Modification time should NOT have been updated"
    );
}

#[test]
fn test_modification_flag_only_updates_modification_time() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file and get initial times
    File::create(&test_file).expect("Failed to create test file");
    let (initial_atime, initial_mtime) = get_file_times(&test_file);

    sleep_for_time_resolution();

    // Run zap with -m flag (should update only modification time)
    let output = Command::new("cargo")
        .args(["run", "--", "-m", test_file.to_str().unwrap()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (new_atime, new_mtime) = get_file_times(&test_file);

    // Only modification time should have been updated
    assert_eq!(
        new_atime, initial_atime,
        "Access time should NOT have been updated"
    );
    assert!(
        new_mtime > initial_mtime,
        "Modification time should have been updated"
    );
}

#[test]
fn test_both_flags_update_both_times() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file and get initial times
    File::create(&test_file).expect("Failed to create test file");
    let (initial_atime, initial_mtime) = get_file_times(&test_file);

    sleep_for_time_resolution();

    // Run zap with both -a and -m flags (should update both times)
    let output = Command::new("cargo")
        .args(["run", "--", "-a", "-m", test_file.to_str().unwrap()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (new_atime, new_mtime) = get_file_times(&test_file);

    // Both times should have been updated
    assert!(
        new_atime > initial_atime,
        "Access time should have been updated"
    );
    assert!(
        new_mtime > initial_mtime,
        "Modification time should have been updated"
    );
}

#[test]
fn test_adjust_flag_with_access_only() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file and get initial times
    File::create(&test_file).expect("Failed to create test file");
    let (initial_atime, initial_mtime) = get_file_times(&test_file);

    sleep_for_time_resolution();

    // Run zap with -A (adjust) and -a flags (should adjust only access time)
    let output = Command::new("cargo")
        .args(["run", "--", "-A", "+60", "-a", test_file.to_str().unwrap()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (new_atime, new_mtime) = get_file_times(&test_file);

    // Access time should be adjusted by +60 seconds, modification time should remain unchanged
    let expected_atime = initial_atime + Duration::from_secs(60);
    let atime_diff = new_atime
        .duration_since(expected_atime)
        .unwrap_or_else(|_| expected_atime.duration_since(new_atime).unwrap());

    // Allow for small timing differences (within 1 second)
    assert!(
        atime_diff < Duration::from_secs(1),
        "Access time should be adjusted by +60 seconds"
    );
    assert_eq!(
        new_mtime, initial_mtime,
        "Modification time should NOT have been adjusted"
    );
}

#[test]
fn test_adjust_flag_with_modification_only() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file and get initial times
    File::create(&test_file).expect("Failed to create test file");
    let (initial_atime, initial_mtime) = get_file_times(&test_file);

    sleep_for_time_resolution();

    // Run zap with -A (adjust) and -m flags (should adjust only modification time)
    let output = Command::new("cargo")
        .args(["run", "--", "-A", "-30", "-m", test_file.to_str().unwrap()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (new_atime, new_mtime) = get_file_times(&test_file);

    // Modification time should be adjusted by -30 seconds, access time should remain unchanged
    let expected_mtime = initial_mtime - Duration::from_secs(30);
    let mtime_diff = new_mtime
        .duration_since(expected_mtime)
        .unwrap_or_else(|_| expected_mtime.duration_since(new_mtime).unwrap());

    // Allow for small timing differences (within 1 second)
    assert!(
        mtime_diff < Duration::from_secs(1),
        "Modification time should be adjusted by -30 seconds"
    );
    assert_eq!(
        new_atime, initial_atime,
        "Access time should NOT have been adjusted"
    );
}
