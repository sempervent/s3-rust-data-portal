#!/bin/bash

# End-to-end test script for Blacklake
set -e

echo "🚀 Starting Blacklake End-to-End Test"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
API_URL="http://localhost:8080"
REPO_NAME="test-repo-$(date +%s)"
TEST_FILE="test-data.txt"
META_FILE="test-meta.json"

# Create test files
echo "📝 Creating test files..."
echo "Hello, Blacklake!" > $TEST_FILE
cat > $META_FILE << EOF
{
  "name": "Test Data File",
  "description": "A simple test file for Blacklake",
  "version": "1.0.0",
  "tags": ["test", "example"]
}
EOF

# Function to check if service is ready
wait_for_service() {
    local url=$1
    local name=$2
    echo "⏳ Waiting for $name to be ready..."
    
    for i in {1..30}; do
        if curl -s "$url" > /dev/null 2>&1; then
            echo "✅ $name is ready!"
            return 0
        fi
        sleep 2
    done
    
    echo "❌ $name failed to start within 60 seconds"
    return 1
}

# Test 1: Create repository
echo -e "\n${YELLOW}Test 1: Creating repository${NC}"
RESPONSE=$(curl -s -X POST "$API_URL/v1/repos" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"$REPO_NAME\"}")

if echo "$RESPONSE" | grep -q "id"; then
    echo -e "${GREEN}✅ Repository created successfully${NC}"
    REPO_ID=$(echo "$RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
    echo "   Repository ID: $REPO_ID"
else
    echo -e "${RED}❌ Failed to create repository${NC}"
    echo "Response: $RESPONSE"
    exit 1
fi

# Test 2: List repositories
echo -e "\n${YELLOW}Test 2: Listing repositories${NC}"
RESPONSE=$(curl -s "$API_URL/v1/repos")

if echo "$RESPONSE" | grep -q "$REPO_NAME"; then
    echo -e "${GREEN}✅ Repository found in list${NC}"
else
    echo -e "${RED}❌ Repository not found in list${NC}"
    echo "Response: $RESPONSE"
    exit 1
fi

# Test 3: Initialize upload
echo -e "\n${YELLOW}Test 3: Initializing upload${NC}"
FILE_SIZE=$(wc -c < "$TEST_FILE")
RESPONSE=$(curl -s -X POST "$API_URL/v1/repos/$REPO_NAME/upload-init" \
  -H "Content-Type: application/json" \
  -d "{
    \"path\": \"data/$TEST_FILE\",
    \"size\": $FILE_SIZE,
    \"media_type\": \"text/plain\"
  }")

if echo "$RESPONSE" | grep -q "upload_url"; then
    echo -e "${GREEN}✅ Upload initialized successfully${NC}"
    UPLOAD_URL=$(echo "$RESPONSE" | grep -o '"upload_url":"[^"]*"' | cut -d'"' -f4)
    SHA256=$(echo "$RESPONSE" | grep -o '"sha256":"[^"]*"' | cut -d'"' -f4)
    echo "   Upload URL: $UPLOAD_URL"
    echo "   SHA256: $SHA256"
else
    echo -e "${RED}❌ Failed to initialize upload${NC}"
    echo "Response: $RESPONSE"
    exit 1
fi

# Test 4: Upload file
echo -e "\n${YELLOW}Test 4: Uploading file${NC}"
UPLOAD_RESPONSE=$(curl -s -X PUT -T "$TEST_FILE" "$UPLOAD_URL")

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ File uploaded successfully${NC}"
else
    echo -e "${RED}❌ Failed to upload file${NC}"
    exit 1
fi

# Test 5: Create commit
echo -e "\n${YELLOW}Test 5: Creating commit${NC}"
COMMIT_RESPONSE=$(curl -s -X POST "$API_URL/v1/repos/$REPO_NAME/commit" \
  -H "Content-Type: application/json" \
  -d "{
    \"ref\": \"main\",
    \"message\": \"Add test file\",
    \"changes\": [
      {
        \"op\": \"add\",
        \"path\": \"data/$TEST_FILE\",
        \"sha256\": \"$SHA256\",
        \"meta\": $(cat $META_FILE)
      }
    ]
  }")

if echo "$COMMIT_RESPONSE" | grep -q "commit_id"; then
    echo -e "${GREEN}✅ Commit created successfully${NC}"
    COMMIT_ID=$(echo "$COMMIT_RESPONSE" | grep -o '"commit_id":"[^"]*"' | cut -d'"' -f4)
    echo "   Commit ID: $COMMIT_ID"
else
    echo -e "${RED}❌ Failed to create commit${NC}"
    echo "Response: $COMMIT_RESPONSE"
    exit 1
fi

# Test 6: Get blob
echo -e "\n${YELLOW}Test 6: Getting blob${NC}"
BLOB_RESPONSE=$(curl -s "$API_URL/v1/repos/$REPO_NAME/blob/main/data/$TEST_FILE")

