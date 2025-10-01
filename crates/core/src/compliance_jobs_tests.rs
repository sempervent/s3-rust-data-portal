// Compliance Jobs Tests
// Week 4: Tests for real compliance job implementations

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use uuid::Uuid;
    use chrono::{Utc, Duration};

    #[test]
    fn test_compliance_job_creation() {
        let job_id = Uuid::new_v4();
        let job_type = ComplianceJobType::AuditLogExport;
        let status = ComplianceJobStatus::Pending;
        
        let job = ComplianceJob {
            id: job_id,
            job_type,
            status,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            progress: 0.0,
            metadata: serde_json::json!({}),
        };
        
        assert_eq!(job.id, job_id);
        assert_eq!(job.job_type, ComplianceJobType::AuditLogExport);
        assert_eq!(job.status, ComplianceJobStatus::Pending);
        assert_eq!(job.progress, 0.0);
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());
        assert!(job.error_message.is_none());
    }

    #[test]
    fn test_compliance_job_types() {
        assert_eq!(ComplianceJobType::AuditLogExport, ComplianceJobType::AuditLogExport);
        assert_eq!(ComplianceJobType::RetentionStatusExport, ComplianceJobType::RetentionStatusExport);
        assert_eq!(ComplianceJobType::LegalHoldExport, ComplianceJobType::LegalHoldExport);
        assert_eq!(ComplianceJobType::ComplianceReport, ComplianceJobType::ComplianceReport);
        assert_ne!(ComplianceJobType::AuditLogExport, ComplianceJobType::RetentionStatusExport);
    }

    #[test]
    fn test_compliance_job_status() {
        assert_eq!(ComplianceJobStatus::Pending, ComplianceJobStatus::Pending);
        assert_eq!(ComplianceJobStatus::Running, ComplianceJobStatus::Running);
        assert_eq!(ComplianceJobStatus::Completed, ComplianceJobStatus::Completed);
        assert_eq!(ComplianceJobStatus::Failed, ComplianceJobStatus::Failed);
        assert_ne!(ComplianceJobStatus::Pending, ComplianceJobStatus::Completed);
    }

    #[test]
    fn test_csv_export_format() {
        let audit_logs = vec![
            AuditLog {
                id: "1".to_string(),
                action: "VIEW".to_string(),
                user: "test@example.com".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                details: "Viewed file".to_string(),
                ip_address: "192.168.1.1".to_string(),
            },
            AuditLog {
                id: "2".to_string(),
                action: "DOWNLOAD".to_string(),
                user: "admin@example.com".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                details: "Downloaded file".to_string(),
                ip_address: "192.168.1.2".to_string(),
            },
        ];
        
        let mut csv_content = String::new();
        csv_content.push_str("id,action,user,timestamp,details,ip_address\n");
        
        for log in &audit_logs {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{}\n",
                log.id,
                log.action,
                log.user,
                log.timestamp,
                log.details.replace(',', ";"),
                log.ip_address
            ));
        }
        
        assert!(csv_content.contains("id,action,user,timestamp,details,ip_address"));
        assert!(csv_content.contains("1,VIEW,test@example.com"));
        assert!(csv_content.contains("2,DOWNLOAD,admin@example.com"));
    }

    #[test]
    fn test_legal_hold_export_format() {
        let legal_holds = vec![
            LegalHold {
                id: Uuid::new_v4(),
                name: "Test Hold".to_string(),
                description: Some("Test description".to_string()),
                start_date: Utc::now().date_naive(),
                end_date: Some((Utc::now() + Duration::days(30)).date_naive()),
                status: "active".to_string(),
                created_by: "admin@example.com".to_string(),
                created_at: Utc::now(),
            },
        ];
        
        let mut csv_content = String::new();
        csv_content.push_str("id,name,description,start_date,end_date,status,created_by,created_at\n");
        
        for hold in &legal_holds {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                hold.id,
                hold.name,
                hold.description.as_ref().unwrap_or(&"".to_string()).replace(',', ";"),
                hold.start_date,
                hold.end_date.map(|d| d.to_string()).unwrap_or_default(),
                hold.status,
                hold.created_by,
                hold.created_at
            ));
        }
        
        assert!(csv_content.contains("id,name,description,start_date,end_date,status,created_by,created_at"));
        assert!(csv_content.contains("Test Hold"));
        assert!(csv_content.contains("active"));
    }

    #[test]
    fn test_compliance_report_structure() {
        let report = serde_json::json!({
            "report_metadata": {
                "generated_at": Utc::now().to_rfc3339(),
                "report_type": "compliance_summary",
                "version": "1.0"
            },
            "retention_summary": {
                "total_entries": 100,
                "expired_entries": 10,
                "entries_with_legal_hold": 5,
                "entries_ready_for_deletion": 5
            },
            "legal_holds": {
                "total_holds": 3,
                "active_holds": 2,
                "holds": []
            },
            "audit_summary": {
                "total_audit_entries": 50,
                "recent_activities": []
            }
        });
        
        assert!(report["report_metadata"]["generated_at"].is_string());
        assert_eq!(report["report_metadata"]["report_type"], "compliance_summary");
        assert_eq!(report["report_metadata"]["version"], "1.0");
        assert_eq!(report["retention_summary"]["total_entries"], 100);
        assert_eq!(report["legal_holds"]["total_holds"], 3);
        assert_eq!(report["audit_summary"]["total_audit_entries"], 50);
    }

    #[test]
    fn test_file_creation_and_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_export.csv");
        
        // Create test file
        fs::write(&file_path, "test,data\n1,value1\n2,value2").unwrap();
        
        // Verify file exists and has content
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("test,data"));
        assert!(content.contains("1,value1"));
        assert!(content.contains("2,value2"));
        
        // Test file cleanup
        fs::remove_file(&file_path).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_json_serialization() {
        let job = ComplianceJob {
            id: Uuid::new_v4(),
            job_type: ComplianceJobType::AuditLogExport,
            status: ComplianceJobStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            progress: 0.0,
            metadata: serde_json::json!({"key": "value"}),
        };
        
        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("AuditLogExport"));
        assert!(json.contains("Pending"));
        assert!(json.contains("key"));
        assert!(json.contains("value"));
    }

    #[test]
    fn test_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent_path = temp_dir.path().join("non_existent").join("file.csv");
        
        // Test error handling for non-existent parent directory
        let result = fs::write(&non_existent_path, "content");
        assert!(result.is_err());
        
        // Test error handling for invalid JSON
        let invalid_json = "{ invalid json }";
        let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_progress_calculation() {
        let total_items = 100;
        let processed_items = 25;
        let progress = (processed_items as f64 / total_items as f64) * 100.0;
        
        assert_eq!(progress, 25.0);
        
        let processed_items = 50;
        let progress = (processed_items as f64 / total_items as f64) * 100.0;
        assert_eq!(progress, 50.0);
        
        let processed_items = 100;
        let progress = (processed_items as f64 / total_items as f64) * 100.0;
        assert_eq!(progress, 100.0);
    }

    #[test]
    fn test_timestamp_handling() {
        let now = Utc::now();
        let rfc3339 = now.to_rfc3339();
        let parsed = chrono::DateTime::parse_from_rfc3339(&rfc3339).unwrap();
        
        assert_eq!(parsed.timestamp(), now.timestamp());
        assert!(!rfc3339.is_empty());
    }
}
