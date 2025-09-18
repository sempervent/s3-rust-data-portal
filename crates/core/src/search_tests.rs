#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_solr_client_creation() {
        let client = SolrClient::new("http://localhost:8983", "test_collection").unwrap();
        assert_eq!(client.collection, "test_collection");
    }

    #[test]
    fn test_solr_client_invalid_url() {
        let result = SolrClient::new("invalid-url", "test_collection");
        assert!(result.is_err());
    }

    #[test]
    fn test_search_query_creation() {
        let query = SearchQuery {
            q: "test query".to_string(),
            fq: Some(vec!["file_type:csv".to_string(), "org_lab:ornl".to_string()]),
            sort: Some("creation_dt desc".to_string()),
            limit: Some(20),
            offset: Some(0),
            json_facet: Some(json!({
                "file_type": {
                    "type": "terms",
                    "field": "file_type",
                    "limit": 10
                }
            })),
        };

        assert_eq!(query.q, "test query");
        assert_eq!(query.fq.as_ref().unwrap().len(), 2);
        assert_eq!(query.limit, Some(20));
        assert_eq!(query.offset, Some(0));
        assert!(query.json_facet.is_some());
    }

    #[test]
    fn test_search_query_minimal() {
        let query = SearchQuery {
            q: "*:*".to_string(),
            fq: None,
            sort: None,
            limit: None,
            offset: None,
            json_facet: None,
        };

        assert_eq!(query.q, "*:*");
        assert!(query.fq.is_none());
        assert!(query.sort.is_none());
        assert!(query.limit.is_none());
        assert!(query.offset.is_none());
        assert!(query.json_facet.is_none());
    }

    #[test]
    fn test_search_response_creation() {
        let docs = vec![
            json!({
                "id": "doc1",
                "title": "Test Document 1",
                "file_type": "csv"
            }),
            json!({
                "id": "doc2", 
                "title": "Test Document 2",
                "file_type": "parquet"
            })
        ];

        let facets = json!({
            "file_type": {
                "buckets": [
                    {"val": "csv", "count": 1},
                    {"val": "parquet", "count": 1}
                ]
            }
        });

        let response = SearchResponse {
            docs: docs.clone(),
            num_found: 2,
            facets: Some(facets.clone()),
        };

        assert_eq!(response.docs.len(), 2);
        assert_eq!(response.num_found, 2);
        assert!(response.facets.is_some());
    }

    #[test]
    fn test_solr_status_creation() {
        let status = SolrStatus {
            collection: "test_collection".to_string(),
            doc_count: 1000,
            last_commit_at: Some("2024-01-15T10:30:00Z".to_string()),
            health: "OK".to_string(),
        };

        assert_eq!(status.collection, "test_collection");
        assert_eq!(status.doc_count, 1000);
        assert_eq!(status.health, "OK");
        assert!(status.last_commit_at.is_some());
    }

    #[test]
    fn test_solr_error_display() {
        let errors = vec![
            SolrError::Api("Solr server error".to_string()),
            SolrError::InvalidUrl("Invalid URL format".to_string()),
            SolrError::Other(anyhow::anyhow!("Generic error")),
        ];

        for error in errors {
            let error_string = format!("{}", error);
            assert!(!error_string.is_empty());
        }
    }

    #[test]
    fn test_search_query_serialization() {
        let query = SearchQuery {
            q: "test query".to_string(),
            fq: Some(vec!["file_type:csv".to_string()]),
            sort: Some("creation_dt desc".to_string()),
            limit: Some(10),
            offset: Some(0),
            json_facet: Some(json!({
                "file_type": {
                    "type": "terms",
                    "field": "file_type"
                }
            })),
        };

        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: SearchQuery = serde_json::from_str(&serialized).unwrap();

        assert_eq!(query.q, deserialized.q);
        assert_eq!(query.fq, deserialized.fq);
        assert_eq!(query.sort, deserialized.sort);
        assert_eq!(query.limit, deserialized.limit);
        assert_eq!(query.offset, deserialized.offset);
    }

    #[test]
    fn test_search_response_serialization() {
        let response = SearchResponse {
            docs: vec![json!({"id": "doc1", "title": "Test"})],
            num_found: 1,
            facets: Some(json!({"file_type": {"buckets": []}})),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: SearchResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response.docs.len(), deserialized.docs.len());
        assert_eq!(response.num_found, deserialized.num_found);
        assert!(response.facets.is_some());
        assert!(deserialized.facets.is_some());
    }

    #[test]
    fn test_solr_status_serialization() {
        let status = SolrStatus {
            collection: "test".to_string(),
            doc_count: 100,
            last_commit_at: Some("2024-01-01T00:00:00Z".to_string()),
            health: "OK".to_string(),
        };

        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: SolrStatus = serde_json::from_str(&serialized).unwrap();

        assert_eq!(status.collection, deserialized.collection);
        assert_eq!(status.doc_count, deserialized.doc_count);
        assert_eq!(status.health, deserialized.health);
        assert_eq!(status.last_commit_at, deserialized.last_commit_at);
    }

    #[test]
    fn test_solr_client_build_url() {
        let client = SolrClient::new("http://localhost:8983", "test_collection").unwrap();
        let url = client.build_url("select");
        
        assert!(url.to_string().contains("test_collection"));
        assert!(url.to_string().contains("select"));
    }

    #[test]
    fn test_search_query_with_complex_facets() {
        let json_facet = json!({
            "file_type": {
                "type": "terms",
                "field": "file_type",
                "limit": 10
            },
            "creation_dt": {
                "type": "range",
                "field": "creation_dt",
                "ranges": [
                    {
                        "from": "2024-01-01T00:00:00Z",
                        "to": "2024-12-31T23:59:59Z",
                        "label": "2024"
                    }
                ]
            }
        });

        let query = SearchQuery {
            q: "*:*".to_string(),
            fq: None,
            sort: None,
            limit: Some(20),
            offset: Some(0),
            json_facet: Some(json_facet),
        };

        assert!(query.json_facet.is_some());
        let facets = query.json_facet.unwrap();
        assert!(facets.get("file_type").is_some());
        assert!(facets.get("creation_dt").is_some());
    }

    #[test]
    fn test_search_query_with_multiple_filters() {
        let query = SearchQuery {
            q: "machine learning".to_string(),
            fq: Some(vec![
                "file_type:(csv OR parquet)".to_string(),
                "org_lab:ornl".to_string(),
                "creation_dt:[2024-01-01T00:00:00Z TO *]".to_string(),
            ]),
            sort: Some("creation_dt desc".to_string()),
            limit: Some(50),
            offset: Some(100),
            json_facet: None,
        };

        assert_eq!(query.q, "machine learning");
        assert_eq!(query.fq.as_ref().unwrap().len(), 3);
        assert_eq!(query.sort, Some("creation_dt desc".to_string()));
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.offset, Some(100));
    }
}
