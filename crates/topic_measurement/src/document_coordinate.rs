//! Fit-owned document ALR locations paired with diagonal-Laplace variance.
//!
//! The reference estimator retains document topic proportions and one diagonal
//! variance per additive log-ratio coordinate. This module keeps those two
//! numerical quantities together with the exact fitted topic-basis identity so
//! downstream code cannot publish curvature while silently dropping either the
//! fitted location it approximates or the numerical basis that gives local topic
//! indexes meaning. The public projection path accepts only an owner-issued
//! [`ReferenceTopicFit`], preventing a caller from pairing a model from one fit
//! with document coordinates from another. The view remains fit-local numerical
//! evidence; it does not authenticate source/vocabulary provenance or represent
//! joint covariance.

use uuid::Uuid;

use crate::{
    FittedTopicBasisIdentity, ReferenceTopicFit, ReferenceTopicInput, ReferenceTopicModel,
    TopicMeasurementError, additive_log_ratio,
};

/// Version of the fit-local document-coordinate summary contract.
pub const FITTED_DOCUMENT_COORDINATE_SUMMARY_VERSION: &str =
    "tepp.fitted_document_coordinate_summary.v1";

/// One fitted ALR location paired with its retained diagonal-Laplace variance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FittedDocumentCoordinate {
    numerator_topic_index: usize,
    reference_topic_index: usize,
    location: f64,
    variance: f64,
}

impl FittedDocumentCoordinate {
    /// Return the numerator topic index in the fitted local topic order.
    #[must_use]
    pub const fn numerator_topic_index(&self) -> usize {
        self.numerator_topic_index
    }

    /// Return the reference topic index used as the ALR denominator.
    #[must_use]
    pub const fn reference_topic_index(&self) -> usize {
        self.reference_topic_index
    }

    /// Return the fitted MAP ALR location for this coordinate.
    #[must_use]
    pub const fn location(&self) -> f64 {
        self.location
    }

    /// Return the retained positive diagonal-Laplace variance.
    #[must_use]
    pub const fn variance(&self) -> f64 {
        self.variance
    }
}

/// All fitted ALR coordinates for one admitted modeled document.
#[derive(Clone, Debug, PartialEq)]
pub struct FittedDocumentCoordinateRow {
    document_id: Uuid,
    coordinates: Vec<FittedDocumentCoordinate>,
}

impl FittedDocumentCoordinateRow {
    /// Return the admitted modeled document identity.
    #[must_use]
    pub const fn document_id(&self) -> Uuid {
        self.document_id
    }

    /// Return fitted coordinates in numerator-topic order.
    #[must_use]
    pub fn coordinates(&self) -> &[FittedDocumentCoordinate] {
        &self.coordinates
    }
}

/// Fit-local document ALR locations and diagonal variances in input row order.
///
/// This summary is deliberately narrower than an externally released posterior
/// artifact. It binds each retained diagonal variance to its fitted ALR location,
/// document row, and exact fitted topic basis inside one owner-issued fit
/// aggregate, but it does not supply joint covariance, plausible values,
/// calibration evidence, semantic topic labels, or Evidence-owned
/// source/vocabulary provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct FittedDocumentCoordinateSummary {
    topic_count: usize,
    topic_basis_identity: FittedTopicBasisIdentity,
    rows: Vec<FittedDocumentCoordinateRow>,
}

impl FittedDocumentCoordinateSummary {
    /// Build a coordinate summary from an owner-issued nominal fit aggregate.
    ///
    /// [`ReferenceTopicFit`] can only be minted by executing the reference
    /// estimator over the input/configuration retained inside that aggregate,
    /// so this public path does not accept a detached model/input pair.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidModelInput`] if retained fitted
    /// dimensions or numerical state violate the coordinate contract.
    pub fn from_bound_fit(fit: &ReferenceTopicFit) -> Result<Self, TopicMeasurementError> {
        Self::from_pair(fit.input(), fit.model())
    }

