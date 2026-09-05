use validation_core::wilson_coverage_interval;

#[test]
fn interior_wilson_lower_bound_survives_finite_extreme_z() {
    let truth = [0.0, 1.0];
    let lower = [-1.0, 2.0];
    let upper = [1.0, 3.0];
    let z = 1.0e154_f64;

    let n = 2.0_f64;
    let p = 0.5_f64;
    let z_squared = z * z;
    assert!(z_squared.is_finite());

    // Algebraically rationalized Wilson lower endpoint. This form avoids the
    // nearly-equal center-minus-margin subtraction used by the predecessor.
    let expected_lower = (2.0 * n * p * p / z_squared)
        / (1.0
            + 2.0 * n * p / z_squared
            + (1.0 + 4.0 * n * p * (1.0 - p) / z_squared).sqrt());
    assert!(expected_lower.is_finite());
    assert!(expected_lower > 0.0);

    let (actual_lower, actual_upper) =
        wilson_coverage_interval(&truth, &lower, &upper, z).expect("finite Wilson interval");

    assert_eq!(actual_lower.to_bits(), expected_lower.to_bits());
    assert!(actual_upper >= p);
    assert!(actual_upper <= 1.0);
}
