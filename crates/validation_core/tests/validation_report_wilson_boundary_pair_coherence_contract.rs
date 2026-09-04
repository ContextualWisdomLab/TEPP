//! Wilson-pair boundary regression for a representable minority-coverage lower root.

use validation_core::{ValidationError, ValidationReport};

const COVERAGE_ONE_IN_ONE_HUNDRED_MILLION: f64 = f64::from_bits(0x3e45_798e_e230_8c3a);
const PRODUCER_LOWER: f64 = f64::from_bits(0x3c9c_d2b2_8e2c_a873);
const PRODUCER_UPPER: f64 = f64::from_bits(0x3fe0_0000_055e_63b8);

fn report_with_wilson_pair(lower: f64, upper: f64) -> ValidationReport {
    ValidationReport {
        study_label: "wilson-boundary-pair".to_owned(),
        rmse: 1.0,
        rmse_standard_error: 0.0,
        mean_bias: 0.0,
        bias_standard_error: 0.0,
        interval_coverage: COVERAGE_ONE_IN_ONE_HUNDRED_MILLION,
        coverage_wilson_lower: lower,
        coverage_wilson_upper: upper,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: None,
    }
}

/// Rejects a zero lower endpoint when the peer endpoint implies a representable positive root.
#[test]
fn zero_lower_cannot_hide_representable_wilson_peer_root() {
    let artifact = report_with_wilson_pair(0.0, PRODUCER_UPPER);

    assert_eq!(artifact.validate(), Err(ValidationError::InvalidInput));
}

/// Preserves the actual rounded Wilson pair for `n = 100_000_000`, one cover, and `z = 10_000`.
#[test]
fn producer_pair_remains_admissible_at_the_same_coverage() {
    let artifact = report_with_wilson_pair(PRODUCER_LOWER, PRODUCER_UPPER);

    assert!(artifact.validate().is_ok());
}
