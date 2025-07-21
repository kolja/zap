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
fn test_create_empty_file_with_current_time() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("new_file.txt");

    // Ensure file doesn't exist
    assert!(!test_file.exists());

    let before_time = SystemTime::now();

    // Run zap to create empty file
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

    let after_time = SystemTime::now();

    // File should now exist
    assert!(test_file.exists());

    let (atime, mtime) = get_file_times(&test_file);

    // Times should be between before and after
    assert!(atime >= before_time && atime <= after_time);
    assert!(mtime >= before_time && mtime <= after_time);
}

#[test]
fn test_set_specific_time_then_adjust() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file first
    File::create(&test_file).expect("Failed to create test file");

    // Run zap with specific time and adjustment
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-t",
            "202301010000", // Set to Jan 1, 2023 00:00
            "-A",
            "010000", // Then adjust by +1 hour
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

    let (atime, mtime) = get_file_times(&test_file);

    // Expected time: Jan 1, 2023 01:00 in LOCAL time (00:00 local + 1 hour)
    // The -t option parses time as local time, not UTC
    use chrono::{Local, TimeZone};
    let local_dt = Local.with_ymd_and_hms(2023, 1, 1, 1, 0, 0).unwrap();
    let expected_timestamp = local_dt.timestamp() as u64;
    let expected_time = SystemTime::UNIX_EPOCH + Duration::from_secs(expected_timestamp);

    // Allow for small timing differences (within 2 seconds)
    let atime_diff = atime
        .duration_since(expected_time)
        .unwrap_or_else(|_| expected_time.duration_since(atime).unwrap());
    let mtime_diff = mtime
        .duration_since(expected_time)
        .unwrap_or_else(|_| expected_time.duration_since(mtime).unwrap());

    println!("Expected time: {:?}", expected_time);
    println!("Actual atime: {:?}", atime);
    println!("Actual mtime: {:?}", mtime);
    println!("Atime diff: {:?}", atime_diff);
    println!("Mtime diff: {:?}", mtime_diff);

    assert!(
        atime_diff < Duration::from_secs(2),
        "Access time should be Jan 1, 2023 01:00 (00:00 + 1 hour). Expected: {:?}, Got: {:?}, Diff: {:?}",
        expected_time,
        atime,
        atime_diff
    );
    assert!(
        mtime_diff < Duration::from_secs(2),
        "Modification time should be Jan 1, 2023 01:00 (00:00 + 1 hour). Expected: {:?}, Got: {:?}, Diff: {:?}",
        expected_time,
        mtime,
        mtime_diff
    );
}

#[test]
fn test_set_time_access_only_then_adjust_access_only() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file and get initial times
    File::create(&test_file).expect("Failed to create test file");
    let (_, initial_mtime) = get_file_times(&test_file);

    sleep_for_time_resolution();

    // Run zap with specific time for access only, then adjust access only
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-t",
            "202301010000", // Set to Jan 1, 2023 00:00
            "-A",
            "3000", // Then adjust by +30 minutes
            "-a",   // Only affect access time
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

    // Expected access time: Jan 1, 2023 00:30 in LOCAL time (00:00 local + 30 minutes)
    use chrono::{Local, TimeZone};
    let expected_dt = Local.with_ymd_and_hms(2023, 1, 1, 0, 30, 0).unwrap();
    let expected_timestamp = expected_dt.timestamp() as u64;
    let expected_atime = SystemTime::UNIX_EPOCH + Duration::from_secs(expected_timestamp);

    let atime_diff = new_atime
        .duration_since(expected_atime)
        .unwrap_or_else(|_| expected_atime.duration_since(new_atime).unwrap());

    assert!(
        atime_diff < Duration::from_secs(2),
        "Access time should be Jan 1, 2023 00:30"
    );
    assert_eq!(
        new_mtime, initial_mtime,
        "Modification time should remain unchanged"
    );
}

#[test]
fn test_adjustment_only_without_initial_time_setting() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file and get initial times
    File::create(&test_file).expect("Failed to create test file");
    let (initial_atime, initial_mtime) = get_file_times(&test_file);

    sleep_for_time_resolution();

    // Run zap with only adjustment (no time setting)
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-A",
            "0200", // Adjust by +2 minutes
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

    // Both times should be adjusted by +2 minutes from their initial values
    let expected_atime = initial_atime + Duration::from_secs(120);
    let expected_mtime = initial_mtime + Duration::from_secs(120);

    let atime_diff = new_atime
        .duration_since(expected_atime)
        .unwrap_or_else(|_| expected_atime.duration_since(new_atime).unwrap());
    let mtime_diff = new_mtime
        .duration_since(expected_mtime)
        .unwrap_or_else(|_| expected_mtime.duration_since(new_mtime).unwrap());

    assert!(
        atime_diff < Duration::from_secs(1),
        "Access time should be adjusted by +2 minutes"
    );
    assert!(
        mtime_diff < Duration::from_secs(1),
        "Modification time should be adjusted by +2 minutes"
    );
}

