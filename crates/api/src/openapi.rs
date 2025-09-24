// OpenAPI specification generation
// Week 7: API versioning and contract tests

use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use crate::ApiResponse;
use serde_json::Value;
use crate::AppState;

/// OpenAPI 3.0 specification for BlackLake API v1
pub fn generate_openapi_spec() -> Value {
    serde_json::json!({
        "openapi": "3.0.3",
        "info": {
            "title": "BlackLake Data Portal API",
            "description": "Enterprise data portal with multi-tenant access controls and governance",
            "version": "1.0.0",
            "contact": {
                "name": "BlackLake Support",
                "email": "support@blacklake.dev"
            },
            "license": {
                "name": "MIT",
                "url": "https://opensource.org/licenses/MIT"
            }
        },
        "servers": [
            {
                "url": "http://localhost:8080",
                "description": "Development server"
            },
            {
                "url": "https://api.blacklake.dev",
                "description": "Production server"
            }
        ],
        "paths": {
            "/v1/repos": {
                "get": {
                    "summary": "List repositories",
                    "description": "List all repositories accessible to the authenticated user",
                    "tags": ["Repositories"],
                    "responses": {
                        "200": {
                            "description": "List of repositories",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/RepositoryListResponse"
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create repository",
                    "description": "Create a new repository",
                    "tags": ["Repositories"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/CreateRepositoryRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Repository created successfully",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/RepositoryResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/repos/{repo}": {
                "get": {
                    "summary": "Get repository",
                    "description": "Get repository details",
                    "tags": ["Repositories"],
                    "parameters": [
                        {
                            "name": "repo",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Repository details",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/RepositoryResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/repos/{repo}/tree/{ref}": {
                "get": {
                    "summary": "Get repository tree",
                    "description": "Get the file tree for a specific reference",
                    "tags": ["Repositories"],
                    "parameters": [
                        {
                            "name": "repo",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        },
                        {
                            "name": "ref",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        },
                        {
                            "name": "path",
                            "in": "query",
                            "required": false,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Repository tree",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/TreeResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/search": {
                "get": {
                    "summary": "Search repositories",
                    "description": "Search across all accessible repositories",
                    "tags": ["Search"],
                    "parameters": [
                        {
                            "name": "q",
                            "in": "query",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        },
                        {
                            "name": "limit",
                            "in": "query",
                            "required": false,
                            "schema": {
                                "type": "integer",
                                "default": 20
                            }
                        },
                        {
                            "name": "offset",
                            "in": "query",
                            "required": false,
                            "schema": {
                                "type": "integer",
                                "default": 0
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Search results",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/SearchResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/admin/tenants": {
                "get": {
                    "summary": "List tenants",
                    "description": "List all tenants (admin only)",
                    "tags": ["Admin"],
                    "responses": {
                        "200": {
                            "description": "List of tenants",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/TenantListResponse"
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create tenant",
                    "description": "Create a new tenant (admin only)",
                    "tags": ["Admin"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/CreateTenantRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Tenant created successfully",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/TenantResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "Repository": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "format": "uuid"
                        },
                        "name": {
                            "type": "string"
                        },
                        "description": {
                            "type": "string"
                        },
                        "created_at": {
                            "type": "string",
                            "format": "date-time"
                        },
                        "updated_at": {
                            "type": "string",
                            "format": "date-time"
                        }
                    }
                },
                "RepositoryListResponse": {
                    "type": "object",
                    "properties": {
                        "success": {
                            "type": "boolean"
                        },
                        "data": {
                            "type": "array",
                            "items": {
                                "$ref": "#/components/schemas/Repository"
                            }
                        }
                    }
                },
                "RepositoryResponse": {
                    "type": "object",
                    "properties": {
                        "success": {
                            "type": "boolean"
                        },
                        "data": {
                            "$ref": "#/components/schemas/Repository"
                        }
                    }
                },
                "CreateRepositoryRequest": {
                    "type": "object",
                    "required": ["name"],
                    "properties": {
                        "name": {
                            "type": "string"
                        },
                        "description": {
                            "type": "string"
                        }
                    }
                },
                "TreeEntry": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "type": {
                            "type": "string",
                            "enum": ["file", "directory"]
                        },
                        "size": {
                            "type": "integer"
                        },
                        "modified_at": {
                            "type": "string",
                            "format": "date-time"
                        }
                    }
                },
                "TreeResponse": {
                    "type": "object",
                    "properties": {
                        "success": {
                            "type": "boolean"
                        },
                        "data": {
                            "type": "array",
                            "items": {
                                "$ref": "#/components/schemas/TreeEntry"
                            }
                        }
                    }
                },
                "SearchResult": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string"
                        },
                        "repo_name": {
                            "type": "string"
                        },
                        "path": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "content_type": {
                            "type": "string"
                        },
                        "size": {
                            "type": "integer"
                        },
                        "modified_at": {
                            "type": "string",
                            "format": "date-time"
                        }
                    }
                },
                "SearchResponse": {
                    "type": "object",
                    "properties": {
                        "success": {
                            "type": "boolean"
                        },
                        "data": {
                            "type": "object",
                            "properties": {
                                "results": {
                                    "type": "array",
                                    "items": {
                                        "$ref": "#/components/schemas/SearchResult"
                                    }
                                },
                                "total": {
                                    "type": "integer"
                                },
                                "limit": {
                                    "type": "integer"
                                },
                                "offset": {
                                    "type": "integer"
                                }
                            }
                        }
                    }
                },
                "Tenant": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "format": "uuid"
                        },
                        "name": {
                            "type": "string"
                        },
                        "created_at": {
                            "type": "string",
                            "format": "date-time"
                        }
                    }
                },
                "TenantListResponse": {
                    "type": "object",
                    "properties": {
                        "success": {
                            "type": "boolean"
                        },
                        "data": {
                            "type": "array",
                            "items": {
                                "$ref": "#/components/schemas/Tenant"
                            }
                        }
                    }
                },
                "TenantResponse": {
                    "type": "object",
                    "properties": {
                        "success": {
                            "type": "boolean"
                        },
                        "data": {
                            "$ref": "#/components/schemas/Tenant"
                        }
                    }
                },
                "CreateTenantRequest": {
                    "type": "object",
                    "required": ["name"],
                    "properties": {
                        "name": {
                            "type": "string"
                        }
                    }
                }
            },
            "securitySchemes": {
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                },
                "SessionAuth": {
                    "type": "apiKey",
                    "in": "cookie",
                    "name": "session"
                }
            }
        },
        "security": [
            {
                "BearerAuth": []
            },
            {
                "SessionAuth": []
            }
        ],
        "tags": [
            {
                "name": "Repositories",
                "description": "Repository management operations"
            },
            {
                "name": "Search",
                "description": "Search and discovery operations"
            },
            {
                "name": "Admin",
                "description": "Administrative operations (admin only)"
            }
        ]
    })
}

/// Get OpenAPI specification
async fn get_openapi_spec(
    State(_state): State<AppState>,
) -> Result<Json<Value>, blacklake_core::ApiError> {
    let spec = generate_openapi_spec();
    Ok(Json(spec))
}

/// Create OpenAPI routes
pub fn create_openapi_routes() -> Router<AppState> {
    Router::new()
        .route("/openapi.json", get(get_openapi_spec))
}
