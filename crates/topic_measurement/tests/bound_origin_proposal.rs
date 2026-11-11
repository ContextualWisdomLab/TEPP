//! Known-truth recovery contract for the CPU `f64` TRSL-TM reference.

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
    ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix, fit_origin_bound_topic_model,
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

fn separated_counts() -> SparseMatrix {
    SparseMatrix::from_csr(
        6,
        4,
        vec![0, 2, 4, 6, 8, 10, 12],
        vec![0, 1, 0, 1, 0, 1, 2, 3, 2, 3, 2, 3],
        vec![
            90.0, 10.0, 85.0, 15.0, 80.0, 20.0, 10.0, 90.0, 15.0, 85.0, 20.0, 80.0,
        ],
    )
    .expect("counts")
}

/// Build the admitted synthetic corpus used by these origin-binding diagnostics.
fn precision_origin_input() -> ReferenceTopicInput {
    let (snapshot, ids, times, memberships, relations) = fixture();
    ReferenceTopicInput::new(
        &snapshot,
        ids,
        &separated_counts(),
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("admitted input")
}

/// Fit only the synthetic six-document corpus through the public CPU f64 entry.
fn precision_origin_fit(
    input: &ReferenceTopicInput,
) -> (
    ReferenceTopicModelConfig,
    topic_measurement::FitOriginBoundModel,
) {
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2_000, 1e-5)
        .expect("config")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters");
    let model = fit_origin_bound_topic_model(input.clone(), config.clone())
        .expect("real converged CPU fit");
    assert!(model.model().objective.is_finite());
    (config, model)
}

/// Matching input, configuration, and actual fit retain identified finite draws.
#[test]
fn precision_origin_matching_public_fit_positive() {
    let input = precision_origin_input();
    let (config, model) = precision_origin_fit(&input);
    let ids = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let precision = model
        .build_joint_coordinate_precision(&input, &config, ids.clone())
        .expect("matching precision");
    assert_eq!(
        precision.document_ids(),
        (1_u128..=6).map(Uuid::from_u128).collect::<Vec<_>>()
    );
    assert_eq!(precision.topic_ids(), ids);
    assert_eq!(precision.coordinate_means().len(), 6);
    assert_eq!(precision.values().len(), 6);
    assert!(
        precision
            .values()
            .iter()
            .all(|r| r.len() == 6 && r.iter().all(|x| x.is_finite()))
    );
    let draws = precision
        .draw_joint_gaussian(11, 2)
        .expect("finite public draws");
    assert_eq!(draws.draws().len(), 2);
    assert!(
        draws
            .draws()
            .iter()
            .all(|r| r.len() == 6 && r.iter().all(|x| x.is_finite()))
    );
    println!(
        "MATCHING iterations={} objective={} precision00={} draw_set_id={}",
        model.model().iterations,
        model.model().objective,
        precision.values()[0][0],
        draws.draw_set_id()
    );
}

/// A fit-bound claim must not silently replace the fitted prior and network penalty.
#[test]
fn precision_origin_rejects_different_fitted_hyperparameters() {
    let input = precision_origin_input();
    let (config, model) = precision_origin_fit(&input);
    let ids = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let original = model
        .build_joint_coordinate_precision(&input, &config, ids.clone())
        .expect("matching precision");
    let changed = config
        .clone()
        .with_hyperparameters(4.0, 0.0, 0.01, 0.05, 0.2)
        .expect("independently valid but not fitted configuration");
    let result = model.build_joint_coordinate_precision(&input, &changed, ids);
    if let Ok(other) = &result {
        assert_eq!(other.coordinate_means(), original.coordinate_means());
        assert_ne!(other.values(), original.values());
        let a = original.draw_joint_gaussian(11, 2).expect("original draws");
        let b = other
            .draw_joint_gaussian(11, 2)
            .expect("mismatched draws still admitted");
        assert_ne!(a.draw_set_id(), b.draw_set_id());
        println!(
            "MISMATCH_CONFIG admitted=true original00={} other00={} original_draw={} other_draw={}",
            original.values()[0][0],
            other.values()[0][0],
            a.draw_set_id(),
            b.draw_set_id()
        );
    }
    assert!(
        result.is_err(),
        "fit-origin configuration mismatch must not be admitted as fit-bound precision"
    );
}

/// A fit-bound claim must not relabel unchanged fitted rows with another input order.
#[test]
fn precision_origin_rejects_different_fitted_document_order() {
    let input = precision_origin_input();
    let (config, model) = precision_origin_fit(&input);
    let (snapshot, mut ids, mut times, memberships, relations) = fixture();
    ids.reverse();
    times.reverse();
    let counts = SparseMatrix::from_csr(
        6,
        4,
        vec![0, 2, 4, 6, 8, 10, 12],
        vec![2, 3, 2, 3, 2, 3, 0, 1, 0, 1, 0, 1],
        vec![
            20.0, 80.0, 15.0, 85.0, 10.0, 90.0, 80.0, 20.0, 85.0, 15.0, 90.0, 10.0,
        ],
    )
    .expect("aligned reversed sparse rows");
    let reversed = ReferenceTopicInput::new(
        &snapshot,
        ids.clone(),
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("aligned row permutation is a valid independent input");
    let (_, reverse_fit) = precision_origin_fit(&reversed);
    let topics = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let positive = reverse_fit
        .build_joint_coordinate_precision(&reversed, &config, topics.clone())
        .expect("matching reversed fit");
    assert_eq!(positive.document_ids(), ids);
    let original = model
        .build_joint_coordinate_precision(&input, &config, topics.clone())
        .expect("original matching precision");
    let result = model.build_joint_coordinate_precision(&reversed, &config, topics);
    if let Ok(other) = &result {
        assert_eq!(other.document_ids(), ids);
        assert_eq!(other.coordinate_means(), original.coordinate_means());
        assert_ne!(other.coordinate_means(), positive.coordinate_means());
        let a = original.draw_joint_gaussian(11, 2).expect("original draws");
        let b = other
            .draw_joint_gaussian(11, 2)
            .expect("foreign-origin draws still admitted");
        assert_ne!(a.document_ids(), b.document_ids());
        assert_eq!(b.event_times(), times);
        let records = b.plausible_values();
        assert_eq!(records[0].document_id, ids[0]);
        assert_eq!(records[0].event_time, times[0]);
        assert_eq!(records[0].logistic_normal_coordinates, b.draws()[0][..1]);
        println!(
            "MISMATCH_ORDER admitted=true original_first={} other_first={} same_coordinate_means=true same_numerical_draws={}",
            original.document_ids()[0],
            other.document_ids()[0],
            a.draws() == b.draws()
        );
    }
    assert!(
        result.is_err(),
        "different originating document order must not relabel the original fit"
    );
}
