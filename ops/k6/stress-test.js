// K6 Stress Test for BlackLake API
// Week 5: Stress testing to find breaking points

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
export let errorRate = new Rate('errors');
export let responseTime = new Trend('response_time');

// Stress test configuration
export let options = {
  stages: [
    { duration: '1m', target: 10 },   // Ramp up to 10 users
    { duration: '2m', target: 50 },   // Ramp up to 50 users
    { duration: '3m', target: 100 },  // Ramp up to 100 users
    { duration: '5m', target: 100 },  // Stay at 100 users
    { duration: '2m', target: 200 },  // Ramp up to 200 users
    { duration: '5m', target: 200 },  // Stay at 200 users
    { duration: '2m', target: 0 },    // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<5000'], // 95% of requests must complete below 5s
    http_req_failed: ['rate<0.2'],     // Error rate must be below 20%
    errors: ['rate<0.2'],              // Custom error rate must be below 20%
  },
};

// Base URL
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

// Test data
const testRepos = [
  'stress-test-repo-1',
  'stress-test-repo-2',
  'stress-test-repo-3',
  'stress-test-repo-4',
  'stress-test-repo-5',
  'stress-test-repo-6',
  'stress-test-repo-7',
  'stress-test-repo-8',
  'stress-test-repo-9',
  'stress-test-repo-10'
];

export default function() {
  // Test health endpoints under stress
  testHealthEndpoints();
  
  // Test API endpoints under stress
  testAPIEndpoints();
  
  // Simulate realistic user behavior
  sleep(Math.random() * 1 + 0.5); // Random sleep between 0.5-1.5 seconds
}

function testHealthEndpoints() {
  // Test /live endpoint
  let response = http.get(`${BASE_URL}/live`);
  let success = check(response, {
    'health check /live status is 200': (r) => r.status === 200,
    'health check /live response time < 200ms': (r) => r.timings.duration < 200,
  });
  errorRate.add(!success);
  responseTime.add(response.timings.duration);

  // Test /ready endpoint
  response = http.get(`${BASE_URL}/ready`);
  success = check(response, {
    'health check /ready status is 200': (r) => r.status === 200,
    'health check /ready response time < 200ms': (r) => r.timings.duration < 200,
  });
  errorRate.add(!success);
  responseTime.add(response.timings.duration);
}

function testAPIEndpoints() {
  // Test repositories endpoint
  let response = http.get(`${BASE_URL}/v1/repos`);
  let success = check(response, {
    'repos endpoint responds': (r) => r.status === 200 || r.status === 401,
    'repos endpoint response time < 1000ms': (r) => r.timings.duration < 1000,
  });
  errorRate.add(!success);
  responseTime.add(response.timings.duration);

  // Test specific repository
  const repo = testRepos[Math.floor(Math.random() * testRepos.length)];
  response = http.get(`${BASE_URL}/v1/repos/${repo}`);
  success = check(response, {
    'repo endpoint responds': (r) => r.status === 200 || r.status === 401 || r.status === 404,
    'repo endpoint response time < 1000ms': (r) => r.timings.duration < 1000,
  });
  errorRate.add(!success);
  responseTime.add(response.timings.duration);

  // Test repository entries
  response = http.get(`${BASE_URL}/v1/repos/${repo}/tree/main`);
  success = check(response, {
    'tree endpoint responds': (r) => r.status === 200 || r.status === 401 || r.status === 404,
    'tree endpoint response time < 2000ms': (r) => r.timings.duration < 2000,
  });
  errorRate.add(!success);
  responseTime.add(response.timings.duration);

  // Test search endpoint with various queries
  const searchQueries = ['test', 'data', 'model', 'csv', 'json', 'parquet'];
  const query = searchQueries[Math.floor(Math.random() * searchQueries.length)];
  const searchParams = {
    q: query,
    limit: '20'
  };
  response = http.get(`${BASE_URL}/v1/search`, { params: searchParams });
  success = check(response, {
    'search endpoint responds': (r) => r.status === 200 || r.status === 401,
    'search endpoint response time < 2000ms': (r) => r.timings.duration < 2000,
  });
  errorRate.add(!success);
  responseTime.add(response.timings.duration);

  // Test metrics endpoint (should be protected but test anyway)
  response = http.get(`${BASE_URL}/metrics`);
  success = check(response, {
    'metrics endpoint responds': (r) => r.status === 200 || r.status === 401 || r.status === 403,
    'metrics endpoint response time < 1000ms': (r) => r.timings.duration < 1000,
  });
  errorRate.add(!success);
  responseTime.add(response.timings.duration);
}

