//! Fit-owned topic-basis and document-coordinate contracts for release consumers.

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
    FittedDocumentCoordinateSummary, FittedTopicBasisIdentity, ReferenceTopicFit, ReferenceTopicInput,
    ReferenceTopicModel, ReferenceTopicModelConfig, SparseMatrix, TopicMeasurementError,
    additive_log_ratio, fit_reference_topic_model,
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
                EventTime::parse_rfc3339(&format!("2026-07-{day:02}T12:00:00Z")).expect("end"),
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

fn fitted_input_and_config() -> (ReferenceTopicInput, ReferenceTopicModelConfig) {
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
        .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("reference config");
    (input, config)
}

fn fitted_model() -> ReferenceTopicModel {
    let (input, config) = fitted_input_and_config();
    fit_reference_topic_model(&input, &config).expect("converged reference fit")
}

#[test]
fn fitted_basis_identity_is_deterministic_and_topic_content_follows_a_permutation() {
    let model = fitted_model();
    let identity = FittedTopicBasisIdentity::from_model(&model, 4).expect("basis identity");
    let repeated = FittedTopicBasisIdentity::from_model(&model, 4).expect("repeated identity");

    assert_eq!(identity, repeated);
    assert_eq!(identity.vocabulary_size(), 4);
    assert_eq!(identity.topics().len(), 2);
    assert_eq!(identity.topics()[0].topic_index(), 0);
    assert_eq!(identity.topics()[1].topic_index(), 1);
    assert_eq!(identity.sha256().len(), 64);
    assert_eq!(identity.topics()[0].sha256().len(), 64);
    assert_eq!(identity.topics()[1].sha256().len(), 64);
    assert_ne!(identity.topics()[0].sha256(), identity.topics()[1].sha256());

    let mut permuted = model.clone();
    permuted.topic_term_probabilities.swap(0, 1);
    let permuted_identity =
        FittedTopicBasisIdentity::from_model(&permuted, 4).expect("permuted identity");

    assert_eq!(identity.topics()[0].sha256(), permuted_identity.topics()[1].sha256());
    assert_eq!(identity.topics()[1].sha256(), permuted_identity.topics()[0].sha256());
    assert_ne!(identity.sha256(), permuted_identity.sha256());
}

#[test]
fn malformed_or_ambiguous_fitted_topic_rows_fail_closed() {
    let model = fitted_model();

    assert_eq!(
        FittedTopicBasisIdentity::from_model(&model, 3),
        Err(TopicMeasurementError::InvalidModelInput)
    );

    let mut malformed_width = model.clone();
    malformed_width.topic_term_probabilities[0].pop();
    assert_eq!(
        FittedTopicBasisIdentity::from_model(&malformed_width, 4),
        Err(TopicMeasurementError::InvalidModelInput)
    );

    let mut non_finite = model.clone();
    non_finite.topic_term_probabilities[0][0] = f64::NAN;
    assert_eq!(
        FittedTopicBasisIdentity::from_model(&non_finite, 4),
        Err(TopicMeasurementError::InvalidModelInput)
    );

    let mut duplicate = model;
    duplicate.topic_term_probabilities[1] = duplicate.topic_term_probabilities[0].clone();
    assert_eq!(
        FittedTopicBasisIdentity::from_model(&duplicate, 4),
        Err(TopicMeasurementError::InvalidModelInput)
    );
}

#[test]
fn bound_fit_coordinates_pair_alr_location_with_diagonal_variance() {
    let (input, config) = fitted_input_and_config();
    let fit = ReferenceTopicFit::fit(&input, &config).expect("owner-issued fit");
    let summary =
        FittedDocumentCoordinateSummary::from_bound_fit(&fit).expect("coordinate summary");

    assert_eq!(summary.version(), "tepp.fitted_document_coordinate_summary.v1");
    assert_eq!(summary.topic_count(), 2);
    assert_eq!(summary.rows().len(), input.document_count());
    for (document_index, (row, document_id)) in summary
        .rows()
        .iter()
        .zip(fit.input().document_ids())
        .enumerate()
    {
        assert_eq!(row.document_id(), *document_id);
        assert_eq!(row.coordinates().len(), 1);
        let coordinate = &row.coordinates()[0];
        assert_eq!(coordinate.numerator_topic_index(), 0);
        assert_eq!(coordinate.reference_topic_index(), 1);
        let expected_location =
            additive_log_ratio(&fit.model().document_topic_proportions[document_index])
                .expect("ALR location")[0];
        assert_eq!(coordinate.location().to_bits(), expected_location.to_bits());
        assert_eq!(
            coordinate.variance().to_bits(),
            fit.model().document_coordinate_variances[document_index][0].to_bits()
        );
    }
}

#[test]
fn public_basis_identity_is_minted_from_owner_issued_fit() {
    let (input, config) = fitted_input_and_config();
    let fit = ReferenceTopicFit::fit(&input, &config).expect("owner-issued fit");

    let identity = FittedTopicBasisIdentity::from_bound_fit(&fit).expect("basis identity");
    let repeated = FittedTopicBasisIdentity::from_bound_fit(&fit).expect("repeated basis identity");

    assert_eq!(identity, repeated);
    assert_eq!(identity.vocabulary_size(), fit.input().vocabulary_size());
    assert_eq!(identity.topics().len(), fit.model().topic_term_probabilities.len());
}
