//! CPU `f64` streamed reference arithmetic.

use crate::error::ComputeBackendError;

/// Stream a compensated weighted sum on the CPU `f64` reference path.
///
/// Neumaier-style compensation preserves low-order terms in cancellation-heavy
/// inputs while keeping deterministic input order. This sequential function is
/// the numerical reference for later fixed-pool CPU and GPU implementations.
///
/// # Errors
///
/// Returns [`ComputeBackendError::InvalidBudget`] when the slices are empty or
/// unequal, and [`ComputeBackendError::NonFiniteOutput`] when any term or
/// accumulator is non-finite.
pub fn streamed_weighted_sum(weights: &[f64], values: &[f64]) -> Result<f64, ComputeBackendError> {
    if weights.is_empty() || weights.len() != values.len() {
        return Err(ComputeBackendError::InvalidBudget);
    }
    let mut total = 0.0_f64;
    let mut compensation = 0.0_f64;
    for (weight, value) in weights.iter().zip(values) {
        let term = require_finite(*weight)? * require_finite(*value)?;
        let term = require_finite(term)?;
        let next = require_finite(total + term)?;
        let correction = if total.abs() >= term.abs() {
            (total - next) + term
        } else {
            (term - next) + total
        };
        compensation = require_finite(compensation + correction)?;
        total = next;
    }
    require_finite(total + compensation)
}

/// Reject a non-finite diagnostic quantity.
///
/// # Errors
///
/// Returns [`ComputeBackendError::NonFiniteOutput`] when `value` is NaN or
/// infinite.
pub fn require_finite(value: f64) -> Result<f64, ComputeBackendError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ComputeBackendError::NonFiniteOutput)
    }
}

/// Compare a candidate quantity against the CPU `f64` reference.
///
/// # Errors
///
/// Returns [`ComputeBackendError::NonFiniteOutput`] when either value or the
/// tolerance is non-finite, [`ComputeBackendError::InvalidTolerance`] for a
/// negative tolerance, and [`ComputeBackendError::ParityFailure`] when the
/// normalized gap exceeds `tolerance`, where the gap is divided by
/// `max(1, |reference|, |candidate|)`. Computing the normalized gap first
/// prevents a finite tolerance bound from overflowing before comparison.
pub fn require_cpu_gpu_parity(
    cpu_reference: f64,
    candidate: f64,
    tolerance: f64,
) -> Result<(), ComputeBackendError> {
    let left = require_finite(cpu_reference)?;
    let right = require_finite(candidate)?;
    let bound = require_finite(tolerance)?;
    if bound < 0.0 {
        return Err(ComputeBackendError::InvalidTolerance);
    }
    let scale = left.abs().max(right.abs()).max(1.0);
    let normalized_gap = require_finite((left - right).abs() / scale)?;
    if normalized_gap <= bound {
        Ok(())
    } else {
        Err(ComputeBackendError::ParityFailure)
    }
}

#[cfg(test)]
mod tests {
    use super::{require_cpu_gpu_parity, require_finite, streamed_weighted_sum};
    use crate::error::ComputeBackendError;

    #[test]
    fn compensated_reference_recovers_low_order_cancellation_term() {
        let result =
            streamed_weighted_sum(&[1.0, 1.0, 1.0], &[1e16, 1.0, -1e16]).expect("compensated sum");
        assert!((result - 1.0).abs() < 1e-15);
        let reverse = streamed_weighted_sum(&[1.0, 1.0, 1.0], &[-1e16, 1.0, 1e16])
            .expect("reverse compensation branch");
        assert!((reverse - 1.0).abs() < 1e-15);
    }

    #[test]
    fn reference_path_rejects_invalid_and_non_finite_input() {
        assert_eq!(
            streamed_weighted_sum(&[], &[1.0]),
            Err(ComputeBackendError::InvalidBudget)
        );
        assert_eq!(
            streamed_weighted_sum(&[1.0], &[1.0, 2.0]),
            Err(ComputeBackendError::InvalidBudget)
        );
        assert_eq!(
            streamed_weighted_sum(&[f64::NAN], &[1.0]),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        assert_eq!(
            streamed_weighted_sum(&[1.0], &[f64::INFINITY]),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        assert_eq!(
            streamed_weighted_sum(&[1e308], &[1e308]),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        assert_eq!(
            require_finite(f64::NEG_INFINITY),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        let finite = require_finite(1.5).expect("finite");
        assert!((finite - 1.5).abs() < 1e-15);
        require_cpu_gpu_parity(1.0, 1.0, 0.0).expect("exact parity");
        assert_eq!(
            require_cpu_gpu_parity(1.0, 2.0, 0.1),
            Err(ComputeBackendError::ParityFailure)
        );
        require_cpu_gpu_parity(1.0e12, 1.0e12 + 1.0e6, 1.0e-6)
            .expect("relative parity at large scale");
        assert_eq!(
            require_cpu_gpu_parity(1.0e12, 1.0e12 + 2.0e6, 1.0e-6),
            Err(ComputeBackendError::ParityFailure)
        );
        assert_eq!(
            require_cpu_gpu_parity(1.0, 1.0, -0.1),
            Err(ComputeBackendError::InvalidTolerance)
        );
        assert_eq!(
            require_cpu_gpu_parity(f64::NAN, 1.0, 0.1),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        assert_eq!(
            require_cpu_gpu_parity(1.0, f64::NAN, 0.1),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        assert_eq!(
            require_cpu_gpu_parity(1.0, 1.0, f64::NAN),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        assert_eq!(
            require_cpu_gpu_parity(f64::MAX, -f64::MAX, f64::MAX),
            Err(ComputeBackendError::NonFiniteOutput)
        );
    }
}
