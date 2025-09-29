use std::fs;
use std::path::Path;
use tempfile::TempDir;
use serde_yaml;

#[tokio::test]
async fn test_init_directory() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dataset");
    fs::create_dir(&test_dir).unwrap();
    
    // Create test files
    fs::write(test_dir.join("data.csv"), "name,age\nAlice,30\nBob,25").unwrap();
    fs::write(test_dir.join("README.md"), "# Test Dataset").unwrap();
    
    let args = blacklake_cli::cmd::init::InitArgs {
        path: test_dir.to_string_lossy().to_string(),
        recursive: false,
        max_depth: 1,
        include_hidden: false,
        follow_symlinks: false,
        namespace: "test".to_string(),
        label: vec![("domain".to_string(), "demo".to_string())],
        meta: vec![("source".to_string(), "test".to_string())],
        class: "restricted".to_string(),
        owner: Some("test@example.com".to_string()),
        no_hash: false,
        hash: "blake3,sha256".to_string(),
        set: vec![],
        overwrite: false,
        dry_run: false,
        with_authorization: false,
    };
    
    blacklake_cli::cmd::init::init_command(args).await.unwrap();
    
    // Check that .bl directory was created
    let bl_dir = test_dir.join(".bl");
    assert!(bl_dir.exists());
    
    // Check that metadata files were created
    assert!(bl_dir.join("data.csv.metadata.yaml").exists());
    assert!(bl_dir.join("README.md.metadata.yaml").exists());
    assert!(bl_dir.join("_artifact.metadata.yaml").exists());
    assert!(bl_dir.join("policy.yaml").exists());
    assert!(bl_dir.join("authorization.yaml").exists());
    assert!(bl_dir.join("README.md").exists());
    assert!(bl_dir.join(".gitignore").exists());
    assert!(bl_dir.join("provenance.yaml").exists());
    
    // Verify metadata content
    let metadata_content = fs::read_to_string(bl_dir.join("data.csv.metadata.yaml")).unwrap();
    let metadata: serde_yaml::Value = serde_yaml::from_str(&metadata_content).unwrap();
    
    assert_eq!(metadata["schema_version"], "1");
    assert_eq!(metadata["artifact_type"], "file");
    assert_eq!(metadata["name"], "data.csv");
    assert_eq!(metadata["namespace"], "test");
    assert_eq!(metadata["labels"]["domain"], "demo");
    assert_eq!(metadata["user_metadata"]["source"], "test");
    assert_eq!(metadata["policy"]["classification"], "restricted");
    assert_eq!(metadata["policy"]["owner"], "test@example.com");
    
    // Verify checksums were computed
    assert!(metadata["checksums"]["blake3"].is_string());
    assert!(metadata["checksums"]["sha256"].is_string());
}

#[tokio::test]
async fn test_init_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("model.onnx");
    fs::write(&test_file, "fake onnx data").unwrap();
    
    let args = blacklake_cli::cmd::init::InitArgs {
        path: test_file.to_string_lossy().to_string(),
        recursive: false,
        max_depth: 1,
        include_hidden: false,
        follow_symlinks: false,
        namespace: "ml".to_string(),
        label: vec![("framework".to_string(), "onnx".to_string())],
        meta: vec![],
        class: "internal".to_string(),
        owner: Some("ml@example.com".to_string()),
        no_hash: false,
        hash: "blake3,sha256".to_string(),
        set: vec![],
        overwrite: false,
        dry_run: false,
        with_authorization: true,
    };
    
    blacklake_cli::cmd::init::init_command(args).await.unwrap();
    
    // Check that sidecar files were created
    let metadata_file = test_file.with_extension("bl.metadata.yaml");
    let auth_file = test_file.with_extension("bl.authorization.yaml");
    
    assert!(metadata_file.exists());
    assert!(auth_file.exists());
    
    // Verify metadata content
    let metadata_content = fs::read_to_string(&metadata_file).unwrap();
    let metadata: serde_yaml::Value = serde_yaml::from_str(&metadata_content).unwrap();
    
    assert_eq!(metadata["schema_version"], "1");
    assert_eq!(metadata["artifact_type"], "file");
    assert_eq!(metadata["name"], "model.onnx");
    assert_eq!(metadata["namespace"], "ml");
    assert_eq!(metadata["labels"]["framework"], "onnx");
    assert_eq!(metadata["policy"]["classification"], "internal");
    assert_eq!(metadata["policy"]["owner"], "ml@example.com");
}

