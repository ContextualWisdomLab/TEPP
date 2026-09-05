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
