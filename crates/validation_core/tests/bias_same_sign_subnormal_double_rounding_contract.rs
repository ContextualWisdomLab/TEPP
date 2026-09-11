//! Contracts exact binary64 mean-bias rounding for same-sign subnormal inputs.
//!
//! These cases guard the final represented-value decision against double rounding:
//! accumulation may cross the normal/subnormal boundary or land exactly halfway
//! between subnormal units, so the public metric must round once at the final scale
//! with IEEE 754 ties-to-even semantics.

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
    assert_eq!(
        mirrored_bias.to_bits(),
        (1_u64 << 63) | (minimum_normal_units - 21)
    );
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
        f64::from_bits(0x000c_b668_19cb_622e),
        f64::from_bits(0x0009_4719_9efa_898b),
        f64::from_bits(0x0006_f17b_1c5d_1435),
        f64::from_bits(0x0000_1701_d709_e8e5),
        f64::from_bits(0x000c_4faf_0b45_b7f2),
        f64::from_bits(0x0006_1875_4530_9072),
        f64::from_bits(0x000a_85ae_8ead_81bb),
        f64::from_bits(0x0008_c0d3_d161_a365),
        f64::from_bits(0x000d_d3b5_2339_f696),
        f64::from_bits(0x000d_7e53_f0d4_c246),
        f64::from_bits(0x0001_7524_7a31_7115),
        f64::from_bits(0x000c_74ca_f908_0251),
        f64::from_bits(0x000a_8a2b_1fba_bae0),
        f64::from_bits(0x0004_ddbb_f25f_1735),
        f64::from_bits(0x000e_fefd_feb8_32bd),
        f64::from_bits(0x0008_878e_f311_141d),
    ];

    // The exact unit sum leaves remainder 8 on division by 16, exactly halfway
    // between adjacent subnormals. The lower candidate is even and must win.
    let bias = mean_bias(&truth, &recovered).expect("halfway represented mean bias");
    assert_eq!(bias.to_bits(), 0x0009_2df1_1e7d_d9b8);
}
