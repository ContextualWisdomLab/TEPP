use validation_core::bias_standard_error;

#[test]
fn equal_rounded_residuals_preserve_nontranslatable_low_term_dispersion() {
    let minimum_subnormal = f64::from_bits(1);
    let truth = [
        f64::from_bits(0x3c90_0000_0000_0000),
        f64::from_bits(0x3c80_0000_0000_0000),
        minimum_subnormal,
    ];
    let recovered = [1.0; 3];

    // All three represented subtractions round to 1.0, while their exact low
    // terms are -2^-54, -2^-55, and -2^-1074. No single low-term anchor can
    // translate all three differences exactly, so the general scaled fallback
    // must retain the nonzero dispersion instead of collapsing it to zero.
    let standard_error =
        bias_standard_error(&truth, &recovered).expect("low-term spread remains representable");
    assert_eq!(standard_error.to_bits(), 0x3c72_79a7_4590_331c);

    let reversed_truth = [minimum_subnormal, truth[1], truth[0]];
    let reversed = bias_standard_error(&reversed_truth, &recovered)
        .expect("permutation preserves low-term spread");
    assert_eq!(reversed.to_bits(), standard_error.to_bits());
}
