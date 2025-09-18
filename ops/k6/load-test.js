// BlackLake K6 Load Test Script
// Week 5: Performance testing for core workflows

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// ===== CUSTOM METRICS =====
export const errorRate = new Rate('errors');
export const responseTime = new Trend('response_time');

// ===== TEST CONFIGURATION =====
export const options = {
  stages: [
    { duration: '2m', target: 10 }, // Ramp up to 10 users
    { duration: '5m', target: 10 }, // Stay at 10 users
    { duration: '2m', target: 20 }, // Ramp up to 20 users
    { duration: '5m', target: 20 }, // Stay at 20 users
    { duration: '2m', target: 0 },  // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<2000'], // 95% of requests must complete below 2s
    http_req_failed: ['rate<0.1'],     // Error rate must be below 10%
    errors: ['rate<0.1'],              // Custom error rate must be below 10%
  },
};

// ===== TEST DATA =====
const BASE_URL = 'http://localhost:8080';
const API_BASE = `${BASE_URL}/api/v1`;

// Test repositories
const testRepos = [
  'test-repo-1',
  'test-repo-2',
  'test-repo-3',
];

// Test files
const testFiles = [
  { name: 'data.csv', content: 'id,name,value\n1,test,100\n2,example,200' },
  { name: 'model.onnx', content: 'fake-onnx-model-data' },
  { name: 'config.json', content: '{"key": "value", "nested": {"data": 123}}' },
];

