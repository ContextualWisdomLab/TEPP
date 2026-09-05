use validation_core::bias_standard_error;

fn assert_large_reduced_ratio_contract(recovered: [f64; 4]) {
    let truth = [0.0; 4];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x41b3_a706_d408_9e32,
        "the exact reduced pair-distance ratio remains authoritative even when its numerator exceeds 2^53; the floating ratio/sqrt fallback rounds one ULP low"
    );
}

#[test]
fn exact_four_observation_dispersion_keeps_large_reduced_ratio_in_midpoint_proof() {
    let samples = [
        [19_274_968.0, 693_729_138.0, 711_353_557.0, 1_625_519_116.0],
        [1_625_519_116.0, 711_353_557.0, 693_729_138.0, 19_274_968.0],
        [693_729_138.0, 1_625_519_116.0, 19_274_968.0, 711_353_557.0],
        [711_353_557.0, 19_274_968.0, 1_625_519_116.0, 693_729_138.0],
    ];
    for recovered in samples {
        assert_large_reduced_ratio_contract(recovered);
        assert_large_reduced_ratio_contract(recovered.map(|value| -value));
    }
}
