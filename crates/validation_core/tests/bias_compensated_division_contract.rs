use validation_core::mean_bias;

#[test]
fn mean_bias_divides_compensated_numerator_without_double_rounding() {
    let two_to_minus_53 = 2.0_f64.powi(-53);
    let two_to_minus_52 = 2.0_f64.powi(-52);
    let truth = [0.0, 0.0, 0.0];
    let recovered = [
        two_to_minus_53,
        -two_to_minus_52,
        -(1.0 + two_to_minus_52),
    ];

    let bias = mean_bias(&truth, &recovered).expect("represented mean bias");
    assert_eq!(bias.to_bits(), 0xbfd5_5555_5555_5557);

    let mirrored: Vec<_> = recovered.iter().map(|value| -*value).collect();
    let mirrored_bias = mean_bias(&truth, &mirrored).expect("mirrored represented mean bias");
    assert_eq!(mirrored_bias.to_bits(), 0x3fd5_5555_5555_5557);
}
