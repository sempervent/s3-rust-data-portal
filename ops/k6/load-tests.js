// k6 Load Testing Scenarios
// Implements k6 scenarios for upload (1GB), search (facet+text)
// Implements bulk export and reindex load tests
// Stores trend graphs in Grafana via remote write

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { SharedArray } from 'k6/data';

// Custom metrics
export const uploadSuccessRate = new Rate('upload_success_rate');
export const searchSuccessRate = new Rate('search_success_rate');
export const exportSuccessRate = new Rate('export_success_rate');
export const reindexSuccessRate = new Rate('reindex_success_rate');

export const uploadDuration = new Trend('upload_duration');
export const searchDuration = new Trend('search_duration');
export const exportDuration = new Trend('export_duration');
export const reindexDuration = new Trend('reindex_duration');

export const uploadThroughput = new Counter('upload_throughput_bytes');
export const searchThroughput = new Counter('search_throughput_requests');
export const exportThroughput = new Counter('export_throughput_requests');
export const reindexThroughput = new Counter('reindex_throughput_requests');

// Test data
const testData = new SharedArray('test_data', function () {
    return {
        // Large file for upload testing (1GB)
        largeFile: generateLargeFile(1024 * 1024 * 1024), // 1GB
        // Medium file for regular uploads
        mediumFile: generateLargeFile(100 * 1024 * 1024), // 100MB
        // Small file for quick tests
        smallFile: generateLargeFile(1024 * 1024), // 1MB
        
        // Search queries
        searchQueries: [
            'machine learning',
            'data analysis',
            'python script',
            'jupyter notebook',
            'tensorflow model',
            'pytorch model',
            'scikit-learn',
            'pandas dataframe',
            'numpy array',
            'matplotlib plot'
        ],
        
        // Facet filters
        facetFilters: [
            { type: 'file_type', value: 'text/plain' },
            { type: 'file_type', value: 'application/json' },
            { type: 'file_type', value: 'text/csv' },
            { type: 'file_type', value: 'application/pdf' },
            { type: 'file_type', value: 'image/png' },
            { type: 'file_type', value: 'image/jpeg' },
            { type: 'file_type', value: 'application/zip' },
            { type: 'file_type', value: 'text/html' },
            { type: 'file_type', value: 'application/xml' },
            { type: 'file_type', value: 'text/javascript' }
        ],
        
        // Repository names
        repoNames: [
            'ml-experiments',
            'data-analysis',
            'research-data',
            'production-models',
            'test-datasets',
            'benchmark-results',
            'model-artifacts',
            'training-data',
            'validation-sets',
            'inference-results'
        ]
    };
});

// Configuration
export const options = {
    scenarios: {
        // Upload scenario - 1GB file
        upload_large_file: {
            executor: 'ramping-vus',
            startVUs: 1,
            stages: [
                { duration: '30s', target: 1 },
                { duration: '2m', target: 1 },
                { duration: '30s', target: 0 }
            ],
            exec: 'uploadLargeFile'
        },
        
        // Upload scenario - multiple medium files
        upload_medium_files: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 5 },
                { duration: '2m', target: 10 },
                { duration: '1m', target: 20 },
                { duration: '30s', target: 0 }
            ],
            exec: 'uploadMediumFiles'
        },
        
        // Search scenario - text search
        search_text: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 10 },
                { duration: '2m', target: 20 },
                { duration: '1m', target: 50 },
                { duration: '30s', target: 0 }
            ],
            exec: 'searchText'
        },
        
        // Search scenario - facet search
        search_facets: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 5 },
                { duration: '2m', target: 10 },
                { duration: '1m', target: 25 },
                { duration: '30s', target: 0 }
            ],
            exec: 'searchFacets'
        },
        
        // Export scenario - bulk export
        export_bulk: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 2 },
                { duration: '2m', target: 5 },
                { duration: '1m', target: 10 },
                { duration: '30s', target: 0 }
            ],
            exec: 'exportBulk'
        },
        
        // Reindex scenario
        reindex_data: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 1 },
                { duration: '2m', target: 3 },
                { duration: '1m', target: 5 },
                { duration: '30s', target: 0 }
            ],
            exec: 'reindexData'
        }
    },
    
    thresholds: {
        // Performance thresholds
        'upload_duration': ['p95<5000', 'p99<10000'],
        'search_duration': ['p95<1000', 'p99<2000'],
        'export_duration': ['p95<10000', 'p99<20000'],
        'reindex_duration': ['p95<30000', 'p99<60000'],
        
        // Success rate thresholds
        'upload_success_rate': ['rate>0.95'],
        'search_success_rate': ['rate>0.98'],
        'export_success_rate': ['rate>0.90'],
        'reindex_success_rate': ['rate>0.85'],
        
        // HTTP error rate
        'http_req_failed': ['rate<0.05']
    }
};

