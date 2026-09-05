use validation_core::wilson_coverage_interval;

#[test]
fn all_covered_wilson_lower_bound_survives_finite_extreme_z() {
    let truth = [0.0];
    let lower = [-1.0];
    let upper = [1.0];
    let z = 1.0e154_f64;

    let z_squared = z * z;
    assert!(z_squared.is_finite());
    let expected_lower = 1.0 / (1.0 + z_squared);
    assert!(expected_lower.is_finite());
    assert!(expected_lower > 0.0);

    let (actual_lower, actual_upper) =
        wilson_coverage_interval(&truth, &lower, &upper, z).expect("finite Wilson interval");

    assert_eq!(actual_lower.to_bits(), expected_lower.to_bits());
    assert_eq!(actual_upper, 1.0);
}
