# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

Please report security vulnerabilities to security@blacklake.dev.

## Security Features

### Authentication & Authorization
- OIDC integration with JWT tokens
- Session-based authentication with secure cookies
- CSRF protection for state-changing operations
- Multi-tenant access controls with ABAC policies

### Data Protection
- Data classification (public, internal, restricted, secret)
- Encryption at rest and in transit
- Secure file uploads with presigned URLs
- Audit logging for all operations

### Network Security
- Security headers (CSP, HSTS, X-Frame-Options)
- Rate limiting and DDoS protection
- Network policies in Kubernetes
- TLS 1.3 encryption

### Session Management
- Secure session cookies with HttpOnly and Secure flags
- Session timeout and idle timeout
- Session revocation capabilities
- IP-based session validation

## Security Headers

The application implements comprehensive security headers:

- **Content-Security-Policy**: Restricts resource loading
- **Strict-Transport-Security**: Enforces HTTPS
- **X-Content-Type-Options**: Prevents MIME sniffing
- **X-Frame-Options**: Prevents clickjacking
- **X-XSS-Protection**: XSS protection
- **Referrer-Policy**: Controls referrer information
- **Permissions-Policy**: Restricts browser features

## Data Classification

Data is classified into four levels:

1. **Public**: No restrictions
2. **Internal**: Company use only
3. **Restricted**: Limited access with approval
4. **Secret**: Highly sensitive, encrypted storage

## Policy Enforcement

Access control is enforced through:

- Attribute-based access control (ABAC)
- Path-prefix policies
- Resource-level permissions
- Time-based access controls
- IP-based restrictions

## Audit Logging

All operations are logged with:

- User identification
- Action performed
- Resource accessed
- Timestamp
- IP address
- Policy decisions
- Success/failure status
