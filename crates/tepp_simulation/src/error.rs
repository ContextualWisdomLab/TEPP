//! Fail-closed simulation errors.

use std::fmt;

/// A fail-closed simulation-domain error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SimulationError {
    /// Configuration values were empty, out of range, or inconsistent.
    InvalidConfiguration,
    /// A required temporal order invariant was violated.
    TemporalInvariantViolation,
    /// Truth-manifest invariants failed after generation.
    ManifestInvariantViolation,
}

impl fmt::Display for SimulationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidConfiguration => "invalid simulation configuration",
            Self::TemporalInvariantViolation => "temporal invariant violation",
            Self::ManifestInvariantViolation => "truth manifest invariant violation",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SimulationError {}

#[cfg(test)]
mod tests {
    use super::SimulationError;

    #[test]
    fn messages_are_stable() {
        assert_eq!(
            SimulationError::InvalidConfiguration.to_string(),
            "invalid simulation configuration"
        );
        assert_eq!(
            SimulationError::TemporalInvariantViolation.to_string(),
            "temporal invariant violation"
        );
        assert_eq!(
            SimulationError::ManifestInvariantViolation.to_string(),
            "truth manifest invariant violation"
        );
    }
}
