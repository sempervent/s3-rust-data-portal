// Postgres connector for foreign table federation
// Week 8: Federation across data sources

use super::traits::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio_postgres::{Client, NoTls, Row};
use uuid::Uuid;

/// Postgres connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConnectorConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub table_name: String,
    pub id_column: String,
    pub title_column: String,
    pub description_column: Option<String>,
    pub url_column: Option<String>,
    pub content_type_column: Option<String>,
    pub size_column: Option<String>,
    pub modified_at_column: Option<String>,
    pub tags_column: Option<String>,
    pub ssl_mode: String,
}

/// Postgres connector implementation
pub struct PostgresConnector {
    config: PostgresConnectorConfig,
    client: Client,
    name: String,
}

impl PostgresConnector {
    pub async fn new(name: String, config: PostgresConnectorConfig) -> Result<Self, ConnectorError> {
        let connection_string = format!(
            "host={} port={} dbname={} user={} password={} sslmode={}",
            config.host,
            config.port,
            config.database,
            config.username,
            config.password,
            config.ssl_mode
        );
        
        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;
        
        // Spawn connection task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Postgres connection error: {}", e);
            }
        });
        
        Ok(Self {
            config,
            client,
            name,
        })
    }
    
    /// Build SELECT query for entries
    fn build_select_query(&self) -> String {
        let mut columns = vec![
            self.config.id_column.clone(),
            self.config.title_column.clone(),
        ];
        
        if let Some(ref col) = self.config.description_column {
            columns.push(col.clone());
        }
        if let Some(ref col) = self.config.url_column {
            columns.push(col.clone());
        }
        if let Some(ref col) = self.config.content_type_column {
            columns.push(col.clone());
        }
        if let Some(ref col) = self.config.size_column {
            columns.push(col.clone());
        }
        if let Some(ref col) = self.config.modified_at_column {
            columns.push(col.clone());
        }
        if let Some(ref col) = self.config.tags_column {
            columns.push(col.clone());
        }
        
        format!("SELECT {} FROM {}", columns.join(", "), self.config.table_name)
    }
    
    /// Convert database row to ExternalEntry
    fn row_to_entry(&self, row: &Row, source_id: Uuid) -> Result<ExternalEntry, ConnectorError> {
        let id = row.get::<_, String>(&self.config.id_column);
        let title = row.get::<_, String>(&self.config.title_column);
        
        let description = if let Some(ref col) = self.config.description_column {
            row.get::<_, Option<String>>(col)
        } else {
            None
        };
        
        let url = if let Some(ref col) = self.config.url_column {
            row.get::<_, Option<String>>(col)
        } else {
            None
        }.unwrap_or_else(|| format!("postgres://{}/{}", self.config.database, id));
        
        let content_type = if let Some(ref col) = self.config.content_type_column {
            row.get::<_, Option<String>>(col)
        } else {
            None
        };
        
        let size = if let Some(ref col) = self.config.size_column {
            row.get::<_, Option<i64>>(col).map(|s| s as u64)
        } else {
            None
        };
        
        let modified_at = if let Some(ref col) = self.config.modified_at_column {
            row.get::<_, Option<chrono::DateTime<chrono::Utc>>>(col)
        } else {
            None
        };
        
        let tags = if let Some(ref col) = self.config.tags_column {
            row.get::<_, Option<Vec<String>>>(col).unwrap_or_default()
        } else {
            vec![]
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("database".to_string(), serde_json::Value::String(self.config.database.clone()));
        metadata.insert("table".to_string(), serde_json::Value::String(self.config.table_name.clone()));
        metadata.insert("id".to_string(), serde_json::Value::String(id.clone()));
        
        Ok(ExternalEntry {
            id: format!("postgres:{}:{}:{}", self.config.database, self.config.table_name, id),
            title,
            description,
            url,
            content_type,
            size,
            modified_at,
            tags,
            metadata,
            source_id,
            source_type: "postgres".to_string(),
        })
    }
}

#[async_trait]
impl Connector for PostgresConnector {
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::Postgres
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn test_connection(&self) -> Result<(), ConnectorError> {
        // Test connection by running a simple query
        let query = format!("SELECT 1 FROM {} LIMIT 1", self.config.table_name);
        self.client.query(&query, &[]).await?;
        Ok(())
    }
    
    async fn list_entries(&self) -> Result<Vec<ExternalEntry>, ConnectorError> {
        let query = self.build_select_query();
        let rows = self.client.query(&query, &[]).await?;
        
        let mut entries = Vec::new();
        let source_id = Uuid::new_v4(); // This would be the connector ID
        
        for row in rows {
            let entry = self.row_to_entry(&row, source_id)?;
            entries.push(entry);
        }
        
        Ok(entries)
    }
    
    async fn get_entry(&self, id: &str) -> Result<Option<ExternalEntry>, ConnectorError> {
        // Parse Postgres ID format: postgres:database:table:id
        let parts: Vec<&str> = id.split(':').collect();
        if parts.len() != 4 || parts[0] != "postgres" {
            return Err(ConnectorError::EntryNotFound(format!("Invalid Postgres entry ID: {}", id)));
        }
        
        let database = parts[1];
        let table = parts[2];
        let entry_id = parts[3];
        
        if database != self.config.database || table != self.config.table_name {
            return Ok(None);
        }
        
        let query = format!("{} WHERE {} = $1", self.build_select_query(), self.config.id_column);
        let rows = self.client.query(&query, &[&entry_id]).await?;
        
        if let Some(row) = rows.first() {
            let source_id = Uuid::new_v4();
            Ok(Some(self.row_to_entry(row, source_id)?))
        } else {
            Ok(None)
        }
    }
    
    async fn get_presigned_url(&self, entry: &ExternalEntry) -> Result<String, ConnectorError> {
        // For Postgres, return the URL from the entry
        Ok(entry.url.clone())
    }
    
    async fn sync_entries(&self) -> Result<SyncResult, ConnectorError> {
        let start_time = std::time::Instant::now();
        
        // Test connection first
        self.test_connection().await?;
        
        // List all entries
        let entries = self.list_entries().await?;
        
        let duration = start_time.elapsed();
        
        Ok(SyncResult {
            entries_processed: entries.len() as u64,
            entries_added: entries.len() as u64,
            entries_updated: 0,
            entries_removed: 0,
            errors: vec![],
            duration_seconds: duration.as_secs_f64(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_postgres_connector_config() {
        let config = PostgresConnectorConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            table_name: "documents".to_string(),
            id_column: "id".to_string(),
            title_column: "title".to_string(),
            description_column: Some("description".to_string()),
            url_column: Some("url".to_string()),
            content_type_column: Some("content_type".to_string()),
            size_column: Some("size".to_string()),
            modified_at_column: Some("updated_at".to_string()),
            tags_column: Some("tags".to_string()),
            ssl_mode: "prefer".to_string(),
        };
        
        assert_eq!(config.host, "localhost");
        assert_eq!(config.database, "testdb");
        assert_eq!(config.table_name, "documents");
    }
    
    #[test]
    fn test_postgres_entry_id_parsing() {
        let id = "postgres:mydb:documents:123";
        let parts: Vec<&str> = id.split(':').collect();
        
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "postgres");
        assert_eq!(parts[1], "mydb");
        assert_eq!(parts[2], "documents");
        assert_eq!(parts[3], "123");
    }
}
