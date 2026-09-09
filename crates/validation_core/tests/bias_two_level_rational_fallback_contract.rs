//! Public contracts for exact two-level rational-scale fallback paths.

use validation_core::bias_standard_error;

#[test]
fn balanced_low_term_levels_use_normal_rational_restoration_after_exact_route_refuses() {
    let quarter_ulp_at_one = 2.0_f64.powi(-54);
    let truth = [
        quarter_ulp_at_one,
        quarter_ulp_at_one,
        quarter_ulp_at_one,
        quarter_ulp_at_one,
        quarter_ulp_at_one,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    ];
    let recovered = [1.0; 10];

    // Every represented subtraction rounds to the same high part, 1.0, so the
    // bounded exact pair-distance route correctly refuses its no-roundoff input
    // contract. Error-free subtraction leaves five low terms at -2^-54 and five
    // at zero. Their two-level count factor is exactly 1/6, making the represented
    // SE 2^-54 / 6. That result is normal, not subnormal, so the fallback's
    // bounded subnormal-unit shortcut must decline and ordinary exact
    // sum-over-count restoration must preserve the final rounding.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("balanced represented low-term dispersion remains finite");
    assert_eq!(standard_error.to_bits(), 0x3c65_5555_5555_5555);

    let permuted_truth = [
        quarter_ulp_at_one,
        0.0,
        quarter_ulp_at_one,
        0.0,
        quarter_ulp_at_one,
        0.0,
        quarter_ulp_at_one,
        0.0,
        quarter_ulp_at_one,
        0.0,
    ];
    let permuted_standard_error = bias_standard_error(&permuted_truth, &recovered)
        .expect("permutation preserves represented low-term dispersion");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}

#[test]
fn balanced_four_observation_low_terms_fall_through_nonrational_count_scale() {
    let quarter_ulp_at_one = 2.0_f64.powi(-54);
    let truth = [quarter_ulp_at_one, quarter_ulp_at_one, 0.0, 0.0];
    let recovered = [1.0; 4];

    // Subtraction roundoff makes the bounded pair-distance route refuse. The
    // fallback then sees two exact translated levels with counts 2 and 2. Their
    // count-only SE factor is 1/sqrt(12), which is not a rational square root, so
    // the rational-scale shortcut must decline and the general represented-input
    // path must return the correctly rounded 2^-54/sqrt(12).
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("nonrational two-level count geometry remains representable");
    assert_eq!(standard_error.to_bits(), 0x3c72_79a7_4590_331c);

    let permuted_truth = [quarter_ulp_at_one, 0.0, quarter_ulp_at_one, 0.0];
    let permuted_standard_error = bias_standard_error(&permuted_truth, &recovered)
        .expect("permutation preserves nonrational two-level geometry");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}

#[test]
fn unbalanced_five_observation_low_terms_refuse_nonsquare_reduced_numerator() {
    let quarter_ulp_at_one = 2.0_f64.powi(-54);
    let truth = [quarter_ulp_at_one, quarter_ulp_at_one, 0.0, 0.0, 0.0];
    let recovered = [1.0; 5];

    // The bounded exact pair-distance route refuses because two subtractions
    // carry error-free low terms. The fallback sees two exact levels with counts
    // 2 and 3. Reducing m(n-m) / (n^2(n-1)) gives 3/50, whose numerator is not a
    // perfect square, so the rational shortcut must decline at its numerator
    // proof and the generic translated path must round 2^-54 * sqrt(3/50).
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("nonsquare numerator geometry remains representable");
    assert_eq!(standard_error.to_bits(), 0x3c6f_5a7c_ecdb_684a);

    let permuted_truth = [quarter_ulp_at_one, 0.0, 0.0, quarter_ulp_at_one, 0.0];
    let permuted_standard_error = bias_standard_error(&permuted_truth, &recovered)
        .expect("permutation preserves nonsquare numerator geometry");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
