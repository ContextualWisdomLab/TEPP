//! Preserve low-order opposing residual mass through represented mean-bias division.
//!
//! Four quarter-ULP residuals opposing a unit residual combine to one full ULP before the
//! scientific denominator is applied. Aggregation must therefore retain their joint represented
//! mass instead of rounding each contribution away, and sign mirroring must change only the sign
//! of the correctly rounded mean bias.

use validation_core::mean_bias;

#[test]
fn mixed_sign_bias_preserves_small_opposing_mass_before_division() {
    let quarter_ulp_at_one = 2.0_f64.powi(-54);
    let truth = [
        0.0,
        quarter_ulp_at_one,
        quarter_ulp_at_one,
        quarter_ulp_at_one,
        quarter_ulp_at_one,
    ];
    let recovered = [1.0, 0.0, 0.0, 0.0, 0.0];

    let bias = mean_bias(&truth, &recovered)
        .expect("the represented-input mean bias is finite and representable");

    assert_eq!(
        bias.to_bits(),
        0x3fc9_9999_9999_9998,
        "four quarter-ulp opposing residuals sum to one full ulp before division and must not be rounded away one at a time"
    );
}

#[test]
fn mixed_sign_bias_roundoff_contract_is_sign_symmetric() {
    let quarter_ulp_at_one = 2.0_f64.powi(-54);
    let truth = [
        0.0,
        -quarter_ulp_at_one,
        -quarter_ulp_at_one,
        -quarter_ulp_at_one,
        -quarter_ulp_at_one,
    ];
    let recovered = [-1.0, 0.0, 0.0, 0.0, 0.0];

    let bias = mean_bias(&truth, &recovered)
        .expect("the represented-input mean bias is finite and representable");

    assert_eq!(bias.to_bits(), (-f64::from_bits(0x3fc9_9999_9999_9998)).to_bits());
}
