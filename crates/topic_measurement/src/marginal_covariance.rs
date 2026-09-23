//! Fit-bound document marginal covariance from the joint ALR precision.
//!
//! The reference estimator already retains a bounded dense generalized-Gauss-
//! Newton precision for scientific validation. This module exposes one or more
//! document marginal covariance blocks from the inverse of that full relation-
//! coupled precision. It does not treat reciprocal precision diagonals or an
//! isolated document block inverse as a marginal posterior covariance.

use std::collections::BTreeSet;

use temporal_core::EventTime;
use uuid::Uuid;

use crate::{JointCoordinatePrecision, TopicMeasurementError, reference::cholesky};

const SYMMETRY_RELATIVE_TOLERANCE: f64 = 1.0e-10;

/// One fit-bound document marginal covariance in the retained ALR topic order.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentMarginalCovariance {
    document_id: Uuid,
    event_time: EventTime,
    topic_ids: Vec<Uuid>,
    values: Vec<Vec<f64>>,
}

impl DocumentMarginalCovariance {
    /// Return the fitted document whose marginal block was solved.
    #[must_use]
    pub const fn document_id(&self) -> Uuid {
        self.document_id
    }

    /// Return the event time retained for the fitted document row.
    #[must_use]
    pub const fn event_time(&self) -> EventTime {
        self.event_time
    }

    /// Return ALR numerator topics followed by the retained reference topic.
    #[must_use]
    pub fn topic_ids(&self) -> &[Uuid] {
        &self.topic_ids
    }

    /// Return the `(K-1) × (K-1)` marginal covariance block.
    #[must_use]
    pub fn values(&self) -> &[Vec<f64>] {
        &self.values
    }
}

impl JointCoordinatePrecision {
    /// Solve one document's marginal covariance block from the full precision.
    ///
    /// The scalar API delegates to [`Self::document_marginal_covariances`] so
    /// single- and multi-document scientific paths share identical selected-
    /// inverse arithmetic and validation semantics.
    ///
    /// # Errors
    ///
    /// Propagates retained-geometry, missing-document, factorization, solve,
    /// symmetry, and positive-definiteness failures from the batch owner.
    pub fn document_marginal_covariance(
        &self,
        document_id: Uuid,
    ) -> Result<DocumentMarginalCovariance, TopicMeasurementError> {
        self.document_marginal_covariances(&[document_id])?
            .into_iter()
            .next()
            .ok_or(TopicMeasurementError::InvalidModelInput)
    }

