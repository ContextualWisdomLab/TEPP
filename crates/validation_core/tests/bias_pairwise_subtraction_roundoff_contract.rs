use validation_core::mean_bias;

#[test]
fn mean_bias_preserves_pairwise_subtraction_roundoff_before_averaging() {
    let truth = [2.0_f64.powi(-108), 2.0_f64.powi(-53)];
    let recovered = [2.0_f64.powi(-54), 1.0];

    let bias = mean_bias(&truth, &recovered).expect("represented mean bias");
    assert_eq!(bias.to_bits(), 0x3fdf_ffff_ffff_ffff);

    let mirrored_truth = truth.map(|value| -value);
    let mirrored_recovered = recovered.map(|value| -value);
    let mirrored_bias = mean_bias(&mirrored_truth, &mirrored_recovered)
        .expect("mirrored represented mean bias");
    assert_eq!(mirrored_bias.to_bits(), 0xbfdf_ffff_ffff_ffff);
}
