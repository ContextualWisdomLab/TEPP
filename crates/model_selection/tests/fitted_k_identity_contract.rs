//! Fitted candidate-`K` scoring uses real TRSL-TM reference fits.

use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{ModelSelectionError, statistical_candidate_from_fit};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use topic_measurement::{
    ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix, fit_reference_topic_model,
};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("event time")
}

fn relation(source: Uuid, target: Uuid, source_day: u8, target_day: u8) -> RelationEdge {
    let interval = |day| {
        TemporalInterval::bounded(
            TemporalBoundary::Included(event_time(day)),
            TemporalBoundary::Included(
                EventTime::parse_rfc3339(&format!("2026-01-{day:02}T12:00:00Z"))
                    .expect("interval end"),
            ),
            TemporalPrecision::Second,
        )
        .expect("bounded interval")
    };
    RelationEdge::new(
        RelationKind::TransitionsTo,
        RelationEndpointId::from_uuid(source),
        RelationEndpointId::from_uuid(target),
        RelationEvidenceStatus::Observed,
        interval(source_day),
        interval(target_day),
    )
    .expect("forward relation")
}

fn two_document_input(term_values: [f64; 4]) -> ReferenceTopicInput {
    let document_ids = vec![Uuid::from_u128(1), Uuid::from_u128(2)];
    let times = vec![event_time(1), event_time(2)];
    let available = AvailableTime::parse_rfc3339("2026-01-10T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff");
    let mut snapshot = CorpusSnapshot::new();
    for id in &document_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*id, available), &cutoff)
            .expect("eligible");
    }
    let mut memberships = MembershipNetwork::new();
    memberships
        .insert(
            MembershipAssignment::new(
                MemberId::from_uuid(document_ids[0]),
                GroupId::from_uuid(Uuid::from_u128(100)),
                MembershipRole::Organization,
                MembershipWeight::full().expect("full"),
                event_time(1),
                event_time(9),
            )
            .expect("membership"),
        )
        .expect("insert");
    memberships
        .insert(
            MembershipAssignment::new(
                MemberId::from_uuid(document_ids[1]),
                GroupId::from_uuid(Uuid::from_u128(100)),
                MembershipRole::Organization,
                MembershipWeight::full().expect("full"),
                event_time(1),
                event_time(9),
            )
            .expect("membership"),
        )
        .expect("insert");
    let mut relations = RelationGraph::new();
    relations
        .insert(relation(document_ids[0], document_ids[1], 1, 2))
        .expect("relation");
    let counts =
        SparseMatrix::from_csr(2, 2, vec![0, 2, 4], vec![0, 1, 0, 1], term_values.to_vec())
            .expect("counts");
    ReferenceTopicInput::new(
        &snapshot,
        document_ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("two-document input")
}

/// Matching candidate identity uses a real public fit, not an injected model DTO.
#[test]
fn candidate_k_matches_real_fit() {
    let input = two_document_input([8.0, 0.0, 0.0, 4.0]);
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2000, 1e-5).expect("config");
    let model = fit_reference_topic_model(&input, &config).expect("actual CPU fit");
    assert_eq!(model.topic_term_probabilities.len(), 2);
    let candidate = statistical_candidate_from_fit(&input, 2, &model).expect("matching K");
    assert_eq!(candidate.candidate_k(), 2);
    assert!(candidate.is_statistically_supported());
}
/// A real two-topic fit must not supply evidence for any other candidate K.
#[test]
fn wrong_candidate_k_is_rejected() {
    let input = two_document_input([8.0, 0.0, 0.0, 4.0]);
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2000, 1e-5).expect("config");
    let model = fit_reference_topic_model(&input, &config).expect("actual CPU fit");
    assert_eq!(model.topic_term_probabilities.len(), 2);
    for k in [0, 1, 3, 4, u32::MAX] {
        let actual = statistical_candidate_from_fit(&input, k, &model);
        println!("REAL_FIT_K actual_model_topics=2 supplied_k={k} result={actual:?}");
        let expected = if k < 2 {
            ModelSelectionError::NonPositiveCandidateK
        } else {
            ModelSelectionError::InvalidDiagnostic
        };
        assert_eq!(
            actual,
            Err(expected),
            "candidate K must bind to the real fitted dimension"
        );
    }
}
