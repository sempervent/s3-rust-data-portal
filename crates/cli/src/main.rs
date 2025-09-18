use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

mod api;
mod cmd;
mod prompt;

use api::ApiClient;
use cmd::{put, meta};

#[derive(Parser)]
#[command(name = "blacklake")]
#[command(about = "Blacklake CLI - Git-style data artifact service")]
#[command(version)]
struct Cli {
    /// API base URL
    #[arg(long, default_value = "http://localhost:8080")]
    api_url: String,
    
    /// Authentication token
    #[arg(long)]
    token: Option<String>,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Upload and commit files with interactive metadata
    Put {
        /// Repository name
        repo: String,
        /// Branch or ref name
        r#ref: String,
        /// Local file path
        local_file: String,
        /// Logical path in repository
        #[arg(long)]
        path: String,
        /// MIME type (auto-detected if not specified)
        #[arg(long)]
        r#type: Option<String>,
        /// Emit RDF metadata
        #[arg(long)]
        emit_rdf: bool,
        /// Open editor for metadata
        #[arg(long)]
        open_editor: bool,
        /// Metadata as JSON/YAML file
        #[arg(long)]
        meta: Option<String>,
        /// Metadata key-value pairs
        #[arg(long, value_parser = parse_key_value)]
        meta_key: Vec<(String, String)>,
        /// Template name
        #[arg(long)]
        template: Option<String>,
        /// Dry run (don't commit)
        #[arg(long)]
        dry_run: bool,
        /// Non-interactive mode
        #[arg(long)]
        non_interactive: bool,
    },
    /// Edit metadata for existing files
    Meta {
        #[command(subcommand)]
        command: MetaCommands,
    },
    /// Get/download files
    Get {
        /// Repository name
        repo: String,
        /// Branch or ref name
        r#ref: String,
        /// File path in repository
        path: String,
        /// Output file path
        #[arg(long)]
        out: Option<String>,
    },
    /// Search repository
    Search {
        /// Repository name
        repo: String,
        /// File type filter
        #[arg(long)]
        file_type: Option<String>,
        /// Organization/Lab filter
        #[arg(long)]
        org: Option<String>,
        /// Tag filter
        #[arg(long)]
        tag: Vec<String>,
        /// Created after date
        #[arg(long)]
        from: Option<String>,
        /// Created before date
        #[arg(long)]
        to: Option<String>,
        /// Query string
        #[arg(short, long)]
        q: Option<String>,
        /// Limit results
        #[arg(long)]
        limit: Option<u32>,
        /// Sort by field (path, size, creation_dt, file_type, org_lab)
        #[arg(long)]
        sort: Option<String>,
        /// Fields to display (comma-separated)
        #[arg(long)]
        fields: Option<String>,
        /// JSON output
        #[arg(long)]
        json: bool,
    },
    /// Repository operations
    Repo {
        #[command(subcommand)]
        command: RepoCommands,
    },
    /// RDF operations
    Rdf {
        #[command(subcommand)]
        command: RdfCommands,
    },
    /// Generate shell completions
    Completions {
        /// Shell type
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand)]
enum MetaCommands {
    /// Edit metadata for a file
    Edit {
        /// Repository name
        repo: String,
        /// Branch or ref name
        r#ref: String,
        /// File path in repository
        path: String,
        /// Open editor for metadata
        #[arg(long)]
        open_editor: bool,
        /// Metadata as JSON/YAML file
        #[arg(long)]
        meta: Option<String>,
        /// Metadata key-value pairs
        #[arg(long, value_parser = parse_key_value)]
        meta_key: Vec<(String, String)>,
        /// Dry run (don't commit)
        #[arg(long)]
        dry_run: bool,
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

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid key=value format: {}", s));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Set up logging
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    }

    let api_client = ApiClient::new(cli.api_url.clone())
        .with_token(cli.token.unwrap_or_default());

