use validation_core::bias_standard_error;

fn assert_fifteen_observation_pair_distance_contract(recovered: [f64; 15]) {
    let truth = [0.0; 15];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x41a1_254f_de99_720d,
        "the exact fifteen-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP low"
    );
}

#[test]
fn exact_fifteen_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            224_611_356.0,
            291_740_781.0,
            326_373_353.0,
            461_196_594.0,
            686_313_913.0,
            812_468_689.0,
            889_538_573.0,
            1_089_098_727.0,
            1_093_012_981.0,
            1_094_199_400.0,
            1_387_143_595.0,
            1_412_604_591.0,
            1_556_072_759.0,
            1_847_457_618.0,
            1_990_087_657.0,
        ],
        [
            1_990_087_657.0,
            1_847_457_618.0,
            1_556_072_759.0,
            1_412_604_591.0,
            1_387_143_595.0,
            1_094_199_400.0,
            1_093_012_981.0,
            1_089_098_727.0,
            889_538_573.0,
            812_468_689.0,
            686_313_913.0,
            461_196_594.0,
            326_373_353.0,
            291_740_781.0,
            224_611_356.0,
        ],
        [
            1_089_098_727.0,
            224_611_356.0,
            1_990_087_657.0,
            461_196_594.0,
            1_412_604_591.0,
            291_740_781.0,
            1_094_199_400.0,
            686_313_913.0,
            1_847_457_618.0,
            326_373_353.0,
            1_387_143_595.0,
            812_468_689.0,
            1_556_072_759.0,
            889_538_573.0,
            1_093_012_981.0,
        ],
    ];
    for recovered in samples {
        assert_fifteen_observation_pair_distance_contract(recovered);
        assert_fifteen_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
