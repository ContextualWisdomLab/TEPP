//! Frozen prevalence coordinates for leakage-safe evaluation projection.
//!
//! A fitted prevalence coefficient matrix is meaningful only in the numerical
//! coordinate system used by its admitted training input. Rolling-origin or
//! otherwise held-out evaluation must therefore reuse the training event-time
//! transform and ordered prevalence-feature basis instead of recomputing them
//! from evaluation-horizon rows.

use std::collections::BTreeSet;

use corpus_split::CorpusSnapshot;
use membership_core::{MemberId, MembershipNetwork};
use relation_graph::RelationGraph;
use temporal_core::EventTime;
use uuid::Uuid;

use crate::{PrevalenceFeature, ReferenceTopicInput, SparseMatrix, TopicMeasurementError};

const NANOS_PER_SECOND: f64 = 1_000_000_000.0;

/// Immutable numerical coordinate system learned from one admitted training input.
///
/// This value owns only prevalence-coordinate semantics: the training event-time
/// origin/location/scale and the exact ordered feature basis used by the reference
/// input. It is not Evidence provenance, Membership authority, relation promotion,
/// topic identity, or release activation authority.
#[derive(Clone, Debug, PartialEq)]
pub struct PrevalenceDesignBasis {
    event_time_origin: EventTime,
    event_time_location_seconds: f64,
    event_time_scale_seconds: f64,
    features: Vec<PrevalenceFeature>,
}

