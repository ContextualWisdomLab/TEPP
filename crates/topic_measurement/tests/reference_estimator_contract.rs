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
    PosteriorApproximation, PrevalenceFeature, ReferenceTopicInput, ReferenceTopicModelConfig,
    SparseMatrix, TopicMeasurementError, fit_reference_topic_model,
};
use uuid::Uuid;
use validation_core::root_mean_square_error;

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

#[test]
fn direct_input_refuses_high_sparse_covariates_before_dense_expansion() {
    let (snapshot, ids, times, memberships, relations) = fixture();
    for columns in [3, usize::MAX] {
        let covariates =
            SparseMatrix::from_csr(6, columns, vec![0, 1, 1, 1, 1, 1, 1], vec![2], vec![1.0])
                .expect("small populated covariate with unused trailing columns");
        let input = ReferenceTopicInput::new(
            &snapshot,
            ids.clone(),
            &separated_counts(),
            &times,
            Some(&covariates),
            &memberships,
            &relations,
        )
        .expect("actual small feature expansion is admitted");
        assert_eq!(input.features().len(), 8);
        let config = ReferenceTopicModelConfig::new(2, vec![7], 4, 1e30).expect("config");
        assert!(fit_reference_topic_model(&input, &config).is_ok());
    }
    for column in [1_000_000, usize::MAX - 1] {
        let covariates = SparseMatrix::from_csr(
            6,
            column + 1,
            vec![0, 1, 1, 1, 1, 1, 1],
            vec![column],
            vec![1.0],
        )
        .expect("one sparse high-index covariate");
        assert!(matches!(
            ReferenceTopicInput::new(
                &snapshot,
                ids.clone(),
                &separated_counts(),
                &times,
                Some(&covariates),
                &memberships,
                &relations,
            ),
            Err(TopicMeasurementError::ResourceLimitExceeded)
        ));
    }
}

#[test]
fn direct_fit_refuses_large_sparse_vocabulary_before_seed_allocation() {
    let (snapshot, ids, times, memberships, relations) = fixture();
    let counts = SparseMatrix::from_csr(
        6,
        100_000,
        vec![0, 1, 2, 3, 4, 5, 6],
        vec![0; 6],
        vec![1.0; 6],
    )
    .expect("small sparse counts with large declared vocabulary");
    let input = ReferenceTopicInput::new(
        &snapshot,
        ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("small design");
    let config = ReferenceTopicModelConfig::new(20_000, vec![7], 2, 1e-5).expect("config");
    assert!(matches!(
        fit_reference_topic_model(&input, &config),
        Err(TopicMeasurementError::ResourceLimitExceeded)
    ));
}

#[test]
fn reference_input_requires_observed_not_proposed_transitions() {
    let (snapshot, ids, times, memberships, observed) = fixture();
    let counts = separated_counts();
    let mut inferred = RelationGraph::new();
    for edge in observed.edges() {
        inferred
            .insert(
                RelationEdge::new(
                    edge.kind(),
                    edge.source(),
                    edge.target(),
                    RelationEvidenceStatus::Inferred,
                    edge.source_event_time(),
                    edge.target_event_time(),
                )
                .expect("valid proposal"),
            )
            .expect("insert proposal");
    }
    let build = |graph: &RelationGraph| {
        ReferenceTopicInput::new(
            &snapshot,
            ids.clone(),
            &counts,
            &times,
            None,
            &memberships,
            graph,
        )
    };
    assert!(matches!(
        build(&inferred),
        Err(TopicMeasurementError::InvalidModelInput)
    ));
    assert!(build(&observed).is_ok());
    let mut mixed = observed.clone();
    for edge in inferred.edges() {
        mixed.insert(*edge).expect("distinct status");
    }
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2_000, 1e-5).expect("config");
    let baseline = fit_reference_topic_model(&build(&observed).expect("observed"), &config)
        .expect("observed fit");
    let candidate =
        fit_reference_topic_model(&build(&mixed).expect("mixed"), &config).expect("mixed fit");
    assert_eq!(candidate, baseline);
}

