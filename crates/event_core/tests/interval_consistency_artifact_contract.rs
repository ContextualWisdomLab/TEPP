//! Contract tests for durable bounded interval-consistency exports.

use event_core::{
    EventError, INTERVAL_CONSISTENCY_ARTIFACT_TYPE, IntervalConsistencyArtifact,
    IntervalConsistencyNetwork,
};
use temporal_core::{AllenRelation, RelationSet};

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn artifact() -> IntervalConsistencyArtifact {
    let mut network = IntervalConsistencyNetwork::with_limits(3, 3, 128).expect("limits");
    let first = network.add_variable().expect("first");
    let second = network.add_variable().expect("second");
    let third = network.add_variable().expect("third");
    network
        .assert_qualitative_relations(first, second, RelationSet::singleton(AllenRelation::Before))
        .expect("first assertion");
    network
        .assert_qualitative_relations(second, third, RelationSet::singleton(AllenRelation::Before))
        .expect("second assertion");
    network.close().expect("closure");
    IntervalConsistencyArtifact::from_network(
        "run<&1",
        "snapshot-1",
        DIGEST,
        &network,
        &[
            ("event<&1".to_owned(), first),
            ("event-2".to_owned(), second),
            ("event-3".to_owned(), third),
        ],
    )
    .expect("artifact")
}

#[test]
fn derived_branch_is_durable_typed_and_provenance_bearing() {
    let artifact = artifact();
    assert_eq!(
        INTERVAL_CONSISTENCY_ARTIFACT_TYPE,
        "tdt_chronos_interval_consistency_v1"
    );
    assert_eq!(artifact.relations.len(), 3);
    let derived = artifact
        .relations
        .iter()
        .find(|relation| {
            relation.left_event_id == "event<&1" && relation.right_event_id == "event-3"
        })
        .expect("derived relation");
    assert_eq!(derived.allen_relations, vec![AllenRelation::Before]);
    assert!(!derived.observed);
    assert_eq!(derived.support_assertion_ordinals, vec![0, 1]);

    let json = artifact.to_json().expect("json");
    assert_eq!(
        IntervalConsistencyArtifact::from_json(&json).expect("round trip"),
        artifact
    );
    assert_eq!(artifact.sha256().expect("digest").len(), 64);
    let graphml = artifact.to_graphml().expect("graphml");
    assert!(graphml.contains("allen_relations"));
    assert!(graphml.contains(&format!("<data key=\"input_digest\">{DIGEST}</data>")));
    assert!(graphml.contains("<data key=\"observed\">false</data>"));
    assert!(graphml.contains("<data key=\"support\">0,1</data>"));
    assert!(graphml.contains("event&lt;&amp;1"));
}