    match cli.command {
        Commands::Put(args) => {
            put::put_command(put::PutArgs {
                repo: args.repo,
                r#ref: args.r#ref,
                local_file: args.local_file,
                path: args.path,
                r#type: args.r#type,
                emit_rdf: args.emit_rdf,
                open_editor: args.open_editor,
                meta: args.meta,
                meta_key: args.meta_key,
                template: args.template,
                dry_run: args.dry_run,
                non_interactive: args.non_interactive,
            }, &api_client).await?;
        },
        Commands::Meta { command } => {
            match command {
                MetaCommands::Edit(args) => {
                    meta::meta_edit_command(meta::MetaEditArgs {
                        repo: args.repo,
                        r#ref: args.r#ref,
                        path: args.path,
                        open_editor: args.open_editor,
                        meta: args.meta,
                        meta_key: args.meta_key,
                        dry_run: args.dry_run,
                    }, &api_client).await?;
                },
            }
        },
        Commands::Get { repo, r#ref, path, out } => {
            get_command(repo, r#ref, path, out, &api_client).await?;
        },
        Commands::Search { repo, file_type, org, tag, from, to, q, limit, sort, fields, json } => {
            search_command(repo, file_type, org, tag, from, to, q, limit, sort, fields, json, &api_client).await?;
        },
        Commands::Repo { command } => {
            match command {
                RepoCommands::Create { name } => {
                    create_repo_command(name, &api_client).await?;
                },
                RepoCommands::List => {
                    list_repos_command(&api_client).await?;
                },
                RepoCommands::Features { command } => {
                    match command {
                        RepoFeatureCommands::Set { repo, key, value } => {
                            set_repo_feature_command(repo, key, value, &api_client).await?;
                        },
                    }
                },
            }
        },
        Commands::Rdf { command } => {
            match command {
                RdfCommands::Get { repo, r#ref, path, format } => {
                    get_rdf_command(repo, r#ref, path, format, &api_client).await?;
                },
            }
        },
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "blacklake", &mut std::io::stdout());
        },
    }

    Ok(())
}

