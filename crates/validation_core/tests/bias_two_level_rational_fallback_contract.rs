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
