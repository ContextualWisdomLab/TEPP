//! Public contract for a three-level fallback whose dyadic normalization rounds but does not erase the minor offset.

use validation_core::bias_standard_error;

#[test]
fn three_level_irreversible_normalization_preserves_correctly_rounded_se() {
    let huge = f64::from_bits(0x7fe0_0000_0000_0000); // 2^1023
    let minor = 3.0 * f64::EPSILON; // 3 * 2^-52, exactly represented.
    let truth = [0.0; 3];

    // The bounded exact route refuses this 1075-bit coefficient span. In the
    // translated fallback, dividing `minor` by the exact 2^1023 scale yields
    // two minimum-subnormal units: nonzero, but not exactly reversible to the
    // original 3 * 2^-52 offset. That proof must refuse without changing the
    // correctly rounded public target, whose minor contribution is far below
    // the midpoint around 2^1023 / 3.
    let recovered_minor_first = [0.0, minor, huge];
    let standard_error = bias_standard_error(&truth, &recovered_minor_first)
        .expect("representable three-level standard error");
    assert_eq!(standard_error.to_bits(), 0x7fc5_5555_5555_5555);

    // Reverse the two nonzero translated levels so both normalization-reversal
    // guards are exercised without making route admission depend on order.
    let recovered_minor_second = [0.0, huge, minor];
    let permuted_standard_error = bias_standard_error(&truth, &recovered_minor_second)
        .expect("permuted representable three-level standard error");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
