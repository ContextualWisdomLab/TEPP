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
    EventTime::parse_rfc3339(&format!("2026-07-{day:02}T00:00:00Z")).expect("event time")
}

fn relation(source: Uuid, target: Uuid, source_day: u8, target_day: u8) -> RelationEdge {
    let interval = |day| {
        TemporalInterval::bounded(
            TemporalBoundary::Included(event_time(day)),
            TemporalBoundary::Included(
                EventTime::parse_rfc3339(&format!("2026-07-{day:02}T12:00:00Z"))
                    .expect("end"),
            ),
            TemporalPrecision::Second,
        )
        .expect("interval")
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

fn reference_fixture() -> (ReferenceTopicInput, ReferenceTopicModelConfig) {
    let ids: Vec<_> = (1_u128..=4).map(Uuid::from_u128).collect();
    let times: Vec<_> = (1_u8..=4).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff");
    let mut snapshot = CorpusSnapshot::new();
    let mut memberships = MembershipNetwork::new();
    for id in &ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*id, available), &cutoff)
            .expect("eligible document");
        memberships
            .insert(
                MembershipAssignment::new(
                    MemberId::from_uuid(*id),
                    GroupId::from_uuid(Uuid::from_u128(100)),
                    MembershipRole::Project,
                    MembershipWeight::full().expect("full membership"),
                    event_time(1),
                    event_time(9),
                )
                .expect("membership"),
            )
            .expect("insert membership");
    }

    let mut relations = RelationGraph::new();
    for (source, target, source_day, target_day) in [(0, 1, 1, 2), (1, 2, 2, 3), (2, 3, 3, 4)] {
        relations
            .insert(relation(ids[source], ids[target], source_day, target_day))
            .expect("insert relation");
    }

    let counts = SparseMatrix::from_csr(
        4,
        4,
        vec![0, 2, 4, 6, 8],
        vec![0, 1, 0, 1, 2, 3, 2, 3],
        vec![90.0, 10.0, 85.0, 15.0, 10.0, 90.0, 15.0, 85.0],
    )
    .expect("counts");
    let input = ReferenceTopicInput::new(
        &snapshot,
        ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("reference input");
    let config = ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 0.001)
        .and_then(|value| value.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("reference config");
    (input, config)
}

#[test]
fn diagonal_laplace_variance_is_reciprocal_of_owner_ggn_diagonal() {
    let (input, config) = reference_fixture();
    let model = fit_reference_topic_model(&input, &config).expect("converged reference fit");
    let precision = input
        .build_joint_coordinate_precision(
            &model,
            &config,
            vec![Uuid::from_u128(101), Uuid::from_u128(102)],
        )
        .expect("owner joint precision");

    assert_eq!(model.document_coordinate_variances.len(), 4);
    assert_eq!(precision.values().len(), 4);

    for (document, variances) in model.document_coordinate_variances.iter().enumerate() {
        assert_eq!(variances.len(), 1);
        let expected = precision.values()[document][document].recip();
        let actual = variances[0];
        assert!(
            (actual - expected).abs() <= 1.0e-12,
            "document {document}: retained diagonal variance {actual:.17e} must equal reciprocal owner GGN diagonal {expected:.17e}"
        );
    }

    // This fixture must distinguish the former degree*relation_strength shortcut
    // from the actual softmax-Jacobian GGN contribution.
    let degrees = [1.0_f64, 2.0, 2.0, 1.0];
    assert!(model
        .document_topic_proportions
        .iter()
        .zip(&model.document_coordinate_variances)
        .zip(degrees)
        .any(|((theta, variances), degree)| {
            let legacy = (100.0 * theta[0] * (1.0 - theta[0]) + 1.0 + degree * 0.5).recip();
            (variances[0] - legacy).abs() > 1.0e-9
        }));
}
