//! Fit-origin binding, precision, and draw contracts for the CPU `f64` reference.

use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
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

fn fixture() -> (
    CorpusSnapshot,
    Vec<Uuid>,
    Vec<EventTime>,
    MembershipNetwork,
    RelationGraph,
) {
    let document_ids: Vec<_> = (1_u128..=6).map(Uuid::from_u128).collect();
    let times: Vec<_> = (1_u8..=6).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339("2026-01-10T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff");
    let mut snapshot = CorpusSnapshot::new();
    for id in &document_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*id, available), &cutoff)
            .expect("eligible");
    }

    let organization = GroupId::from_uuid(Uuid::from_u128(100));
    let projects = [
        GroupId::from_uuid(Uuid::from_u128(101)),
        GroupId::from_uuid(Uuid::from_u128(102)),
    ];
    let validity_start = event_time(1);
    let validity_end = event_time(9);
    let mut memberships = MembershipNetwork::new();
    for (index, id) in document_ids.iter().enumerate() {
        let member = MemberId::from_uuid(*id);
        memberships
            .insert(
                MembershipAssignment::new(
                    member,
                    organization,
                    MembershipRole::Organization,
                    MembershipWeight::full().expect("full"),
                    validity_start,
                    validity_end,
                )
                .expect("organization membership"),
            )
            .expect("insert organization");
        memberships
            .insert(
                MembershipAssignment::new(
                    member,
                    projects[usize::from(index >= 3)],
                    MembershipRole::Project,
                    MembershipWeight::new(0.75).expect("partial"),
                    validity_start,
                    validity_end,
                )
                .expect("project membership"),
            )
            .expect("insert project");
    }

    let mut relations = RelationGraph::new();
    for (source, target, source_day, target_day) in [
        (0, 1, 1, 2),
        (1, 2, 2, 3),
        (2, 3, 3, 4),
        (3, 4, 4, 5),
        (4, 5, 5, 6),
    ] {
        relations
            .insert(relation(
                document_ids[source],
                document_ids[target],
                source_day,
                target_day,
            ))
            .expect("insert relation");
    }
    (snapshot, document_ids, times, memberships, relations)
}

/// New characterization: public input and a real K=3 CPU fit feed the complete public builder.
#[test]
fn actual_three_topic_fit_retains_joint_precision_at_two_count_scales() {
    for scale in [1.0_f64, 1_000.0_f64] {
        let counts = SparseMatrix::from_csr(
            6,
            4,
            vec![0, 2, 4, 6, 8, 10, 12],
            vec![0, 1, 0, 1, 0, 1, 2, 3, 2, 3, 2, 3],
            vec![90., 10., 85., 15., 80., 20., 10., 90., 15., 85., 20., 80.]
                .into_iter()
                .map(|x| x * scale)
                .collect(),
        )
        .expect("finite positive public CSR counts");
        let (snapshot, ids, times, memberships, relations) = fixture();
        let input = ReferenceTopicInput::new(
            &snapshot,
            ids,
            &counts,
            &times,
            None,
            &memberships,
            &relations,
        )
        .expect("fully admitted public input");
        let config = ReferenceTopicModelConfig::new(3, vec![7], 5_000, 1e-5)
            .expect("K=3 configuration")
            .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
            .expect("hyperparameters");
        let fit =
            fit_reference_topic_model(&input, &config).expect("actual converged public CPU fit");
        assert!(fit.objective.is_finite());
        assert_eq!(fit.document_topic_proportions.len(), 6);
        assert_eq!(fit.topic_term_probabilities.len(), 3);
        assert!(fit.document_topic_proportions.iter().all(|r| r.len() == 3
            && r.iter().all(|x| x.is_finite() && *x > 0.)
            && (r.iter().sum::<f64>() - 1.).abs() < 1e-12));
        let tokens = 100. * scale;
        let asymmetries: Vec<f64> = fit
            .document_topic_proportions
            .iter()
            .map(|r| ((-tokens * r[0] * r[1]) - (-tokens * r[1] * r[0])).abs())
            .collect();
        println!(
            "FIT scale={scale} tokens_per_document={tokens} iterations={} objective={} theta={:?} likelihood_asymmetries={asymmetries:?}",
            fit.iterations, fit.objective, fit.document_topic_proportions
        );
        let topics = (201_u128..=203).map(Uuid::from_u128).collect();
        let result = input.build_joint_coordinate_precision(&fit, &config, topics);
        println!(
            "BUILDER scale={scale} result={:?}",
            result.as_ref().map(|x| (
                x.values().len(),
                x.values().iter().map(Vec::len).collect::<Vec<_>>()
            ))
        );
        let precision =
            result.expect("actual admitted fitted model should retain finite joint precision");
        assert_eq!(precision.values().len(), 12);
        assert!(
            precision
                .values()
                .iter()
                .all(|r| r.len() == 12 && r.iter().all(|x| x.is_finite()))
        );
    }
}
