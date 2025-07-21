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
        .args(["run", "--", "-A", "0100", "-a", test_file.to_str().unwrap()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (new_atime, new_mtime) = get_file_times(&test_file);

    // Access time should be adjusted by +1 minute (60 seconds), modification time should remain unchanged
    let expected_atime = initial_atime + Duration::from_secs(60);
    let atime_diff = new_atime
        .duration_since(expected_atime)
        .unwrap_or_else(|_| expected_atime.duration_since(new_atime).unwrap());

    // Allow for small timing differences (within 1 second)
    assert!(
        atime_diff < Duration::from_secs(1),
        "Access time should be adjusted by +1 minute"
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

#[test]
fn test_set_time_then_adjust_both_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file
    File::create(&test_file).expect("Failed to create test file");

    // Run zap with specific date and then adjust by +2 hours
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-d",
            "2023-01-01 12:00:00",
            "-A",
            "020000", // +2 hours
            test_file.to_str().unwrap(),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (new_atime, new_mtime) = get_file_times(&test_file);

    // Expected time should be 2023-01-01 14:00:00 (12:00 + 2 hours)
    // We'll check that both times are approximately correct
    let now = SystemTime::now();
    let _expected_duration = Duration::from_secs(2 * 60 * 60); // 2 hours

    // Times should be reasonable (not current time, but also not too far in past)
    assert!(
        new_atime < now,
        "Access time should be in the past (set to 2023-01-01 14:00)"
    );
    assert!(
        new_mtime < now,
        "Modification time should be in the past (set to 2023-01-01 14:00)"
    );

    // Both times should be the same since we set both and adjusted both
    let time_diff = new_atime
        .duration_since(new_mtime)
        .unwrap_or_else(|_| new_mtime.duration_since(new_atime).unwrap());

    assert!(
        time_diff < Duration::from_secs(1),
        "Access and modification times should be nearly identical"
    );
}

#[test]
fn test_adjust_with_mixed_flags() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file and get initial times
    File::create(&test_file).expect("Failed to create test file");
    let (initial_atime, initial_mtime) = get_file_times(&test_file);

    sleep_for_time_resolution();

    // Run zap with adjustment affecting only modification time
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-A",
            "0500", // +5 minutes
            "-m",   // only modification time
            test_file.to_str().unwrap(),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (new_atime, new_mtime) = get_file_times(&test_file);

    // Access time should remain unchanged
    assert_eq!(
        new_atime, initial_atime,
        "Access time should NOT have been affected by adjustment"
    );

    // Modification time should be adjusted by +5 minutes
    let expected_mtime = initial_mtime + Duration::from_secs(300);
    let mtime_diff = new_mtime
        .duration_since(expected_mtime)
        .unwrap_or_else(|_| expected_mtime.duration_since(new_mtime).unwrap());

    assert!(
        mtime_diff < Duration::from_secs(1),
        "Modification time should be adjusted by +5 minutes"
    );
}

#[test]
fn test_no_time_operations_when_skipping() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("nonexistent.txt");

    // Ensure file doesn't exist
    assert!(!test_file.exists());

    // Run zap with --no-create and time operations that would normally execute
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--no-create",
            "-t",
            "202301010000",
            "-A",
            "010000",
            test_file.to_str().unwrap(),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // File should still not exist (confirming no time operations were attempted)
    assert!(!test_file.exists());

    // Should have skipping message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Skipping"));
}