    /// Solve requested document marginal covariance blocks with one factorization.
    ///
    /// The retained joint precision is validated and Cholesky-factorized once.
    /// For every requested document, one unit right-hand side is solved for each
    /// ALR coordinate and only that document's `(K-1) × (K-1)` block of `P^-1`
    /// is retained. Requested order is preserved exactly; duplicate, empty, or
    /// absent document requests fail closed rather than silently changing the
    /// scientific denominator.
    ///
    /// With joint dimension `N <= 4096`, coordinate count `C = K-1`, and `R`
    /// requested documents, the bounded dense reference path costs `O(N^3)` for
    /// one Cholesky factorization plus `O(R C N^2)` for triangular solves and
    /// `O(N^2)` memory. This is a validation primitive, not a calibrated
    /// posterior or accelerated/sparse covariance claim.
    ///
    /// Returned coordinates remain bound to each fitted document, retained
    /// EventTime, and topic order. Evidence provenance, release topic identity,
    /// empirical coverage, and backend parity remain separate owner
    /// responsibilities.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidModelInput`] when the request is
    /// empty/duplicated, a requested document is absent, or retained matrix
    /// geometry is inconsistent. Returns [`TopicMeasurementError::NonFiniteEstimate`]
    /// when factorization, linear solves, symmetry reconciliation, or positive-
    /// definiteness fails.
    pub fn document_marginal_covariances(
        &self,
        requested_document_ids: &[Uuid],
    ) -> Result<Vec<DocumentMarginalCovariance>, TopicMeasurementError> {
        if requested_document_ids.is_empty()
            || requested_document_ids.iter().copied().collect::<BTreeSet<_>>().len()
                != requested_document_ids.len()
        {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        let (coordinate_count, dimension) = self.validated_geometry()?;
        let document_indices: Vec<_> = requested_document_ids
            .iter()
            .map(|document_id| {
                self.document_ids
                    .iter()
                    .position(|candidate| candidate == document_id)
                    .ok_or(TopicMeasurementError::InvalidModelInput)
            })
            .collect::<Result<_, _>>()?;
        let lower = cholesky(&self.values)?;

        requested_document_ids
            .iter()
            .copied()
            .zip(document_indices)
            .map(|(document_id, document_index)| {
                let start = document_index * coordinate_count;
                let mut covariance = vec![vec![0.0; coordinate_count]; coordinate_count];
                for local_column in 0..coordinate_count {
                    let mut right_hand_side = vec![0.0; dimension];
                    right_hand_side[start + local_column] = 1.0;
                    let solution = solve_cholesky(&lower, &right_hand_side)?;
                    for local_row in 0..coordinate_count {
                        covariance[local_row][local_column] = solution[start + local_row];
                    }
                }
                reconcile_roundoff_and_validate(&mut covariance)?;
                Ok(DocumentMarginalCovariance {
                    document_id,
                    event_time: self.event_times[document_index],
                    topic_ids: self.topic_ids.clone(),
                    values: covariance,
                })
            })
            .collect()
    }

    fn validated_geometry(&self) -> Result<(usize, usize), TopicMeasurementError> {
        let coordinate_count = self
            .topic_ids
            .len()
            .checked_sub(1)
            .filter(|count| *count > 0)
            .ok_or(TopicMeasurementError::InvalidModelInput)?;
        let dimension = self
            .document_ids
            .len()
            .checked_mul(coordinate_count)
            .filter(|dimension| *dimension > 0)
            .ok_or(TopicMeasurementError::InvalidModelInput)?;
        if self.values.len() != dimension
            || self.coordinate_means.len() != dimension
            || self.event_times.len() != self.document_ids.len()
            || self.values.iter().any(|row| row.len() != dimension)
        {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        Ok((coordinate_count, dimension))
    }
}

fn solve_cholesky(
    lower: &[Vec<f64>],
    right_hand_side: &[f64],
) -> Result<Vec<f64>, TopicMeasurementError> {
    let dimension = lower.len();
    if dimension == 0
        || right_hand_side.len() != dimension
        || lower.iter().any(|row| row.len() != dimension)
        || right_hand_side.iter().any(|value| !value.is_finite())
    {
        return Err(TopicMeasurementError::InvalidModelInput);
    }

    let mut intermediate = vec![0.0; dimension];
    for row in 0..dimension {
        let diagonal = lower[row][row];
        let product = (0..row)
            .map(|column| lower[row][column] * intermediate[column])
            .sum::<f64>();
        let value = (right_hand_side[row] - product) / diagonal;
        if !value.is_finite() {
            return Err(TopicMeasurementError::NonFiniteEstimate);
        }
        intermediate[row] = value;
    }

    let mut solution = vec![0.0; dimension];
    for row in (0..dimension).rev() {
        let diagonal = lower[row][row];
        let product = ((row + 1)..dimension)
            .map(|column| lower[column][row] * solution[column])
            .sum::<f64>();
        let value = (intermediate[row] - product) / diagonal;
        if !value.is_finite() {
            return Err(TopicMeasurementError::NonFiniteEstimate);
        }
        solution[row] = value;
    }
    Ok(solution)
}

fn reconcile_roundoff_and_validate(
    covariance: &mut [Vec<f64>],
) -> Result<(), TopicMeasurementError> {
    let dimension = covariance.len();
    if dimension == 0
        || covariance.iter().any(|row| {
            row.len() != dimension || row.iter().any(|value| !value.is_finite())
        })
    {
        return Err(TopicMeasurementError::NonFiniteEstimate);
    }

    for row in 0..dimension {
        for column in (row + 1)..dimension {
            let left = covariance[row][column];
            let right = covariance[column][row];
            let scale = 1.0 + left.abs().max(right.abs());
            if (left - right).abs() > SYMMETRY_RELATIVE_TOLERANCE * scale {
                return Err(TopicMeasurementError::NonFiniteEstimate);
            }
            let reconciled = (left + right) / 2.0;
            if !reconciled.is_finite() {
                return Err(TopicMeasurementError::NonFiniteEstimate);
            }
            covariance[row][column] = reconciled;
            covariance[column][row] = reconciled;
        }
    }
    cholesky(covariance).map(|_| ())
}

#[cfg(test)]
mod tests {
    use temporal_core::EventTime;
    use uuid::Uuid;

    use super::{DocumentMarginalCovariance, reconcile_roundoff_and_validate, solve_cholesky};
    use crate::{JointCoordinatePrecision, TopicMeasurementError, reference::cholesky};

    fn event_time(day: u8) -> EventTime {
        EventTime::parse_rfc3339(&format!("2026-09-{day:02}T00:00:00Z")).expect("event time")
    }

    fn coupled_precision() -> JointCoordinatePrecision {
        JointCoordinatePrecision {
            document_ids: vec![Uuid::from_u128(1), Uuid::from_u128(2)],
            topic_ids: vec![Uuid::from_u128(11), Uuid::from_u128(12)],
            event_times: vec![event_time(1), event_time(2)],
            coordinate_means: vec![0.0, 0.0],
            values: vec![vec![2.0, -1.0], vec![-1.0, 2.0]],
        }
    }

    #[test]
    fn full_inverse_marginal_differs_from_local_reciprocal() {
        let precision = coupled_precision();
        let marginal = precision
            .document_marginal_covariance(Uuid::from_u128(1))
            .expect("marginal covariance");
        assert_eq!(marginal.document_id(), Uuid::from_u128(1));
        assert_eq!(marginal.event_time(), event_time(1));
        assert_eq!(marginal.topic_ids(), precision.topic_ids());
        assert!((marginal.values()[0][0] - 2.0 / 3.0).abs() < 1.0e-12);
        assert!((marginal.values()[0][0] - 0.5).abs() > 1.0e-6);
    }

    #[test]
    fn multi_coordinate_block_comes_from_the_full_joint_inverse() {
        let precision = JointCoordinatePrecision {
            document_ids: vec![Uuid::from_u128(1), Uuid::from_u128(2)],
            topic_ids: vec![
                Uuid::from_u128(11),
                Uuid::from_u128(12),
                Uuid::from_u128(13),
            ],
            event_times: vec![event_time(1), event_time(2)],
            coordinate_means: vec![0.0; 4],
            values: vec![
                vec![4.0, 1.0, -1.0, 0.2],
                vec![1.0, 3.0, 0.1, -0.5],
                vec![-1.0, 0.1, 3.5, 0.8],
                vec![0.2, -0.5, 0.8, 2.8],
            ],
        };
        let marginal = precision
            .document_marginal_covariance(Uuid::from_u128(1))
            .expect("two-coordinate marginal");

        assert_eq!(marginal.values().len(), 2);
        assert!((marginal.values()[0][0] - 0.311_047_30).abs() < 1.0e-7);
        assert!((marginal.values()[0][1] + 0.119_807_86).abs() < 1.0e-7);
        assert!((marginal.values()[1][0] + 0.119_807_86).abs() < 1.0e-7);
        assert!((marginal.values()[1][1] - 0.391_846_59).abs() < 1.0e-7);

        let isolated_block_inverse_00 = 3.0 / 11.0;
        assert!((marginal.values()[0][0] - isolated_block_inverse_00).abs() > 1.0e-3);
    }

    #[test]
    fn identity_and_geometry_fail_closed() {
        let precision = coupled_precision();
        assert_eq!(
            precision.document_marginal_covariance(Uuid::from_u128(999)),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        let mut malformed = precision.clone();
        malformed.values.pop();
        assert_eq!(
            malformed.document_marginal_covariance(Uuid::from_u128(1)),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        let mut malformed_means = precision.clone();
        malformed_means.coordinate_means.pop();
        assert_eq!(
            malformed_means.document_marginal_covariance(Uuid::from_u128(1)),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        let mut malformed_times = precision;
        malformed_times.event_times.pop();
        assert_eq!(
            malformed_times.document_marginal_covariance(Uuid::from_u128(1)),
            Err(TopicMeasurementError::InvalidModelInput)
        );
    }

    #[test]
    fn triangular_solve_and_roundoff_paths_are_bounded() {
        let lower = cholesky(&[vec![4.0, 1.0], vec![1.0, 3.0]]).expect("cholesky");
        let solution = solve_cholesky(&lower, &[1.0, 0.0]).expect("solve");
        assert!((solution[0] - 3.0 / 11.0).abs() < 1.0e-12);
        assert!((solution[1] + 1.0 / 11.0).abs() < 1.0e-12);

        assert_eq!(
            solve_cholesky(&[], &[]),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        assert_eq!(
            solve_cholesky(&[vec![1.0]], &[f64::NAN]),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        let tiny = cholesky(&[vec![1.0e-320]]).expect("tiny positive precision");
        assert_eq!(
            solve_cholesky(&tiny, &[1.0]),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );

        let mut near_symmetric = vec![vec![2.0, 0.5], vec![0.5 + 1.0e-12, 2.0]];
        reconcile_roundoff_and_validate(&mut near_symmetric).expect("roundoff reconciliation");
        assert_eq!(
            near_symmetric[0][1].to_bits(),
            near_symmetric[1][0].to_bits()
        );

        let mut asymmetric = vec![vec![2.0, 0.5], vec![0.6, 2.0]];
        assert_eq!(
            reconcile_roundoff_and_validate(&mut asymmetric),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        let mut non_spd = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        assert_eq!(
            reconcile_roundoff_and_validate(&mut non_spd),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
    }

    #[test]
    fn public_value_object_accessors_preserve_owner_coordinates() {
        let value = DocumentMarginalCovariance {
            document_id: Uuid::from_u128(1),
            event_time: event_time(1),
            topic_ids: vec![Uuid::from_u128(11), Uuid::from_u128(12)],
            values: vec![vec![0.25]],
        };
        assert_eq!(value.document_id(), Uuid::from_u128(1));
        assert_eq!(value.event_time(), event_time(1));
        assert_eq!(value.topic_ids(), &[Uuid::from_u128(11), Uuid::from_u128(12)]);
        assert!((value.values()[0][0] - 0.25).abs() < f64::EPSILON);
    }
}
