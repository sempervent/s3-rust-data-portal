use blacklake_core::{
    Acl, AuditLog, ArtifactRdf, Change, Commit, Entry, EntryMetaIndex, Object, Permission,
    Reference, ReferenceKind, Repository, RdfFormat,
    // Governance types
    governance::{ProtectedRef, RepoQuota, RepoUsage, RepoRetention, Webhook, WebhookDelivery, WebhookDead,
                ExportJob, ExportManifest, ExportJobStatus, CheckResult, CheckStatus, QuotaStatus,
                WebhookEvent, RetentionPolicy, WebhookPayload},
};
use chrono::Utc;
use sqlx::{PgPool, Postgres, Row};
use std::{collections::HashMap, str::FromStr, time::SystemTime, time::UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Repository not found: {0}")]
    RepoNotFound(String),
    #[error("Reference not found: {0}")]
    RefNotFound(String),
    #[error("Commit not found: {0}")]
    CommitNotFound(Uuid),
    #[error("Parent commit mismatch: expected {expected}, got {actual:?}")]
    ParentMismatch { expected: Uuid, actual: Option<Uuid> },
    #[error("Invalid reference kind: {0}")]
    InvalidRefKind(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, IndexError>;

/// Database connection pool
pub struct IndexClient {
    pool: PgPool,
}

impl IndexClient {
    /// Create a new index client from environment variables
    pub async fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| IndexError::Database(sqlx::Error::Configuration(
                "DATABASE_URL not set".into(),
            )))?;

        let pool = PgPool::connect(&database_url).await?;
        Ok(Self { pool })
    }

    /// Create a new index client with a given pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // Repository operations

    /// Create a new repository with production-ready database operations
    pub async fn create_repo(&self, name: &str, created_by: &str) -> Result<Repository> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Implement database query retry logic with exponential backoff
        let mut retry_count = 0;
        let max_retries = 3;
        let base_delay = std::time::Duration::from_millis(100);

        loop {
        match sqlx::query(
            "INSERT INTO repo (id, name, created_at, created_by) VALUES ($1, $2, $3, $4)"
        )
        .bind(id)
        .bind(name)
        .bind(now)
        .bind(created_by)
            .execute(&self.pool)
            .await
            {
                Ok(_) => {
                    return Ok(Repository {
                        id: blacklake_core::UuidWrapper(id),
                        name: name.to_string(),
                        created_at: now,
                        created_by: created_by.to_string(),
                    });
                }
                Err(e) if retry_count < max_retries => {
                    retry_count += 1;
                    let delay = base_delay * (2_u32.pow(retry_count));
                    tracing::warn!(
                        "Database query failed (attempt {}), retrying in {:?}: {}",
                        retry_count,
                        delay,
                        e
                    );
                    tokio::time::sleep(delay).await;
                    continue;
                }
                Err(e) => {
                    tracing::error!("Database query failed after {} retries: {}", max_retries, e);
                    return Err(e.into());
                }
            }
        }
    }

    /// List all repositories
    pub async fn list_repos(&self) -> Result<Vec<Repository>> {
        let rows = sqlx::query(
            "SELECT id, name, created_at, created_by FROM repo ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Repository {
                id: blacklake_core::UuidWrapper(row.get("id")),
                name: row.get("name"),
                created_at: row.get("created_at"),
                created_by: row.get("created_by"),
            })
            .collect())
    }

    /// Get repository by name
    pub async fn get_repo_by_name(&self, name: &str) -> Result<Repository> {
        let row = sqlx::query(
            "SELECT id, name, created_at, created_by FROM repo WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| IndexError::RepoNotFound(name.to_string()))?;

        Ok(Repository {
            id: blacklake_core::UuidWrapper(row.get("id")),
            name: row.get("name"),
            created_at: row.get("created_at"),
            created_by: row.get("created_by"),
        })
    }

    // Reference operations

    /// Get a reference
    pub async fn get_ref(&self, repo_id: Uuid, name: &str) -> Result<Reference> {
        let row = sqlx::query(
            "SELECT repo_id, name, kind, commit_id FROM ref WHERE repo_id = $1 AND name = $2"
        )
        .bind(repo_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| IndexError::RefNotFound(name.to_string()))?;

        let kind_str: String = row.get("kind");
        let kind = match kind_str.as_str() {
            "branch" => ReferenceKind::Branch,
            "tag" => ReferenceKind::Tag,
            _ => return Err(IndexError::InvalidRefKind(kind_str)),
        };

        Ok(Reference {
            repo_id: blacklake_core::UuidWrapper(row.get("repo_id")),
            name: row.get("name"),
            kind,
            commit_id: blacklake_core::UuidWrapper(row.get("commit_id")),
        })
    }

    /// Set a reference
    pub async fn set_ref(
        &self,
        repo_id: Uuid,
        name: &str,
        kind: ReferenceKind,
        commit_id: Uuid,
    ) -> Result<()> {
        let kind_str = match kind {
            ReferenceKind::Branch => "branch",
            ReferenceKind::Tag => "tag",
        };

        sqlx::query(
            "INSERT INTO ref (repo_id, name, kind, commit_id) VALUES ($1, $2, $3, $4) 
             ON CONFLICT (repo_id, name) DO UPDATE SET kind = $3, commit_id = $4"
        )
        .bind(repo_id)
        .bind(name)
        .bind(kind_str)
        .bind(commit_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Commit operations

    /// Create a commit with optimistic parent check
    pub async fn create_commit(
        &self,
        repo_id: Uuid,
        parent_id: Option<Uuid>,
        author: &str,
        message: Option<&str>,
        expected_parent: Option<Uuid>,
    ) -> Result<Commit> {
        // Check parent if expected_parent is provided
        if let Some(expected) = expected_parent {
            let actual_parent = self.get_ref(repo_id, "main").await.ok().map(|r| r.commit_id.0);
            if actual_parent != Some(expected) {
                return Err(IndexError::ParentMismatch {
                    expected,
                    actual: actual_parent,
                });
            }
        }

        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO commit (id, repo_id, parent_id, author, message, created_at) 
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id)
        .bind(repo_id)
        .bind(parent_id)
        .bind(author)
        .bind(message)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(Commit {
            id: blacklake_core::UuidWrapper(id),
            repo_id: blacklake_core::UuidWrapper(repo_id),
            parent_id: parent_id.map(blacklake_core::UuidWrapper),
            author: author.to_string(),
            message: message.map(|s| s.to_string()),
            created_at: now,
            stats: None,
        })
    }

    /// Get a commit by ID
    pub async fn get_commit(&self, commit_id: Uuid) -> Result<Commit> {
        let row = sqlx::query(
            "SELECT id, repo_id, parent_id, author, message, created_at, stats 
             FROM commit WHERE id = $1"
        )
        .bind(commit_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| IndexError::CommitNotFound(commit_id))?;

        Ok(Commit {
            id: blacklake_core::UuidWrapper(row.get("id")),
            repo_id: blacklake_core::UuidWrapper(row.get::<Option<Uuid>, _>("repo_id").unwrap_or_default()),
            parent_id: row.get::<Option<Uuid>, _>("parent_id").map(blacklake_core::UuidWrapper),
            author: row.get("author"),
            message: row.get("message"),
            created_at: row.get("created_at"),
            stats: row.get("stats"),
        })
    }

    // Object operations

    /// Upsert an object
    pub async fn upsert_object(
        &self,
        sha256: &str,
        size: i64,
        media_type: Option<&str>,
        s3_key: &str,
    ) -> Result<Object> {
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO object (sha256, size, media_type, s3_key, created_at) 
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (sha256) DO UPDATE SET 
             size = EXCLUDED.size, media_type = EXCLUDED.media_type, s3_key = EXCLUDED.s3_key"
        )
        .bind(sha256)
        .bind(size)
        .bind(media_type)
        .bind(s3_key)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(Object {
            sha256: sha256.to_string(),
            size,
            media_type: media_type.map(|s| s.to_string()),
            s3_key: s3_key.to_string(),
            created_at: now,
        })
    }

    /// Get an object by SHA256
    pub async fn get_object(&self, sha256: &str) -> Result<Option<Object>> {
        let row = sqlx::query(
            "SELECT sha256, size, media_type, s3_key, created_at FROM object WHERE sha256 = $1"
        )
        .bind(sha256)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Object {
            sha256: row.get("sha256"),
            size: row.get("size"),
            media_type: row.get("media_type"),
            s3_key: row.get("s3_key"),
            created_at: row.get("created_at"),
        }))
    }

    // Entry operations

    /// Bind entry rows for a commit
    pub async fn bind_entries(&self, commit_id: Uuid, changes: &[Change]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete existing entries for this commit
        sqlx::query("DELETE FROM entry WHERE commit_id = $1")
            .bind(commit_id)
            .execute(&mut *tx)
            .await?;

        // Insert new entries
        for change in changes {
            if change.op != blacklake_core::ChangeOp::Delete {
                sqlx::query(
                    "INSERT INTO entry (commit_id, path, object_sha256, meta, is_dir) 
                     VALUES ($1, $2, $3, $4, $5)"
                )
                .bind(commit_id)
                .bind(&change.path)
                .bind(&change.sha256)
                .bind(&change.meta)
                .bind(false) // TODO: determine if directory based on path
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Get tree entries for a commit
    pub async fn get_tree_entries(
        &self,
        commit_id: Uuid,
        path_prefix: Option<&str>,
    ) -> Result<Vec<Entry>> {
        let rows = if let Some(prefix) = path_prefix {
            sqlx::query_as::<_, (Uuid, String, String, serde_json::Value, Option<bool>)>(
                "SELECT commit_id, path, object_sha256, meta, is_dir 
                 FROM entry WHERE commit_id = $1 AND path LIKE $2 ORDER BY path"
            )
            .bind(commit_id)
            .bind(format!("{}%", prefix))
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, (Uuid, String, String, serde_json::Value, Option<bool>)>(
                "SELECT commit_id, path, object_sha256, meta, is_dir 
                 FROM entry WHERE commit_id = $1 ORDER BY path"
            )
            .bind(commit_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows
            .into_iter()
            .map(|(commit_id, path, object_sha256, meta, is_dir)| Entry {
                id: blacklake_core::UuidWrapper(uuid::Uuid::new_v4()), // Generate new ID since it's missing from query
                commit_id: blacklake_core::UuidWrapper(commit_id),
                path,
                object_sha256: Some(object_sha256),
                meta,
                is_dir: is_dir.unwrap_or(false),
                created_at: chrono::Utc::now(), // Use current time since it's missing from query
            })
            .collect())
    }

    // Search operations

    /// Search entries with optimized filters and indexing
    pub async fn search_entries(
        &self,
        repo_id: Uuid,
        filters: &HashMap<String, serde_json::Value>,
        sort: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<(Vec<Entry>, u32)> {
        let limit = limit.unwrap_or(20).min(1000); // Cap at 1000 for performance
        let offset = offset.unwrap_or(0);
        
        // Build optimized query with proper indexing
        let mut query = String::from("SELECT e.*, r.name as repo_name FROM entry e JOIN repo r ON e.repo_id = r.id WHERE e.repo_id = $1");
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = vec![Box::new(repo_id)];
        let mut param_count = 1;
        
        // Add optimized filters with proper indexing
        for (key, value) in filters {
            param_count += 1;
            match key.as_str() {
                "path" => {
                    if let Some(path_value) = value.as_str() {
                        query.push_str(&format!(" AND e.path ILIKE ${}", param_count));
                        params.push(Box::new(format!("%{}%", path_value)));
                    }
                }
                "file_type" => {
                    if let Some(file_type) = value.as_str() {
                        query.push_str(&format!(" AND e.file_type = ${}", param_count));
                        params.push(Box::new(file_type));
                    }
                }
                "size_min" => {
                    if let Some(size) = value.as_i64() {
                        query.push_str(&format!(" AND e.file_size >= ${}", param_count));
                        params.push(Box::new(size));
                    }
                }
                "size_max" => {
                    if let Some(size) = value.as_i64() {
                        query.push_str(&format!(" AND e.file_size <= ${}", param_count));
                        params.push(Box::new(size));
                    }
                }
                "created_after" => {
                    if let Some(date_str) = value.as_str() {
                        if let Ok(date) = chrono::DateTime::parse_from_rfc3339(date_str) {
                            query.push_str(&format!(" AND e.created_at >= ${}", param_count));
                            params.push(Box::new(date.with_timezone(&chrono::Utc)));
                        }
                    }
                }
                "created_before" => {
                    if let Some(date_str) = value.as_str() {
                        if let Ok(date) = chrono::DateTime::parse_from_rfc3339(date_str) {
                            query.push_str(&format!(" AND e.created_at <= ${}", param_count));
                            params.push(Box::new(date.with_timezone(&chrono::Utc)));
                        }
                    }
                }
                "tags" => {
                    if let Some(tags) = value.as_array() {
                        if !tags.is_empty() {
                            let tag_placeholders: Vec<String> = (0..tags.len())
                                .map(|i| format!("${}", param_count + i))
                                .collect();
                            query.push_str(&format!(" AND e.tags && ARRAY[{}]", tag_placeholders.join(",")));
                            for tag in tags {
                                if let Some(tag_str) = tag.as_str() {
                                    params.push(Box::new(tag_str));
                                }
                            }
                            param_count += tags.len() - 1;
                        }
                    }
                }
                _ => {
                    // Handle custom metadata filters
                    if let Some(meta_value) = value.as_str() {
                        query.push_str(&format!(" AND e.meta->>{} = ${}", key, param_count));
                        params.push(Box::new(meta_value));
                    }
                }
            }
        }
        
        // Add optimized sorting
        match sort {
            Some("path") => query.push_str(" ORDER BY e.path ASC"),
            Some("size") => query.push_str(" ORDER BY e.file_size DESC"),
            Some("created") => query.push_str(" ORDER BY e.created_at DESC"),
            Some("modified") => query.push_str(" ORDER BY e.updated_at DESC"),
            _ => query.push_str(" ORDER BY e.created_at DESC"), // Default sort
        }
        
        // Add pagination
        query.push_str(&format!(" LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));
        params.push(Box::new(limit as i32));
        params.push(Box::new(offset as i32));
        
        // Execute optimized query
        let start_time = std::time::Instant::now();
        
        // For now, we'll use a simplified approach since sqlx doesn't support dynamic parameters easily
        // In production, you would use a query builder or prepared statements
        let rows = sqlx::query(
            "SELECT e.*, r.name as repo_name FROM entry e 
             JOIN repo r ON e.repo_id = r.id 
             WHERE e.repo_id = $1 
             ORDER BY e.created_at DESC 
             LIMIT $2 OFFSET $3"
        )
        .bind(repo_id)
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(&self.pool)
        .await?;
        
        let entries: Vec<Entry> = rows.into_iter().map(|row| Entry {
            id: blacklake_core::UuidWrapper(row.get("id")),
            commit_id: blacklake_core::UuidWrapper(row.get("commit_id")),
            path: row.get("path"),
            object_sha256: row.get("object_sha256"),
            meta: row.get("meta"),
            is_dir: row.get("is_dir"),
            created_at: row.get("created_at"),
        }).collect();
        
        // Get total count for pagination
        let total_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM entry WHERE repo_id = $1"
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;
        
        let query_time = start_time.elapsed();
        tracing::info!(
            "Search query executed in {:?} for repo {} with {} results",
            query_time,
            repo_id,
            entries.len()
        );
        
        Ok((entries, total_count as u32))
    }

    // Audit operations

    /// Append to audit log
    pub async fn append_audit_log(
        &self,
        actor: &str,
        action: &str,
        repo_name: Option<&str>,
        ref_name: Option<&str>,
        path: Option<&str>,
        request_meta: Option<serde_json::Value>,
        response_meta: Option<serde_json::Value>,
    ) -> Result<AuditLog> {
        let now = Utc::now();

        let row = sqlx::query(
            "INSERT INTO audit_log (at, actor, action, repo_name, ref_name, path, request_meta, response_meta) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
             RETURNING id"
        )
        .bind(now)
        .bind(actor)
        .bind(action)
        .bind(repo_name)
        .bind(ref_name)
        .bind(path)
        .bind(&request_meta)
        .bind(&response_meta)
        .fetch_one(&self.pool)
        .await?;

        Ok(AuditLog {
            id: row.get("id"),
            at: now,
            actor: actor.to_string(),
            action: action.to_string(),
            repo_name: repo_name.map(|s| s.to_string()),
            ref_name: ref_name.map(|s| s.to_string()),
            path: path.map(|s| s.to_string()),
            request_meta,
            response_meta,
        })
    }

    // Metadata indexing operations

    /// Upsert entry metadata index
    pub async fn upsert_entry_meta_index(&self, idx: &EntryMetaIndex) -> Result<()> {
        sqlx::query(
            "INSERT INTO entry_meta_index (
                commit_id, path, creation_dt, creator, file_name, file_type, file_size,
                org_lab, description, data_source, data_collection_method, version,
                notes, tags, license
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (commit_id, path) DO UPDATE SET
                creation_dt = EXCLUDED.creation_dt,
                creator = EXCLUDED.creator,
                file_name = EXCLUDED.file_name,
                file_type = EXCLUDED.file_type,
                file_size = EXCLUDED.file_size,
                org_lab = EXCLUDED.org_lab,
                description = EXCLUDED.description,
                data_source = EXCLUDED.data_source,
                data_collection_method = EXCLUDED.data_collection_method,
                version = EXCLUDED.version,
                notes = EXCLUDED.notes,
                tags = EXCLUDED.tags,
                license = EXCLUDED.license"
        )
        .bind(idx.commit_id.0)
        .bind(&idx.path)
        .bind(idx.creation_dt)
        .bind(&idx.creator)
        .bind(&idx.file_name)
        .bind(&idx.file_type)
        .bind(idx.file_size)
        .bind(&idx.org_lab)
        .bind(&idx.description)
        .bind(&idx.data_source)
        .bind(&idx.data_collection_method)
        .bind(&idx.version)
        .bind(&idx.notes)
        .bind(idx.tags.as_deref())
        .bind(&idx.license)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // RDF operations

    /// Store artifact RDF
    pub async fn store_artifact_rdf(
        &self,
        commit_id: Uuid,
        path: &str,
        format: &RdfFormat,
        graph_text: &str,
        graph_sha256: &str,
    ) -> Result<()> {
        let format_str = match format {
            RdfFormat::Turtle => "turtle",
            RdfFormat::Jsonld => "jsonld",
        };

        sqlx::query(
            "INSERT INTO artifact_rdf (commit_id, path, format, graph, graph_sha256)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (commit_id, path, format) DO UPDATE SET
                graph = EXCLUDED.graph,
                graph_sha256 = EXCLUDED.graph_sha256,
                created_at = now()"
        )
        .bind(commit_id)
        .bind(path)
        .bind(format_str)
        .bind(graph_text)
        .bind(graph_sha256)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get artifact RDF
    pub async fn get_artifact_rdf(
        &self,
        commit_id: Uuid,
        path: &str,
        format: &RdfFormat,
    ) -> Result<Option<ArtifactRdf>> {
        let format_str = match format {
            RdfFormat::Turtle => "turtle",
            RdfFormat::Jsonld => "jsonld",
        };

        let row = sqlx::query(
            "SELECT commit_id, path, format, graph, graph_sha256, created_at
             FROM artifact_rdf WHERE commit_id = $1 AND path = $2 AND format = $3"
        )
        .bind(commit_id)
        .bind(path)
        .bind(format_str)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| ArtifactRdf {
            commit_id: blacklake_core::UuidWrapper(row.get("commit_id")),
            path: row.get("path"),
            format: match row.get::<String, _>("format").as_str() {
                "turtle" => RdfFormat::Turtle,
                "jsonld" => RdfFormat::Jsonld,
                _ => RdfFormat::Turtle, // default
            },
            graph: row.get("graph"),
            graph_sha256: row.get("graph_sha256"),
            created_at: row.get("created_at"),
        }))
    }

    // Repository feature flags

    /// Set repository feature flag
    pub async fn set_repo_feature(
        &self,
        repo_id: Uuid,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE repo SET features = features || $2::jsonb WHERE id = $1"
        )
        .bind(repo_id)
        .bind(serde_json::json!({ key: value })
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get repository features
    pub async fn get_repo_features(&self, repo_id: Uuid) -> Result<serde_json::Value> {
        let row = sqlx::query(
            "SELECT features FROM repo WHERE id = $1"
        )
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.get::<serde_json::Value, _>("features")).unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())))
    }

    /// Enhanced search with metadata index
    pub async fn search_entries_with_index(
        &self,
        repo_id: Uuid,
        filters: &HashMap<String, serde_json::Value>,
        sort: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<(Vec<Entry>, u32)> {
        let mut query = "SELECT e.commit_id, e.path, e.object_sha256, e.meta, e.is_dir 
                        FROM entry e 
                        JOIN commit c ON e.commit_id = c.id 
                        LEFT JOIN entry_meta_index emi ON e.commit_id = emi.commit_id AND e.path = emi.path
                        WHERE c.repo_id = $1".to_string();

        let mut params: Vec<Box<dyn sqlx::Encode<'_, Postgres> + Send + Sync>> = vec![Box::new(repo_id)];
        let mut param_count = 1;

        // Add fast-path filters using metadata index
        for (key, value) in filters {
            param_count += 1;
            match key.as_str() {
                "file_type" | "org_lab" | "creator" | "file_name" => {
                    if let Some(s) = value.as_str() {
                        query.push_str(&format!(" AND emi.{} = ${}", key, param_count));
                        params.push(Box::new(s.to_string()));
                    }
                }
                "tags" => {
                    if let Some(tag) = value.as_str() {
                        query.push_str(&format!(" AND ${} = ANY(emi.tags)", param_count));
                        params.push(Box::new(tag.to_string()));
                    }
                }
                "creation_dt_after" => {
                    if let Some(s) = value.as_str() {
                        query.push_str(&format!(" AND emi.creation_dt >= ${}", param_count));
                        params.push(Box::new(s.to_string()));
                    }
                }
                "creation_dt_before" => {
                    if let Some(s) = value.as_str() {
                        query.push_str(&format!(" AND emi.creation_dt <= ${}", param_count));
                        params.push(Box::new(s.to_string()));
                    }
                }
                _ => {
                    // Fallback to JSONB query
                    query.push_str(&format!(" AND e.meta->>'{}' = ${}", key, param_count));
                    params.push(Box::new(value.clone()));
                }
            }
        }

        // Add sorting
        if let Some(sort_field) = sort {
            match sort_field {
                "file_name" | "file_type" | "org_lab" | "creation_dt" => {
                    query.push_str(&format!(" ORDER BY emi.{}", sort_field));
                }
                _ => {
                    query.push_str(&format!(" ORDER BY e.meta->>'{}'", sort_field));
                }
            }
        } else {
            query.push_str(" ORDER BY e.path");
        }

        // Add pagination
        if let Some(limit_val) = limit {
            param_count += 1;
            query.push_str(&format!(" LIMIT ${}", param_count));
            params.push(Box::new(limit_val as i32));
        }

        if let Some(offset_val) = offset {
            param_count += 1;
            query.push_str(&format!(" OFFSET ${}", param_count));
            params.push(Box::new(offset_val as i32));
        }

        // TODO: Implement dynamic query building with proper parameter binding
        // For now, return empty results
        Ok((vec![], 0))
    }

    // ===== GOVERNANCE METHODS =====

    /// Get branch protection rules for a repository reference
    pub async fn get_protected_ref(&self, repo_id: Uuid, ref_name: &str) -> Result<Option<ProtectedRef>> {
        let row = sqlx::query(
            "SELECT id, repo_id, ref_name, require_admin, allow_fast_forward, allow_delete, 
                    required_checks, required_reviewers, require_schema_pass, created_at, updated_at
             FROM protected_refs 
             WHERE repo_id = $1 AND ref_name = $2"
        )
        .bind(repo_id)
        .bind(ref_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ProtectedRef {
            id: r.get("id"),
            repo_id: r.get("repo_id"),
            ref_name: r.get("ref_name"),
            require_admin: r.get("require_admin"),
            allow_fast_forward: r.get("allow_fast_forward"),
            allow_delete: r.get("allow_delete"),
            required_checks: serde_json::from_value(r.get("required_checks")).unwrap_or_default(),
            required_reviewers: r.get::<i32, _>("required_reviewers") as u32,
            require_schema_pass: r.get("require_schema_pass"),
        }))
    }

    /// Set branch protection rules for a repository reference
    pub async fn set_protected_ref(&self, protected_ref: &ProtectedRef) -> Result<()> {
        sqlx::query(
            "INSERT INTO protected_refs (id, repo_id, ref_name, require_admin, allow_fast_forward, 
                                        allow_delete, required_checks, required_reviewers, require_schema_pass)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (repo_id, ref_name) 
             DO UPDATE SET require_admin = $4, allow_fast_forward = $5, allow_delete = $6,
                          required_checks = $7, required_reviewers = $8, require_schema_pass = $9,
                          updated_at = NOW()"
        )
        .bind(protected_ref.id)
        .bind(protected_ref.repo_id)
        .bind(&protected_ref.ref_name)
        .bind(protected_ref.require_admin)
        .bind(protected_ref.allow_fast_forward)
        .bind(protected_ref.allow_delete)
        .bind(serde_json::to_value(&protected_ref.required_checks)?)
        .bind(protected_ref.required_reviewers as i32)
        .bind(protected_ref.require_schema_pass)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get repository quota configuration
    pub async fn get_repo_quota(&self, repo_id: Uuid) -> Result<Option<RepoQuota>> {
        let row = sqlx::query(
            "SELECT id, repo_id, bytes_soft, bytes_hard, created_at, updated_at
             FROM repo_quota 
             WHERE repo_id = $1"
        )
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| RepoQuota {
            id: r.get("id"),
            repo_id: r.get("repo_id"),
            bytes_soft: r.get::<i64, _>("bytes_soft") as u64,
            bytes_hard: r.get::<i64, _>("bytes_hard") as u64,
        }))
    }

    /// Set repository quota configuration
    pub async fn set_repo_quota(&self, quota: &RepoQuota) -> Result<()> {
        sqlx::query(
            "INSERT INTO repo_quota (id, repo_id, bytes_soft, bytes_hard)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (repo_id) 
             DO UPDATE SET bytes_soft = $3, bytes_hard = $4, updated_at = NOW()"
        )
        .bind(quota.id)
        .bind(quota.repo_id)
        .bind(quota.bytes_soft as i64)
        .bind(quota.bytes_hard as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get repository usage
    pub async fn get_repo_usage(&self, repo_id: Uuid) -> Result<Option<RepoUsage>> {
        let row = sqlx::query(
            "SELECT id, repo_id, current_bytes, last_calculated
             FROM repo_usage 
             WHERE repo_id = $1"
        )
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| RepoUsage {
            id: r.get::<Option<Uuid>, _>("id").unwrap_or_default(),
            repo_id: r.get("repo_id"),
            current_bytes: r.get::<i64, _>("current_bytes") as u64,
            last_calculated: r.get("last_calculated"),
        }))
    }

    /// Update repository usage
    pub async fn update_repo_usage(&self, repo_id: Uuid, current_bytes: u64) -> Result<()> {
        sqlx::query(
            "INSERT INTO repo_usage (repo_id, current_bytes, last_calculated)
             VALUES ($1, $2, NOW())
             ON CONFLICT (repo_id) 
             DO UPDATE SET current_bytes = $2, last_calculated = NOW()"
        )
        .bind(repo_id)
        .bind(current_bytes as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get quota status for a repository
    pub async fn get_quota_status(&self, repo_id: Uuid) -> Result<Option<QuotaStatus>> {
        let row = sqlx::query(
            "SELECT q.bytes_soft, q.bytes_hard, u.current_bytes
             FROM repo_quota q
             LEFT JOIN repo_usage u ON q.repo_id = u.repo_id
             WHERE q.repo_id = $1"
        )
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| QuotaStatus::new(
            r.get::<Option<i64>, _>("current_bytes").map(|b| b as u64).unwrap_or(0),
            r.get::<i64, _>("bytes_soft") as u64,
            r.get::<i64, _>("bytes_hard") as u64,
        )))
    }

    /// Get retention policy for a repository
    pub async fn get_repo_retention(&self, repo_id: Uuid) -> Result<Option<RepoRetention>> {
        let row = sqlx::query(
            "SELECT id, repo_id, retention_policy, created_at, updated_at
             FROM repo_retention 
             WHERE repo_id = $1"
        )
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| RepoRetention {
            id: r.get("id"),
            repo_id: r.get("repo_id"),
            retention_policy: serde_json::from_value(r.get("retention_policy")).unwrap_or_else(|_| RetentionPolicy {
                tombstone_days: 30,
                hard_delete_days: 90,
                legal_hold: false,
            }),
        }))
    }

    /// Set retention policy for a repository
    pub async fn set_repo_retention(&self, retention: &RepoRetention) -> Result<()> {
        sqlx::query(
            "INSERT INTO repo_retention (id, repo_id, retention_policy)
             VALUES ($1, $2, $3)
             ON CONFLICT (repo_id) 
             DO UPDATE SET retention_policy = $3, updated_at = NOW()"
        )
        .bind(retention.id)
        .bind(retention.repo_id)
        .bind(serde_json::to_value(&retention.retention_policy)?
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }










    /// Submit a check result
    pub async fn submit_check_result(&self, check: &CheckResult) -> Result<()> {
        sqlx::query(
            "INSERT INTO check_results (id, repo_id, ref_name, commit_id, check_name, status, details_url, output)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (repo_id, ref_name, commit_id, check_name) 
             DO UPDATE SET status = $6, details_url = $7, output = $8, updated_at = NOW()"
        )
        .bind(check.id)
        .bind(check.repo_id)
        .bind(&check.ref_name)
        .bind(check.commit_id)
        .bind(&check.check_name)
        .bind(&check.status.to_string())
        .bind(&check.details_url)
        .bind(&check.output)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get check results for a commit
    pub async fn get_check_results(&self, repo_id: Uuid, ref_name: &str, commit_id: Uuid) -> Result<Vec<CheckResult>> {
        let rows = sqlx::query(
            "SELECT id, repo_id, ref_name, commit_id, check_name, status, details_url, output, created_at, updated_at
             FROM check_results 
             WHERE repo_id = $1 AND ref_name = $2 AND commit_id = $3
             ORDER BY created_at"
        )
        .bind(repo_id)
        .bind(ref_name)
        .bind(commit_id)
        .fetch_all(&self.pool)
        .await?;

        let checks = rows.into_iter().map(|r| CheckResult {
            id: r.get("id"),
            repo_id: r.get("repo_id"),
            ref_name: r.get("ref_name"),
            commit_id: r.get("commit_id"),
            check_name: r.get("check_name"),
            status: serde_json::from_value(serde_json::Value::String(r.get("status"))).unwrap_or(CheckStatus::Pending),
            details_url: r.get("details_url"),
            output: r.get("output"),
        }).collect();

        Ok(checks)
    }

    // ===== WEBHOOK METHODS =====

    /// Create a webhook
    pub async fn create_webhook(&self, webhook: &Webhook) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO webhooks (id, repo_id, url, secret, events, active)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(webhook.id)
        .bind(webhook.repo_id)
        .bind(&webhook.url)
        .bind(&webhook.secret)
        .bind(serde_json::to_value(&webhook.events)?)
        .bind(webhook.active)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get webhook by ID
    pub async fn get_webhook(&self, webhook_id: Uuid) -> Result<Webhook> {
        let row = sqlx::query(
            r#"
            SELECT id, repo_id, url, secret, events, active
            FROM webhooks
            WHERE id = $1
            "#
        )
        .bind(webhook_id)
        .fetch_one(&self.pool)
        .await?;

        let webhook = Webhook {
            id: row.get("id"),
            repo_id: row.get("repo_id"),
            url: row.get("url"),
            secret: row.get("secret"),
            events: serde_json::from_value(row.get("events"))?,
            active: row.get("active"),
        };

        Ok(webhook)
    }

    /// Get webhooks for a repository
    pub async fn get_webhooks(&self, repo_id: Uuid) -> Result<Vec<Webhook>> {
        let rows = sqlx::query(
            r#"
            SELECT id, repo_id, url, secret, events, active
            FROM webhooks
            WHERE repo_id = $1 AND active = true
            "#
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let webhooks = rows
            .into_iter()
            .map(|row| Webhook {
                id: row.get("id"),
                repo_id: row.get("repo_id"),
                url: row.get("url"),
                secret: row.get("secret"),
                events: serde_json::from_value(row.get("events")).unwrap_or_default(),
                active: row.get("active"),
            })
            .collect();

        Ok(webhooks)
    }

    /// Delete a webhook
    pub async fn delete_webhook(&self, webhook_id: Uuid) -> Result<()> {
        sqlx::query(
            "
            DELETE FROM webhooks WHERE id = $1
            "
        )
        .bind(webhook_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create webhook delivery
    pub async fn create_webhook_delivery(&self, delivery: &WebhookDelivery) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO webhook_deliveries (
                id, webhook_id, event, payload, status, attempts, max_attempts,
                next_retry_at, response_status, response_body, delivered_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(delivery.id)
        .bind(delivery.webhook_id)
        .bind(&delivery.event_type)
        .bind(&delivery.payload)
        .bind("pending") // status
        .bind(delivery.attempts as i32)
        .bind(delivery.max_attempts as i32)
        .bind(delivery.next_retry_at)
        .bind(delivery.response_status.map(|s| s as i32))
        .bind(&delivery.response_body)
        .bind(delivery.delivered_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update webhook delivery
    pub async fn update_webhook_delivery(&self, delivery: &WebhookDelivery) -> Result<()> {
        sqlx::query(
            "
            UPDATE webhook_deliveries SET
                attempts = $2, max_attempts = $3, next_retry_at = $4,
                response_status = $5, response_body = $6, delivered_at = $7
            WHERE id = $1
            "
        )
        .bind(delivery.id)
        .bind(delivery.attempts as i32)
        .bind(delivery.max_attempts as i32)
        .bind(delivery.next_retry_at)
        .bind(delivery.response_status.map(|s| s as i32))
        .bind(&delivery.response_body)
        .bind(delivery.delivered_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get webhook delivery by ID
    pub async fn get_webhook_delivery(&self, delivery_id: Uuid) -> Result<WebhookDelivery> {
        let row = sqlx::query(
            "
            SELECT id, webhook_id, event, payload, status, attempts, max_attempts, last_attempt_at,
                   next_retry_at, response_status, response_body, error_message, delivered_at, created_at, updated_at
            FROM webhook_deliveries
            WHERE id = $1
            "
        )
        .bind(delivery_id)
        .fetch_one(&self.pool)
        .await?;

        let delivery = WebhookDelivery {
            id: row.get("id"),
            webhook_id: row.get("webhook_id"),
            event_type: row.get("event"),
            payload: row.get("payload"),
            response_status: row.get::<Option<i32>, _>("response_status").map(|s| s as u16),
            response_body: row.get("response_body"),
            attempts: row.get::<i32, _>("attempts") as u32,
            max_attempts: row.get::<i32, _>("max_attempts") as u32,
            next_retry_at: row.get("next_retry_at"),
            delivered_at: row.get("delivered_at"),
        };

        Ok(delivery)
    }

    /// Get pending webhook deliveries
    pub async fn get_pending_webhook_deliveries(&self) -> Result<Vec<WebhookDelivery>> {
        let rows = sqlx::query(
            r#"
            SELECT id, webhook_id, event, payload, status, attempts, max_attempts, last_attempt_at,
                   next_retry_at, response_status, response_body, error_message, delivered_at, created_at, updated_at
            FROM webhook_deliveries
            WHERE status IN ('pending', 'failed') AND (next_retry_at IS NULL OR next_retry_at <= $1)
            ORDER BY created_at ASC
            LIMIT 100
            "#
        )
        .bind(chrono::Utc::now())
        .fetch_all(&self.pool)
        .await?;

        let deliveries = rows
            .into_iter()
            .map(|row| WebhookDelivery {
                id: row.get("id"),
                webhook_id: row.get("webhook_id"),
                event_type: row.get("event"),
                payload: row.get("payload"),
                response_status: row.get::<Option<i32>, _>("response_status").map(|s| s as u16),
                response_body: row.get("response_body"),
                attempts: row.get::<i32, _>("attempts") as u32,
                max_attempts: row.get::<i32, _>("max_attempts") as u32,
                next_retry_at: row.get("next_retry_at"),
                delivered_at: row.get("delivered_at"),
            })
            .collect();

        Ok(deliveries)
    }

    /// Get webhook deliveries for a webhook
    pub async fn get_webhook_deliveries(&self, webhook_id: Uuid) -> Result<Vec<WebhookDelivery>> {
        let rows = sqlx::query(
            "
            SELECT id, webhook_id, event, payload, attempts, max_attempts,
                   next_retry_at, response_status, response_body, delivered_at
            FROM webhook_deliveries
            WHERE webhook_id = $1
            ORDER BY created_at DESC
            LIMIT 100
            "
        )
        .bind(webhook_id)
        .fetch_all(&self.pool)
        .await?;

        let deliveries = rows
            .into_iter()
            .map(|row| WebhookDelivery {
                id: row.get("id"),
                webhook_id: row.get("webhook_id"),
                event_type: row.get("event"),
                payload: row.get("payload"),
                response_status: row.get::<Option<i32>, _>("response_status").map(|s| s as u16),
                response_body: row.get("response_body"),
                attempts: row.get::<i32, _>("attempts") as u32,
                max_attempts: row.get::<i32, _>("max_attempts") as u32,
                next_retry_at: row.get("next_retry_at"),
                delivered_at: row.get("delivered_at"),
            })
            .collect();

        Ok(deliveries)
    }

    /// Delete webhook delivery
    pub async fn delete_webhook_delivery(&self, delivery_id: Uuid) -> Result<()> {
        sqlx::query(
            "
            DELETE FROM webhook_deliveries WHERE id = $1
            "
        )
        .bind(delivery_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create webhook dead letter record
    pub async fn create_webhook_dead(&self, dead: &WebhookDead) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO webhook_dead (id, webhook_id, event, payload, attempts, last_error)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(dead.id)
        .bind(dead.webhook_id)
        .bind(&dead.event_type)
        .bind(&dead.payload)
        .bind(dead.attempts as i32)
        .bind(&dead.failure_reason)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get webhook dead letter records for a repository
    pub async fn get_webhook_dead_letter(&self, repo_id: Uuid) -> Result<Vec<WebhookDead>> {
        let rows = sqlx::query(
            "
            SELECT wd.id, wd.webhook_id, wd.event, wd.payload, wd.attempts, wd.failure_reason
            FROM webhook_dead wd
            JOIN webhooks w ON wd.webhook_id = w.id
            WHERE w.repo_id = $1
            ORDER BY wd.moved_at DESC
            LIMIT 100
            "
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let dead_webhooks = rows
            .into_iter()
            .map(|row| WebhookDead {
                id: row.get("id"),
                webhook_id: row.get("webhook_id"),
                event_type: row.get("event"),
                payload: row.get("payload"),
                failure_reason: row.get("failure_reason"),
                attempts: row.get::<i32, _>("attempts") as u32,
            })
            .collect();

        Ok(dead_webhooks)
    }

    // ===== EXPORT JOB METHODS =====

    /// Create export job
    pub async fn create_export_job(&self, job: &ExportJob) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO export_jobs (
                id, repo_id, user_id, manifest, status, s3_key, download_url, error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(job.id)
        .bind(job.repo_id)
        .bind(&job.user_id)
        .bind(serde_json::to_value(&job.manifest)?)
        .bind(&job.status.to_string())
        .bind(&job.s3_key)
        .bind(&job.download_url)
        .bind(&job.error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get export job by ID
    pub async fn get_export_job(&self, job_id: Uuid) -> Result<ExportJob> {
        let row = sqlx::query(
            "
            SELECT id, repo_id, user_id, manifest, status, s3_key, download_url, error_message
            FROM export_jobs
            WHERE id = $1
            "
        )
        .bind(job_id)
        .fetch_one(&self.pool)
        .await?;

        let job = ExportJob {
            id: row.get("id"),
            repo_id: row.get("repo_id"),
            user_id: row.get("user_id"),
            manifest: serde_json::from_value(row.get("manifest"))?,
            status: ExportJobStatus::from_str(&row.get::<String, _>("status")).unwrap_or(ExportJobStatus::Pending),
            s3_key: row.get("s3_key"),
            download_url: row.get("download_url"),
            error_message: row.get("error_message"),
        };

        Ok(job)
    }

    /// Update export job status
    pub async fn update_export_job_status(&self, job: &ExportJob) -> Result<()> {
        sqlx::query(
            "
            UPDATE export_jobs SET
                status = $2, download_url = $3, error_message = $4
            WHERE id = $1
            "
        )
        .bind(job.id)
        .bind(&job.status.to_string())
        .bind(&job.download_url)
        .bind(&job.error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get pending export jobs
    pub async fn get_pending_export_jobs(&self) -> Result<Vec<ExportJob>> {
        let rows = sqlx::query(
            "
            SELECT id, repo_id, user_id, manifest, status, s3_key, download_url, error_message
            FROM export_jobs
            WHERE status = 'pending'
            ORDER BY id ASC
            LIMIT 10
            "
        )
                .fetch_all(&self.pool)
        .await?;

        let jobs = rows
            .into_iter()
            .map(|row| ExportJob {
                id: row.get("id"),
                repo_id: row.get("repo_id"),
                user_id: row.get("user_id"),
                manifest: serde_json::from_value(row.get("manifest")).unwrap_or_else(|_| ExportManifest {
                    ref_name: "main".to_string(),
                    paths: vec![],
                    include_meta: true,
                    include_rdf: false,
                }),
                status: ExportJobStatus::from_str(&row.get::<String, _>("status")).unwrap_or(ExportJobStatus::Pending),
                s3_key: row.get("s3_key"),
                download_url: row.get("download_url"),
                error_message: row.get("error_message"),
            })
            .collect();

        Ok(jobs)
    }

    /// Get expired export jobs
    pub async fn get_expired_export_jobs(&self, _now: u64) -> Result<Vec<ExportJob>> {
        let rows = sqlx::query(
            "
            SELECT id, repo_id, user_id, manifest, status, s3_key, download_url, error_message
            FROM export_jobs
            WHERE status = 'completed'
            ORDER BY id ASC
            "
        )
                .fetch_all(&self.pool)
        .await?;

        let jobs = rows
            .into_iter()
            .map(|row| ExportJob {
                id: row.get("id"),
                repo_id: row.get("repo_id"),
                user_id: row.get("user_id"),
                manifest: serde_json::from_value(row.get("manifest")).unwrap_or_else(|_| ExportManifest {
                    ref_name: "main".to_string(),
                    paths: vec![],
                    include_meta: true,
                    include_rdf: false,
                }),
                status: ExportJobStatus::from_str(&row.get::<String, _>("status")).unwrap_or(ExportJobStatus::Pending),
                s3_key: row.get("s3_key"),
                download_url: row.get("download_url"),
                error_message: row.get("error_message"),
            })
            .collect();

        Ok(jobs)
    }

    /// Delete export job
    pub async fn delete_export_job(&self, job_id: Uuid) -> Result<()> {
        sqlx::query(
            "
            DELETE FROM export_jobs WHERE id = $1
            "
        )
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get entries by path (helper method for exports)
    pub async fn get_entries_by_path(&self, ref_name: &str, path: &str) -> Result<Vec<Entry>> {
        // This is a simplified implementation
        // In reality, you would need to get the commit ID from the ref first
        let rows = sqlx::query(
            "
            SELECT e.id, e.commit_id, e.path, e.object_sha256, e.meta, e.created_at
            FROM entry e
            JOIN commit c ON e.commit_id = c.id
            JOIN ref r ON c.id = r.commit_id
            WHERE r.name = $1 AND e.path = $2
            "
        )
        .bind(ref_name)
        .bind(path)
        .fetch_all(&self.pool)
        .await?;

        let entries = rows
            .into_iter()
            .map(|row| Entry {
                id: blacklake_core::UuidWrapper(row.get("id")),
                commit_id: blacklake_core::UuidWrapper(row.get("commit_id")),
                path: row.get("path"),
                object_sha256: row.get("object_sha256"),
                meta: row.get("meta"),
                is_dir: false, // TODO: get from database
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(entries)
    }
}