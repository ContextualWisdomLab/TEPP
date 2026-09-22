//! Fit-owned document ALR locations paired with diagonal-Laplace variance.
//!
//! The reference estimator retains document topic proportions and one diagonal
//! variance per additive log-ratio coordinate. This module keeps those two
//! numerical quantities in one owner-issued coordinate view so downstream code
//! cannot publish curvature while silently dropping the fitted location it
//! approximates. The view is fit-local numerical evidence only; it does not
//! authenticate source/vocabulary provenance or represent joint covariance.

use uuid::Uuid;

use crate::{
    FittedTopicBasisIdentity, ReferenceTopicInput, ReferenceTopicModel, TopicMeasurementError,
    additive_log_ratio,
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

/// Fit-owned document ALR locations and diagonal variances in admitted row order.
///
/// This summary is deliberately narrower than an externally released posterior
/// artifact. It binds each retained diagonal variance to its fitted ALR location
/// and the admitted document row, but does not supply joint covariance,
/// plausible values, calibration evidence, semantic topic labels, or
/// Evidence-owned source/vocabulary provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct FittedDocumentCoordinateSummary {
    topic_count: usize,
    rows: Vec<FittedDocumentCoordinateRow>,
}

impl FittedDocumentCoordinateSummary {
    /// Build the coordinate summary from one admitted input and fitted model.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidModelInput`] when fitted topic,
    /// document, or coordinate dimensions disagree, a fitted topic proportion
    /// cannot be represented in ALR coordinates, or a retained diagonal
    /// variance is non-finite or not strictly positive.
    pub fn from_fit(
        input: &ReferenceTopicInput,
        model: &ReferenceTopicModel,
    ) -> Result<Self, TopicMeasurementError> {
        let topic_count = model.topic_term_probabilities.len();
        if topic_count < 2 {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
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

        Ok(Self { topic_count, rows })
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

    /// Return document rows in the admitted estimator input order.
    #[must_use]
    pub fn rows(&self) -> &[FittedDocumentCoordinateRow] {
        &self.rows
    }
}