#[test]
fn separated_topics_recover_and_emit_predecessor_successor_counts() {
    let (snapshot, document_ids, times, memberships, relations) = fixture();
    let counts = separated_counts();
    let input = ReferenceTopicInput::new(
        &snapshot,
        document_ids.clone(),
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("input");
    assert_eq!(input.document_count(), 6);
    assert_eq!(input.vocabulary_size(), 4);
    assert!(matches!(input.features()[0], PrevalenceFeature::Intercept));
    assert!(matches!(input.features()[1], PrevalenceFeature::EventTime));

    let config = ReferenceTopicModelConfig::new(2, vec![7, 11, 19], 2_000, 1e-5)
        .expect("configuration")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters");
    let result = fit_reference_topic_model(&input, &config).expect("converged fit");
    assert_eq!(
        result.posterior_approximation(),
        PosteriorApproximation::DiagonalLaplace
    );
    assert_eq!(
        result.joint_coordinate_precision(),
        Err(TopicMeasurementError::JointPosteriorUnavailable)
    );
    let topic_ids = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let precision = input
        .build_joint_coordinate_precision(&result, &config, topic_ids.clone())
        .expect("joint precision");
    assert_eq!(
        precision.approximation(),
        PosteriorApproximation::JointGaussNewtonLaplace
    );
    assert_eq!(precision.document_ids(), document_ids);
    assert_eq!(precision.topic_ids(), topic_ids);
    assert_eq!(precision.values().len(), 6);
    assert_eq!(precision.coordinate_means().len(), 6);
    assert!(precision.values().iter().enumerate().all(|(row, values)| {
        values.iter().enumerate().all(|(column, value)| {
            value.is_finite() && (value - precision.values()[column][row]).abs() < f64::EPSILON
        })
    }));
    assert!(precision.values()[0][1] < 0.0);
    assert!(result.objective.is_finite());
    assert!(result.iterations <= 2_000);
    assert_eq!(result.connected_post_count, 6);
    assert_eq!(result.lineage_count, 2);
    assert_eq!(result.sequence_edges.len(), 4);
    assert!(
        result
            .sequence_edges
            .iter()
            .all(|edge| edge.association_strength > 0.5)
    );
    assert!(
        result
            .document_coordinate_variances
            .iter()
            .flatten()
            .all(|value| *value > 0.0)
    );

    let recovered: Vec<f64> = result
        .document_topic_proportions
        .iter()
        .map(|row| row[0])
        .collect();
    let truth_a = [0.9, 0.85, 0.8, 0.1, 0.15, 0.2];
    let truth_b = [0.1, 0.15, 0.2, 0.9, 0.85, 0.8];
    let rmse = root_mean_square_error(&truth_a, &recovered)
        .expect("rmse")
        .min(root_mean_square_error(&truth_b, &recovered).expect("label-swapped rmse"));
    assert!(rmse < 0.25, "known-truth topic RMSE {rmse} exceeded 0.25");

    let log_likelihood = input
        .in_sample_log_likelihood(&result)
        .expect("in-sample mixture log-likelihood");
    assert!(log_likelihood.is_finite());
    let tokens = input.token_count().expect("token count");
    assert!((tokens - 600.0).abs() < f64::EPSILON);
}

#[test]
fn invalid_configuration_and_topic_dimension_fail_closed() {
    let (snapshot, document_ids, times, memberships, relations) = fixture();
    let counts = SparseMatrix::from_csr(
        6,
        2,
        vec![0, 1, 2, 3, 4, 5, 6],
        vec![0, 0, 0, 1, 1, 1],
        vec![1.0; 6],
    )
    .expect("counts");
    let input = ReferenceTopicInput::new(
        &snapshot,
        document_ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("input");
    assert!(ReferenceTopicModelConfig::new(1, vec![1], 10, 1e-6).is_err());
    let too_many = ReferenceTopicModelConfig::new(3, vec![1], 10, 1e-6).expect("config");
    assert!(fit_reference_topic_model(&input, &too_many).is_err());

    for (topics, seeds, iterations, tolerance) in [
        (2, vec![], 10, 1e-6),
        (2, vec![1], 1, 1e-6),
        (2, vec![1], 10, f64::NAN),
        (2, vec![1], 10, 0.0),
    ] {
        assert!(ReferenceTopicModelConfig::new(topics, seeds, iterations, tolerance).is_err());
    }
    let base = ReferenceTopicModelConfig::new(2, vec![1], 10, 1e-6).expect("base");
    for values in [
        (f64::NAN, 0.5, 0.01, 0.05, 0.2),
        (0.0, 0.5, 0.01, 0.05, 0.2),
        (1.0, f64::NAN, 0.01, 0.05, 0.2),
        (1.0, -1.0, 0.01, 0.05, 0.2),
        (1.0, 0.5, f64::NAN, 0.05, 0.2),
        (1.0, 0.5, -1.0, 0.05, 0.2),
        (1.0, 0.5, 0.01, f64::NAN, 0.2),
        (1.0, 0.5, 0.01, 0.0, 0.2),
        (1.0, 0.5, 0.01, 0.05, f64::NAN),
        (1.0, 0.5, 0.01, 0.05, 0.0),
    ] {
        assert!(
            base.clone()
                .with_hyperparameters(values.0, values.1, values.2, values.3, values.4)
                .is_err()
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn invalid_structural_inputs_and_nonconvergence_fail_closed() {
    let (snapshot, document_ids, times, memberships, relations) = fixture();
    let counts = separated_counts();
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids[..1].to_vec(),
            &counts,
            &times,
            None,
            &memberships,
            &relations,
        )
        .is_err()
    );
    let wrong_rows =
        SparseMatrix::from_csr(2, 2, vec![0, 1, 2], vec![0, 1], vec![1.0; 2]).expect("wrong rows");
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids.clone(),
            &wrong_rows,
            &times,
            None,
            &memberships,
            &relations,
        )
        .is_err()
    );
    let one_column =
        SparseMatrix::from_csr(6, 1, vec![0, 1, 2, 3, 4, 5, 6], vec![0; 6], vec![1.0; 6])
            .expect("one column");
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids.clone(),
            &one_column,
            &times,
            None,
            &memberships,
            &relations,
        )
        .is_err()
    );
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids.clone(),
            &counts,
            &times[..5],
            None,
            &memberships,
            &relations,
        )
        .is_err()
    );
    let mut missing_snapshot = CorpusSnapshot::new();
    missing_snapshot
        .insert_if_eligible(
            CorpusDocument::new(
                document_ids[0],
                AvailableTime::parse_rfc3339("2026-01-10T00:00:00Z").expect("available"),
            ),
            &KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff"),
        )
        .expect("eligible");
    assert!(
        ReferenceTopicInput::new(
            &missing_snapshot,
            document_ids.clone(),
            &counts,
            &times,
            None,
            &memberships,
            &relations,
        )
        .is_err()
    );
    let mut outside_relations = RelationGraph::new();
    outside_relations
        .insert(relation(Uuid::from_u128(999), document_ids[0], 1, 2))
        .expect("outside source");
    outside_relations
        .insert(relation(document_ids[0], Uuid::from_u128(998), 2, 3))
        .expect("outside target");
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids.clone(),
            &counts,
            &times,
            None,
            &memberships,
            &outside_relations,
        )
        .is_err()
    );

    let empty_row = SparseMatrix::from_csr(
        6,
        2,
        vec![0, 0, 1, 2, 3, 4, 5],
        vec![0, 0, 1, 1, 1],
        vec![1.0; 5],
    )
    .expect("empty row");
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids.clone(),
            &empty_row,
            &times,
            None,
            &memberships,
            &relations,
        )
        .is_err()
    );
    let zero_row = SparseMatrix::from_csr(
        6,
        2,
        vec![0, 1, 2, 3, 4, 5, 6],
        vec![0, 0, 0, 1, 1, 1],
        vec![0.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    )
    .expect("zero row");
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids.clone(),
            &zero_row,
            &times,
            None,
            &memberships,
            &relations,
        )
        .is_err()
    );

    let mut duplicate_ids = document_ids.clone();
    duplicate_ids[1] = duplicate_ids[0];
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            duplicate_ids,
            &counts,
            &times,
            None,
            &memberships,
            &relations,
        )
        .is_err()
    );

    let negative = SparseMatrix::from_csr(
        6,
        2,
        vec![0, 1, 2, 3, 4, 5, 6],
        vec![0, 0, 0, 1, 1, 1],
        vec![-1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    )
    .expect("finite sparse values");
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids.clone(),
            &negative,
            &times,
            None,
            &memberships,
            &relations,
        )
        .is_err()
    );

    let covariate = SparseMatrix::from_csc(
        6,
        1,
        vec![0, 6],
        vec![0, 1, 2, 3, 4, 5],
        vec![-1.0, -0.5, 0.0, 0.0, 0.5, 1.0],
    )
    .expect("covariate");
    let with_covariate = ReferenceTopicInput::new(
        &snapshot,
        document_ids.clone(),
        &counts,
        &times,
        Some(&covariate),
        &memberships,
        &relations,
    )
    .expect("covariate input");
    assert!(matches!(
        with_covariate.features()[2],
        PrevalenceFeature::Covariate(0)
    ));
    let wrong_rows =
        SparseMatrix::from_csr(2, 1, vec![0, 0, 0], vec![], vec![]).expect("covariate");
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids.clone(),
            &counts,
            &times,
            Some(&wrong_rows),
            &memberships,
            &relations,
        )
        .is_err()
    );
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids.clone(),
            &counts,
            &times,
            None,
            &MembershipNetwork::new(),
            &relations,
        )
        .is_err()
    );
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids.clone(),
            &counts,
            &times,
            None,
            &memberships,
            &RelationGraph::new(),
        )
        .is_err()
    );
    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids.clone(),
            &counts,
            &[event_time(1); 6],
            None,
            &memberships,
            &relations,
        )
        .is_err()
    );

    let input = ReferenceTopicInput::new(
        &snapshot,
        document_ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("valid input");
    let exhausted = ReferenceTopicModelConfig::new(2, vec![1], 2, 1e-12).expect("exhausted");
    assert!(fit_reference_topic_model(&input, &exhausted).is_err());
    let quick = ReferenceTopicModelConfig::new(2, vec![1], 10, f64::MAX).expect("quick");
    assert!(fit_reference_topic_model(&input, &quick).is_ok());
    let unstable = ReferenceTopicModelConfig::new(2, vec![1], 10, 1e-6)
        .expect("unstable")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, f64::MAX)
        .expect("finite hyperparameters");
    assert!(fit_reference_topic_model(&input, &unstable).is_err());
}

