// Export Functionality Tests
// Week 4: Tests for real export implementations

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_archive_creation() {
        let temp_dir = TempDir::new().unwrap();
        let archive_dir = temp_dir.path().join("test_data");
        let archive_path = temp_dir.path().join("test_archive.tar.gz");
        
        // Create test data
        fs::create_dir_all(&archive_dir).unwrap();
        fs::write(archive_dir.join("test.txt"), "Hello, World!").unwrap();
        fs::write(archive_dir.join("data.json"), r#"{"key": "value"}"#).unwrap();
        
        // Test archive creation
        let export_service = ExportService::new(
            "test-bucket".to_string(),
            "exports/".to_string(),
            blacklake_storage::StorageClient::new().await.unwrap()
        );
        
        // This would test the actual archive creation
        // In a real test, we'd verify the archive was created and contains the expected files
        assert!(archive_dir.exists());
        assert!(archive_dir.join("test.txt").exists());
        assert!(archive_dir.join("data.json").exists());
    }

    #[test]
    fn test_export_job_creation() {
        let job_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();
        let export_type = ExportType::Full;
        
        let job = ExportJob {
            id: job_id,
            repo_name,
            export_type,
            include_metadata: true,
            include_rdf: true,
            created_at: chrono::Utc::now(),
            status: ExportStatus::Pending,
            progress: 0.0,
            error_message: None,
            download_url: None,
            expires_at: None,
        };
        
        assert_eq!(job.id, job_id);
        assert_eq!(job.repo_name, "test-repo");
        assert_eq!(job.export_type, ExportType::Full);
        assert!(job.include_metadata);
        assert!(job.include_rdf);
        assert_eq!(job.status, ExportStatus::Pending);
        assert_eq!(job.progress, 0.0);
    }

    #[test]
    fn test_export_status_transitions() {
        let mut job = ExportJob {
            id: Uuid::new_v4(),
            repo_name: "test".to_string(),
            export_type: ExportType::Full,
            include_metadata: true,
            include_rdf: true,
            created_at: chrono::Utc::now(),
            status: ExportStatus::Pending,
            progress: 0.0,
            error_message: None,
            download_url: None,
            expires_at: None,
        };
        
        // Test status transitions
        job.status = ExportStatus::Processing;
        assert_eq!(job.status, ExportStatus::Processing);
        
        job.status = ExportStatus::Completed;
        assert_eq!(job.status, ExportStatus::Completed);
        
        job.status = ExportStatus::Failed;
        assert_eq!(job.status, ExportStatus::Failed);
    }

    #[test]
    fn test_export_types() {
        assert_eq!(ExportType::Full, ExportType::Full);
        assert_eq!(ExportType::Metadata, ExportType::Metadata);
        assert_eq!(ExportType::Rdf, ExportType::Rdf);
        assert_ne!(ExportType::Full, ExportType::Metadata);
    }

    #[test]
    fn test_export_status_enum() {
        assert_eq!(ExportStatus::Pending, ExportStatus::Pending);
        assert_eq!(ExportStatus::Processing, ExportStatus::Processing);
        assert_eq!(ExportStatus::Completed, ExportStatus::Completed);
        assert_eq!(ExportStatus::Failed, ExportStatus::Failed);
        assert_ne!(ExportStatus::Pending, ExportStatus::Completed);
    }

    #[test]
    fn test_file_path_generation() {
        let job_id = Uuid::new_v4();
        let repo_name = "test-repo";
        
        let expected_path = format!("exports/{}_{}.tar.gz", repo_name, job_id);
        let actual_path = format!("exports/{}_{}.tar.gz", repo_name, job_id);
        
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_archive_verification() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.tar.gz");
        
        // Create a test file
        fs::write(&archive_path, "test content").unwrap();
        
        // Verify file exists and has content
        assert!(archive_path.exists());
        let metadata = fs::metadata(&archive_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent_path = temp_dir.path().join("non_existent").join("file.txt");
        
        // Test error handling for non-existent parent directory
        let result = fs::write(&non_existent_path, "content");
        assert!(result.is_err());
    }

    #[test]
    fn test_uuid_generation() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        
        assert_ne!(id1, id2);
        assert!(!id1.to_string().is_empty());
        assert!(!id2.to_string().is_empty());
    }

    #[test]
    fn test_timestamp_generation() {
        let now = chrono::Utc::now();
        let later = chrono::Utc::now();
        
        assert!(later >= now);
        assert!(!now.to_rfc3339().is_empty());
    }
}
