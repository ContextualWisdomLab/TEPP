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

#[test]
fn mean_bias_uses_exact_subnormal_units_when_float_compensation_hits_a_halfway_case() {
    let truth = [0.0; 16];
    let recovered = [
        f64::from_bits(0x0cb6_6819_cb62_2e),
        f64::from_bits(0x0947_199e_fa89_8b),
        f64::from_bits(0x06f1_7b1c_5d14_35),
        f64::from_bits(0x0017_01d7_09e8_e5),
        f64::from_bits(0x0c4f_af0b_45b7_f2),
        f64::from_bits(0x0618_7545_3090_72),
        f64::from_bits(0x0a85_ae8e_ad81_bb),
        f64::from_bits(0x08c0_d3d1_61a3_65),
        f64::from_bits(0x0dd3_b523_39f6_96),
        f64::from_bits(0x0d7e_53f0_d4c2_46),
        f64::from_bits(0x0175_247a_3171_15),
        f64::from_bits(0x0c74_caf9_0802_51),
        f64::from_bits(0x0a8a_2b1f_baba_e0),
        f64::from_bits(0x04dd_bbf2_5f17_35),
        f64::from_bits(0x0efe_fdfe_b832_bd),
        f64::from_bits(0x0887_8ef3_1114_1d),
    ];

    // The exact unit sum leaves remainder 8 on division by 16, exactly halfway
    // between adjacent subnormals. The lower candidate is even and must win.
    let bias = mean_bias(&truth, &recovered).expect("halfway represented mean bias");
    assert_eq!(bias.to_bits(), 0x092d_f11e_7dd9_b8);
}
