// K6 Configuration for BlackLake
// Week 5: Performance testing configuration

export const options = {
  // Default configuration
  scenarios: {
    // Load test scenario
    load_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 10 },
        { duration: '5m', target: 10 },
        { duration: '2m', target: 20 },
        { duration: '5m', target: 20 },
        { duration: '2m', target: 0 },
      ],
      gracefulRampDown: '30s',
    },
    
    // Stress test scenario
    stress_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 10 },
        { duration: '2m', target: 50 },
        { duration: '3m', target: 100 },
        { duration: '5m', target: 100 },
        { duration: '2m', target: 200 },
        { duration: '5m', target: 200 },
        { duration: '2m', target: 0 },
      ],
      gracefulRampDown: '30s',
    },
    
    // Spike test scenario
    spike_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 10 },
        { duration: '1m', target: 100 },
        { duration: '1m', target: 10 },
        { duration: '1m', target: 200 },
        { duration: '1m', target: 10 },
        { duration: '1m', target: 0 },
      ],
      gracefulRampDown: '30s',
    },
  },
  
  // Thresholds
  thresholds: {
    // Load test thresholds
    'http_req_duration': ['p(95)<2000'],
    'http_req_failed': ['rate<0.1'],
    'errors': ['rate<0.1'],
    
    // Stress test thresholds
    'http_req_duration': ['p(95)<5000'],
    'http_req_failed': ['rate<0.2'],
    'errors': ['rate<0.2'],
    
    // Spike test thresholds
    'http_req_duration': ['p(95)<3000'],
    'http_req_failed': ['rate<0.15'],
    'errors': ['rate<0.15'],
  },
  
  // Output options
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
  summaryTimeUnit: 'ms',
  
  // Environment variables
  env: {
    BASE_URL: __ENV.BASE_URL || 'http://localhost:8080',
    TEST_DURATION: __ENV.TEST_DURATION || '15m',
    MAX_VUS: __ENV.MAX_VUS || '100',
  },
};

// Test data
export const testData = {
  repos: [
    'test-repo-1',
    'test-repo-2',
    'test-repo-3',
    'test-repo-4',
    'test-repo-5',
  ],
  files: [
    'data/sample.csv',
    'data/sample.json',
    'data/sample.parquet',
    'models/sample.onnx',
    'docs/README.md',
  ],
  searchQueries: [
    'test',
    'data',
    'model',
    'csv',
    'json',
    'parquet',
    'onnx',
    'pytorch',
  ],
};

// Helper functions
export function getRandomRepo() {
  return testData.repos[Math.floor(Math.random() * testData.repos.length)];
}

export function getRandomFile() {
  return testData.files[Math.floor(Math.random() * testData.files.length)];
}

export function getRandomSearchQuery() {
  return testData.searchQueries[Math.floor(Math.random() * testData.searchQueries.length)];
}

export function sleepRandom(min = 1, max = 3) {
  sleep(Math.random() * (max - min) + min);
}

// Custom metrics
import { Rate, Trend, Counter } from 'k6/metrics';

export const errorRate = new Rate('errors');
export const responseTime = new Trend('response_time');
export const requestCount = new Counter('requests');

// Test scenarios
export const scenarios = {
  load: {
    executor: 'ramping-vus',
    startVUs: 0,
    stages: [
      { duration: '2m', target: 10 },
      { duration: '5m', target: 10 },
      { duration: '2m', target: 20 },
      { duration: '5m', target: 20 },
      { duration: '2m', target: 0 },
    ],
    gracefulRampDown: '30s',
  },
  
  stress: {
    executor: 'ramping-vus',
    startVUs: 0,
    stages: [
      { duration: '1m', target: 10 },
      { duration: '2m', target: 50 },
      { duration: '3m', target: 100 },
      { duration: '5m', target: 100 },
      { duration: '2m', target: 200 },
      { duration: '5m', target: 200 },
      { duration: '2m', target: 0 },
    ],
    gracefulRampDown: '30s',
  },
  
  spike: {
    executor: 'ramping-vus',
    startVUs: 0,
    stages: [
      { duration: '1m', target: 10 },
      { duration: '1m', target: 100 },
      { duration: '1m', target: 10 },
      { duration: '1m', target: 200 },
      { duration: '1m', target: 10 },
      { duration: '1m', target: 0 },
    ],
    gracefulRampDown: '30s',
  },
};
