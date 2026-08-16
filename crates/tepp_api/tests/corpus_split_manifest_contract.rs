//! Corpus-split leakage-audit wire contract.
//!
//! A buyer and naruon consumer must see which documents entered an analysis,
//! which were excluded at the knowledge cutoff, and a digest that matches the
//! `corpus_split_manifest` row stored by migration `0003`.

use corpus_split::{CorpusDocument, LeakageLink, LeakageLinkKind};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{
    CORPUS_SPLIT_MANIFEST_CONTRACT_VERSION, CorpusSplitManifest, CorpusSplitPartitions,
};
use uuid::Uuid;

fn document(id: u128, stamp: &str) -> CorpusDocument {
    CorpusDocument::new(
        Uuid::from_u128(id),
        AvailableTime::parse_rfc3339(stamp).expect("available"),
    )
}

fn cutoff(stamp: &str) -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339(stamp).expect("cutoff")
}

fn sample_partitions() -> CorpusSplitPartitions {
    CorpusSplitPartitions {
        train_document_ids: vec![Uuid::from_u128(1), Uuid::from_u128(2)],
        validation_document_ids: vec![Uuid::from_u128(3)],
        test_document_ids: vec![Uuid::from_u128(4)],
    }
}

fn sample_candidates() -> Vec<CorpusDocument> {
    vec![
        document(1, "2026-01-01T00:00:00Z"),
        document(2, "2026-01-02T00:00:00Z"),
        document(3, "2026-01-03T00:00:00Z"),
        document(4, "2026-01-04T00:00:00Z"),
        document(9, "2026-08-01T00:00:00Z"),
    ]
}

fn sample_links() -> Vec<LeakageLink> {
    vec![LeakageLink {
        left: Uuid::from_u128(1),
        right: Uuid::from_u128(2),
        kind: LeakageLinkKind::Translation,
    }]
}

#[test]
fn late_available_document_is_counted_excluded_and_stays_out_of_partitions() {
    let manifest = CorpusSplitManifest::from_domain(
        "split-demo-001",
        &cutoff("2026-03-01T00:00:00Z"),
        "relation-aware-v1",
        &sample_candidates(),
        &sample_links(),
        &sample_partitions(),
    )
    .expect("eligible split");

    assert_eq!(
        manifest.contract_version,
        CORPUS_SPLIT_MANIFEST_CONTRACT_VERSION
    );
    assert_eq!(manifest.included_document_count, 4);
    assert_eq!(manifest.excluded_unavailable_at_cutoff_count, 1);
    assert_eq!(manifest.connected_group_count, 3);
    assert_eq!(manifest.governed_link_kinds, vec!["translation".to_owned()]);
    assert_eq!(manifest.split_manifest_digest_sha256.len(), 64);
    assert!(
        manifest
            .split_manifest_digest_sha256
            .chars()
            .all(|digit| digit.is_ascii_hexdigit() && !digit.is_ascii_uppercase())
    );
    let json = manifest.to_json().expect("json");
    let parsed = CorpusSplitManifest::from_json(&json).expect("round-trip");
    assert_eq!(parsed, manifest);
}

#[test]
fn relation_leakage_fails_closed_when_linked_members_split() {
    let mut leaked = sample_partitions();
    leaked.train_document_ids = vec![Uuid::from_u128(1)];
    leaked.validation_document_ids = vec![Uuid::from_u128(2)];
    leaked.test_document_ids = vec![Uuid::from_u128(3), Uuid::from_u128(4)];
    assert_eq!(
        CorpusSplitManifest::from_domain(
            "split-leaked",
            &cutoff("2026-03-01T00:00:00Z"),
            "relation-aware-v1",
            &sample_candidates(),
            &sample_links(),
            &leaked,
        ),
        Err(tepp_api::ApiError::InvalidWirePayload)
    );
}

#[test]
fn unknown_fields_and_tampered_digest_fail_closed() {
    let manifest = CorpusSplitManifest::from_domain(
        "split-demo-001",
        &cutoff("2026-03-01T00:00:00Z"),
        "relation-aware-v1",
        &sample_candidates(),
        &sample_links(),
        &sample_partitions(),
    )
    .expect("eligible split");
    let mut json = manifest.to_json().expect("json");
    json.insert(json.len() - 1, ',');
    json.insert_str(json.len() - 1, r#""extra":"no""#);
    assert_eq!(
        CorpusSplitManifest::from_json(&json),
        Err(tepp_api::ApiError::InvalidWirePayload)
    );

    let mut tampered = manifest.clone();
    tampered.split_manifest_digest_sha256 = "ab".repeat(32);
    assert_eq!(
        tampered.to_json(),
        Err(tepp_api::ApiError::InvalidWirePayload)
    );
}

#[test]
fn committed_example_parses_through_the_live_contract() {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("examples");
    path.push("corpus_split_manifest_v1.json");
    let payload = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("missing example {}: {error}", path.display()));
    let parsed = CorpusSplitManifest::from_json(&payload).expect("example");
    assert_eq!(parsed.manifest_id, "split-demo-001");
    assert_eq!(parsed.excluded_unavailable_at_cutoff_count, 1);
}
