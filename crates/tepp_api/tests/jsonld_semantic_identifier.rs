//! Contract regression for semantic Rust identifiers on the JSON-LD export seam.

use tepp_api::JsonLdExport;

#[test]
fn jsonld_export_uses_node_id_in_rust_and_preserves_v1_wire_id() {
    let export = JsonLdExport::new(
        "https://example.org/tepp/context.jsonld",
        "urn:tepp:artifact:1",
        "ValidationReport",
        "abc123",
    )
    .expect("valid JSON-LD export");

    assert_eq!(export.node_id, "urn:tepp:artifact:1");

    let wire_json = export.to_json().expect("serialize JSON-LD export");
    assert!(wire_json.contains(r#""id":"urn:tepp:artifact:1""#));
    assert!(!wire_json.contains("node_id"));

    let decoded = JsonLdExport::from_json(&wire_json).expect("deserialize v1 JSON-LD export");
    assert_eq!(decoded.node_id, export.node_id);
}
