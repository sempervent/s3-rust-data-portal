#!/bin/bash
# BlackLake SolrCloud Initialization Script
# Week 6: Initialize SolrCloud collection with BlackLake configset

set -e

SOLR_URL="${SOLR_URL:-http://localhost:8983/solr}"
COLLECTION_NAME="${COLLECTION_NAME:-blacklake}"
CONFIGSET_NAME="${CONFIGSET_NAME:-blacklake}"
NUM_SHARDS="${NUM_SHARDS:-1}"
REPLICATION_FACTOR="${REPLICATION_FACTOR:-1}"

echo "Initializing SolrCloud collection: $COLLECTION_NAME"
echo "Solr URL: $SOLR_URL"
echo "Configset: $CONFIGSET_NAME"
echo "Shards: $NUM_SHARDS"
echo "Replication Factor: $REPLICATION_FACTOR"

# Wait for Solr to be ready
echo "Waiting for Solr to be ready..."
until curl -f "$SOLR_URL/admin/ping" > /dev/null 2>&1; do
    echo "Solr not ready, waiting..."
    sleep 5
done
echo "Solr is ready!"

# Check if collection already exists
if curl -f "$SOLR_URL/admin/collections?action=LIST" | grep -q "\"$COLLECTION_NAME\""; then
    echo "Collection $COLLECTION_NAME already exists"
    exit 0
fi

# Check if configset exists
if ! curl -f "$SOLR_URL/admin/configs?action=LIST" | grep -q "\"$CONFIGSET_NAME\""; then
    echo "Configset $CONFIGSET_NAME not found, creating..."
    
    # Upload configset
    curl -X POST "$SOLR_URL/admin/configs?action=UPLOAD&name=$CONFIGSET_NAME" \
        -F "file=@/opt/solr/configsets/blacklake/managed-schema" \
        -F "file=@/opt/solr/configsets/blacklake/solrconfig.xml" \
        -F "file=@/opt/solr/configsets/blacklake/synonyms.txt" \
        -F "file=@/opt/solr/configsets/blacklake/stopwords.txt"
    
    echo "Configset $CONFIGSET_NAME created"
else
    echo "Configset $CONFIGSET_NAME already exists"
fi

# Create collection
echo "Creating collection $COLLECTION_NAME..."
curl -X POST "$SOLR_URL/admin/collections?action=CREATE" \
    -d "name=$COLLECTION_NAME" \
    -d "numShards=$NUM_SHARDS" \
    -d "replicationFactor=$REPLICATION_FACTOR" \
    -d "collection.configName=$CONFIGSET_NAME"

echo "Collection $COLLECTION_NAME created successfully!"

# Verify collection
echo "Verifying collection..."
curl -f "$SOLR_URL/$COLLECTION_NAME/admin/ping" > /dev/null 2>&1
echo "Collection verification successful!"

# Create suggesters
echo "Building suggesters..."
curl -X POST "$SOLR_URL/$COLLECTION_NAME/suggest?build=true"

echo "SolrCloud initialization complete!"
echo "Collection: $COLLECTION_NAME"
echo "URL: $SOLR_URL/$COLLECTION_NAME"