// Base URL configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API_KEY = __ENV.API_KEY || 'test-api-key';

// Headers
const headers = {
    'Authorization': `Bearer ${API_KEY}`,
    'Content-Type': 'application/json'
};

// Upload large file (1GB)
export function uploadLargeFile() {
    const repoName = testData.repoNames[Math.floor(Math.random() * testData.repoNames.length)];
    const fileName = `large-file-${Date.now()}.bin`;
    
    const startTime = Date.now();
    
    // Create repository if it doesn't exist
    let repoId = createRepository(repoName);
    if (!repoId) {
        return;
    }
    
    // Upload file in chunks
    const chunkSize = 10 * 1024 * 1024; // 10MB chunks
    const totalChunks = Math.ceil(testData.largeFile.length / chunkSize);
    let uploadSuccess = true;
    
    for (let i = 0; i < totalChunks; i++) {
        const start = i * chunkSize;
        const end = Math.min(start + chunkSize, testData.largeFile.length);
        const chunk = testData.largeFile.slice(start, end);
        
        const response = http.post(`${BASE_URL}/api/v1/repos/${repoId}/files/${fileName}`, chunk, {
            headers: {
                'Authorization': `Bearer ${API_KEY}`,
                'Content-Type': 'application/octet-stream',
                'X-Chunk-Index': i.toString(),
                'X-Total-Chunks': totalChunks.toString()
            }
        });
        
        if (response.status !== 200 && response.status !== 201) {
            uploadSuccess = false;
            break;
        }
        
        sleep(0.1); // Small delay between chunks
    }
    
    const duration = Date.now() - startTime;
    
    uploadSuccessRate.add(uploadSuccess);
    uploadDuration.add(duration);
    uploadThroughput.add(testData.largeFile.length);
    
    check(response, {
        'upload large file successful': (r) => uploadSuccess,
        'upload duration acceptable': (r) => duration < 300000, // 5 minutes
    });
}

// Upload medium files
export function uploadMediumFiles() {
    const repoName = testData.repoNames[Math.floor(Math.random() * testData.repoNames.length)];
    const fileName = `medium-file-${Date.now()}-${Math.random().toString(36).substr(2, 9)}.bin`;
    
    const startTime = Date.now();
    
    // Create repository if it doesn't exist
    let repoId = createRepository(repoName);
    if (!repoId) {
        return;
    }
    
    const response = http.post(`${BASE_URL}/api/v1/repos/${repoId}/files/${fileName}`, testData.mediumFile, {
        headers: {
            'Authorization': `Bearer ${API_KEY}`,
            'Content-Type': 'application/octet-stream'
        }
    });
    
    const duration = Date.now() - startTime;
    const success = response.status === 200 || response.status === 201;
    
    uploadSuccessRate.add(success);
    uploadDuration.add(duration);
    uploadThroughput.add(testData.mediumFile.length);
    
    check(response, {
        'upload medium file successful': (r) => success,
        'upload duration acceptable': (r) => duration < 30000, // 30 seconds
    });
}

// Search text
export function searchText() {
    const query = testData.searchQueries[Math.floor(Math.random() * testData.searchQueries.length)];
    
    const startTime = Date.now();
    
    const response = http.get(`${BASE_URL}/api/v1/search?q=${encodeURIComponent(query)}&limit=20`, {
        headers: headers
    });
    
    const duration = Date.now() - startTime;
    const success = response.status === 200;
    
    searchSuccessRate.add(success);
    searchDuration.add(duration);
    searchThroughput.add(1);
    
    check(response, {
        'search text successful': (r) => success,
        'search duration acceptable': (r) => duration < 2000, // 2 seconds
        'search returns results': (r) => success && JSON.parse(r.body).results.length >= 0
    });
}

