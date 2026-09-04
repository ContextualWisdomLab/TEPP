use validation_core::mean_bias;

fn assert_bias_bits(truth: [f64; 2], recovered: [f64; 2], expected: u64) {
    let bias = mean_bias(&truth, &recovered).expect("represented mean bias");
    assert_eq!(bias.to_bits(), expected);

    let mirrored_truth = truth.map(|value| -value);
    let mirrored_recovered = recovered.map(|value| -value);
    let mirrored_bias = mean_bias(&mirrored_truth, &mirrored_recovered)
        .expect("mirrored represented mean bias");
    assert_eq!(mirrored_bias.to_bits(), expected | (1_u64 << 63));
}

#[test]
fn mean_bias_preserves_pairwise_subtraction_roundoff_before_averaging() {
    assert_bias_bits(
        [2.0_f64.powi(-108), 2.0_f64.powi(-53)],
        [2.0_f64.powi(-54), 1.0],
        0x3fdf_ffff_ffff_ffff,
    );

    assert_bias_bits(
        [2.0_f64.powi(-105), 2.0_f64.powi(-53)],
        [2.0_f64.powi(-51), 1.0],
        0x3fe0_0000_0000_0001,
    );
}
