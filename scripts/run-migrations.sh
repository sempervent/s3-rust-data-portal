#!/bin/bash
# BlackLake Database Migration Runner
# This script runs all migrations in the correct order

set -e

echo "🚀 Starting BlackLake database migrations..."

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "❌ Error: DATABASE_URL environment variable is not set"
    exit 1
fi

# Wait for database to be ready
echo "⏳ Waiting for database to be ready..."
until pg_isready -d "$DATABASE_URL" > /dev/null 2>&1; do
    echo "   Database not ready, waiting 2 seconds..."
    sleep 2
done

echo "✅ Database is ready!"

# Run migrations in order
echo "📦 Running migrations..."

# Run SQLx migrations (if any)
if [ -d "migrations" ] && [ "$(ls -A migrations/*.sql 2>/dev/null)" ]; then
    echo "   Running SQLx migrations..."
    sqlx migrate run
    echo "   ✅ SQLx migrations completed"
else
    echo "   ⚠️  No SQLx migrations found"
fi

# Run custom migrations
echo "   Running custom migrations..."

# Migration 1: Initial schema
if [ -f "migrations/0001_init.sql" ]; then
    echo "   📄 Running 0001_init.sql..."
    psql "$DATABASE_URL" -f migrations/0001_init.sql
fi

# Migration 2: Metadata index and RDF
if [ -f "migrations/0002_metadata_index_and_rdf.sql" ]; then
    echo "   📄 Running 0002_metadata_index_and_rdf.sql..."
    psql "$DATABASE_URL" -f migrations/0002_metadata_index_and_rdf.sql
fi

# Migration 3: Governance features
if [ -f "migrations/0003_governance_features.sql" ]; then
    echo "   📄 Running 0003_governance_features.sql..."
    psql "$DATABASE_URL" -f migrations/0003_governance_features.sql
fi

# Migration 4: Lineage and quotas
if [ -f "migrations/0003_lineage_and_quotas.sql" ]; then
    echo "   📄 Running 0003_lineage_and_quotas.sql..."
    psql "$DATABASE_URL" -f migrations/0003_lineage_and_quotas.sql
fi

# Migration 5: Multitenant ABAC
if [ -f "migrations/0004_multitenant_abac.sql" ]; then
    echo "   📄 Running 0004_multitenant_abac.sql..."
    psql "$DATABASE_URL" -f migrations/0004_multitenant_abac.sql
fi

# Migration 6: Data classification
if [ -f "migrations/0005_data_classification.sql" ]; then
    echo "   📄 Running 0005_data_classification.sql..."
    psql "$DATABASE_URL" -f migrations/0005_data_classification.sql
fi

# Migration 7: External sources
if [ -f "migrations/0006_external_sources.sql" ]; then
    echo "   📄 Running 0006_external_sources.sql..."
    psql "$DATABASE_URL" -f migrations/0006_external_sources.sql
fi

# Migration 8: Compliance features
if [ -f "migrations/0007_compliance_features.sql" ]; then
    echo "   📄 Running 0007_compliance_features.sql..."
    psql "$DATABASE_URL" -f migrations/0007_compliance_features.sql
fi

# Migration 9: Compliance jobs
if [ -f "migrations/0008_compliance_jobs.sql" ]; then
    echo "   📄 Running 0008_compliance_jobs.sql..."
    psql "$DATABASE_URL" -f migrations/0008_compliance_jobs.sql
fi

# Migration 10: Fix missing columns
if [ -f "migrations/0009_fix_missing_columns.sql" ]; then
    echo "   📄 Running 0009_fix_missing_columns.sql..."
    psql "$DATABASE_URL" -f migrations/0009_fix_missing_columns.sql
fi

# Migration 11: API missing tables
if [ -f "migrations/0010_api_missing_tables.sql" ]; then
    echo "   📄 Running 0010_api_missing_tables.sql..."
    psql "$DATABASE_URL" -f migrations/0010_api_missing_tables.sql
fi

echo "🎉 All migrations completed successfully!"
echo "📊 Database schema is now up to date"