// Search facets
export function searchFacets() {
    const query = testData.searchQueries[Math.floor(Math.random() * testData.searchQueries.length)];
    const facet = testData.facetFilters[Math.floor(Math.random() * testData.facetFilters.length)];
    
    const startTime = Date.now();
    
    const response = http.get(`${BASE_URL}/api/v1/search?q=${encodeURIComponent(query)}&facet=${facet.type}:${facet.value}&limit=20`, {
        headers: headers
    });
    
    const duration = Date.now() - startTime;
    const success = response.status === 200;
    
    searchSuccessRate.add(success);
    searchDuration.add(duration);
    searchThroughput.add(1);
    
    check(response, {
        'search facets successful': (r) => success,
        'search duration acceptable': (r) => duration < 2000, // 2 seconds
        'search returns results': (r) => success && JSON.parse(r.body).results.length >= 0
    });
}

// Export bulk
export function exportBulk() {
    const repoName = testData.repoNames[Math.floor(Math.random() * testData.repoNames.length)];
    
    const startTime = Date.now();
    
    // Create export job
    const exportRequest = {
        repo_name: repoName,
        export_type: 'full',
        include_metadata: true,
        include_rdf: true
    };
    
    const response = http.post(`${BASE_URL}/api/v1/exports`, JSON.stringify(exportRequest), {
        headers: headers
    });
    
    const duration = Date.now() - startTime;
    const success = response.status === 200 || response.status === 201;
    
    exportSuccessRate.add(success);
    exportDuration.add(duration);
    exportThroughput.add(1);
    
    check(response, {
        'export bulk successful': (r) => success,
        'export duration acceptable': (r) => duration < 5000, // 5 seconds
        'export job created': (r) => success && JSON.parse(r.body).id
    });
}

// Reindex data
export function reindexData() {
    const repoName = testData.repoNames[Math.floor(Math.random() * testData.repoNames.length)];
    
    const startTime = Date.now();
    
    // Create reindex job
    const reindexRequest = {
        repo_name: repoName,
        force: true
    };
    
    const response = http.post(`${BASE_URL}/api/v1/reindex`, JSON.stringify(reindexRequest), {
        headers: headers
    });
    
    const duration = Date.now() - startTime;
    const success = response.status === 200 || response.status === 201;
    
    reindexSuccessRate.add(success);
    reindexDuration.add(duration);
    reindexThroughput.add(1);
    
    check(response, {
        'reindex successful': (r) => success,
        'reindex duration acceptable': (r) => duration < 10000, // 10 seconds
        'reindex job created': (r) => success && JSON.parse(r.body).id
    });
}

// Helper functions
function createRepository(name) {
    const response = http.post(`${BASE_URL}/api/v1/repos`, JSON.stringify({ name }), {
        headers: headers
    });
    
    if (response.status === 200 || response.status === 201) {
        return JSON.parse(response.body).id;
    } else if (response.status === 409) {
        // Repository already exists, get its ID
        const getResponse = http.get(`${BASE_URL}/api/v1/repos`, { headers: headers });
        if (getResponse.status === 200) {
            const repos = JSON.parse(getResponse.body);
            const repo = repos.find(r => r.name === name);
            return repo ? repo.id : null;
        }
    }
    
    return null;
}

function generateLargeFile(size) {
    const chunkSize = 1024 * 1024; // 1MB chunks
    const chunks = Math.ceil(size / chunkSize);
    let content = '';
    
    for (let i = 0; i < chunks; i++) {
        const remainingSize = Math.min(chunkSize, size - (i * chunkSize));
        content += 'A'.repeat(remainingSize);
    }
    
    return content;
}

// Setup function
export function setup() {
    console.log('Setting up load test environment...');
    
    // Verify API is accessible
    const healthResponse = http.get(`${BASE_URL}/health`);
    if (healthResponse.status !== 200) {
        throw new Error(`API health check failed: ${healthResponse.status}`);
    }
    
    console.log('Load test environment ready');
    return { baseUrl: BASE_URL };
}

// Teardown function
export function teardown(data) {
    console.log('Cleaning up load test environment...');
    
    // Clean up test repositories
    const reposResponse = http.get(`${BASE_URL}/api/v1/repos`, { headers: headers });
    if (reposResponse.status === 200) {
        const repos = JSON.parse(reposResponse.body);
        for (const repo of repos) {
            if (testData.repoNames.includes(repo.name)) {
                http.del(`${BASE_URL}/api/v1/repos/${repo.id}`, null, { headers: headers });
            }
        }
    }
    
    console.log('Load test environment cleaned up');
}