// ===== SETUP =====
export function setup() {
  console.log('Setting up load test...');
  
  // Create test repositories
  for (const repo of testRepos) {
    const response = http.post(`${API_BASE}/repos`, JSON.stringify({
      name: repo,
      description: `Test repository for load testing - ${repo}`
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
    
    if (response.status !== 201) {
      console.log(`Failed to create repo ${repo}: ${response.status}`);
    }
  }
  
  return { testRepos, testFiles };
}

// ===== MAIN TEST FUNCTION =====
export default function(data) {
  const { testRepos, testFiles } = data;
  
  // Randomly select a repository and file
  const repo = testRepos[Math.floor(Math.random() * testRepos.length)];
  const file = testFiles[Math.floor(Math.random() * testFiles.length)];
  
  // Test scenario: Upload and commit workflow
  testUploadAndCommitWorkflow(repo, file);
  
  // Test scenario: Search workflow
  testSearchWorkflow();
  
  // Test scenario: Repository management
  testRepositoryManagement(repo);
  
  // Wait between requests
  sleep(Math.random() * 2 + 1); // 1-3 seconds
}

// ===== TEST SCENARIOS =====

function testUploadAndCommitWorkflow(repo, file) {
  // 1. Initialize upload
  const uploadResponse = http.post(`${API_BASE}/repos/${repo}/upload-init`, JSON.stringify({
    path: `test/${file.name}`,
    size: file.content.length,
    content_type: getContentType(file.name),
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  
  const uploadSuccess = check(uploadResponse, {
    'upload init status is 200': (r) => r.status === 200,
    'upload init has presigned URL': (r) => r.json('presigned_url') !== undefined,
  });
  
  errorRate.add(!uploadSuccess);
  responseTime.add(uploadResponse.timings.duration);
  
  if (uploadSuccess) {
    const presignedUrl = uploadResponse.json('presigned_url');
    const objectKey = uploadResponse.json('object_key');
    
    // 2. Upload file to S3 (simulate)
    const uploadS3Response = http.put(presignedUrl, file.content, {
      headers: { 'Content-Type': getContentType(file.name) },
    });
    
    const s3UploadSuccess = check(uploadS3Response, {
      'S3 upload status is 200': (r) => r.status === 200,
    });
    
    errorRate.add(!s3UploadSuccess);
    
    if (s3UploadSuccess) {
      // 3. Commit the upload
      const commitResponse = http.post(`${API_BASE}/repos/${repo}/commit`, JSON.stringify({
        message: `Load test commit for ${file.name}`,
        changes: [{
          op: 'put',
          path: `test/${file.name}`,
          object_key: objectKey,
          meta: {
            file_name: file.name,
            file_type: getFileType(file.name),
            file_size: file.content.length,
            creator: 'load-test-user',
            description: `Load test file: ${file.name}`,
          }
        }]
      }), {
        headers: { 'Content-Type': 'application/json' },
      });
      
      const commitSuccess = check(commitResponse, {
        'commit status is 200': (r) => r.status === 200,
        'commit has commit_id': (r) => r.json('commit_id') !== undefined,
      });
      
      errorRate.add(!commitSuccess);
      responseTime.add(commitResponse.timings.duration);
    }
  }
}

function testSearchWorkflow() {
  // 1. Search for files
  const searchResponse = http.get(`${API_BASE}/search?q=test&limit=10`);
  
  const searchSuccess = check(searchResponse, {
    'search status is 200': (r) => r.status === 200,
    'search has results': (r) => r.json('data.results') !== undefined,
  });
  
  errorRate.add(!searchSuccess);
  responseTime.add(searchResponse.timings.duration);
  
  // 2. Search with filters
  const filteredSearchResponse = http.get(`${API_BASE}/search?file_type=csv&org_lab=test&limit=5`);
  
  const filteredSearchSuccess = check(filteredSearchResponse, {
    'filtered search status is 200': (r) => r.status === 200,
    'filtered search has results': (r) => r.json('data.results') !== undefined,
  });
  
  errorRate.add(!filteredSearchSuccess);
  responseTime.add(filteredSearchResponse.timings.duration);
}

function testRepositoryManagement(repo) {
  // 1. Get repository info
  const repoResponse = http.get(`${API_BASE}/repos/${repo}`);
  
  const repoSuccess = check(repoResponse, {
    'repo info status is 200': (r) => r.status === 200,
    'repo info has name': (r) => r.json('data.name') === repo,
  });
  
  errorRate.add(!repoSuccess);
  responseTime.add(repoResponse.timings.duration);
  
  // 2. List repository entries
  const entriesResponse = http.get(`${API_BASE}/repos/${repo}/tree/main`);
  
  const entriesSuccess = check(entriesResponse, {
    'entries status is 200': (r) => r.status === 200,
    'entries has data': (r) => r.json('data.entries') !== undefined,
  });
  
  errorRate.add(!entriesSuccess);
  responseTime.add(entriesResponse.timings.duration);
  
  // 3. Get repository usage
  const usageResponse = http.get(`${API_BASE}/repos/${repo}/usage`);
  
  const usageSuccess = check(usageResponse, {
    'usage status is 200': (r) => r.status === 200,
    'usage has current_bytes': (r) => r.json('data.current_bytes') !== undefined,
  });
  
  errorRate.add(!usageSuccess);
  responseTime.add(usageResponse.timings.duration);
}

// ===== UTILITY FUNCTIONS =====

function getContentType(filename) {
  const ext = filename.split('.').pop().toLowerCase();
  const types = {
    'csv': 'text/csv',
    'json': 'application/json',
    'onnx': 'application/octet-stream',
    'txt': 'text/plain',
  };
  return types[ext] || 'application/octet-stream';
}

function getFileType(filename) {
  const ext = filename.split('.').pop().toLowerCase();
  const types = {
    'csv': 'csv',
    'json': 'json',
    'onnx': 'onnx',
    'txt': 'text',
  };
  return types[ext] || 'binary';
}

// ===== TEARDOWN =====
export function teardown(data) {
  console.log('Cleaning up load test...');
  
  // Clean up test repositories
  for (const repo of data.testRepos) {
    const response = http.del(`${API_BASE}/repos/${repo}`);
    if (response.status !== 204) {
      console.log(`Failed to delete repo ${repo}: ${response.status}`);
    }
  }
  
  console.log('Load test cleanup completed');
}