    fn from_pair(
        input: &ReferenceTopicInput,
        model: &ReferenceTopicModel,
    ) -> Result<Self, TopicMeasurementError> {
        let topic_count = model.topic_term_probabilities.len();
        if topic_count < 2 {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        let topic_basis_identity =
            FittedTopicBasisIdentity::from_model(model, input.vocabulary_size())?;
        if model.document_topic_proportions.len() != input.document_count() {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        if model.document_coordinate_variances.len() != input.document_count() {
            return Err(TopicMeasurementError::InvalidModelInput);
        }

        let coordinate_count = topic_count - 1;
        let reference_topic_index = coordinate_count;
        let mut rows = Vec::with_capacity(input.document_count());
        for ((document_id, proportions), variances) in input
            .document_ids()
            .iter()
            .copied()
            .zip(&model.document_topic_proportions)
            .zip(&model.document_coordinate_variances)
        {
            if proportions.len() != topic_count {
                return Err(TopicMeasurementError::InvalidModelInput);
            }
            if variances.len() != coordinate_count {
                return Err(TopicMeasurementError::InvalidModelInput);
            }
            let locations = additive_log_ratio(proportions)
                .map_err(|_| TopicMeasurementError::InvalidModelInput)?;
            let mut coordinates = Vec::with_capacity(coordinate_count);
            for (numerator_topic_index, (location, variance)) in
                locations.into_iter().zip(variances.iter().copied()).enumerate()
            {
                if !variance.is_finite() || variance <= 0.0 {
                    return Err(TopicMeasurementError::InvalidModelInput);
                }
                coordinates.push(FittedDocumentCoordinate {
                    numerator_topic_index,
                    reference_topic_index,
                    location,
                    variance,
                });
            }
            rows.push(FittedDocumentCoordinateRow {
                document_id,
                coordinates,
            });
        }

        Ok(Self {
            topic_count,
            topic_basis_identity,
            rows,
        })
    }

    /// Return this fit-local contract's exact version.
    #[must_use]
    pub const fn version(&self) -> &'static str {
        FITTED_DOCUMENT_COORDINATE_SUMMARY_VERSION
    }

    /// Return the fitted global topic count.
    #[must_use]
    pub const fn topic_count(&self) -> usize {
        self.topic_count
    }

    /// Return the exact fitted topic basis that gives coordinate indexes meaning.
    #[must_use]
    pub const fn topic_basis_identity(&self) -> &FittedTopicBasisIdentity {
        &self.topic_basis_identity
    }

    /// Return document rows in the retained estimator input order.
    #[must_use]
    pub fn rows(&self) -> &[FittedDocumentCoordinateRow] {
        &self.rows
    }
}

#[cfg(test)]
mod tests {
    use corpus_split::{CorpusDocument, CorpusSnapshot};
    use membership_core::{
        GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole,
        MembershipWeight,
    };
    use relation_graph::{
        RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
    };
    use temporal_core::{
        AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
        TemporalPrecision,
    };
    use uuid::Uuid;

    use super::FittedDocumentCoordinateSummary;
    use crate::{
        ReferenceTopicInput, ReferenceTopicModel, ReferenceTopicModelConfig, SparseMatrix,
        TopicMeasurementError, fit_reference_topic_model,
    };

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

    fn fitted_input_and_model() -> (ReferenceTopicInput, ReferenceTopicModel) {
        let ids: Vec<_> = (1_u128..=4).map(Uuid::from_u128).collect();
        let times: Vec<_> = (1_u8..=4).map(event_time).collect();
        let available =
            AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available");
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
        for (source, target, source_day, target_day) in
            [(0, 1, 1, 2), (1, 2, 2, 3), (2, 3, 3, 4)]
        {
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
        let model = fit_reference_topic_model(&input, &config).expect("converged reference fit");
        (input, model)
    }

    #[test]
    fn private_pair_validation_preserves_location_and_refuses_malformed_state() {
        let (input, model) = fitted_input_and_model();
        let baseline = FittedDocumentCoordinateSummary::from_pair(&input, &model)
            .expect("coordinate summary");

        let mut shifted = model.clone();
        shifted.document_topic_proportions[0] = vec![0.8, 0.2];
        let shifted_summary = FittedDocumentCoordinateSummary::from_pair(&input, &shifted)
            .expect("shifted summary");
        assert_ne!(
            baseline.rows()[0].coordinates()[0].location().to_bits(),
            shifted_summary.rows()[0].coordinates()[0].location().to_bits(),
            "equal diagonal variance must not erase a changed fitted ALR location"
        );
        assert_eq!(
            baseline.rows()[0].coordinates()[0].variance().to_bits(),
            shifted_summary.rows()[0].coordinates()[0].variance().to_bits()
        );

        let mut one_topic = model.clone();
        one_topic.topic_term_probabilities.truncate(1);
        assert_eq!(
            FittedDocumentCoordinateSummary::from_pair(&input, &one_topic),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut malformed_basis = model.clone();
        malformed_basis.topic_term_probabilities[0].pop();
        assert_eq!(
            FittedDocumentCoordinateSummary::from_pair(&input, &malformed_basis),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut missing_document = model.clone();
        missing_document.document_topic_proportions.pop();
        assert_eq!(
            FittedDocumentCoordinateSummary::from_pair(&input, &missing_document),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut missing_variance_document = model.clone();
        missing_variance_document.document_coordinate_variances.pop();
        assert_eq!(
            FittedDocumentCoordinateSummary::from_pair(&input, &missing_variance_document),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut malformed_topic_width = model.clone();
        malformed_topic_width.document_topic_proportions[0].pop();
        assert_eq!(
            FittedDocumentCoordinateSummary::from_pair(&input, &malformed_topic_width),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut malformed_variance_width = model.clone();
        malformed_variance_width.document_coordinate_variances[0].clear();
        assert_eq!(
            FittedDocumentCoordinateSummary::from_pair(&input, &malformed_variance_width),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut non_finite_variance = model.clone();
        non_finite_variance.document_coordinate_variances[0][0] = f64::NAN;
        assert_eq!(
            FittedDocumentCoordinateSummary::from_pair(&input, &non_finite_variance),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut non_positive_variance = model.clone();
        non_positive_variance.document_coordinate_variances[0][0] = 0.0;
        assert_eq!(
            FittedDocumentCoordinateSummary::from_pair(&input, &non_positive_variance),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut invalid_location = model;
        invalid_location.document_topic_proportions[0] = vec![1.0, 0.0];
        assert_eq!(
            FittedDocumentCoordinateSummary::from_pair(&input, &invalid_location),
            Err(TopicMeasurementError::InvalidModelInput)
        );
    }
}
