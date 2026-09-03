use validation_core::wilson_coverage_interval;

#[test]
fn all_uncovered_wilson_upper_bound_does_not_round_to_false_one() {
    let truth = [0.0, 1.0];
    let lower = [1.0, 2.0];
    let upper = [2.0, 3.0];
    let z = 134_217_728.0_f64; // 2^27, so z^2 / n = 2^53 exactly for n = 2.

    let z_squared = z * z;
    assert_eq!(z_squared, 18_014_398_509_481_984.0_f64); // 2^54.

    // For p-hat = 0, the Wilson upper endpoint is z^2 / (n + z^2).
    // At this represented input its correctly rounded binary64 value is the
    // immediate predecessor of 1.0, not the exact endpoint 1.0.
    let expected_upper = f64::from_bits(1.0_f64.to_bits() - 1);

    let (actual_lower, actual_upper) =
        wilson_coverage_interval(&truth, &lower, &upper, z).expect("finite Wilson interval");

    assert_eq!(actual_lower, 0.0);
    assert_eq!(actual_upper.to_bits(), expected_upper.to_bits());
}