#[test]
fn test_create_with_template_and_specific_time() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("templated.txt");

    // Create a simple template file for testing
    let config_dir = temp_dir.path().join(".config").join("zap");
    let template_dir = config_dir.join("templates");
    let plugins_dir = config_dir.join("plugins");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template directory");
    std::fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    let template_file = template_dir.join("simple");
    std::fs::write(&template_file, "Hello, world!").expect("Failed to create template");

    // Ensure test file doesn't exist
    assert!(!test_file.exists());

    // Calculate a future time (current time + 5 minutes) truncated to minute precision
    use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
    let now_datetime: DateTime<Local> = SystemTime::now().into();
    let future_datetime = now_datetime + chrono::Duration::minutes(5);

    // Truncate to minute precision (remove seconds and subseconds)
    let future_datetime_truncated = Local
        .with_ymd_and_hms(
            future_datetime.year(),
            future_datetime.month(),
            future_datetime.day(),
            future_datetime.hour(),
            future_datetime.minute(),
            0,
        )
        .unwrap();

    let future_time = SystemTime::from(future_datetime_truncated);
    let future_timestamp = future_datetime_truncated.format("%Y%m%d%H%M").to_string();

    // Run zap with template and specific future time
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--template",
            "simple",
            "-t",
            &future_timestamp, // Set to specific future time
            test_file.to_str().unwrap(),
        ])
        .env("HOME", temp_dir.path()) // Point HOME to temp dir so it finds our template
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // File should now exist with template content
    assert!(test_file.exists());
    let content = std::fs::read_to_string(&test_file).expect("Failed to read file");
    assert_eq!(content, "Hello, world!");

    let (atime, mtime) = get_file_times(&test_file);

    // Times should be approximately the future time we set (within a small tolerance)
    let atime_diff = atime
        .duration_since(future_time)
        .unwrap_or_else(|_| future_time.duration_since(atime).unwrap());
    let mtime_diff = mtime
        .duration_since(future_time)
        .unwrap_or_else(|_| future_time.duration_since(mtime).unwrap());

    assert!(
        atime_diff < Duration::from_secs(2),
        "Access time should be approximately the set future time"
    );
    assert!(
        mtime_diff < Duration::from_secs(2),
        "Modification time should be approximately the set future time"
    );
}

#[test]
fn test_no_create_flag_with_nonexistent_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("nonexistent.txt");

    // Ensure file doesn't exist
    assert!(!test_file.exists());

    // Run zap with --no-create flag
    let output = Command::new("cargo")
        .args(["run", "--", "--no-create", test_file.to_str().unwrap()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute zap command");

    assert!(
        output.status.success(),
        "zap command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // File should still not exist
    assert!(!test_file.exists());

    // Should have skipping message in output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Skipping"));
    assert!(stdout.contains("--no-create flag is set"));
}

#[test]
fn test_multiple_sequential_adjustments() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");

    // Create a file
    File::create(&test_file).expect("Failed to create test file");

    // First adjustment: +1 hour
    let output1 = Command::new("cargo")
        .args(["run", "--", "-A", "010000", test_file.to_str().unwrap()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute first zap command");

    assert!(output1.status.success());

    let (atime1, mtime1) = get_file_times(&test_file);

    sleep_for_time_resolution();

    // Second adjustment: -30 minutes
    let output2 = Command::new("cargo")
        .args(["run", "--", "-A", "-3000", test_file.to_str().unwrap()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute second zap command");

    assert!(output2.status.success());

    let (atime2, mtime2) = get_file_times(&test_file);

    // Second times should be 30 minutes earlier than first times
    let expected_atime2 = atime1 - Duration::from_secs(1800);
    let expected_mtime2 = mtime1 - Duration::from_secs(1800);

    let atime_diff = atime2
        .duration_since(expected_atime2)
        .unwrap_or_else(|_| expected_atime2.duration_since(atime2).unwrap());
    let mtime_diff = mtime2
        .duration_since(expected_mtime2)
        .unwrap_or_else(|_| expected_mtime2.duration_since(mtime2).unwrap());

    assert!(
        atime_diff < Duration::from_secs(1),
        "Access time should be 30 minutes earlier than previous"
    );
    assert!(
        mtime_diff < Duration::from_secs(1),
        "Modification time should be 30 minutes earlier than previous"
    );
}
