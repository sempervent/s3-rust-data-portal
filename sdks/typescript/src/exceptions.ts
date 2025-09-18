/**
 * BlackLake TypeScript SDK Exceptions
 * 
 * Custom exceptions for the BlackLake SDK.
 */

export class BlackLakeError extends Error {
  public readonly details?: Record<string, any>

  constructor(message: string, details?: Record<string, any>) {
    super(message)
    this.name = 'BlackLakeError'
    this.details = details
  }
}

export class AuthenticationError extends BlackLakeError {
  constructor(message: string = 'Authentication failed', details?: Record<string, any>) {
    super(message, details)
    this.name = 'AuthenticationError'
  }
}

export class AuthorizationError extends BlackLakeError {
  constructor(message: string = 'Access denied', details?: Record<string, any>) {
    super(message, details)
    this.name = 'AuthorizationError'
  }
}

export class NotFoundError extends BlackLakeError {
  constructor(message: string = 'Resource not found', details?: Record<string, any>) {
    super(message, details)
    this.name = 'NotFoundError'
  }
}

export class ValidationError extends BlackLakeError {
  constructor(message: string = 'Validation failed', details?: Record<string, any>) {
    super(message, details)
    this.name = 'ValidationError'
  }
}

export class RateLimitError extends BlackLakeError {
  constructor(message: string = 'Rate limit exceeded', details?: Record<string, any>) {
    super(message, details)
    this.name = 'RateLimitError'
  }
}

export class ServerError extends BlackLakeError {
  constructor(message: string = 'Server error', details?: Record<string, any>) {
    super(message, details)
    this.name = 'ServerError'
  }
}

export class NetworkError extends BlackLakeError {
  constructor(message: string = 'Network error', details?: Record<string, any>) {
    super(message, details)
    this.name = 'NetworkError'
  }
}

export class TimeoutError extends BlackLakeError {
  constructor(message: string = 'Request timeout', details?: Record<string, any>) {
    super(message, details)
    this.name = 'TimeoutError'
  }
}
