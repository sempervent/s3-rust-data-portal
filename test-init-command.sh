#!/bin/bash
set -e

echo "🧪 Testing BlackLake init command..."

# Create test directory
mkdir -p test_dataset
echo "name,age" > test_dataset/data.csv
echo "Alice,30" >> test_dataset/data.csv
echo "Bob,25" >> test_dataset/data.csv
echo "# Test Dataset" > test_dataset/README.md

# Test 1: Initialize directory
echo "📁 Testing directory initialization..."
cargo run -p blacklake-cli -- init test_dataset \
  --namespace test \
  --label domain=demo \
  --meta source=test \
  --class restricted \
  --owner test@example.com

# Check that .bl directory was created
if [ -d "test_dataset/.bl" ]; then
    echo "✅ .bl directory created"
else
    echo "❌ .bl directory not created"
    exit 1
fi

# Check that metadata files were created
if [ -f "test_dataset/.bl/data.csv.metadata.yaml" ]; then
    echo "✅ Per-file metadata created"
else
    echo "❌ Per-file metadata not created"
    exit 1
fi

if [ -f "test_dataset/.bl/_artifact.metadata.yaml" ]; then
    echo "✅ Directory manifest created"
else
    echo "❌ Directory manifest not created"
    exit 1
fi

if [ -f "test_dataset/.bl/policy.yaml" ]; then
    echo "✅ Policy template created"
else
    echo "❌ Policy template not created"
    exit 1
fi

# Test 2: Initialize single file
echo "📄 Testing file initialization..."
echo "fake onnx data" > model.onnx

cargo run -p blacklake-cli -- init model.onnx \
  --namespace ml \
  --label framework=onnx \
  --class internal \
  --with-authorization

# Check that sidecar files were created
if [ -f "model.onnx.bl.metadata.yaml" ]; then
    echo "✅ File metadata sidecar created"
else
    echo "❌ File metadata sidecar not created"
    exit 1
fi

if [ -f "model.onnx.bl.authorization.yaml" ]; then
    echo "✅ Authorization sidecar created"
else
    echo "❌ Authorization sidecar not created"
    exit 1
fi

# Test 3: Dry run
echo "🔍 Testing dry run..."
cargo run -p blacklake-cli -- init test_dataset --dry-run

# Test 4: Dot notation overrides
echo "⚙️  Testing dot notation overrides..."
cargo run -p blacklake-cli -- init test_dataset \
  --set policy.readers[0]=group:data-science \
  --set auth.allowed_audiences[0]=urn:ml:prod \
  --overwrite

echo "✅ All tests passed!"

# Cleanup
rm -rf test_dataset model.onnx model.onnx.bl.metadata.yaml model.onnx.bl.authorization.yaml