async fn get_command(repo: String, r#ref: String, path: String, out: Option<String>, api_client: &ApiClient) -> Result<()> {
    println!("📥 Downloading {}/{}", repo, path);
    
    let download_url = api_client.get_blob(&repo, &r#ref, &path).await?;
    
    let response = reqwest::get(&download_url).await?;
    let content = response.bytes().await?;
    
    let output_path = out.unwrap_or_else(|| {
        PathBuf::from(&path).file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("downloaded_file")
            .to_string()
    });
    
    std::fs::write(&output_path, content)?;
    println!("✅ Downloaded to: {}", output_path);
    
    Ok(())
}

async fn search_command(
    repo: String,
    file_type: Option<String>,
    org: Option<String>,
    tag: Vec<String>,
    from: Option<String>,
    created_before: Option<String>,
    q: Option<String>,
    limit: Option<u32>,
    sort: Option<String>,
    fields: Option<String>,
    json: bool,
    api_client: &ApiClient,
) -> Result<()> {
    let search_request = blacklake_core::SearchRequest {
        query: q,
        file_type,
        org_lab: org,
        tags: if tag.is_empty() { None } else { Some(tag) },
        created_after: from,
        created_before,
        limit,
        offset: None,
    };
    
    let response = api_client.search(&repo, &search_request).await?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!("🔍 Search results for {} ({} total):", repo, response.total);
        println!();
        
        // Parse fields to display
        let fields_to_show = if let Some(fields_str) = fields {
            fields_str.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            vec!["path".to_string(), "size".to_string(), "description".to_string(), "org_lab".to_string()]
        };
        
        // Sort entries if requested
        let mut entries = response.entries;
        if let Some(sort_field) = sort {
            entries.sort_by(|a, b| {
                match sort_field.as_str() {
                    "path" => a.path.cmp(&b.path),
                    "size" => a.size.cmp(&b.size),
                    "creation_dt" => {
                        let a_dt = a.meta.as_ref()
                            .and_then(|m| m.get("creation_dt"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let b_dt = b.meta.as_ref()
                            .and_then(|m| m.get("creation_dt"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        a_dt.cmp(b_dt)
                    },
                    "file_type" => {
                        let a_type = a.meta.as_ref()
                            .and_then(|m| m.get("file_type"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let b_type = b.meta.as_ref()
                            .and_then(|m| m.get("file_type"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        a_type.cmp(b_type)
                    },
                    "org_lab" => {
                        let a_org = a.meta.as_ref()
                            .and_then(|m| m.get("org_lab"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let b_org = b.meta.as_ref()
                            .and_then(|m| m.get("org_lab"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        a_org.cmp(b_org)
                    },
                    _ => a.path.cmp(&b.path),
                }
            });
        }
        
        for entry in entries {
            for field in &fields_to_show {
                match field.as_str() {
                    "path" => println!("📄 {}", entry.path),
                    "size" => println!("   Size: {} bytes", entry.size),
                    "sha256" => println!("   SHA256: {}", entry.sha256),
                    "description" => {
                        if let Some(meta) = &entry.meta {
                            if let Some(description) = meta.get("description").and_then(|v| v.as_str()) {
                                println!("   Description: {}", description);
                            }
                        }
                    },
                    "org_lab" => {
                        if let Some(meta) = &entry.meta {
                            if let Some(org_lab) = meta.get("org_lab").and_then(|v| v.as_str()) {
                                println!("   Organization: {}", org_lab);
                            }
                        }
                    },
                    "file_type" => {
                        if let Some(meta) = &entry.meta {
                            if let Some(file_type) = meta.get("file_type").and_then(|v| v.as_str()) {
                                println!("   Type: {}", file_type);
                            }
                        }
                    },
                    "creation_dt" => {
                        if let Some(meta) = &entry.meta {
                            if let Some(creation_dt) = meta.get("creation_dt").and_then(|v| v.as_str()) {
                                println!("   Created: {}", creation_dt);
                            }
                        }
                    },
                    "tags" => {
                        if let Some(meta) = &entry.meta {
                            if let Some(tags) = meta.get("tags").and_then(|v| v.as_array()) {
                                let tag_strs: Vec<String> = tags.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect();
                                if !tag_strs.is_empty() {
                                    println!("   Tags: {}", tag_strs.join(", "));
                                }
                            }
                        }
                    },
                    _ => {
                        if let Some(meta) = &entry.meta {
                            if let Some(value) = meta.get(field).and_then(|v| v.as_str()) {
                                println!("   {}: {}", field, value);
                            }
                        }
                    }
                }
            }
            println!();
        }
    }
    
    Ok(())
}

async fn create_repo_command(name: String, api_client: &ApiClient) -> Result<()> {
    println!("📁 Creating repository: {}", name);
    
    let response = api_client.post_request(&format!("{}/v1/repos", api_client.base_url))
        .json(&json!({ "name": name }))
        .send()
        .await?;
    
    if response.status().is_success() {
        println!("✅ Repository created successfully");
    } else {
        let error_text = response.text().await?;
        return Err(format!("Failed to create repository: {}", error_text).into());
    }
    
    Ok(())
}

async fn list_repos_command(api_client: &ApiClient) -> Result<()> {
    println!("📁 Repositories:");
    
    let response = api_client.request()
        .url(&format!("{}/v1/repos", api_client.base_url))
        .send()
        .await?;
    
    if response.status().is_success() {
        let repos: Vec<serde_json::Value> = response.json().await?;
        for repo in repos {
            if let Some(name) = repo.get("name").and_then(|v| v.as_str()) {
                println!("  📁 {}", name);
            }
        }
    } else {
        let error_text = response.text().await?;
        return Err(format!("Failed to list repositories: {}", error_text).into());
    }
    
    Ok(())
}

async fn set_repo_feature_command(repo: String, key: String, value: String, api_client: &ApiClient) -> Result<()> {
    println!("⚙️ Setting feature {}={} for repository {}", key, value, repo);
    
    let response = api_client.post_request(&format!("{}/v1/repos/{}/features", api_client.base_url, repo))
        .json(&json!({ "key": key, "value": value }))
        .send()
        .await?;
    
    if response.status().is_success() {
        println!("✅ Feature set successfully");
    } else {
        let error_text = response.text().await?;
        return Err(format!("Failed to set feature: {}", error_text).into());
    }
    
    Ok(())
}

async fn get_rdf_command(repo: String, r#ref: String, path: String, format: String, api_client: &ApiClient) -> Result<()> {
    println!("🔗 Getting RDF for {}/{}", repo, path);
    
    let rdf_content = api_client.get_rdf(&repo, &r#ref, &path, &format).await?;
    
    println!("{}", rdf_content);
    
    Ok(())
}