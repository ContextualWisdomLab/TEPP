use validation_core::{ValidationError, summarize_replications};

#[test]
fn representable_extreme_symmetric_moments_survive_intermediate_overflow() {
    let samples = [f64::MAX, 0.0, 0.0, -f64::MAX];
    let summary = summarize_replications(&samples, 0.0, 1.0)
        .expect("representable extreme Monte Carlo moments");

    let expected_standard_deviation = f64::MAX * (2.0_f64 / 3.0).sqrt();
    let expected_standard_error = expected_standard_deviation / 2.0;
    assert_eq!(summary.mean.to_bits(), 0.0_f64.to_bits());
    assert!(
        ((summary.standard_deviation - expected_standard_deviation)
            / expected_standard_deviation)
            .abs()
            <= 2.0 * f64::EPSILON
    );
    assert!(
        ((summary.standard_error - expected_standard_error) / expected_standard_error).abs()
            <= 2.0 * f64::EPSILON
    );
    assert_eq!(summary.percentile_lower, -f64::MAX);
    assert_eq!(summary.percentile_upper, f64::MAX);
}

#[test]
fn nonzero_monte_carlo_uncertainty_cannot_collapse_to_exact_zero() {
    let minimum_subnormal = f64::from_bits(1);
    let samples = [
        minimum_subnormal,
        -minimum_subnormal,
        minimum_subnormal,
        -minimum_subnormal,
        minimum_subnormal,
        -minimum_subnormal,
        minimum_subnormal,
        -minimum_subnormal,
        minimum_subnormal,
        -minimum_subnormal,
        minimum_subnormal,
        -minimum_subnormal,
        minimum_subnormal,
        -minimum_subnormal,
        minimum_subnormal,
        -minimum_subnormal,
    ];

    assert_eq!(
        summarize_replications(&samples, 0.0, 1.0),
        Err(ValidationError::InvalidInput)
    );
}
