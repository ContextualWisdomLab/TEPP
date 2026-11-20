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

/// Parse a deterministic event-time fixture for a January day.
fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("event time")
}

/// Construct an observed forward relation with bounded same-day intervals.
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

/// Build a cutoff-eligible two-document corpus with shared membership and a transition.
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

/// Public likelihood and candidate scoring admit exact zeros beside a positive tiny mixture.
#[test]
fn zero_factor_subnormal_public_likelihood_and_candidate() {
    let input = two_document_input([1.0, 0.0, 0.0, 1.0]);
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2000, 1e-5).expect("config");
    let fitted = fit_reference_topic_model(&input, &config).expect("actual CPU fit");
    for tiny in [0.25, f64::from_bits(1_u64 << 44)] {
        for (theta, beta) in [
            (
                vec![1.0, 0.0],
                vec![vec![tiny, 1.0 - tiny], vec![tiny, 1.0 - tiny]],
            ),
            (vec![0.5, 0.5], vec![vec![tiny, 1.0 - tiny], vec![0.0, 1.0]]),
            (
                vec![0.0, 1.0],
                vec![vec![tiny, 1.0 - tiny], vec![tiny, 1.0 - tiny]],
            ),
            (
                vec![0.5, 0.5],
                vec![vec![-0.0, 1.0], vec![tiny, 1.0 - tiny]],
            ),
        ] {
            let mut model = fitted.clone();
            model.document_topic_proportions = vec![theta, vec![0.5, 0.5]];
            model.topic_term_probabilities = beta;
            let expected = model
                .document_topic_proportions
                .iter()
                .enumerate()
                .map(|(row, weights)| {
                    let probability = weights
                        .iter()
                        .enumerate()
                        .map(|(topic, weight)| weight * model.topic_term_probabilities[topic][row])
                        .sum::<f64>();
                    assert!(probability.is_finite() && probability > 0.0);
                    probability.ln()
                })
                .sum::<f64>();
            let actual = input
                .in_sample_log_likelihood(&model)
                .expect("finite positive mixtures");
            assert!((actual - expected).abs() <= 1e-12);
            let candidate = statistical_candidate_from_fit(&input, 2, &model)
                .expect("same public likelihood consumer");
            assert!(candidate.is_statistically_supported());
        }
    }
    let mut invalid = fitted;
    invalid.document_topic_proportions = vec![vec![1.0, 0.0]; 2];
    invalid.topic_term_probabilities = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
    assert!(input.in_sample_log_likelihood(&invalid).is_err());
    assert!(statistical_candidate_from_fit(&input, 2, &invalid).is_err());
}

/// Public likelihood rejects an overflowing raw mixture instead of rescuing its logarithm.
#[test]
fn nonfinite_mixture_public_likelihood_and_candidate_are_rejected() {
    let input = two_document_input([1.0, 0.0, 0.0, 1.0]);
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2000, 1e-5).expect("config");
    let fitted = fit_reference_topic_model(&input, &config).expect("actual CPU fit");
    for (theta, beta) in [
        (
            vec![f64::MAX, 0.0],
            vec![vec![f64::MAX, 1.0], vec![0.0, 1.0]],
        ),
        (
            vec![f64::MAX, f64::MAX],
            vec![vec![1.0, 1.0], vec![1.0, 1.0]],
        ),
    ] {
        let raw = theta
            .iter()
            .enumerate()
            .map(|(topic, weight)| weight * beta[topic][0])
            .sum::<f64>();
        assert!(raw.is_infinite() && raw.is_sign_positive());
        let mut model = fitted.clone();
        model.document_topic_proportions = vec![theta, vec![0.5, 0.5]];
        model.topic_term_probabilities = beta;
        assert_eq!(
            input.in_sample_log_likelihood(&model),
            Err(topic_measurement::TopicMeasurementError::NonFiniteEstimate)
        );
        assert!(statistical_candidate_from_fit(&input, 2, &model).is_err());
    }
}

/// Finite ordinary, subnormal and rounded-zero positive mixtures retain shared scoring.
#[test]
fn nonfinite_mixture_guard_preserves_finite_and_underflow_controls() {
    let input = two_document_input([1.0, 0.0, 0.0, 1.0]);
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2000, 1e-5).expect("config");
    let fitted = fit_reference_topic_model(&input, &config).expect("actual CPU fit");
    let tiny = 2.0_f64.powi(-600);
    let subnormal = f64::from_bits(1_u64 << 44);
    for (theta, beta, first_log, raw_is_zero) in [
        (
            vec![0.5, 0.5],
            vec![vec![0.5, 0.5], vec![0.5, 0.5]],
            0.5_f64.ln(),
            false,
        ),
        (
            vec![1.0, 0.0],
            vec![vec![subnormal, 1.0], vec![0.0, 1.0]],
            subnormal.ln(),
            false,
        ),
        (
            vec![tiny, 1.0],
            vec![vec![tiny, 1.0], vec![0.0, 1.0]],
            -1200.0 * std::f64::consts::LN_2,
            true,
        ),
    ] {
        let raw = theta
            .iter()
            .enumerate()
            .map(|(topic, weight)| weight * beta[topic][0])
            .sum::<f64>();
        assert!(raw.is_finite());
        assert_eq!(raw.to_bits() == 0.0_f64.to_bits(), raw_is_zero);
        let mut model = fitted.clone();
        model.document_topic_proportions = vec![theta, vec![0.5, 0.5]];
        model.topic_term_probabilities = beta;
        let second =
            0.5 * model.topic_term_probabilities[0][1] + 0.5 * model.topic_term_probabilities[1][1];
        let actual = input
            .in_sample_log_likelihood(&model)
            .expect("finite or recovered tiny mixture");
        assert!((actual - (first_log + second.ln())).abs() <= 1e-12);
        assert!(statistical_candidate_from_fit(&input, 2, &model).is_ok());
    }
}
