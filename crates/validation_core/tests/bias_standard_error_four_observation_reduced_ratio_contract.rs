use validation_core::bias_standard_error;

fn assert_reduced_ratio_contract(recovered: [f64; 4]) {
    let truth = [0.0; 4];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x4174_46e5_76f8_7445,
        "exact pair-distance ratio reduces by gcd 4 before the bounded sqrt proof; fallback rounds one ULP low"
    );
}

#[test]
fn exact_four_observation_dispersion_reduces_ratio_before_bounded_sqrt_proof() {
    let samples = [
        [0.0, 14_099_687.0, 16_729_100.0, 94_045_527.0],
        [94_045_527.0, 16_729_100.0, 14_099_687.0, 0.0],
        [14_099_687.0, 94_045_527.0, 0.0, 16_729_100.0],
        [16_729_100.0, 0.0, 94_045_527.0, 14_099_687.0],
    ];
    for recovered in samples {
        assert_reduced_ratio_contract(recovered);
        assert_reduced_ratio_contract(recovered.map(|value| -value));
    }
}
