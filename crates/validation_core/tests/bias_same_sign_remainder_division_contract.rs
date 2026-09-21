//! Preserve compensated same-sign remainder mass through the scientific sample-count division.
//!
//! After mixed-sign cancellation leaves a same-sign low-order remainder, the compensated numerator
//! must be divided before it is collapsed to a coarser binary64 value. The mirrored fixture fixes
//! the same correctly rounded magnitude with the opposite sign.

use validation_core::mean_bias;

#[test]
fn mean_bias_keeps_same_sign_remainder_compensation_through_division() {
    let truth = [0.0, 0.0, 0.0];
    let recovered = [
        f64::from_bits(0x3fc0_0000_0000_0004),
        f64::from_bits(0x3fbf_ffff_ffff_fffc),
        f64::from_bits(0xbfbf_ffff_ffff_fffd),
    ];

    let bias = mean_bias(&truth, &recovered).expect("represented mean bias");
    assert_eq!(bias.to_bits(), 0x3fa5_5555_5555_555a);

    let mirrored: Vec<_> = recovered.iter().map(|value| -*value).collect();
    let mirrored_bias = mean_bias(&truth, &mirrored).expect("mirrored represented mean bias");
    assert_eq!(mirrored_bias.to_bits(), 0xbfa5_5555_5555_555a);
}
