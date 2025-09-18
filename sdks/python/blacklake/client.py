"""
BlackLake Python SDK Client

Main client for interacting with the BlackLake Data Portal API.
"""

import asyncio
from typing import Dict, List, Optional, Union, Any
from urllib.parse import urljoin

import httpx
from pydantic import BaseModel, Field

from .exceptions import BlackLakeError, AuthenticationError, AuthorizationError, NotFoundError
from .models import Repository, SearchResult, SearchResponse, TreeEntry, TreeResponse


class BlackLakeClient:
    """BlackLake Data Portal API client."""
    
    def __init__(
        self,
        base_url: str = "http://localhost:8080",
        api_key: Optional[str] = None,
        timeout: float = 30.0,
        verify_ssl: bool = True,
    ):
        """
        Initialize the BlackLake client.
        
        Args:
            base_url: Base URL of the BlackLake API
            api_key: API key for authentication (optional)
            timeout: Request timeout in seconds
            verify_ssl: Whether to verify SSL certificates
        """
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.timeout = timeout
        self.verify_ssl = verify_ssl
        
        # Create HTTP client
        self._client = httpx.AsyncClient(
            base_url=self.base_url,
            timeout=timeout,
            verify=verify_ssl,
            headers=self._get_headers(),
        )
    
    def _get_headers(self) -> Dict[str, str]:
        """Get default headers for API requests."""
        headers = {
            "Content-Type": "application/json",
            "User-Agent": f"blacklake-sdk-python/{__import__('blacklake').__version__}",
        }
        
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        
        return headers
    
    async def __aenter__(self):
        """Async context manager entry."""
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.close()
    
    async def close(self):
        """Close the HTTP client."""
        await self._client.aclose()
    
    async def _request(
        self,
        method: str,
        endpoint: str,
        params: Optional[Dict[str, Any]] = None,
        json: Optional[Dict[str, Any]] = None,
        headers: Optional[Dict[str, str]] = None,
    ) -> Dict[str, Any]:
        """
        Make an HTTP request to the API.
        
        Args:
            method: HTTP method
            endpoint: API endpoint
            params: Query parameters
            json: JSON payload
            headers: Additional headers
            
        Returns:
            Response data
            
        Raises:
            BlackLakeError: For API errors
        """
        request_headers = self._get_headers()
        if headers:
            request_headers.update(headers)
        
        try:
            response = await self._client.request(
                method=method,
                url=endpoint,
                params=params,
                json=json,
                headers=request_headers,
            )
            
            # Handle different status codes
            if response.status_code == 401:
                raise AuthenticationError("Authentication failed")
            elif response.status_code == 403:
                raise AuthorizationError("Access denied")
            elif response.status_code == 404:
                raise NotFoundError("Resource not found")
            elif response.status_code >= 400:
                error_data = response.json() if response.headers.get("content-type", "").startswith("application/json") else {}
                error_message = error_data.get("error", f"HTTP {response.status_code}")
                raise BlackLakeError(f"API error: {error_message}")
            
            # Parse response
            if response.headers.get("content-type", "").startswith("application/json"):
                return response.json()
            else:
                return {"data": response.text}
                
        except httpx.RequestError as e:
            raise BlackLakeError(f"Request failed: {e}")
    
    # Repository operations
    
    async def list_repositories(self) -> List[Repository]:
        """List all accessible repositories."""
        response = await self._request("GET", "/v1/repos")
        return [Repository(**repo) for repo in response.get("data", [])]
    
    async def get_repository(self, name: str) -> Repository:
        """Get repository details."""
        response = await self._request("GET", f"/v1/repos/{name}")
        return Repository(**response["data"])
    
    async def create_repository(
        self,
        name: str,
        description: Optional[str] = None,
    ) -> Repository:
        """Create a new repository."""
        payload = {"name": name}
        if description:
            payload["description"] = description
        
        response = await self._request("POST", "/v1/repos", json=payload)
        return Repository(**response["data"])
    
    async def get_repository_tree(
        self,
        repo_name: str,
        ref: str = "main",
        path: Optional[str] = None,
    ) -> TreeResponse:
        """Get repository file tree."""
        params = {}
        if path:
            params["path"] = path
        
        response = await self._request(
            "GET",
            f"/v1/repos/{repo_name}/tree/{ref}",
            params=params,
        )
        return TreeResponse(**response)
    
    # Search operations
    
    async def search(
        self,
        query: str,
        limit: int = 20,
        offset: int = 0,
        repo: Optional[str] = None,
        classification: Optional[str] = None,
    ) -> SearchResponse:
        """
        Search across repositories.
        
        Args:
            query: Search query
            limit: Maximum number of results
            offset: Number of results to skip
            repo: Filter by repository name
            classification: Filter by data classification
        """
        params = {
            "q": query,
            "limit": limit,
            "offset": offset,
        }
        
        if repo:
            params["repo"] = repo
        if classification:
            params["classification"] = classification
        
        response = await self._request("GET", "/v1/search", params=params)
        return SearchResponse(**response)
    
    async def search_suggestions(
        self,
        query: str,
        count: int = 10,
    ) -> List[str]:
        """Get search suggestions."""
        params = {"q": query, "count": count}
        response = await self._request("GET", "/v1/search/suggest", params=params)
        return response.get("data", [])
    
    # File operations
    
    async def get_file_metadata(
        self,
        repo_name: str,
        ref: str,
        path: str,
    ) -> Dict[str, Any]:
        """Get file metadata."""
        response = await self._request(
            "GET",
            f"/v1/repos/{repo_name}/metadata/{ref}/{path}",
        )
        return response["data"]
    
    async def update_file_metadata(
        self,
        repo_name: str,
        ref: str,
        path: str,
        metadata: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Update file metadata."""
        response = await self._request(
            "PUT",
            f"/v1/repos/{repo_name}/metadata/{ref}/{path}",
            json=metadata,
        )
        return response["data"]
    
    # Upload operations
    
    async def initiate_upload(
        self,
        repo_name: str,
        path: str,
        size: int,
        content_type: str,
    ) -> Dict[str, Any]:
        """Initiate file upload."""
        payload = {
            "path": path,
            "size": size,
            "content_type": content_type,
        }
        
        response = await self._request(
            "POST",
            f"/v1/repos/{repo_name}/upload/init",
            json=payload,
        )
        return response["data"]
    
    async def commit_upload(
        self,
        repo_name: str,
        upload_id: str,
        message: str,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        """Commit uploaded files."""
        payload = {
            "upload_id": upload_id,
            "message": message,
        }
        
        if metadata:
            payload["metadata"] = metadata
        
        response = await self._request(
            "POST",
            f"/v1/repos/{repo_name}/commit",
            json=payload,
        )
        return response["data"]
    
    # Export operations
    
    async def export_repository(
        self,
        repo_name: str,
        ref: str = "main",
        format: str = "zip",
    ) -> Dict[str, Any]:
        """Export repository."""
        params = {"ref": ref, "format": format}
        response = await self._request(
            "POST",
            f"/v1/repos/{repo_name}/export",
            params=params,
        )
        return response["data"]
    
    async def get_export_status(
        self,
        export_id: str,
    ) -> Dict[str, Any]:
        """Get export status."""
        response = await self._request("GET", f"/v1/exports/{export_id}")
        return response["data"]
    
    # Health check
    
    async def health_check(self) -> Dict[str, Any]:
        """Check API health."""
        response = await self._request("GET", "/health")
        return response


# Synchronous wrapper for convenience
class BlackLakeClientSync:
    """Synchronous wrapper for BlackLakeClient."""
    
    def __init__(self, *args, **kwargs):
        self._client = BlackLakeClient(*args, **kwargs)
        self._loop = None
    
    def __enter__(self):
        self._loop = asyncio.new_event_loop()
        asyncio.set_event_loop(self._loop)
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self._loop:
            self._loop.close()
    
    def __getattr__(self, name):
        """Delegate method calls to the async client."""
        method = getattr(self._client, name)
        
        if asyncio.iscoroutinefunction(method):
            def sync_wrapper(*args, **kwargs):
                return self._loop.run_until_complete(method(*args, **kwargs))
            return sync_wrapper
        else:
            return method