export function handleSummary(data) {
  return {
    'stress-summary.json': JSON.stringify(data, null, 2),
    'stress-summary.html': htmlReport(data),
  };
}

function htmlReport(data) {
  const maxVUs = data.metrics.vus.values.max;
  const errorRate = data.metrics.http_req_failed.values.rate * 100;
  const avgResponseTime = data.metrics.http_req_duration.values.avg;
  const p95ResponseTime = data.metrics.http_req_duration.values.p95;
  
  let status = 'success';
  if (errorRate > 10 || p95ResponseTime > 3000) {
    status = 'warning';
  }
  if (errorRate > 20 || p95ResponseTime > 5000) {
    status = 'error';
  }

  return `
    <!DOCTYPE html>
    <html>
      <head>
        <title>BlackLake Stress Test Results</title>
        <style>
          body { font-family: Arial, sans-serif; margin: 20px; }
          .metric { margin: 10px 0; padding: 10px; border: 1px solid #ddd; }
          .success { background-color: #d4edda; }
          .warning { background-color: #fff3cd; }
          .error { background-color: #f8d7da; }
          .header { background-color: #f8f9fa; padding: 20px; border-radius: 5px; }
        </style>
      </head>
      <body>
        <div class="header">
          <h1>BlackLake Stress Test Results</h1>
          <p><strong>Test Status:</strong> <span class="${status}">${status.toUpperCase()}</span></p>
        </div>
        
        <div class="metric">
          <h3>Test Configuration</h3>
          <p><strong>Duration:</strong> ${data.state.testRunDurationMs / 1000}s</p>
          <p><strong>Max VUs:</strong> ${maxVUs}</p>
          <p><strong>Iterations:</strong> ${data.metrics.iterations.values.count}</p>
        </div>
        
        <div class="metric ${errorRate < 10 ? 'success' : errorRate < 20 ? 'warning' : 'error'}">
          <h3>Error Rate</h3>
          <p><strong>HTTP Request Failures:</strong> ${errorRate.toFixed(2)}%</p>
          <p><strong>Threshold:</strong> < 20%</p>
        </div>
        
        <div class="metric ${p95ResponseTime < 3000 ? 'success' : p95ResponseTime < 5000 ? 'warning' : 'error'}">
          <h3>Response Time</h3>
          <p><strong>Average:</strong> ${avgResponseTime.toFixed(2)}ms</p>
          <p><strong>95th Percentile:</strong> ${p95ResponseTime.toFixed(2)}ms</p>
          <p><strong>99th Percentile:</strong> ${data.metrics.http_req_duration.values.p99.toFixed(2)}ms</p>
          <p><strong>Threshold:</strong> 95th percentile < 5000ms</p>
        </div>
        
        <div class="metric">
          <h3>Throughput</h3>
          <p><strong>Requests per second:</strong> ${data.metrics.http_reqs.values.rate.toFixed(2)}</p>
          <p><strong>Data received:</strong> ${(data.metrics.data_received.values.count / 1024 / 1024).toFixed(2)} MB</p>
          <p><strong>Data sent:</strong> ${(data.metrics.data_sent.values.count / 1024 / 1024).toFixed(2)} MB</p>
        </div>
        
        <div class="metric">
          <h3>System Performance</h3>
          <p><strong>Max VUs reached:</strong> ${maxVUs}</p>
          <p><strong>VU duration:</strong> ${data.metrics.vu_max_duration.values.max.toFixed(2)}s</p>
          <p><strong>Iteration duration:</strong> ${data.metrics.iteration_duration.values.avg.toFixed(2)}ms</p>
        </div>
        
        <div class="metric">
          <h3>Recommendations</h3>
          ${generateRecommendations(errorRate, p95ResponseTime, maxVUs)}
        </div>
      </body>
    </html>
  `;
}

function generateRecommendations(errorRate, p95ResponseTime, maxVUs) {
  let recommendations = '<ul>';
  
  if (errorRate > 10) {
    recommendations += '<li>Consider implementing rate limiting or request queuing</li>';
  }
  
  if (p95ResponseTime > 3000) {
    recommendations += '<li>Optimize database queries and add caching</li>';
  }
  
  if (maxVUs < 100) {
    recommendations += '<li>System may need horizontal scaling</li>';
  }
  
  if (errorRate < 5 && p95ResponseTime < 1000) {
    recommendations += '<li>System is performing well under stress</li>';
  }
  
  recommendations += '</ul>';
  return recommendations;
}
