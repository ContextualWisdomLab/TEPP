use validation_core::wilson_coverage_interval;

#[test]
fn interior_wilson_lower_bound_does_not_accept_a_roundoff_residual_as_signal() {
    let truth = [0.0, 1.0, 2.0];
    let lower = [-1.0, 2.0, 3.0];
    let upper = [1.0, 3.0, 4.0];
    let z = 1.0e11_f64;

    let n = 3.0_f64;
    let p = 1.0 / 3.0;
    let z_squared = z * z;
    assert!(z_squared.is_finite());

    // The rationalized Wilson root is the same estimand without subtracting
    // nearly equal O(z²) terms. At this finite z the predecessor subtraction
    // leaves a nonzero floating-point residue, so an exact-zero fallback alone
    // cannot detect the cancellation error.
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

#[test]
fn rationalized_lower_root_preserves_small_z_without_dividing_by_tiny_z_squared() {
    let truth = [0.0, 1.0, 2.0];
    let lower = [-1.0, 2.0, 3.0];
    let upper = [1.0, 3.0, 4.0];
    let z = 0.5_f64;

    let n = 3.0_f64;
    let p = 1.0 / 3.0;
    let z_squared = z * z;
    assert!(z_squared < 1.0);

    let expected_lower = (2.0 * n * p * p)
        / (z_squared
            + 2.0 * n * p
            + z * (z_squared + 4.0 * n * p * (1.0 - p)).sqrt());

    let (actual_lower, actual_upper) =
        wilson_coverage_interval(&truth, &lower, &upper, z).expect("finite Wilson interval");

    assert_eq!(actual_lower.to_bits(), expected_lower.to_bits());
    assert!(actual_upper >= p);
    assert!(actual_upper <= 1.0);
}
