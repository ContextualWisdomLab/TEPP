//! Wilson-pair boundary regression for representable peer roots.

use validation_core::{ValidationError, ValidationReport};

const COVERAGE_ONE_IN_ONE_HUNDRED_MILLION: f64 = f64::from_bits(0x3e45_798e_e230_8c3a);
const PRODUCER_LOWER: f64 = f64::from_bits(0x3c9c_d2b2_8e2c_a873);
const PRODUCER_UPPER: f64 = f64::from_bits(0x3fe0_0000_055e_63b8);
const COMPLEMENT_COVERAGE: f64 = f64::from_bits(0x3fef_ffff_faa1_9c47);
const COMPLEMENT_LOWER: f64 = f64::from_bits(0x3fdf_ffff_f543_3890);
const MINIMUM_U64_COVERAGE: f64 = f64::from_bits(0x3bf0_0000_0000_0000);

fn report_with_wilson_pair(coverage: f64, lower: f64, upper: f64) -> ValidationReport {
    ValidationReport {
        study_label: "wilson-boundary-pair".to_owned(),
        rmse: 1.0,
        rmse_standard_error: 0.0,
        mean_bias: 0.0,
        bias_standard_error: 0.0,
        interval_coverage: coverage,
        coverage_wilson_lower: lower,
        coverage_wilson_upper: upper,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: None,
    }
}

/// Rejects a zero lower endpoint when the peer endpoint implies a representable positive root.
#[test]
fn zero_lower_cannot_hide_representable_wilson_peer_root() {
    let artifact = report_with_wilson_pair(
        COVERAGE_ONE_IN_ONE_HUNDRED_MILLION,
        0.0,
        PRODUCER_UPPER,
    );

    assert_eq!(artifact.validate(), Err(ValidationError::InvalidInput));
    assert_eq!(artifact.to_json(), Err(ValidationError::InvalidInput));

    let raw = format!(
        r#"{{"study_label":"wilson-boundary-pair","rmse":1.0,"rmse_standard_error":0.0,"mean_bias":0.0,"bias_standard_error":0.0,"interval_coverage":{coverage},"coverage_wilson_lower":0.0,"coverage_wilson_upper":{upper},"temporal_order_accuracy":1.0,"monte_carlo_rmse":null}}"#,
        coverage = COVERAGE_ONE_IN_ONE_HUNDRED_MILLION,
        upper = PRODUCER_UPPER,
    );
    assert!(serde_json::from_str::<ValidationReport>(&raw).is_err());
}

/// Applies the same peer-root admission rule to the complement-symmetric exact-one upper boundary.
#[test]
fn exact_one_upper_cannot_hide_representable_uncovered_peer_root() {
    let artifact = report_with_wilson_pair(COMPLEMENT_COVERAGE, COMPLEMENT_LOWER, 1.0);

    assert_eq!(artifact.validate(), Err(ValidationError::InvalidInput));
}

/// Preserves the actual rounded Wilson pair for `n = 100_000_000`, one cover, and `z = 10_000`.
#[test]
fn producer_pair_remains_admissible_at_the_same_coverage() {
    let artifact = report_with_wilson_pair(
        COVERAGE_ONE_IN_ONE_HUNDRED_MILLION,
        PRODUCER_LOWER,
        PRODUCER_UPPER,
    );

    assert!(artifact.validate().is_ok());
}

/// Keeps an unresolved extreme boundary admissible when the peer-root reconstruction also rounds to zero.
#[test]
fn rounded_zero_and_one_pair_remains_admissible_when_peer_root_is_unrepresentable() {
    let artifact = report_with_wilson_pair(MINIMUM_U64_COVERAGE, 0.0, 1.0);

    assert!(artifact.validate().is_ok());
}
