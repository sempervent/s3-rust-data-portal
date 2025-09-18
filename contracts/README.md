# BlackLake API Contract Tests

This directory contains contract tests for the BlackLake API to ensure backward compatibility and API consistency.

## Structure

- `schemas/` - OpenAPI schemas for different API versions
- `tests/` - Contract test implementations
- `fixtures/` - Test data and fixtures

## Running Tests

```bash
# Run all contract tests
cargo test --package blacklake-api --test contract_tests

# Run specific version tests
cargo test --package blacklake-api --test contract_tests v1

# Run with schemathesis (Python)
pip install schemathesis
schemathesis run http://localhost:8080/openapi.json --base-url http://localhost:8080
```

## API Versioning

- **v1**: Current stable API (frozen)
- **v2**: Future API version (in development)

## Contract Test Requirements

1. All v1 endpoints must maintain backward compatibility
2. Response schemas must not change without version bump
3. New fields must be optional or have defaults
4. Deprecated fields must be marked and documented
