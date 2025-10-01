// Mobile Search Tests
// Week 3: Tests for mobile search functionality

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mobileSearchApi } from '@/services/mobileSearchApi'
import { searchHelpers } from '@/utils/mobileSearchHelpers'

// Mock fetch
global.fetch = vi.fn()

describe('Mobile Search API', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('mobileSearchApi', () => {
    it('should make search requests with correct parameters', async () => {
      const mockResponse = {
        results: [
          {
            id: '1',
            name: 'test.csv',
            path: '/test.csv',
            description: 'Test file',
            type: 'file',
            size: 1024,
            lastModified: '2023-01-01T00:00:00Z',
            author: 'Test User',
            tags: ['test', 'data']
          }
        ],
        total: 1,
        filters: {},
        suggestions: ['test suggestion'],
        tags: ['test', 'data']
      }

      ;(fetch as any).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockResponse)
      })

      const result = await mobileSearchApi.search({
        query: 'test',
        limit: 20,
        offset: 0
      })

      expect(fetch).toHaveBeenCalledWith(
        expect.stringContaining('/v1/search'),
        expect.objectContaining({
          method: 'GET',
          headers: expect.objectContaining({
            'Content-Type': 'application/json'
          })
        })
      )

      expect(result).toEqual(mockResponse)
    })

    it('should handle search errors gracefully', async () => {
      ;(fetch as any).mockRejectedValueOnce(new Error('Network error'))

      await expect(mobileSearchApi.search({
        query: 'test',
        limit: 20,
        offset: 0
      })).rejects.toThrow('Network error')
    })

    it('should make suggestions requests', async () => {
      const mockResponse = {
        suggestions: ['test suggestion 1', 'test suggestion 2'],
        tags: ['test', 'data']
      }

      ;(fetch as any).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockResponse)
      })

      const result = await mobileSearchApi.getSuggestions({
        query: 'test',
        limit: 10
      })

      expect(fetch).toHaveBeenCalledWith(
        expect.stringContaining('/v1/search/suggest'),
        expect.objectContaining({
          method: 'GET'
        })
      )

      expect(result).toEqual(mockResponse)
    })

    it('should make analytics requests', async () => {
      const mockResponse = {
        totalSearches: 100,
        popularQueries: ['test query'],
        recentSearches: ['recent query'],
        searchTrends: [
          { date: '2023-01-01', count: 10 }
        ]
      }

      ;(fetch as any).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockResponse)
      })

      const result = await mobileSearchApi.getAnalytics({
        query: 'test'
      })

      expect(fetch).toHaveBeenCalledWith(
        expect.stringContaining('/v1/search/analytics'),
        expect.objectContaining({
          method: 'GET'
        })
      )

      expect(result).toEqual(mockResponse)
    })
  })

  describe('searchHelpers', () => {
    it('should perform semantic search', async () => {
      const mockResponse = {
        results: [
          {
            id: '1',
            name: 'test.csv',
            path: '/test.csv',
            description: 'Test file',
            type: 'file',
            size: 1024,
            lastModified: '2023-01-01T00:00:00Z',
            author: 'Test User',
            tags: ['test', 'data']
          }
        ],
        total: 1,
        filters: {},
        suggestions: ['test suggestion'],
        tags: ['test', 'data']
      }

      ;(fetch as any).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockResponse)
      })

      const result = await searchHelpers.performSemanticSearch('test query', true)

      expect(result.results).toHaveLength(1)
      expect(result.results[0]).toMatchObject({
        id: '1',
        name: 'test.csv',
        similarity: expect.any(Number)
      })
      expect(result.suggestedTags).toHaveLength(5)
    })

    it('should get compliance entries', async () => {
      const result = await searchHelpers.getComplianceEntries()

      expect(result).toHaveLength(3)
      expect(result[0]).toMatchObject({
        id: '1',
        name: 'customer_data.csv',
        classification: 'restricted',
        legalHold: true
      })
    })

    it('should get audit logs', async () => {
      const result = await searchHelpers.getAuditLogs()

      expect(result).toHaveLength(3)
      expect(result[0]).toMatchObject({
        id: '1',
        action: 'file_upload',
        user: 'john.doe@example.com'
      })
    })

    it('should get connectors', async () => {
      const result = await searchHelpers.getConnectors()

      expect(result).toHaveLength(3)
      expect(result[0]).toMatchObject({
        id: '1',
        name: 'AWS S3 Connector',
        type: 's3',
        status: 'active'
      })
    })

    it('should test connector', async () => {
      const result = await searchHelpers.testConnector('test-id')

      expect(result).toMatchObject({
        success: true,
        message: 'Connector test successful'
      })
    })

    it('should sync connector', async () => {
      const result = await searchHelpers.syncConnector('test-id')

      expect(result).toMatchObject({
        success: true,
        message: 'Connector sync completed successfully',
        entriesProcessed: expect.any(Number)
      })
    })

    it('should delete connector', async () => {
      const result = await searchHelpers.deleteConnector('test-id')

      expect(result).toMatchObject({
        success: true,
        message: 'Connector deleted successfully'
      })
    })

    it('should create connector', async () => {
      const result = await searchHelpers.createConnector({
        name: 'Test Connector',
        type: 's3',
        configuration: { bucket: 'test-bucket' }
      })

      expect(result).toMatchObject({
        success: true,
        message: 'Connector created successfully',
        connectorId: expect.any(String)
      })
    })
  })
})
