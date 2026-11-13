//! Isolated guard characterization; internal mutations are not public admission proofs.

use crate::{
    ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix, fit_origin_bound_topic_model,
};
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
) -> (ReferenceTopicModelConfig, crate::FitOriginBoundModel) {
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2_000, 1e-5)
        .expect("config")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters");
    let model = fit_origin_bound_topic_model(input.clone(), config.clone())
        .expect("real converged CPU fit");
    assert!(model.model().objective.is_finite());
    (config, model)
}

/// Every input guard independently refuses one changed private numerical field.
/// These synthetic private mutations characterize the guard, not public reachability.
#[test]
fn origin_supplement_each_input_field_is_checked() {
    let input = precision_origin_input();
    let (config, bound) = precision_origin_fit(&input);
    let topics = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let mut cases = Vec::new();
    let mut changed = input.clone();
    changed.document_ids[0] = Uuid::from_u128(900);
    cases.push(("document_ids", changed));
    let mut changed = input.clone();
    changed.event_times[0] = event_time(2);
    cases.push(("event_times", changed));
    let mut changed = input.clone();
    changed.term_rows[0][0].1 += 1.0;
    cases.push(("term_rows", changed));
    let mut changed = input.clone();
    changed.vocabulary_size += 1;
    cases.push(("vocabulary_size", changed));
    let mut changed = input.clone();
    changed.design[0][0] += 0.25;
    cases.push(("design", changed));
    let mut changed = input.clone();
    changed.features.swap(0, 1);
    cases.push(("features", changed));
    let mut changed = input.clone();
    changed.transition_pairs.pop().expect("observed pair");
    cases.push(("transition_pairs", changed));
    for (name, changed) in cases {
        let equal_fields = [
            input.document_ids == changed.document_ids,
            input.event_times == changed.event_times,
            input.term_rows == changed.term_rows,
            input.vocabulary_size == changed.vocabulary_size,
            input.design == changed.design,
            input.features == changed.features,
            input.transition_pairs == changed.transition_pairs,
        ];
        assert_eq!(
            equal_fields.iter().filter(|same| !**same).count(),
            1,
            "single input axis {name}"
        );
        assert!(
            matches!(
                bound.build_joint_coordinate_precision(&changed, &config, topics.clone()),
                Err(crate::TopicMeasurementError::InvalidModelInput)
            ),
            "unbound input axis {name}"
        );
        println!("INPUT_AXIS {name} rejected_typed=true");
    }
    assert!(
        bound
            .build_joint_coordinate_precision(&input, &config, topics)
            .is_ok()
    );
}

/// Every current config field is compared independently, including execution controls.
#[test]
fn origin_supplement_each_config_field_is_checked() {
    let input = precision_origin_input();
    let (config, bound) = precision_origin_fit(&input);
    let topics = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let mut cases = Vec::new();
    let mut changed = config.clone();
    changed.topic_count = 3;
    cases.push(("topic_count", changed));
    let mut changed = config.clone();
    changed.seeds = vec![8];
    cases.push(("seeds", changed));
    let mut changed = config.clone();
    changed.maximum_iterations += 1;
    cases.push(("maximum_iterations", changed));
    let mut changed = config.clone();
    changed.tolerance *= 2.0;
    cases.push(("tolerance", changed));
    let mut changed = config.clone();
    changed.prior_variance = 4.0;
    cases.push(("prior_variance", changed));
    let mut changed = config.clone();
    changed.relation_strength = 0.0;
    cases.push(("relation_strength", changed));
    let mut changed = config.clone();
    changed.ridge = 0.02;
    cases.push(("ridge", changed));
    let mut changed = config.clone();
    changed.topic_smoothing = 0.1;
    cases.push(("topic_smoothing", changed));
    let mut changed = config.clone();
    changed.step_size = 0.1;
    cases.push(("step_size", changed));
    for (name, changed) in cases {
        changed
            .validate()
            .expect("independently valid configuration");
        let equal_fields = [
            config.topic_count == changed.topic_count,
            config.seeds == changed.seeds,
            config.maximum_iterations == changed.maximum_iterations,
            config.tolerance.to_bits() == changed.tolerance.to_bits(),
            config.prior_variance.to_bits() == changed.prior_variance.to_bits(),
            config.relation_strength.to_bits() == changed.relation_strength.to_bits(),
            config.ridge.to_bits() == changed.ridge.to_bits(),
            config.topic_smoothing.to_bits() == changed.topic_smoothing.to_bits(),
            config.step_size.to_bits() == changed.step_size.to_bits(),
        ];
        assert_eq!(
            equal_fields.iter().filter(|same| !**same).count(),
            1,
            "single config axis {name}"
        );
        assert!(
            matches!(
                bound.build_joint_coordinate_precision(&input, &changed, topics.clone()),
                Err(crate::TopicMeasurementError::InvalidModelInput)
            ),
            "unbound config axis {name}"
        );
        println!("CONFIG_AXIS {name} rejected_typed=true");
    }
    assert!(
        bound
            .build_joint_coordinate_precision(&input, &config, topics)
            .is_ok()
    );
}

/// Equivalent public reconstruction, clone immutability and successful labels survive.
#[test]
fn origin_supplement_successful_draw_and_plausible_labels() {
    let input = precision_origin_input();
    let reconstructed = precision_origin_input();
    let (config, bound) = precision_origin_fit(&input);
    let topics = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let reference = bound
        .build_joint_coordinate_precision(&input, &config, topics.clone())
        .expect("matching precision");
    let equivalent = bound
        .build_joint_coordinate_precision(&reconstructed, &config.clone(), topics.clone())
        .expect("equal value public reconstruction");
    assert_eq!(reference, equivalent);
    let cloned = bound.clone();
    let mut detached = cloned.model().clone();
    detached.document_topic_proportions[0][0] = 0.0;
    assert_ne!(
        detached.document_topic_proportions,
        bound.model().document_topic_proportions
    );
    assert_eq!(cloned.model(), bound.model());
    let draws = cloned
        .build_joint_coordinate_precision(&reconstructed, &config, topics.clone())
        .expect("clone precision")
        .draw_joint_gaussian(11, 2)
        .expect("matching draws");
    let expected_ids: Vec<_> = (1_u128..=6).map(Uuid::from_u128).collect();
    let expected_times: Vec<_> = (1_u8..=6).map(event_time).collect();
    assert_eq!(draws.document_ids(), expected_ids);
    assert_eq!(draws.event_times(), expected_times);
    assert_eq!(draws.topic_ids(), topics);
    assert_eq!(draws.seed(), 11);
    assert_eq!(draws.draws().len(), 2);
    for row in draws.draws() {
        assert_eq!(row.len(), 6);
        assert!(row.iter().all(|x| x.is_finite()));
    }
    let records = draws.plausible_values();
    assert_eq!(records.len(), 12);
    for (document, (id, time)) in expected_ids.iter().zip(&expected_times).enumerate() {
        for draw in 0..2 {
            let record = &records[document * 2 + draw];
            assert_eq!(record.document_id, *id);
            assert_eq!(record.event_time, *time);
            assert_eq!(
                record.draw_index,
                u64::try_from(draw).expect("bounded index")
            );
            assert_eq!(record.logistic_normal_coordinates.len(), 1);
            assert_eq!(
                record.logistic_normal_coordinates[0].to_bits(),
                draws.draws()[draw][document].to_bits()
            );
        }
    }
    println!("SUCCESS_LABELS documents=6 draws=2 records=12 clone_immutable=true");
}
