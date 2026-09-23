//! Scientific recovery fits the complete declared K grid through one owner path.

use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{FittedCandidateKConfig, fit_declared_recovery_candidates};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use topic_measurement::{ReferenceTopicModelConfig, ReferenceTopicTrainingInput, SparseMatrix};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-02-{day:02}T00:00:00Z")).expect("event time")
}

fn transition(source: Uuid, target: Uuid, source_day: u8, target_day: u8) -> RelationEdge {
    let interval = |day| {
        TemporalInterval::bounded(
            TemporalBoundary::Included(event_time(day)),
            TemporalBoundary::Included(event_time(day)),
            TemporalPrecision::Second,
        )
        .expect("event interval")
    };
    RelationEdge::new(
        RelationKind::TransitionsTo,
        RelationEndpointId::from_uuid(source),
        RelationEndpointId::from_uuid(target),
        RelationEvidenceStatus::Observed,
        interval(source_day),
        interval(target_day),
    )
    .expect("forward transition")
}

fn training_input() -> ReferenceTopicTrainingInput {
    let document_ids: Vec<_> = (1_u128..=6).map(Uuid::from_u128).collect();
    let event_times: Vec<_> = (1_u8..=6).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-02-20T00:00:00Z").expect("cutoff");

    let mut snapshot = CorpusSnapshot::new();
    let mut memberships = MembershipNetwork::new();
    let group_id = GroupId::from_uuid(Uuid::from_u128(100));
    for document_id in &document_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*document_id, available), &cutoff)
            .expect("training document");
        memberships
            .insert(
                MembershipAssignment::new(
                    MemberId::from_uuid(*document_id),
                    group_id,
                    MembershipRole::Project,
                    MembershipWeight::full().expect("full membership"),
                    event_time(1),
                    event_time(19),
                )
                .expect("membership"),
            )
            .expect("insert membership");
    }

    let mut relations = RelationGraph::new();
    for (index, source_day) in (1_u8..=5).enumerate() {
        relations
            .insert(transition(
                document_ids[index],
                document_ids[index + 1],
                source_day,
                source_day + 1,
            ))
            .expect("insert transition");
    }

    let counts = SparseMatrix::from_csr(
        6,
        6,
        vec![0, 2, 4, 6, 8, 10, 12],
        vec![0, 1, 0, 1, 2, 3, 2, 3, 4, 5, 4, 5],
        vec![
            90.0, 10.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0,
        ],
    )
    .expect("training counts");

    ReferenceTopicTrainingInput::new(
        &snapshot,
        document_ids,
        &counts,
        &event_times,
        None,
        &memberships,
        &relations,
    )
    .expect("training input")
}

#[test]
fn public_recovery_fit_owner_materializes_every_declared_candidate_in_order() {
    let training = training_input();
    let declared = FittedCandidateKConfig::new(vec![2, 3], vec![7, 11, 19], 2_000, 0.001)
        .expect("declared recovery design");

    let fits = fit_declared_recovery_candidates(&training, &declared)
        .expect("complete declared recovery fit grid");

    assert_eq!(fits.len(), 2);
    assert_eq!(
        fits[0]
            .reference_fit()
            .model()
            .topic_term_probabilities
            .len(),
        2
    );
    assert_eq!(
        fits[1]
            .reference_fit()
            .model()
            .topic_term_probabilities
            .len(),
        3
    );
    for (fit, candidate_k) in fits.iter().zip(declared.candidate_topic_counts()) {
        let expected = ReferenceTopicModelConfig::new(
            usize::try_from(*candidate_k).expect("candidate K fits usize"),
            declared.seeds().to_vec(),
            declared.maximum_iterations(),
            declared.tolerance(),
        )
        .expect("default owner reference config");
        assert_eq!(fit.reference_fit().config(), &expected);
        assert!(
            fit.training_input()
                .shares_numerical_training_state(&training)
        );
    }
}
