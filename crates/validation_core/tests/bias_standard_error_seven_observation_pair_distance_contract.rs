use validation_core::bias_standard_error;

fn assert_seven_observation_pair_distance_contract(recovered: [f64; 7]) {
    let truth = [0.0; 7];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x4195_d9b7_0ca9_e6ee,
        "the exact seven-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP high"
    );
}

#[test]
fn exact_seven_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            912_628_433.0,
            991_631_865.0,
            1_109_185_293.0,
            1_253_685_899.0,
            1_354_619_842.0,
            1_368_206_500.0,
            1_611_374_925.0,
        ],
        [
            1_611_374_925.0,
            1_368_206_500.0,
            1_354_619_842.0,
            1_253_685_899.0,
            1_109_185_293.0,
            991_631_865.0,
            912_628_433.0,
        ],
        [
            1_253_685_899.0,
            912_628_433.0,
            1_611_374_925.0,
            1_109_185_293.0,
            1_368_206_500.0,
            991_631_865.0,
            1_354_619_842.0,
        ],
    ];
    for recovered in samples {
        assert_seven_observation_pair_distance_contract(recovered);
        assert_seven_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