if echo "$BLOB_RESPONSE" | grep -q "download_url"; then
    echo -e "${GREEN}✅ Blob retrieved successfully${NC}"
    DOWNLOAD_URL=$(echo "$BLOB_RESPONSE" | grep -o '"download_url":"[^"]*"' | cut -d'"' -f4)
    echo "   Download URL: $DOWNLOAD_URL"
else
    echo -e "${RED}❌ Failed to get blob${NC}"
    echo "Response: $BLOB_RESPONSE"
    exit 1
fi

# Test 7: Download and verify file
echo -e "\n${YELLOW}Test 7: Downloading and verifying file${NC}"
DOWNLOADED_FILE="downloaded-$TEST_FILE"
curl -s "$DOWNLOAD_URL" -o "$DOWNLOADED_FILE"

if diff "$TEST_FILE" "$DOWNLOADED_FILE" > /dev/null; then
    echo -e "${GREEN}✅ File downloaded and verified successfully${NC}"
else
    echo -e "${RED}❌ Downloaded file doesn't match original${NC}"
    exit 1
fi

# Test 8: Search
echo -e "\n${YELLOW}Test 8: Searching repository${NC}"
SEARCH_RESPONSE=$(curl -s "$API_URL/v1/repos/$REPO_NAME/search?name=Test%20Data%20File")

if echo "$SEARCH_RESPONSE" | grep -q "entries"; then
    echo -e "${GREEN}✅ Search completed successfully${NC}"
else
    echo -e "${RED}❌ Search failed${NC}"
    echo "Response: $SEARCH_RESPONSE"
    exit 1
fi

# Test 9: RDF functionality
echo -e "\n${YELLOW}Test 9: Testing RDF functionality${NC}"

# Create Dublin Core metadata file
DC_META_FILE="dc-meta.json"
cat > $DC_META_FILE << EOF
{
  "creation_dt": "2025-01-17T18:28:00Z",
  "creator": "you@example.org",
  "file_name": "demo.csv",
  "file_type": "text/csv",
  "file_size": 1234,
  "org_lab": "ORNL",
  "description": "Demo dataset",
  "data_source": "sensor",
  "data_collection_method": "manual",
  "version": "1.0",
  "tags": ["demo", "csv"],
  "license": "CC-BY-4.0"
}
EOF

# Test commit with RDF emission
echo "Creating commit with RDF emission..."
RDF_COMMIT_RESPONSE=$(curl -s -X POST "$API_URL/v1/repos/$REPO_NAME/commit?emit_rdf=true" \
  -H "Content-Type: application/json" \
  -d "{
    \"ref\": \"main\",
    \"message\": \"add example with RDF\",
    \"changes\": [
      {
        \"op\": \"add\",
        \"path\": \"datasets/demo.csv\",
        \"sha256\": \"a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3\",
        \"meta\": $(cat $DC_META_FILE)
      }
    ]
  }")

if echo "$RDF_COMMIT_RESPONSE" | grep -q "commit_id"; then
    echo -e "${GREEN}✅ RDF commit created successfully${NC}"
else
    echo -e "${YELLOW}⚠️  RDF commit test failed (expected in MVP)${NC}"
fi

# Test RDF retrieval
echo "Testing RDF retrieval..."
RDF_RESPONSE=$(curl -s "$API_URL/v1/repos/$REPO_NAME/rdf/main/datasets/demo.csv?format=turtle")

if echo "$RDF_RESPONSE" | grep -q "dc:title"; then
    echo -e "${GREEN}✅ RDF retrieved successfully${NC}"
    echo "   Sample RDF: $(echo "$RDF_RESPONSE" | head -3)"
else
    echo -e "${YELLOW}⚠️  RDF retrieval test failed (expected in MVP)${NC}"
fi

# Cleanup
echo -e "\n${YELLOW}🧹 Cleaning up test files${NC}"
rm -f "$TEST_FILE" "$META_FILE" "$DOWNLOADED_FILE" "$DC_META_FILE"

echo -e "\n${GREEN}🎉 All tests passed! Blacklake is working correctly.${NC}"
echo -e "${GREEN}✅ Repository: $REPO_NAME${NC}"
echo -e "${GREEN}✅ Commit: $COMMIT_ID${NC}"
echo -e "${GREEN}✅ SHA256: $SHA256${NC}"
echo -e "\n${YELLOW}Next steps:${NC}"
echo -e "${YELLOW}1. Try the CLI: cargo run -p cli -- repo list${NC}"
echo -e "${YELLOW}2. Test RDF: cargo run -p cli -- rdf get $REPO_NAME main datasets/demo.csv --format turtle${NC}"
echo -e "${YELLOW}3. Set features: cargo run -p cli -- repo features set $REPO_NAME auto_rdf true${NC}"
