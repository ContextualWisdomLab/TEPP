use validation_core::mean_bias;

#[test]
fn mean_bias_does_not_double_round_same_sign_subnormal_mean() {
    let truth = [0.0, 0.0, 0.0];
    let minimum_normal_units = 1_u64 << 52;
    let recovered = [
        f64::from_bits(minimum_normal_units - 32),
        f64::from_bits(minimum_normal_units - 12),
        f64::from_bits(minimum_normal_units - 20),
    ];

    // The exact represented-input mean is (3 * 2^52 - 64) / 3 subnormal units,
    // which rounds to 2^52 - 21. A normalized intermediate can round first to
    // a midpoint and then scale back to 2^52 - 22, so the public contract pins
    // the single-rounding result at the final binary64 scale.
    let bias = mean_bias(&truth, &recovered).expect("represented subnormal mean bias");
    assert_eq!(bias.to_bits(), minimum_normal_units - 21);

    let mirrored: Vec<_> = recovered.iter().map(|value| -*value).collect();
    let mirrored_bias = mean_bias(&truth, &mirrored).expect("mirrored subnormal mean bias");
    assert_eq!(mirrored_bias.to_bits(), (1_u64 << 63) | (minimum_normal_units - 21));
}

#[test]
fn mean_bias_rounds_subnormal_halfway_cases_to_even_units() {
    let truth = [0.0, 0.0];
    let minimum_subnormal = f64::from_bits(1);

    let odd_floor = [minimum_subnormal, f64::from_bits(2)];
    let odd_floor_bias = mean_bias(&truth, &odd_floor).expect("odd-floor halfway mean");
    assert_eq!(odd_floor_bias.to_bits(), 2);

    let even_floor = [f64::from_bits(2), f64::from_bits(3)];
    let even_floor_bias = mean_bias(&truth, &even_floor).expect("even-floor halfway mean");
    assert_eq!(even_floor_bias.to_bits(), 2);
}
