//! Exact-midpoint tie-to-even regression contracts for four-observation bias SE.

use validation_core::bias_standard_error;

#[test]
fn exact_midpoint_selects_lower_even_binary64_neighbor() {
    let truth = [0.0; 4];
    let recovered = [
        0.0,
        0.0,
        8_106_479_329_266_891.0,
        9_007_199_254_740_990.0,
    ];

    let standard_error =
        bias_standard_error(&truth, &recovered).expect("finite exact-midpoint standard error");

    assert_eq!(standard_error.to_bits(), 0x4321_9999_9999_9998);
}

#[test]
fn exact_midpoint_selects_upper_even_binary64_neighbor() {
    let truth = [0.0; 4];
    let recovered = [
        0.0,
        0.0,
        8_106_479_329_266_873.0,
        9_007_199_254_740_970.0,
    ];

    let standard_error =
        bias_standard_error(&truth, &recovered).expect("finite exact-midpoint standard error");

    assert_eq!(standard_error.to_bits(), 0x4321_9999_9999_998e);
}
