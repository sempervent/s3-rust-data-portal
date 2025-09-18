#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn test_index_entry_job_serialization() {
        let job = IndexEntryJob {
            repo_id: Uuid::new_v4(),
            repo_name: "test-repo".to_string(),
            ref_name: "main".to_string(),
            path: "data/test.csv".to_string(),
            commit_id: Uuid::new_v4(),
            object_sha256: "abc123".to_string(),
            metadata: json!({
                "title": "Test Dataset",
                "description": "A test dataset",
                "tags": ["test", "csv"]
            }),
            operation: IndexOperation::Index,
        };

        let serialized = serde_json::to_string(&job).unwrap();
        let deserialized: IndexEntryJob = serde_json::from_str(&serialized).unwrap();

        assert_eq!(job.repo_name, deserialized.repo_name);
        assert_eq!(job.path, deserialized.path);
        assert_eq!(job.operation, deserialized.operation);
    }

    #[test]
    fn test_sampling_job_creation() {
        let job = SamplingJob {
            repo_id: Uuid::new_v4(),
            repo_name: "test-repo".to_string(),
            path: "data/test.csv".to_string(),
            commit_id: Uuid::new_v4(),
            object_sha256: "abc123".to_string(),
            file_type: "csv".to_string(),
        };

        assert_eq!(job.file_type, "csv");
        assert_eq!(job.path, "data/test.csv");
    }

    #[test]
    fn test_rdf_emission_job_formats() {
        let job = RdfEmissionJob {
            repo_id: Uuid::new_v4(),
            repo_name: "test-repo".to_string(),
            path: "data/test.csv".to_string(),
            commit_id: Uuid::new_v4(),
            metadata: json!({
                "title": "Test Dataset",
                "creator": "Test User"
            }),
            formats: vec!["jsonld".to_string(), "turtle".to_string()],
        };

        assert_eq!(job.formats.len(), 2);
        assert!(job.formats.contains(&"jsonld".to_string()));
        assert!(job.formats.contains(&"turtle".to_string()));
    }

    #[test]
    fn test_antivirus_scan_job_large_file() {
        let job = AntivirusScanJob {
            repo_id: Uuid::new_v4(),
            repo_name: "test-repo".to_string(),
            path: "data/large.bin".to_string(),
            object_sha256: "abc123".to_string(),
            file_size: 200 * 1024 * 1024, // 200MB
        };

        assert!(job.file_size > 100 * 1024 * 1024);
    }

    #[test]
    fn test_export_job_manifest() {
        let manifest = json!({
            "include_metadata": true,
            "include_rdf": false,
            "filters": {
                "file_types": ["csv", "parquet"],
                "date_range": {
                    "from": "2024-01-01",
                    "to": "2024-12-31"
                }
            }
        });

        let job = ExportJob {
            export_id: Uuid::new_v4(),
            repo_id: Uuid::new_v4(),
            repo_name: "test-repo".to_string(),
            manifest,
            include_metadata: true,
            include_rdf: false,
        };

        assert!(job.include_metadata);
        assert!(!job.include_rdf);
    }

    #[test]
    fn test_job_error_serialization() {
        let error = JobError::Processing("Test error message".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Test error message"));
    }

    #[test]
    fn test_job_metadata_creation() {
        let metadata = JobMetadata {
            id: JobId::new(),
            job_type: "index_entry".to_string(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            attempts: 0,
            max_attempts: 3,
            status: JobStatus::Pending,
            error_message: None,
            progress: 0.0,
            metadata: json!({}),
        };

        assert_eq!(metadata.job_type, "index_entry");
        assert_eq!(metadata.status, JobStatus::Pending);
        assert_eq!(metadata.attempts, 0);
    }

    #[test]
    fn test_job_status_transitions() {
        let mut status = JobStatus::Pending;
        assert_eq!(status, JobStatus::Pending);

        status = JobStatus::Processing;
        assert_eq!(status, JobStatus::Processing);

        status = JobStatus::Completed;
        assert_eq!(status, JobStatus::Completed);
    }

    #[tokio::test]
    async fn test_index_entry_job_trait() {
        let job = IndexEntryJob {
            repo_id: Uuid::new_v4(),
            repo_name: "test-repo".to_string(),
            ref_name: "main".to_string(),
            path: "data/test.csv".to_string(),
            commit_id: Uuid::new_v4(),
            object_sha256: "abc123".to_string(),
            metadata: json!({}),
            operation: IndexOperation::Index,
        };

        assert_eq!(IndexEntryJob::job_type(), "index_entry");
        assert_eq!(IndexEntryJob::max_attempts(), 5);
        assert_eq!(IndexEntryJob::timeout(), Duration::from_secs(120));
    }

    #[tokio::test]
    async fn test_sampling_job_trait() {
        assert_eq!(SamplingJob::job_type(), "sampling");
        assert_eq!(SamplingJob::max_attempts(), 3);
        assert_eq!(SamplingJob::retry_delay(), Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_rdf_emission_job_trait() {
        assert_eq!(RdfEmissionJob::job_type(), "rdf_emission");
        assert_eq!(RdfEmissionJob::max_attempts(), 3);
        assert_eq!(RdfEmissionJob::retry_delay(), Duration::from_secs(45));
    }

    #[tokio::test]
    async fn test_antivirus_scan_job_trait() {
        assert_eq!(AntivirusScanJob::job_type(), "antivirus_scan");
        assert_eq!(AntivirusScanJob::max_attempts(), 2);
        assert_eq!(AntivirusScanJob::retry_delay(), Duration::from_secs(120));
    }

    #[tokio::test]
    async fn test_export_job_trait() {
        assert_eq!(ExportJob::job_type(), "export");
        assert_eq!(ExportJob::max_attempts(), 1);
        assert_eq!(ExportJob::timeout(), Duration::from_secs(1800));
    }
}
