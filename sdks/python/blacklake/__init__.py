"""
BlackLake Data Portal Python SDK

Enterprise data portal with multi-tenant access controls and governance.
"""

from .client import BlackLakeClient
from .exceptions import BlackLakeError, AuthenticationError, AuthorizationError, NotFoundError
from .models import Repository, SearchResult, SearchResponse, TreeEntry, TreeResponse

__version__ = "0.1.0"
__all__ = [
    "BlackLakeClient",
    "BlackLakeError",
    "AuthenticationError", 
    "AuthorizationError",
    "NotFoundError",
    "Repository",
    "SearchResult",
    "SearchResponse",
    "TreeEntry",
    "TreeResponse",
]
