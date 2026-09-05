use validation_core::bias_standard_error;

fn assert_nine_observation_pair_distance_contract(recovered: [f64; 9]) {
    let truth = [0.0; 9];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x41a7_5f1f_6489_5d36,
        "the exact nine-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP low"
    );
}

#[test]
fn exact_nine_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            354_161_013.0,
            513_228_884.0,
            592_175_752.0,
            1_188_877_551.0,
            1_313_358_136.0,
            1_582_012_649.0,
            1_600_441_891.0,
            1_764_736_057.0,
            1_957_134_661.0,
        ],
        [
            1_957_134_661.0,
            1_764_736_057.0,
            1_600_441_891.0,
            1_582_012_649.0,
            1_313_358_136.0,
            1_188_877_551.0,
            592_175_752.0,
            513_228_884.0,
            354_161_013.0,
        ],
        [
            1_313_358_136.0,
            354_161_013.0,
            1_957_134_661.0,
            592_175_752.0,
            1_764_736_057.0,
            513_228_884.0,
            1_600_441_891.0,
            1_188_877_551.0,
            1_582_012_649.0,
        ],
    ];
    for recovered in samples {
        assert_nine_observation_pair_distance_contract(recovered);
        assert_nine_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
