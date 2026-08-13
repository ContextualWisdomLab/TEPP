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

#[cfg(test)]
mod tests {
    use super::NetworkError;

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
}
