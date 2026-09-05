use validation_core::bias_standard_error;

fn assert_five_observation_pair_distance_contract(recovered: [f64; 5]) {
    let truth = [0.0; 5];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x4192_caf1_6406_5ad0,
        "the exact five-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP high"
    );
}

#[test]
fn exact_five_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            1_342_748_146.0,
            1_434_848_064.0,
            1_525_257_611.0,
            1_685_877_224.0,
            1_771_341_094.0,
        ],
        [
            1_771_341_094.0,
            1_525_257_611.0,
            1_342_748_146.0,
            1_685_877_224.0,
            1_434_848_064.0,
        ],
        [
            1_525_257_611.0,
            1_342_748_146.0,
            1_771_341_094.0,
            1_434_848_064.0,
            1_685_877_224.0,
        ],
    ];
    for recovered in samples {
        assert_five_observation_pair_distance_contract(recovered);
        assert_five_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
