use blacklake_core::{
    canonical_to_dc_jsonld, canonical_to_turtle, project_to_index, CanonicalMeta, DC_CONTEXT,
};
use chrono::TimeZone;
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_metadata_integration() {
    // Test data
    let meta = CanonicalMeta {
        creation_dt: chrono::Utc.with_ymd_and_hms(2025, 1, 17, 18, 28, 0).unwrap(),
        creator: "you@example.org".to_string(),
        file_name: "demo.csv".to_string(),
        file_type: "text/csv".to_string(),
        file_size: 1234,
        org_lab: "ORNL".to_string(),
        description: "Demo dataset".to_string(),
        data_source: "sensor".to_string(),
        data_collection_method: "manual".to_string(),
        version: "1.0".to_string(),
        notes: Some("Test notes".to_string()),
        tags: Some(vec!["demo".to_string(), "csv".to_string()]),
        license: Some("CC-BY-4.0".to_string()),
    };

    let subject_iri = "https://blacklake.local/mylab/main/datasets/demo.csv";

    // Test JSON-LD generation
    let jsonld = canonical_to_dc_jsonld(subject_iri, &meta);
    
    // Verify JSON-LD structure
    assert!(jsonld.get("@context").is_some());
    assert_eq!(jsonld.get("@id").unwrap().as_str().unwrap(), subject_iri);
    assert_eq!(jsonld.get("dc:title").unwrap().as_str().unwrap(), "demo.csv");
    assert_eq!(jsonld.get("dc:creator").unwrap().as_str().unwrap(), "you@example.org");
    assert_eq!(jsonld.get("dcterms:extent").unwrap().as_i64().unwrap(), 1234);
    
    // Test Turtle generation
    let turtle = canonical_to_turtle(subject_iri, &meta).unwrap();
    assert!(turtle.contains("<https://blacklake.local/mylab/main/datasets/demo.csv>"));
    assert!(turtle.contains("dc:title"));
    assert!(turtle.contains("demo.csv"));
    assert!(turtle.contains("dcterms:extent"));
    assert!(turtle.contains("1234"));

    // Test metadata indexing
    let commit_id = Uuid::new_v4();
    let meta_json = json!({
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
        "tags": ["demo", "csv"]
    });

    let index = project_to_index(commit_id, "datasets/demo.csv", &meta_json);
    
    assert_eq!(index.commit_id, commit_id);
    assert_eq!(index.path, "datasets/demo.csv");
    assert_eq!(index.file_name, Some("demo.csv".to_string()));
    assert_eq!(index.file_size, Some(1234));
    assert_eq!(index.tags, Some(vec!["demo".to_string(), "csv".to_string()]));
}

#[test]
fn test_dublin_core_context() {
    // Verify DC context structure
    let context = &DC_CONTEXT["@context"];
    assert!(context.get("dc").is_some());
    assert!(context.get("dcterms").is_some());
    assert_eq!(
        context.get("dc").unwrap().as_str().unwrap(),
        "http://purl.org/dc/elements/1.1/"
    );
    assert_eq!(
        context.get("dcterms").unwrap().as_str().unwrap(),
        "http://purl.org/dc/terms/"
    );
}

#[test]
fn test_minimal_metadata() {
    // Test with minimal required fields
    let minimal_meta = CanonicalMeta {
        creation_dt: chrono::Utc::now(),
        creator: "test@example.org".to_string(),
        file_name: "test.txt".to_string(),
        file_type: "text/plain".to_string(),
        file_size: 100,
        org_lab: "TestLab".to_string(),
        description: "Test file".to_string(),
        data_source: "generated".to_string(),
        data_collection_method: "automatic".to_string(),
        version: "1.0".to_string(),
        notes: None,
        tags: None,
        license: None,
    };

    let subject_iri = "https://blacklake.local/test/main/test.txt";
    let jsonld = canonical_to_dc_jsonld(subject_iri, &minimal_meta);
    
    // Should not have optional fields
    assert!(jsonld.get("dcterms:license").is_none());
    assert!(jsonld.get("dc:subject").is_none());
    
    // Should have required fields
    assert!(jsonld.get("dc:title").is_some());
    assert!(jsonld.get("dc:creator").is_some());
    assert!(jsonld.get("dc:description").is_some());
}
