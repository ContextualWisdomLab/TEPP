//! Refusal to treat compositional proportions as Euclidean points.

use crate::NetworkError;

/// Refuse raw simplex proportions as ordinary Euclidean coordinates.
///
/// Topic proportions live on the simplex. Euclidean distances on those
/// coordinates are not a valid network or clustering metric (ADR 0005/0012).
///
/// # Errors
///
/// Returns [`NetworkError::InvalidCoordinate`] when any value is non-finite,
/// or [`NetworkError::RawSimplexIsNotEuclidean`] when the values form a
/// closed simplex (non-negative parts that sum to one).
pub fn refuse_raw_simplex_as_euclidean(parts: &[f64]) -> Result<(), NetworkError> {
    if parts.is_empty() || parts.iter().any(|value| !value.is_finite()) {
        return Err(NetworkError::InvalidCoordinate);
    }
    let all_non_negative = parts.iter().all(|value| *value >= 0.0);
    let sum: f64 = parts.iter().sum();
    if all_non_negative && (sum - 1.0).abs() <= 1e-12 {
        return Err(NetworkError::RawSimplexIsNotEuclidean);
    }
    Err(NetworkError::InvalidCoordinate)
}

#[cfg(test)]
mod tests {
    use super::refuse_raw_simplex_as_euclidean;
    use crate::NetworkError;

    #[test]
    fn empty_and_negative_parts_are_invalid() {
        assert_eq!(
            refuse_raw_simplex_as_euclidean(&[]),
            Err(NetworkError::InvalidCoordinate)
        );
        assert_eq!(
            refuse_raw_simplex_as_euclidean(&[-0.2, 1.2]),
            Err(NetworkError::InvalidCoordinate)
        );
    }
}
