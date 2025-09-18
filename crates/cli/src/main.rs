use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "blacklake")]
#[command(about = "Blacklake CLI - Git-style data artifact service")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Repository operations
    Repo {
        #[command(subcommand)]
        command: RepoCommands,
    },
    /// Upload operations
    UploadInit {
        /// Repository name
        #[arg(long)]
        repo: String,
        /// Logical path in repository
        #[arg(long)]
        path: String,
        /// Local file to upload
        #[arg(long)]
        file: PathBuf,
        /// Media type
        #[arg(long)]
        r#type: Option<String>,
    },
    /// Commit operations
    Commit {
        /// Repository name
        #[arg(long)]
        repo: String,
        /// Reference name
        #[arg(long)]
        r#ref: String,
        /// Commit message
        #[arg(short, long)]
        message: String,
        /// Put change in format: logical-path:sha256:meta-json-file
        #[arg(long)]
        put: String,
        /// Emit RDF for this commit
        #[arg(long)]
        emit_rdf: bool,
    },
    /// RDF operations
    Rdf {
        #[command(subcommand)]
        command: RdfCommands,
    },
}

#[derive(Subcommand)]
enum RepoCommands {
    /// Create a new repository
    Create {
        /// Repository name
        name: String,
    },
    /// List repositories
    List,
    /// Set repository feature flag
    Features {
        #[command(subcommand)]
        command: RepoFeatureCommands,
    },
}

#[derive(Subcommand)]
enum RepoFeatureCommands {
    /// Set a feature flag
    Set {
        /// Repository name
        repo: String,
        /// Feature key
        key: String,
        /// Feature value
        value: String,
    },
}

#[derive(Subcommand)]
enum RdfCommands {
    /// Get RDF for an artifact
    Get {
        /// Repository name
        repo: String,
        /// Reference name
        r#ref: String,
        /// Path to artifact
        path: String,
        /// Output format (turtle or jsonld)
        #[arg(long, default_value = "turtle")]
        format: String,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error: {0}")]
    Api(String),
}

type CliResult<T> = Result<T, CliError>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Repo { command } => match command {
            RepoCommands::Create { name } => create_repo(&name).await?,
            RepoCommands::List => list_repos().await?,
            RepoCommands::Features { command } => match command {
                RepoFeatureCommands::Set { repo, key, value } => set_repo_feature(&repo, &key, &value).await?,
            },
        },
        Commands::UploadInit {
            repo,
            path,
            file,
            r#type,
        } => upload_init(&repo, &path, &file, r#type.as_deref()).await?,
        Commands::Commit {
            repo,
            r#ref,
            message,
            put,
            emit_rdf,
        } => commit(&repo, &r#ref, &message, &put, emit_rdf).await?,
        Commands::Rdf { command } => match command {
            RdfCommands::Get { repo, r#ref, path, format } => get_rdf(&repo, &r#ref, &path, &format).await?,
        },
    }

    Ok(())
}

async fn create_repo(name: &str) -> CliResult<()> {
    let client = Client::new();
    let base_url = get_base_url();

    let payload = json!({
        "name": name
    });

    let response = client
        .post(&format!("{}/v1/repos", base_url))
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("Repository created successfully:");
        println!("  ID: {}", result["id"]);
        println!("  Name: {}", result["name"]);
        println!("  Created: {}", result["created_at"]);
    } else {
        let error: serde_json::Value = response.json().await?;
        return Err(CliError::Api(format!("Failed to create repository: {}", error["error"])));
    }

    Ok(())
}

async fn list_repos() -> CliResult<()> {
    let client = Client::new();
    let base_url = get_base_url();

    let response = client
        .get(&format!("{}/v1/repos", base_url))
        .send()
        .await?;

    if response.status().is_success() {
        let repos: Vec<serde_json::Value> = response.json().await?;
        println!("Repositories:");
        for repo in repos {
            println!("  {} - {} ({})", repo["name"], repo["id"], repo["created_at"]);
        }
    } else {
        let error: serde_json::Value = response.json().await?;
        return Err(CliError::Api(format!("Failed to list repositories: {}", error["error"])));
    }

    Ok(())
}

async fn upload_init(
    repo: &str,
    path: &str,
    file: &PathBuf,
    media_type: Option<&str>,
) -> CliResult<()> {
    // Check if file exists
    if !file.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", file.display()),
        )));
    }

    // Get file size
    let metadata = fs::metadata(file)?;
    let size = metadata.len();

    // Compute SHA256 hash
    let content = fs::read(file)?;
    let sha256 = blacklake_core::hash_bytes(&content);

    // Determine media type
    let detected_type = media_type.unwrap_or_else(|| {
        // Simple media type detection based on extension
        match file.extension().and_then(|s| s.to_str()) {
            Some("onnx") => "application/octet-stream",
            Some("pt") | Some("pth") => "application/octet-stream",
            Some("json") => "application/json",
            Some("yaml") | Some("yml") => "application/x-yaml",
            Some("txt") => "text/plain",
            _ => "application/octet-stream",
        }
    });

    let client = Client::new();
    let base_url = get_base_url();

    let payload = json!({
        "path": path,
        "size": size,
        "media_type": detected_type
    });

    let response = client
        .post(&format!("{}/v1/repos/{}/upload-init", base_url, repo))
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("Upload initialized successfully:");
        println!("  Path: {}", path);
        println!("  SHA256: {}", result["sha256"]);
        println!("  S3 Key: {}", result["s3_key"]);
        println!("  Upload URL: {}", result["upload_url"]);
        println!("  Expires: {}", result["expires_at"]);
        println!();
        println!("To upload the file, run:");
        println!("  curl -X PUT -T {} '{}'", file.display(), result["upload_url"]);
    } else {
        let error: serde_json::Value = response.json().await?;
        return Err(CliError::Api(format!("Failed to initialize upload: {}", error["error"])));
    }

    Ok(())
}

