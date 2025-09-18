// K6 Edge Case Testing for BlackLake
// Week 5: Testing edge cases and error conditions

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
export let errorRate = new Rate('errors');
export let responseTime = new Trend('response_time');

// Edge case test configuration
export let options = {
  stages: [
    { duration: '1m', target: 5 },   // Ramp up to 5 users
    { duration: '2m', target: 5 },   // Stay at 5 users
    { duration: '1m', target: 0 },   // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<10000'], // 95% of requests must complete below 10s
    http_req_failed: ['rate<0.5'],      // Error rate must be below 50%
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function() {
  // Test invalid endpoints
  testInvalidEndpoints();
  
  // Test malformed requests
  testMalformedRequests();
  
  // Test large payloads
  testLargePayloads();
  
  // Test concurrent operations
  testConcurrentOperations();
  
  sleep(1);
}

function testInvalidEndpoints() {
  const invalidEndpoints = [
    '/api/v1/nonexistent',
    '/api/v1/repos/invalid-repo-name!',
    '/api/v1/repos/valid-repo/invalid-ref',
    '/api/v1/search?invalid_param=value',
  ];

  for (const endpoint of invalidEndpoints) {
    const response = http.get(`${BASE_URL}${endpoint}`);
    const success = check(response, {
      'invalid endpoint returns 404 or 400': (r) => r.status === 404 || r.status === 400,
    });
    errorRate.add(!success);
    responseTime.add(response.timings.duration);
  }
}

function testMalformedRequests() {
  // Test malformed JSON
  const malformedJson = '{"invalid": json}';
  const response1 = http.post(`${BASE_URL}/api/v1/repos/test/commit`, malformedJson, {
    headers: { 'Content-Type': 'application/json' },
  });
  
  const success1 = check(response1, {
    'malformed JSON returns 400': (r) => r.status === 400,
  });
  errorRate.add(!success1);
  responseTime.add(response1.timings.duration);

  // Test missing required fields
  const incompleteJson = '{"ref": "main"}';
  const response2 = http.post(`${BASE_URL}/api/v1/repos/test/commit`, incompleteJson, {
    headers: { 'Content-Type': 'application/json' },
  });
  
  const success2 = check(response2, {
    'incomplete JSON returns 400': (r) => r.status === 400,
  });
  errorRate.add(!success2);
  responseTime.add(response2.timings.duration);
}

function testLargePayloads() {
  // Test large search query
  const largeQuery = 'a'.repeat(10000);
  const response = http.get(`${BASE_URL}/api/v1/search?q=${largeQuery}`);
  
  const success = check(response, {
    'large query handled appropriately': (r) => r.status === 400 || r.status === 413,
  });
  errorRate.add(!success);
  responseTime.add(response.timings.duration);
}

function testConcurrentOperations() {
  // Test concurrent requests to same resource
  const promises = [];
  for (let i = 0; i < 5; i++) {
    promises.push(http.get(`${BASE_URL}/api/v1/repos/test`));
  }
  
  const responses = promises;
  for (const response of responses) {
    const success = check(response, {
      'concurrent request handled': (r) => r.status === 200 || r.status === 401 || r.status === 404,
    });
    errorRate.add(!success);
    responseTime.add(response.timings.duration);
  }
}
