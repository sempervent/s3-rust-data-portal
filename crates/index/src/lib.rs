use blacklake_core::{
    Acl, AuditLog, ArtifactRdf, Change, Commit, Entry, EntryMetaIndex, Object, Permission, 
    Reference, ReferenceKind, Repository, RdfFormat,
};
use chrono::Utc;
use sqlx::{PgPool, Postgres, Row};
use std::collections::HashMap;
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
    #[error("Parent commit mismatch: expected {expected}, got {actual}")]
    ParentMismatch { expected: Uuid, actual: Option<Uuid> },
    #[error("Invalid reference kind: {0}")]
    InvalidRefKind(String),
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

    /// Create a new repository
    pub async fn create_repo(&self, name: &str, created_by: &str) -> Result<Repository> {
        // TODO: Add connection pooling optimization and connection health checks
        // TODO: Implement database query retry logic with exponential backoff
        // TODO: Add database connection timeout and circuit breaker patterns
        // TODO: Implement read replicas for better performance
        
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO repo (id, name, created_at, created_by) VALUES ($1, $2, $3, $4)",
            id,
            name,
            now,
            created_by
        )
        .execute(&self.pool)
        .await?;

        Ok(Repository {
            id,
            name: name.to_string(),
            created_at: now,
            created_by: created_by.to_string(),
        })
    }

    /// List all repositories
    pub async fn list_repos(&self) -> Result<Vec<Repository>> {
        let rows = sqlx::query!(
            "SELECT id, name, created_at, created_by FROM repo ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Repository {
                id: row.id,
                name: row.name,
                created_at: row.created_at,
                created_by: row.created_by,
            })
            .collect())
    }

    /// Get repository by name
    pub async fn get_repo_by_name(&self, name: &str) -> Result<Repository> {
        let row = sqlx::query!(
            "SELECT id, name, created_at, created_by FROM repo WHERE name = $1",
            name
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| IndexError::RepoNotFound(name.to_string()))?;

        Ok(Repository {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
            created_by: row.created_by,
        })
    }

    // Reference operations

    /// Get a reference
    pub async fn get_ref(&self, repo_id: Uuid, name: &str) -> Result<Reference> {
        let row = sqlx::query!(
            "SELECT repo_id, name, kind, commit_id FROM ref WHERE repo_id = $1 AND name = $2",
            repo_id,
            name
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| IndexError::RefNotFound(name.to_string()))?;

        let kind = match row.kind.as_str() {
            "branch" => ReferenceKind::Branch,
            "tag" => ReferenceKind::Tag,
            _ => return Err(IndexError::InvalidRefKind(row.kind)),
        };

        Ok(Reference {
            repo_id: row.repo_id,
            name: row.name,
            kind,
            commit_id: row.commit_id,
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

        sqlx::query!(
            "INSERT INTO ref (repo_id, name, kind, commit_id) VALUES ($1, $2, $3, $4) 
             ON CONFLICT (repo_id, name) DO UPDATE SET kind = $3, commit_id = $4",
            repo_id,
            name,
            kind_str,
            commit_id
        )
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
            let actual_parent = self.get_ref(repo_id, "main").await.ok().map(|r| r.commit_id);
            if actual_parent != Some(expected) {
                return Err(IndexError::ParentMismatch {
                    expected,
                    actual: actual_parent,
                });
            }
        }

        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO commit (id, repo_id, parent_id, author, message, created_at) 
             VALUES ($1, $2, $3, $4, $5, $6)",
            id,
            repo_id,
            parent_id,
            author,
            message,
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(Commit {
            id,
            repo_id,
            parent_id,
            author: author.to_string(),
            message: message.map(|s| s.to_string()),
            created_at: now,
            stats: None,
        })
    }

    /// Get a commit by ID
    pub async fn get_commit(&self, commit_id: Uuid) -> Result<Commit> {
        let row = sqlx::query!(
            "SELECT id, repo_id, parent_id, author, message, created_at, stats 
             FROM commit WHERE id = $1",
            commit_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| IndexError::CommitNotFound(commit_id))?;

        Ok(Commit {
            id: row.id,
            repo_id: row.repo_id,
            parent_id: row.parent_id,
            author: row.author,
            message: row.message,
            created_at: row.created_at,
            stats: row.stats,
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

        sqlx::query!(
            "INSERT INTO object (sha256, size, media_type, s3_key, created_at) 
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (sha256) DO UPDATE SET 
             size = EXCLUDED.size, media_type = EXCLUDED.media_type, s3_key = EXCLUDED.s3_key",
            sha256,
            size,
            media_type,
            s3_key,
            now
        )
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
        let row = sqlx::query!(
            "SELECT sha256, size, media_type, s3_key, created_at FROM object WHERE sha256 = $1",
            sha256
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Object {
            sha256: row.sha256,
            size: row.size,
            media_type: row.media_type,
            s3_key: row.s3_key,
            created_at: row.created_at,
        }))
    }

    // Entry operations

    /// Bind entry rows for a commit
    pub async fn bind_entries(&self, commit_id: Uuid, changes: &[Change]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete existing entries for this commit
        sqlx::query!("DELETE FROM entry WHERE commit_id = $1", commit_id)
            .execute(&mut *tx)
            .await?;

        // Insert new entries
        for change in changes {
            if change.op != blacklake_core::ChangeOp::Delete {
                sqlx::query!(
                    "INSERT INTO entry (commit_id, path, object_sha256, meta, is_dir) 
                     VALUES ($1, $2, $3, $4, $5)",
                    commit_id,
                    change.path,
                    change.sha256,
                    change.meta,
                    false // TODO: determine if directory based on path
                )
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
        let query = if let Some(prefix) = path_prefix {
            sqlx::query!(
                "SELECT commit_id, path, object_sha256, meta, is_dir 
                 FROM entry WHERE commit_id = $1 AND path LIKE $2 ORDER BY path",
                commit_id,
                format!("{}%", prefix)
            )
        } else {
            sqlx::query!(
                "SELECT commit_id, path, object_sha256, meta, is_dir 
                 FROM entry WHERE commit_id = $1 ORDER BY path",
                commit_id
            )
        };

        let rows = query.fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|row| Entry {
                commit_id: row.commit_id,
                path: row.path,
                object_sha256: row.object_sha256,
                meta: row.meta,
                is_dir: row.is_dir,
            })
            .collect())
    }

    // Search operations

    /// Search entries with filters
    pub async fn search_entries(
        &self,
        repo_id: Uuid,
        filters: &HashMap<String, serde_json::Value>,
        sort: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<(Vec<Entry>, u32)> {
        // TODO: Implement dynamic query building with proper parameter binding
        // For now, return empty results
        Ok((vec![], 0))
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

        let row = sqlx::query!(
            "INSERT INTO audit_log (at, actor, action, repo_name, ref_name, path, request_meta, response_meta) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
             RETURNING id",
            now,
            actor,
            action,
            repo_name,
            ref_name,
            path,
            request_meta,
            response_meta
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AuditLog {
            id: row.id,
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
        sqlx::query!(
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
                license = EXCLUDED.license",
            idx.commit_id,
            idx.path,
            idx.creation_dt,
            idx.creator,
            idx.file_name,
            idx.file_type,
            idx.file_size,
            idx.org_lab,
            idx.description,
            idx.data_source,
            idx.data_collection_method,
            idx.version,
            idx.notes,
            idx.tags.as_deref(),
            idx.license
        )
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

        sqlx::query!(
            "INSERT INTO artifact_rdf (commit_id, path, format, graph, graph_sha256)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (commit_id, path, format) DO UPDATE SET
                graph = EXCLUDED.graph,
                graph_sha256 = EXCLUDED.graph_sha256,
                created_at = now()",
            commit_id,
            path,
            format_str,
            graph_text,
            graph_sha256
        )
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

        let row = sqlx::query!(
            "SELECT commit_id, path, format, graph, graph_sha256, created_at
             FROM artifact_rdf WHERE commit_id = $1 AND path = $2 AND format = $3",
            commit_id,
            path,
            format_str
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| ArtifactRdf {
            commit_id: row.commit_id,
            path: row.path,
            format: match row.format.as_str() {
                "turtle" => RdfFormat::Turtle,
                "jsonld" => RdfFormat::Jsonld,
                _ => RdfFormat::Turtle, // default
            },
            graph: row.graph,
            graph_sha256: row.graph_sha256,
            created_at: row.created_at,
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
        sqlx::query!(
            "UPDATE repo SET features = features || $2::jsonb WHERE id = $1",
            repo_id,
            serde_json::json!({ key: value })
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get repository features
    pub async fn get_repo_features(&self, repo_id: Uuid) -> Result<serde_json::Value> {
        let row = sqlx::query!(
            "SELECT features FROM repo WHERE id = $1",
            repo_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.features).unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())))
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
}