#[tokio::test]
async fn test_init_no_hash() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("data.txt");
    fs::write(&test_file, "test data").unwrap();
    
    let args = blacklake_cli::cmd::init::InitArgs {
        path: test_file.to_string_lossy().to_string(),
        recursive: false,
        max_depth: 1,
        include_hidden: false,
        follow_symlinks: false,
        namespace: "default".to_string(),
        label: vec![],
        meta: vec![],
        class: "restricted".to_string(),
        owner: None,
        no_hash: true,
        hash: "blake3,sha256".to_string(),
        set: vec![],
        overwrite: false,
        dry_run: false,
        with_authorization: false,
    };
    
    blacklake_cli::cmd::init::init_command(args).await.unwrap();
    
    let metadata_file = test_file.with_extension("bl.metadata.yaml");
    let metadata_content = fs::read_to_string(&metadata_file).unwrap();
    let metadata: serde_yaml::Value = serde_yaml::from_str(&metadata_content).unwrap();
    
    // Verify checksums are null when --no-hash is used
    assert!(metadata["checksums"]["blake3"].is_null());
    assert!(metadata["checksums"]["sha256"].is_null());
}

#[tokio::test]
async fn test_init_dot_notation_overrides() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("data.json");
    fs::write(&test_file, r#"{"test": "data"}"#).unwrap();
    
    let args = blacklake_cli::cmd::init::InitArgs {
        path: test_file.to_string_lossy().to_string(),
        recursive: false,
        max_depth: 1,
        include_hidden: false,
        follow_symlinks: false,
        namespace: "default".to_string(),
        label: vec![],
        meta: vec![],
        class: "restricted".to_string(),
        owner: None,
        no_hash: false,
        hash: "blake3,sha256".to_string(),
        set: vec![
            ("policy.readers[0]".to_string(), "group:data-science".to_string()),
            ("auth.allowed_audiences[0]".to_string(), "urn:ml:prod".to_string()),
            ("user_metadata.calibration".to_string(), r#"{"date":"2025-09-01","operator":"mx-12"}"#.to_string()),
        ],
        overwrite: false,
        dry_run: false,
        with_authorization: false,
    };
    
    blacklake_cli::cmd::init::init_command(args).await.unwrap();
    
    let metadata_file = test_file.with_extension("bl.metadata.yaml");
    let metadata_content = fs::read_to_string(&metadata_file).unwrap();
    let metadata: serde_yaml::Value = serde_yaml::from_str(&metadata_content).unwrap();
    
    // Verify dot notation overrides were applied
    assert_eq!(metadata["policy"]["readers"][0], "group:data-science");
    assert_eq!(metadata["auth"]["allowed_audiences"][0], "urn:ml:prod");
    assert_eq!(metadata["user_metadata"]["calibration"]["date"], "2025-09-01");
    assert_eq!(metadata["user_metadata"]["calibration"]["operator"], "mx-12");
}

#[tokio::test]
async fn test_init_recursive_with_max_depth() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("nested");
    fs::create_dir_all(&test_dir).unwrap();
    fs::create_dir_all(test_dir.join("subdir")).unwrap();
    
    // Create files at different depths
    fs::write(test_dir.join("level1.txt"), "level 1").unwrap();
    fs::write(test_dir.join("subdir").join("level2.txt"), "level 2").unwrap();
    fs::write(test_dir.join("subdir").join("subdir2").join("level3.txt"), "level 3").unwrap();
    
    let args = blacklake_cli::cmd::init::InitArgs {
        path: test_dir.to_string_lossy().to_string(),
        recursive: true,
        max_depth: 2,
        include_hidden: false,
        follow_symlinks: false,
        namespace: "test".to_string(),
        label: vec![],
        meta: vec![],
        class: "restricted".to_string(),
        owner: None,
        no_hash: false,
        hash: "blake3,sha256".to_string(),
        set: vec![],
        overwrite: false,
        dry_run: false,
        with_authorization: false,
    };
    
    blacklake_cli::cmd::init::init_command(args).await.unwrap();
    
    let bl_dir = test_dir.join(".bl");
    assert!(bl_dir.exists());
    
    // Should include level1.txt and level2.txt, but not level3.txt (too deep)
    assert!(bl_dir.join("level1.txt.metadata.yaml").exists());
    assert!(bl_dir.join("subdir").join("level2.txt.metadata.yaml").exists());
    assert!(!bl_dir.join("subdir").join("subdir2").join("level3.txt.metadata.yaml").exists());
}

#[tokio::test]
async fn test_init_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("data.txt");
    fs::write(&test_file, "test data").unwrap();
    
    let args = blacklake_cli::cmd::init::InitArgs {
        path: test_file.to_string_lossy().to_string(),
        recursive: false,
        max_depth: 1,
        include_hidden: false,
        follow_symlinks: false,
        namespace: "default".to_string(),
        label: vec![],
        meta: vec![],
        class: "restricted".to_string(),
        owner: None,
        no_hash: false,
        hash: "blake3,sha256".to_string(),
        set: vec![],
        overwrite: false,
        dry_run: true,
        with_authorization: false,
    };
    
    blacklake_cli::cmd::init::init_command(args).await.unwrap();
    
    // Verify no files were actually created in dry run
    let metadata_file = test_file.with_extension("bl.metadata.yaml");
    assert!(!metadata_file.exists());
}

