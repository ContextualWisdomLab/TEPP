//! Shared finite-vector validation for recovery metrics.

use crate::ValidationError;

/// Validate equal-length non-empty finite vectors.
#[inline(never)]
pub(crate) fn require_paired_finite(
    truth: &[f64],
    recovered: &[f64],
) -> Result<(), ValidationError> {
    if truth.is_empty() {
        return Err(ValidationError::InvalidInput);
    }
    if truth.len() != recovered.len() {
        return Err(ValidationError::InvalidInput);
    }
    if !slice_is_finite(truth) {
        return Err(ValidationError::InvalidInput);
    }
    if !slice_is_finite(recovered) {
        return Err(ValidationError::InvalidInput);
    }
    Ok(())
}

/// Validate a single finite slice.
#[inline(never)]
pub(crate) fn slice_is_finite(values: &[f64]) -> bool {
    let mut ok = true;
    for value in values {
        ok &= value.is_finite();
    }
    ok
}

#[cfg(test)]
mod tests {
    use super::{require_paired_finite, slice_is_finite};
    use crate::ValidationError;

    #[test]
    fn pair_validation_covers_all_arms() {
        assert!(slice_is_finite(&[1.0, 2.0]));
        assert!(!slice_is_finite(&[1.0, f64::NAN]));
        assert!(!slice_is_finite(&[f64::INFINITY]));
        assert_eq!(
            require_paired_finite(&[], &[]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            require_paired_finite(&[1.0], &[1.0, 2.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            require_paired_finite(&[f64::NAN], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            require_paired_finite(&[1.0], &[f64::NAN]),
            Err(ValidationError::InvalidInput)
        );
        require_paired_finite(&[1.0], &[2.0]).expect("ok");
    }
}
