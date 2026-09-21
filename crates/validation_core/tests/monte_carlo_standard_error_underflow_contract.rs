//! Reject a projected nonzero Monte Carlo standard error when its governing ratio rounds to zero.
//!
//! A positive recorded sample spread must imply a representable positive `SD / sqrt(n)` before a
//! durable Monte Carlo summary can claim nonzero uncertainty. These contracts exercise the
//! fail-closed underflow boundary and separately require non-finite relative-error arithmetic to
//! remain invalid rather than being mistaken for an admissible tolerance deviation.

use validation_core::{MonteCarloSummary, ValidationError};

#[test]
fn positive_spread_with_zero_projected_standard_error_is_rejected() {
    let summary = MonteCarloSummary {
        replication_count: 4,
        mean: 0.0,
        standard_deviation: f64::from_bits(1),
        standard_error: f64::from_bits(1),
        percentile_lower: 0.0,
        percentile_upper: 0.0,
    };

    assert_eq!(summary.validate(), Err(ValidationError::InvalidInput));
}

#[test]
fn nonfinite_relative_standard_error_distance_is_rejected() {
    let summary = MonteCarloSummary {
        replication_count: 4,
        mean: 0.0,
        standard_deviation: f64::MIN_POSITIVE,
        standard_error: f64::MAX,
        percentile_lower: 0.0,
        percentile_upper: 0.0,
    };

    assert_eq!(summary.validate(), Err(ValidationError::InvalidInput));
}
