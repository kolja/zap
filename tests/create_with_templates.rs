use std::env;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_create_with_template() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("templated.txt");

    let config_dir = temp_dir.path()
        .join(".config")
        .join("zap");

    let template_dir = config_dir.join("templates");

    std::fs::create_dir_all(&template_dir).expect("Failed to create template directory");

    let template_file = template_dir.join("simple");
    std::fs::write(&template_file, "Hello, world!").expect("Failed to create template");

    // Ensure test file doesn't exist
    assert!(!test_file.exists());

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--template",
            "simple",
            test_file.to_str().unwrap(),
        ])
        .env("ZAP_CONFIG", config_dir)
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
}

#[test]
fn test_create_with_template_and_context() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("templated.txt");

    let config_dir = temp_dir.path()
        .join(".config")
        .join("zap");

    let template_dir = config_dir.join("templates");

    std::fs::create_dir_all(&template_dir).expect("Failed to create template directory");

    let template_file = template_dir.join("simple");
    std::fs::write(&template_file, "Hello, {{ name }}!").expect("Failed to create template");

    // Ensure test file doesn't exist
    assert!(!test_file.exists());

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--template",
            "simple",
            "--context",
            "name=Bob",
            test_file.to_str().unwrap(),
        ])
        .env("ZAP_CONFIG", config_dir)
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
    assert_eq!(content, "Hello, Bob!");
}
