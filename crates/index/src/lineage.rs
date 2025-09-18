use blacklake_core::{EntryLineage, LineageType};
use chrono::Utc;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryLineage {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub commit_id: Uuid,
    pub path: String,
    pub parent_paths: Vec<String>,
    pub child_paths: Vec<String>,
    pub lineage_type: LineageType,
    pub lineage_metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineageType {
    Derived,
    Transformed,
    Aggregated,
    Filtered,
    Joined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoQuota {
    pub repo_id: Uuid,
    pub max_size_bytes: i64,
    pub max_files: i32,
    pub max_commits: i32,
    pub current_size_bytes: i64,
    pub current_files: i32,
    pub current_commits: i32,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQuota {
    pub user_id: String,
    pub max_repos: i32,
    pub max_total_size_bytes: i64,
    pub current_repos: i32,
    pub current_total_size_bytes: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaUsageLog {
    pub id: i64,
    pub user_id: String,
    pub repo_id: Option<Uuid>,
    pub action: String,
    pub size_delta: i64,
    pub file_delta: i32,
    pub commit_delta: i32,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Error, Debug)]
pub enum LineageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
    #[error("Lineage not found: {0}")]
    LineageNotFound(String),
}

pub struct LineageClient {
    pool: PgPool,
}

impl LineageClient {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create lineage relationship
    pub async fn create_lineage(
        &self,
        repo_id: Uuid,
        commit_id: Uuid,
        path: &str,
        parent_paths: Vec<String>,
        child_paths: Vec<String>,
        lineage_type: LineageType,
        lineage_metadata: Option<serde_json::Value>,
    ) -> Result<EntryLineage, LineageError> {
        let lineage_type_str = match lineage_type {
            LineageType::Derived => "derived",
            LineageType::Transformed => "transformed",
            LineageType::Aggregated => "aggregated",
            LineageType::Filtered => "filtered",
            LineageType::Joined => "joined",
        };

        let row = sqlx::query!(
            "INSERT INTO entry_lineage (repo_id, commit_id, path, parent_paths, child_paths, lineage_type, lineage_metadata)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, created_at",
            repo_id,
            commit_id,
            path,
            &parent_paths,
            &child_paths,
            lineage_type_str,
            lineage_metadata
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(EntryLineage {
            id: row.id,
            repo_id,
            commit_id,
            path: path.to_string(),
            parent_paths,
            child_paths,
            lineage_type,
            lineage_metadata,
            created_at: row.created_at,
        })
    }

    /// Get lineage for a specific path
    pub async fn get_lineage(
        &self,
        repo_id: Uuid,
        commit_id: Uuid,
        path: &str,
    ) -> Result<Option<EntryLineage>, LineageError> {
        let row = sqlx::query!(
            "SELECT id, repo_id, commit_id, path, parent_paths, child_paths, lineage_type, lineage_metadata, created_at
             FROM entry_lineage
             WHERE repo_id = $1 AND commit_id = $2 AND path = $3",
            repo_id,
            commit_id,
            path
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| EntryLineage {
            id: r.id,
            repo_id: r.repo_id,
            commit_id: r.commit_id,
            path: r.path,
            parent_paths: r.parent_paths,
            child_paths: r.child_paths,
            lineage_type: match r.lineage_type.as_str() {
                "derived" => LineageType::Derived,
                "transformed" => LineageType::Transformed,
                "aggregated" => LineageType::Aggregated,
                "filtered" => LineageType::Filtered,
                "joined" => LineageType::Joined,
                _ => LineageType::Derived,
            },
            lineage_metadata: r.lineage_metadata,
            created_at: r.created_at,
        }))
    }

    /// Get lineage graph for a repository
    pub async fn get_lineage_graph(
        &self,
        repo_id: Uuid,
        commit_id: Uuid,
    ) -> Result<HashMap<String, EntryLineage>, LineageError> {
        let rows = sqlx::query!(
            "SELECT id, repo_id, commit_id, path, parent_paths, child_paths, lineage_type, lineage_metadata, created_at
             FROM entry_lineage
             WHERE repo_id = $1 AND commit_id = $2",
            repo_id,
            commit_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut graph = HashMap::new();
        for row in rows {
            let lineage = EntryLineage {
                id: row.id,
                repo_id: row.repo_id,
                commit_id: row.commit_id,
                path: row.path.clone(),
                parent_paths: row.parent_paths,
                child_paths: row.child_paths,
                lineage_type: match row.lineage_type.as_str() {
                    "derived" => LineageType::Derived,
                    "transformed" => LineageType::Transformed,
                    "aggregated" => LineageType::Aggregated,
                    "filtered" => LineageType::Filtered,
                    "joined" => LineageType::Joined,
                    _ => LineageType::Derived,
                },
                lineage_metadata: row.lineage_metadata,
                created_at: row.created_at,
            };
            graph.insert(row.path, lineage);
        }

        Ok(graph)
    }

    /// Check repository quota
    pub async fn check_repo_quota(
        &self,
        repo_id: Uuid,
        size_delta: i64,
        file_delta: i32,
        commit_delta: i32,
    ) -> Result<bool, LineageError> {
        let result = sqlx::query!(
            "SELECT check_repo_quota($1, $2, $3, $4) as allowed",
            repo_id,
            size_delta,
            file_delta,
            commit_delta
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.allowed.unwrap_or(false))
    }

    /// Check user quota
    pub async fn check_user_quota(
        &self,
        user_id: &str,
        repo_delta: i32,
        size_delta: i64,
    ) -> Result<bool, LineageError> {
        let result = sqlx::query!(
            "SELECT check_user_quota($1, $2, $3) as allowed",
            user_id,
            repo_delta,
            size_delta
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.allowed.unwrap_or(false))
    }

    /// Update quota usage
    pub async fn update_quota_usage(
        &self,
        user_id: &str,
        repo_id: Uuid,
        action: &str,
        size_delta: i64,
        file_delta: i32,
        commit_delta: i32,
    ) -> Result<(), LineageError> {
        sqlx::query!(
            "SELECT update_quota_usage($1, $2, $3, $4, $5, $6)",
            user_id,
            repo_id,
            action,
            size_delta,
            file_delta,
            commit_delta
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get repository quota
    pub async fn get_repo_quota(&self, repo_id: Uuid) -> Result<Option<RepoQuota>, LineageError> {
        let row = sqlx::query!(
            "SELECT repo_id, max_size_bytes, max_files, max_commits, current_size_bytes, current_files, current_commits, created_at, updated_at
             FROM repo_quota
             WHERE repo_id = $1",
            repo_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| RepoQuota {
            repo_id: r.repo_id,
            max_size_bytes: r.max_size_bytes,
            max_files: r.max_files,
            max_commits: r.max_commits,
            current_size_bytes: r.current_size_bytes,
            current_files: r.current_files,
            current_commits: r.current_commits,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Get user quota
    pub async fn get_user_quota(&self, user_id: &str) -> Result<Option<UserQuota>, LineageError> {
        let row = sqlx::query!(
            "SELECT user_id, max_repos, max_total_size_bytes, current_repos, current_total_size_bytes, created_at, updated_at
             FROM user_quota
             WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| UserQuota {
            user_id: r.user_id,
            max_repos: r.max_repos,
            max_total_size_bytes: r.max_total_size_bytes,
            current_repos: r.current_repos,
            current_total_size_bytes: r.current_total_size_bytes,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Get quota usage history
    pub async fn get_quota_usage_history(
        &self,
        user_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<QuotaUsageLog>, LineageError> {
        let limit = limit.unwrap_or(100);
        
        let rows = sqlx::query!(
            "SELECT id, user_id, repo_id, action, size_delta, file_delta, commit_delta, created_at
             FROM quota_usage_log
             WHERE user_id = $1
             ORDER BY created_at DESC
             LIMIT $2",
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| QuotaUsageLog {
            id: r.id,
            user_id: r.user_id,
            repo_id: r.repo_id,
            action: r.action,
            size_delta: r.size_delta,
            file_delta: r.file_delta,
            commit_delta: r.commit_delta,
            created_at: r.created_at,
        }).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineage_type_serialization() {
        let lineage_type = LineageType::Derived;
        let serialized = serde_json::to_string(&lineage_type).unwrap();
        assert_eq!(serialized, "\"Derived\"");
        
        let deserialized: LineageType = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, LineageType::Derived));
    }

    #[test]
    fn test_repo_quota_creation() {
        let quota = RepoQuota {
            repo_id: Uuid::new_v4(),
            max_size_bytes: 10737418240, // 10GB
            max_files: 10000,
            max_commits: 1000,
            current_size_bytes: 0,
            current_files: 0,
            current_commits: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        assert_eq!(quota.max_size_bytes, 10737418240);
        assert_eq!(quota.max_files, 10000);
        assert_eq!(quota.max_commits, 1000);
    }

    #[test]
    fn test_user_quota_creation() {
        let quota = UserQuota {
            user_id: "test@example.com".to_string(),
            max_repos: 10,
            max_total_size_bytes: 107374182400, // 100GB
            current_repos: 0,
            current_total_size_bytes: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        assert_eq!(quota.user_id, "test@example.com");
        assert_eq!(quota.max_repos, 10);
        assert_eq!(quota.max_total_size_bytes, 107374182400);
    }
}