#[test]
fn reverse_oriented_assertion_is_observed_in_stable_export_order() {
    let mut network = IntervalConsistencyNetwork::with_limits(2, 1, 8).expect("limits");
    let left = network.add_variable().expect("left");
    let right = network.add_variable().expect("right");
    network
        .assert_qualitative_relations(right, left, RelationSet::singleton(AllenRelation::After))
        .expect("reverse assertion");
    let artifact = IntervalConsistencyArtifact::from_network(
        "run-reverse",
        "snapshot-reverse",
        DIGEST,
        &network,
        &[
            ("event-left".to_owned(), left),
            ("event-right".to_owned(), right),
        ],
    )
    .expect("artifact");

    assert!(artifact.relations[0].observed);
    assert_eq!(
        artifact.relations[0].allen_relations,
        vec![AllenRelation::Before]
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn artifact_rejects_unbound_or_noncanonical_payloads() {
    let mut network = IntervalConsistencyNetwork::with_limits(2, 1, 8).expect("limits");
    let first = network.add_variable().expect("first");
    let second = network.add_variable().expect("second");
    network
        .assert_qualitative_relations(first, second, RelationSet::singleton(AllenRelation::Before))
        .expect("assertion");
    assert_eq!(
        IntervalConsistencyArtifact::from_network(
            "run",
            "snapshot",
            "not-a-digest",
            &network,
            &[("same".to_owned(), first), ("same".to_owned(), second)]
        ),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        IntervalConsistencyArtifact::from_network(
            "run",
            "snapshot",
            DIGEST,
            &network,
            &[("event-1".to_owned(), first)]
        ),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        IntervalConsistencyArtifact::from_network(
            "run",
            "snapshot",
            DIGEST,
            &network,
            &[(String::new(), first), ("event-2".to_owned(), second)]
        ),
        Err(EventError::InvalidWirePayload)
    );
    let json = artifact().to_json().expect("json");
    assert_eq!(
        IntervalConsistencyArtifact::from_json(&format!(" {json}")),
        Err(EventError::InvalidWirePayload)
    );
    let unknown = json.replacen('{', "{\"unknown\":true,", 1);
    assert_eq!(
        IntervalConsistencyArtifact::from_json(&unknown),
        Err(EventError::InvalidWirePayload)
    );

    let valid = artifact();
    let mut invalid_binding = valid.clone();
    invalid_binding.input_digest_sha256 = "bad".to_owned();
    assert_eq!(
        invalid_binding.to_json(),
        Err(EventError::InvalidWirePayload)
    );
    let mut invalid_relation = valid.clone();
    invalid_relation.relations[0]
        .support_assertion_ordinals
        .clear();
    assert_eq!(
        invalid_relation.to_graphml(),
        Err(EventError::InvalidWirePayload)
    );
    for invalid in invalid_artifact_variants(&valid) {
        assert_eq!(invalid.to_json(), Err(EventError::InvalidWirePayload));
    }
    let mut numeric_digest = valid.clone();
    numeric_digest.input_digest_sha256 = "1".repeat(64);
    numeric_digest
        .to_json()
        .expect("numeric digest is canonical");
    let mut oversized = valid;
    oversized.run_id = "x".repeat(4 * 1024 * 1024);
    assert_eq!(oversized.to_json(), Err(EventError::InvalidWirePayload));
    assert_eq!(
        IntervalConsistencyArtifact::from_network(
            "x".repeat(4 * 1024 * 1024),
            "snapshot",
            DIGEST,
            &network,
            &[
                ("event-1".to_owned(), first),
                ("event-2".to_owned(), second)
            ]
        ),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        IntervalConsistencyArtifact::from_json(&"x".repeat(4 * 1024 * 1024 + 1)),
        Err(EventError::InvalidWirePayload)
    );

    let unconstrained = IntervalConsistencyNetwork::with_limits(2, 1, 8).expect("limits");
    let mut foreign = IntervalConsistencyNetwork::with_limits(2, 1, 8).expect("foreign");
    let foreign_left = foreign.add_variable().expect("foreign left");
    let foreign_right = foreign.add_variable().expect("foreign right");
    assert_eq!(
        IntervalConsistencyArtifact::from_network(
            "run",
            "snapshot",
            DIGEST,
            &unconstrained,
            &[
                ("event-1".to_owned(), foreign_left),
                ("event-2".to_owned(), foreign_right)
            ]
        ),
        Err(EventError::IntervalConsistencyUnknownVariable)
    );

    let mut quiet = IntervalConsistencyNetwork::with_limits(2, 1, 8).expect("quiet");
    let quiet_left = quiet.add_variable().expect("quiet left");
    let quiet_right = quiet.add_variable().expect("quiet right");
    assert_eq!(
        IntervalConsistencyArtifact::from_network(
            "run",
            "snapshot",
            DIGEST,
            &quiet,
            &[
                ("event-1".to_owned(), quiet_left),
                ("event-2".to_owned(), quiet_right)
            ]
        ),
        Err(EventError::InvalidWirePayload)
    );
}

fn invalid_artifact_variants(
    valid: &IntervalConsistencyArtifact,
) -> Vec<IntervalConsistencyArtifact> {
    let mut variants = Vec::new();
    let mut invalid = valid.clone();
    "unsupported".clone_into(&mut invalid.schema_version);
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.run_id.clear();
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.snapshot_id.clear();
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.relations = vec![valid.relations[0].clone(); 100_001];
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.relations[0].left_event_id.clear();
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.relations[0].right_event_id.clear();
    variants.push(invalid);
    let mut invalid = valid.clone();
    let left_identity = invalid.relations[0].left_event_id.clone();
    invalid.relations[0]
        .right_event_id
        .clone_from(&left_identity);
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.relations[0].allen_relations.clear();
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.relations[0].allen_relations = AllenRelation::ALL.to_vec();
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.relations[0].allen_relations = vec![AllenRelation::Before, AllenRelation::Before];
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.relations[0].support_assertion_ordinals = vec![0, 0];
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.relations.swap(0, 1);
    variants.push(invalid);
    variants
}