async fn commit(
    repo: &str,
    r#ref: &str,
    message: &str,
    put: &str,
    emit_rdf: bool,
) -> CliResult<()> {
    // Parse put format: logical-path:sha256:meta-json-file
    let parts: Vec<&str> = put.split(':').collect();
    if parts.len() != 3 {
        return Err(CliError::Api("Invalid put format. Expected: logical-path:sha256:meta-json-file".to_string()));
    }

    let logical_path = parts[0];
    let sha256 = parts[1];
    let meta_file = PathBuf::from(parts[2]);

    // Read metadata from file
    let meta_content = fs::read_to_string(&meta_file)?;
    let meta: serde_json::Value = serde_json::from_str(&meta_content)?;

    // Create change
    let change = blacklake_core::Change {
        op: blacklake_core::ChangeOp::Add,
        path: logical_path.to_string(),
        sha256: Some(sha256.to_string()),
        meta,
    };

    let client = Client::new();
    let base_url = get_base_url();

    let payload = json!({
        "ref": r#ref,
        "message": message,
        "changes": [change]
    });

    let mut request = client
        .post(&format!("{}/v1/repos/{}/commit", base_url, repo))
        .json(&payload);

    if emit_rdf {
        request = request.query(&[("emit_rdf", "true")]);
    }

    let response = request.send().await?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("Commit created successfully:");
        println!("  Commit ID: {}", result["commit_id"]);
        println!("  Parent ID: {}", result["parent_id"]);
        println!("  Created: {}", result["created_at"]);
    } else {
        let error: serde_json::Value = response.json().await?;
        return Err(CliError::Api(format!("Failed to create commit: {}", error["error"])));
    }

    Ok(())
}

async fn set_repo_feature(repo: &str, key: &str, value: &str) -> CliResult<()> {
    let client = Client::new();
    let base_url = get_base_url();

    let payload = json!({
        "key": key,
        "value": value
    });

    let response = client
        .post(&format!("{}/v1/repos/{}/features", base_url, repo))
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Feature '{}' set to '{}' for repository '{}'", key, value, repo);
    } else {
        let error: serde_json::Value = response.json().await?;
        return Err(CliError::Api(format!("Failed to set feature: {}", error["error"])));
    }

    Ok(())
}

async fn get_rdf(repo: &str, r#ref: &str, path: &str, format: &str) -> CliResult<()> {
    let client = Client::new();
    let base_url = get_base_url();

    let response = client
        .get(&format!("{}/v1/repos/{}/rdf/{}/{}", base_url, repo, r#ref, path))
        .query(&[("format", format)])
        .send()
        .await?;

    if response.status().is_success() {
        let rdf_content = response.text().await?;
        println!("{}", rdf_content);
    } else {
        let error: serde_json::Value = response.json().await?;
        return Err(CliError::Api(format!("Failed to get RDF: {}", error["error"])));
    }

    Ok(())
}

fn get_base_url() -> String {
    std::env::var("BLACKLAKE_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

// Example usage functions

pub fn example_changes_json() -> String {
    json!([
        {
            "op": "add",
            "path": "models/resnet50.onnx",
            "sha256": "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3",
            "meta": {
                "name": "ResNet-50 ONNX Model",
                "description": "Pre-trained ResNet-50 model in ONNX format",
                "version": "1.0.0",
                "tags": ["computer-vision", "image-classification", "onnx"]
            }
        },
        {
            "op": "add",
            "path": "data/train.json",
            "sha256": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            "meta": {
                "name": "Training Data",
                "description": "Training dataset in JSON format",
                "version": "1.0.0",
                "tags": ["dataset", "training"]
            }
        }
    ]).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_changes_json() {
        let json = example_changes_json();
        let changes: Vec<blacklake_core::Change> = serde_json::from_str(&json).unwrap();
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].path, "models/resnet50.onnx");
        assert_eq!(changes[1].path, "data/train.json");
    }
}