/// Build equivalent CSR/CSC fixtures with an unused fifth term and optional zero.
fn normalization_underflow_input(explicit_zero: bool, csc: bool) -> ReferenceTopicInput {
    let (snapshot, ids, times, memberships, relations) = fixture();
    let matrix = if csc {
        SparseMatrix::from_csc(
            6,
            5,
            if explicit_zero {
                vec![0, 3, 6, 9, 12, 13]
            } else {
                vec![0, 3, 6, 9, 12, 12]
            },
            if explicit_zero {
                vec![0, 1, 2, 0, 1, 2, 3, 4, 5, 3, 4, 5, 0]
            } else {
                vec![0, 1, 2, 0, 1, 2, 3, 4, 5, 3, 4, 5]
            },
            if explicit_zero {
                vec![
                    90.0, 85.0, 80.0, 10.0, 15.0, 20.0, 10.0, 15.0, 20.0, 90.0, 85.0, 80.0, 0.0,
                ]
            } else {
                vec![
                    90.0, 85.0, 80.0, 10.0, 15.0, 20.0, 10.0, 15.0, 20.0, 90.0, 85.0, 80.0,
                ]
            },
        )
    } else {
        SparseMatrix::from_csr(
            6,
            5,
            if explicit_zero {
                vec![0, 3, 5, 7, 9, 11, 13]
            } else {
                vec![0, 2, 4, 6, 8, 10, 12]
            },
            if explicit_zero {
                vec![0, 1, 4, 0, 1, 0, 1, 2, 3, 2, 3, 2, 3]
            } else {
                vec![0, 1, 0, 1, 0, 1, 2, 3, 2, 3, 2, 3]
            },
            if explicit_zero {
                vec![
                    90.0, 10.0, 0.0, 85.0, 15.0, 80.0, 20.0, 10.0, 90.0, 15.0, 85.0, 20.0, 80.0,
                ]
            } else {
                vec![
                    90.0, 10.0, 85.0, 15.0, 80.0, 20.0, 10.0, 90.0, 15.0, 85.0, 20.0, 80.0,
                ]
            },
        )
    }
    .expect("finite nonnegative sparse matrix");
    ReferenceTopicInput::new(
        &snapshot,
        ids,
        &matrix,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("legal input with 600 positive total tokens")
}
/// Keep the fit configuration fixed while varying only positive topic smoothing.
fn smoothing_config(smoothing: f64) -> ReferenceTopicModelConfig {
    assert!(smoothing.is_finite() && smoothing > 0.0);
    ReferenceTopicModelConfig::new(2, vec![7], 2000, 1e-5)
        .unwrap()
        .with_hyperparameters(1.0, 0.25, 0.01, smoothing, 0.2)
        .expect("documented finite positive smoothing admitted")
}
/// A representable tiny floor gives identical positive fits for all sparse forms.
#[test]
fn normal_smoothing_preserves_zero_entry_equivalence() {
    let c = smoothing_config(f64::MIN_POSITIVE);
    let baseline =
        fit_reference_topic_model(&normalization_underflow_input(false, false), &c).unwrap();
    for csc in [false, true] {
        for explicit in [false, true] {
            let fitted =
                fit_reference_topic_model(&normalization_underflow_input(explicit, csc), &c)
                    .unwrap();
            assert_eq!(fitted, baseline);
            assert!(
                fitted
                    .topic_term_probabilities
                    .iter()
                    .flatten()
                    .all(|x| x.is_finite() && *x > 0.0)
            );
        }
    }
}
/// An unrepresentable beta floor fails for both orientations and zero spellings.
#[test]
fn normalization_underflow_is_rejected_before_returning_fit() {
    let c = smoothing_config(f64::from_bits(1));
    let mut failures = Vec::new();
    for csc in [false, true] {
        for explicit in [false, true] {
            let result =
                fit_reference_topic_model(&normalization_underflow_input(explicit, csc), &c);
            if result != Err(TopicMeasurementError::NonFiniteEstimate) {
                failures.push((csc, explicit));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "positive smoothing must not return an underflowed zero beta: {failures:?}"
    );
}

/// Reject mapped backward document times despite forward graph intervals.
#[test]
fn reference_input_rejects_reverse_mapped_document_event_times() {
    let (snapshot, ids, times, memberships, relations) = fixture();
    let mut reversed = times.clone();
    reversed.reverse();
    let mut one_reverse_pair = times;
    one_reverse_pair.swap(0, 1);
    for supplied_times in [reversed, one_reverse_pair] {
        assert!(matches!(
            ReferenceTopicInput::new(
                &snapshot,
                ids.clone(),
                &separated_counts(),
                &supplied_times,
                None,
                &memberships,
                &relations,
            ),
            Err(TopicMeasurementError::InvalidModelInput)
        ));
    }
}

/// Admit equal pair times while retaining variation elsewhere in the corpus.
#[test]
fn reference_input_admits_equal_mapped_pair_times_with_other_time_variation() {
    let (snapshot, ids, mut times, memberships, relations) = fixture();
    times[1] = times[0];
    let input = ReferenceTopicInput::new(
        &snapshot,
        ids,
        &separated_counts(),
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("equal pair times do not move backward");
    let config = ReferenceTopicModelConfig::new(2, vec![7], 4, 1e30).expect("config");
    let model = fit_reference_topic_model(&input, &config).expect("finite bounded fit");
    assert!(model.objective.is_finite());
}

/// Preserve forward endpoint identity when documents and matrix rows are permuted.
#[test]
fn reference_input_admits_permuted_document_rows_with_forward_identity_times() {
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
    .expect("permuted matrix");
    let input = ReferenceTopicInput::new(
        &snapshot,
        ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("row order does not define transition order");
    assert_eq!(input.document_count(), 6);
}
