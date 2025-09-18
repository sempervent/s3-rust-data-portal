# Multi-Tenancy Guide

## Overview

BlackLake supports multi-tenant architecture with attribute-based access control (ABAC) for enterprise deployments.

## Tenant Management

### Creating Tenants

```bash
# Create a new tenant
curl -X POST http://localhost:8080/v1/admin/tenants \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "acme-corp"}'
```

### Listing Tenants

```bash
# List all tenants
curl -X GET http://localhost:8080/v1/admin/tenants \
  -H "Authorization: Bearer $TOKEN"
```

## Policy Management

### Creating Policies

```bash
# Create an allow policy
curl -X POST http://localhost:8080/v1/admin/tenants/{tenant_id}/policies \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "allow-data-scientists",
    "effect": "allow",
    "actions": ["read", "search"],
    "resources": ["repo:datasets/**"],
    "condition": {
      "field": "subject.roles",
      "operator": "contains",
      "value": "data-scientist"
    }
  }'
```

### Policy Conditions

Supported operators:
- `equals`: Exact match
- `not_equals`: Not equal
- `in`: Value in array
- `not_in`: Value not in array
- `contains`: Array contains value
- `not_contains`: Array does not contain value
- `startsWith`: String starts with
- `endsWith`: String ends with
- `regex`: Regular expression match
- `greaterThan`: Numeric greater than
- `lessThan`: Numeric less than

### Resource Patterns

- `*`: All resources
- `repo:my-repo`: Specific repository
- `repo:my-repo/**`: Repository and all subpaths
- `repo:my-repo/data/*`: Single level wildcard
- `admin:*`: Admin functions

## Subject Attributes

### Setting Attributes

```bash
# Set user role
curl -X POST http://localhost:8080/v1/admin/attributes \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "user123",
    "key": "roles",
    "value": "data-scientist"
  }'
```

### Common Attributes

- `roles`: User roles (array)
- `org`: Organization
- `department`: Department
- `level`: Access level
- `location`: Geographic location
- `classification`: Security clearance

## Policy Examples

### Data Scientist Access

```json
{
  "name": "data-scientist-read",
  "effect": "allow",
  "actions": ["read", "search"],
  "resources": ["repo:datasets/**"],
  "condition": {
    "field": "subject.roles",
    "operator": "contains",
    "value": "data-scientist"
  }
}
```

### Restricted Data Access

```json
{
  "name": "restricted-data-access",
  "effect": "allow",
  "actions": ["read"],
  "resources": ["repo:secure/**"],
  "condition": {
    "field": "subject.classification",
    "operator": "in",
    "value": ["restricted", "secret"]
  }
}
```

### Admin Functions

```json
{
  "name": "admin-access",
  "effect": "allow",
  "actions": ["admin"],
  "resources": ["admin:*"],
  "condition": {
    "field": "subject.roles",
    "operator": "contains",
    "value": "admin"
  }
}
```

## Testing Policies

### Policy Test Endpoint

```bash
# Test a policy
curl -X POST http://localhost:8080/v1/admin/tenants/{tenant_id}/policies/test \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "user123",
    "action": "read",
    "resource": "repo:datasets/sales.csv",
    "context": {
      "subject.roles": ["data-scientist"],
      "subject.org": "acme-corp"
    }
  }'
```

## Best Practices

1. **Principle of Least Privilege**: Grant minimum required access
2. **Regular Audits**: Review policies and access regularly
3. **Clear Naming**: Use descriptive policy names
4. **Documentation**: Document policy purposes and conditions
5. **Testing**: Test policies before deployment
6. **Monitoring**: Monitor policy decisions and access patterns

## Troubleshooting

### Common Issues

1. **Access Denied**: Check policy conditions and subject attributes
2. **Policy Not Applied**: Verify policy is active and conditions match
3. **Attribute Missing**: Ensure subject attributes are set correctly
4. **Resource Pattern**: Check resource pattern matching

### Debug Mode

Enable debug logging to see policy evaluation:

```bash
export RUST_LOG=debug
```

This will show:
- Policy evaluation steps
- Condition matching
- Final decision
- Matched policies
