//! Fail-closed network-analysis errors.

use std::fmt;

/// A fail-closed network-analysis error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NetworkError {
    /// Raw topic proportions were treated as ordinary Euclidean coordinates.
    RawSimplexIsNotEuclidean,
    /// A coordinate was non-finite.
    InvalidCoordinate,
    /// Cluster-label slices were empty, singleton, or length-mismatched.
    InvalidClusterPayload,
}

impl fmt::Display for NetworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::RawSimplexIsNotEuclidean => "raw simplex is not a Euclidean coordinate",
            Self::InvalidCoordinate => "invalid network coordinate",
            Self::InvalidClusterPayload => "invalid cluster payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for NetworkError {}

/// Fail-closed errors from the posterior network estimator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NetworkEstimatorError {
    /// Posterior draw set is empty.
    EmptyDraws,
    /// Coordinate dimensions are inconsistent across draws or too small.
    DimensionMismatch,
    /// Fewer than 3 ILR coordinates available for pairwise correlation.
    InsufficientCoordinates,
    /// Fewer than 3 observations per coordinate for correlation.
    InsufficientObservations,
    /// Bootstrap replicates must be at least 1.
    ZeroReplicates,
}

impl fmt::Display for NetworkEstimatorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyDraws => "posterior draw set is empty",
            Self::DimensionMismatch => "coordinate dimensions are inconsistent",
            Self::InsufficientCoordinates => "fewer than 3 ILR coordinates for correlation",
            Self::InsufficientObservations => "fewer than 3 observations per coordinate",
            Self::ZeroReplicates => "bootstrap replicates must be at least 1",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for NetworkEstimatorError {}

#[cfg(test)]
mod tests {
    use super::{NetworkError, NetworkEstimatorError};

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                NetworkError::RawSimplexIsNotEuclidean,
                "raw simplex is not a Euclidean coordinate",
            ),
            (
                NetworkError::InvalidCoordinate,
                "invalid network coordinate",
            ),
            (
                NetworkError::InvalidClusterPayload,
                "invalid cluster payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }

    #[test]
    fn estimator_error_messages_are_stable() {
        for (error, message) in [
            (
                NetworkEstimatorError::EmptyDraws,
                "posterior draw set is empty",
            ),
            (
                NetworkEstimatorError::DimensionMismatch,
                "coordinate dimensions are inconsistent",
            ),
            (
                NetworkEstimatorError::InsufficientCoordinates,
                "fewer than 3 ILR coordinates for correlation",
            ),
            (
                NetworkEstimatorError::InsufficientObservations,
                "fewer than 3 observations per coordinate",
            ),
            (
                NetworkEstimatorError::ZeroReplicates,
                "bootstrap replicates must be at least 1",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
