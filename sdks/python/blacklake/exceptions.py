"""
BlackLake Python SDK Exceptions

Custom exceptions for the BlackLake SDK.
"""


class BlackLakeError(Exception):
    """Base exception for BlackLake SDK errors."""
    
    def __init__(self, message: str, details: dict = None):
        super().__init__(message)
        self.message = message
        self.details = details or {}


class AuthenticationError(BlackLakeError):
    """Authentication failed."""
    pass


class AuthorizationError(BlackLakeError):
    """Access denied."""
    pass


class NotFoundError(BlackLakeError):
    """Resource not found."""
    pass


class ValidationError(BlackLakeError):
    """Request validation failed."""
    pass


class RateLimitError(BlackLakeError):
    """Rate limit exceeded."""
    pass


class ServerError(BlackLakeError):
    """Server error."""
    pass


class NetworkError(BlackLakeError):
    """Network error."""
    pass


class TimeoutError(BlackLakeError):
    """Request timeout."""
    pass
