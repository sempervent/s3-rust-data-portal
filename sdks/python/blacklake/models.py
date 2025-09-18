"""
BlackLake Python SDK Models

Pydantic models for API responses and requests.
"""

from datetime import datetime
from typing import Dict, List, Optional, Any, Union
from pydantic import BaseModel, Field


class Repository(BaseModel):
    """Repository model."""
    
    id: str = Field(..., description="Repository ID")
    name: str = Field(..., description="Repository name")
    description: Optional[str] = Field(None, description="Repository description")
    created_at: datetime = Field(..., description="Creation timestamp")
    updated_at: datetime = Field(..., description="Last update timestamp")
    tenant_id: Optional[str] = Field(None, description="Tenant ID")


class TreeEntry(BaseModel):
    """File tree entry model."""
    
    path: str = Field(..., description="File path")
    name: str = Field(..., description="File name")
    type: str = Field(..., description="Entry type (file or directory)")
    size: Optional[int] = Field(None, description="File size in bytes")
    modified_at: Optional[datetime] = Field(None, description="Last modification timestamp")
    content_type: Optional[str] = Field(None, description="MIME type")
    classification: Optional[str] = Field(None, description="Data classification")


class TreeResponse(BaseModel):
    """Tree response model."""
    
    success: bool = Field(..., description="Success status")
    data: List[TreeEntry] = Field(..., description="Tree entries")


class SearchResult(BaseModel):
    """Search result model."""
    
    id: str = Field(..., description="Result ID")
    repo_name: str = Field(..., description="Repository name")
    path: str = Field(..., description="File path")
    name: str = Field(..., description="File name")
    content_type: Optional[str] = Field(None, description="MIME type")
    size: Optional[int] = Field(None, description="File size")
    modified_at: Optional[datetime] = Field(None, description="Last modification timestamp")
    classification: Optional[str] = Field(None, description="Data classification")
    score: Optional[float] = Field(None, description="Search relevance score")
    highlights: Optional[Dict[str, List[str]]] = Field(None, description="Search highlights")


class SearchFacet(BaseModel):
    """Search facet model."""
    
    name: str = Field(..., description="Facet name")
    values: List[Dict[str, Union[str, int]]] = Field(..., description="Facet values")


class SearchResponse(BaseModel):
    """Search response model."""
    
    success: bool = Field(..., description="Success status")
    data: Dict[str, Any] = Field(..., description="Search results data")
    
    @property
    def results(self) -> List[SearchResult]:
        """Get search results."""
        return [SearchResult(**result) for result in self.data.get("results", [])]
    
    @property
    def total(self) -> int:
        """Get total number of results."""
        return self.data.get("total", 0)
    
    @property
    def limit(self) -> int:
        """Get result limit."""
        return self.data.get("limit", 20)
    
    @property
    def offset(self) -> int:
        """Get result offset."""
        return self.data.get("offset", 0)
    
    @property
    def facets(self) -> List[SearchFacet]:
        """Get search facets."""
        facets_data = self.data.get("facets", [])
        return [SearchFacet(**facet) for facet in facets_data]


class UploadInitResponse(BaseModel):
    """Upload initialization response model."""
    
    upload_id: str = Field(..., description="Upload ID")
    presigned_url: str = Field(..., description="Presigned upload URL")
    expires_at: datetime = Field(..., description="Upload expiration timestamp")


class CommitResponse(BaseModel):
    """Commit response model."""
    
    commit_id: str = Field(..., description="Commit ID")
    message: str = Field(..., description="Commit message")
    created_at: datetime = Field(..., description="Commit timestamp")
    files_count: int = Field(..., description="Number of files committed")


class ExportResponse(BaseModel):
    """Export response model."""
    
    export_id: str = Field(..., description="Export ID")
    status: str = Field(..., description="Export status")
    created_at: datetime = Field(..., description="Export creation timestamp")
    download_url: Optional[str] = Field(None, description="Download URL when ready")


class HealthResponse(BaseModel):
    """Health check response model."""
    
    status: str = Field(..., description="Health status")
    timestamp: datetime = Field(..., description="Check timestamp")
    version: Optional[str] = Field(None, description="API version")
    uptime: Optional[float] = Field(None, description="Uptime in seconds")


class Policy(BaseModel):
    """Access policy model."""
    
    id: str = Field(..., description="Policy ID")
    tenant_id: str = Field(..., description="Tenant ID")
    name: str = Field(..., description="Policy name")
    effect: str = Field(..., description="Policy effect (allow/deny)")
    actions: List[str] = Field(..., description="Allowed actions")
    resources: List[str] = Field(..., description="Resource patterns")
    condition: Optional[Dict[str, Any]] = Field(None, description="Policy condition")
    created_at: datetime = Field(..., description="Creation timestamp")
    updated_at: datetime = Field(..., description="Last update timestamp")


class Tenant(BaseModel):
    """Tenant model."""
    
    id: str = Field(..., description="Tenant ID")
    name: str = Field(..., description="Tenant name")
    created_at: datetime = Field(..., description="Creation timestamp")


class SubjectAttribute(BaseModel):
    """Subject attribute model."""
    
    subject: str = Field(..., description="Subject identifier")
    key: str = Field(..., description="Attribute key")
    value: str = Field(..., description="Attribute value")
    created_at: datetime = Field(..., description="Creation timestamp")


class ApiResponse(BaseModel):
    """Generic API response model."""
    
    success: bool = Field(..., description="Success status")
    data: Any = Field(..., description="Response data")
    error: Optional[str] = Field(None, description="Error message")
    message: Optional[str] = Field(None, description="Response message")