impl PrevalenceDesignBasis {
    #[allow(clippy::cast_precision_loss)]
    fn from_training(
        event_times: &[EventTime],
        features: &[PrevalenceFeature],
    ) -> Result<Self, TopicMeasurementError> {
        if event_times.len() < 2 {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        let event_time_origin = event_times[0];
        let origin = event_time_origin.instant().as_nanosecond();
        let offsets: Vec<f64> = event_times
            .iter()
            .map(|time| {
                (time.instant().as_nanosecond() - origin) as f64 / NANOS_PER_SECOND
            })
            .collect();
        let event_time_location_seconds = offsets.iter().sum::<f64>() / offsets.len() as f64;
        let variance = offsets
            .iter()
            .map(|value| (value - event_time_location_seconds).powi(2))
            .sum::<f64>()
            / offsets.len() as f64;
        if variance <= 0.0 {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        let event_time_scale_seconds = variance.sqrt();
        Ok(Self {
            event_time_origin,
            event_time_location_seconds,
            event_time_scale_seconds,
            features: features.to_vec(),
        })
    }

    /// Return the first admitted training event time used as the offset origin.
    #[must_use]
    pub const fn event_time_origin(&self) -> &EventTime {
        &self.event_time_origin
    }

    /// Return the training offset mean, in seconds from [`Self::event_time_origin`].
    #[must_use]
    pub const fn event_time_location_seconds(&self) -> f64 {
        self.event_time_location_seconds
    }

    /// Return the training population standard deviation of event-time offsets.
    #[must_use]
    pub const fn event_time_scale_seconds(&self) -> f64 {
        self.event_time_scale_seconds
    }

    /// Return the exact ordered prevalence features used by the training input.
    #[must_use]
    pub fn features(&self) -> &[PrevalenceFeature] {
        &self.features
    }

    /// Project evaluation observations onto this frozen training coordinate system.
    ///
    /// Event times are transformed with the retained training origin, location,
    /// and scale. Evaluation-batch statistics never enter the transform. The
    /// caller may supply later valid event times; this numerical projection does
    /// not conflate event-valid time with an availability cutoff.
    ///
    /// Covariate dimensions and active membership coordinates must be representable
    /// by the fitted training feature basis. Unknown coordinates fail closed rather
    /// than being reordered or silently dropped.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidModelInput`] for empty, duplicated,
    /// or dimensionally incompatible evaluation rows, missing active membership,
    /// or any covariate/membership coordinate absent from the training basis.
    #[allow(clippy::cast_precision_loss)]
    pub fn project(
        &self,
        document_ids: &[Uuid],
        event_times: &[EventTime],
        covariates: Option<&SparseMatrix>,
        memberships: &MembershipNetwork,
    ) -> Result<Vec<Vec<f64>>, TopicMeasurementError> {
        if document_ids.is_empty()
            || document_ids.len() != event_times.len()
            || document_ids
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                != document_ids.len()
        {
            return Err(TopicMeasurementError::InvalidModelInput);
        }

        let covariate_count = self
            .features
            .iter()
            .filter(|feature| matches!(feature, PrevalenceFeature::Covariate(_)))
            .count();
        let covariate_rows = match (covariate_count, covariates) {
            (0, None) => None,
            (expected, Some(matrix))
                if expected > 0
                    && matrix.rows() == document_ids.len()
                    && matrix.columns() == expected =>
            {
                Some(matrix.row_entries())
            }
            _ => return Err(TopicMeasurementError::InvalidModelInput),
        };

        let origin = self.event_time_origin.instant().as_nanosecond();
        let mut design = vec![vec![0.0; self.features.len()]; document_ids.len()];
        for row in 0..document_ids.len() {
            design[row][0] = 1.0;
            let offset =
                (event_times[row].instant().as_nanosecond() - origin) as f64 / NANOS_PER_SECOND;
            design[row][1] =
                (offset - self.event_time_location_seconds) / self.event_time_scale_seconds;

            if let Some(rows) = &covariate_rows {
                for &(column, value) in &rows[row] {
                    design[row][2 + column] = value;
                }
            }

            let active = memberships
                .active_memberships_for(MemberId::from_uuid(document_ids[row]), event_times[row]);
            if active.is_empty() {
                return Err(TopicMeasurementError::InvalidModelInput);
            }
            for assignment in active {
                let target = PrevalenceFeature::Membership {
                    role: assignment.role(),
                    group_id: assignment.group_id(),
                };
                let feature_column = self
                    .features
                    .iter()
                    .position(|feature| *feature == target)
                    .ok_or(TopicMeasurementError::InvalidModelInput)?;
                design[row][feature_column] = assignment.weight().value();
            }
        }
        Ok(design)
    }
}

/// Training admission that binds a reference input to its frozen prevalence basis.
///
/// Both values are minted atomically from the same raw training observations. A
/// consumer cannot later substitute a dimension-compatible event-time transform or
/// feature ordering while retaining this owner-issued aggregate.
#[derive(Clone, Debug)]
pub struct ReferenceTopicTrainingInput {
    input: ReferenceTopicInput,
    prevalence_design_basis: PrevalenceDesignBasis,
}

impl ReferenceTopicTrainingInput {
    /// Admit one training corpus and freeze the numerical prevalence coordinates.
    ///
    /// The underlying [`ReferenceTopicInput`] remains the estimator owner. This
    /// aggregate adds the training transform required by later held-out projection;
    /// it does not authenticate Evidence, Membership, or relation provenance.
    ///
    /// # Errors
    ///
    /// Propagates reference-input validation failures and fails closed when the
    /// admitted training rows cannot be represented by the frozen basis exactly.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        snapshot: &CorpusSnapshot,
        document_ids: Vec<Uuid>,
        document_term: &SparseMatrix,
        event_times: &[EventTime],
        covariates: Option<&SparseMatrix>,
        memberships: &MembershipNetwork,
        relations: &RelationGraph,
    ) -> Result<Self, TopicMeasurementError> {
        let retained_document_ids = document_ids.clone();
        let input = ReferenceTopicInput::new(
            snapshot,
            document_ids,
            document_term,
            event_times,
            covariates,
            memberships,
            relations,
        )?;
        let prevalence_design_basis =
            PrevalenceDesignBasis::from_training(event_times, input.features())?;
        prevalence_design_basis.project(
            &retained_document_ids,
            event_times,
            covariates,
            memberships,
        )?;
        Ok(Self {
            input,
            prevalence_design_basis,
        })
    }

    /// Return the exact admitted reference-estimator training input.
    #[must_use]
    pub const fn input(&self) -> &ReferenceTopicInput {
        &self.input
    }

    /// Return the immutable training prevalence coordinate system.
    #[must_use]
    pub const fn prevalence_design_basis(&self) -> &PrevalenceDesignBasis {
        &self.prevalence_design_basis
    }
}

#[cfg(test)]
mod tests {
    use super::PrevalenceDesignBasis;
    use crate::{PrevalenceFeature, TopicMeasurementError};
    use temporal_core::EventTime;

    fn event_time(day: u8) -> EventTime {
        EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("event time")
    }

    #[test]
    fn training_transform_rejects_missing_or_constant_time() {
        assert_eq!(
            PrevalenceDesignBasis::from_training(&[], &[PrevalenceFeature::Intercept]),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        assert_eq!(
            PrevalenceDesignBasis::from_training(
                &[event_time(1), event_time(1)],
                &[PrevalenceFeature::Intercept, PrevalenceFeature::EventTime],
            ),
            Err(TopicMeasurementError::InvalidModelInput)
        );
    }
}
