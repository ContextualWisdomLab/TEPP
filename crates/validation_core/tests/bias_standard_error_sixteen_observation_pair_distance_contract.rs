use validation_core::bias_standard_error;

fn assert_sixteen_observation_pair_distance_contract(recovered: [f64; 16]) {
    let truth = [0.0; 16];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x419c_fcbb_b78d_2ad5,
        "the exact sixteen-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP low"
    );
}

#[test]
fn exact_sixteen_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            314_270_929.0,
            327_661_307.0,
            371_854_441.0,
            398_522_837.0,
            413_483_290.0,
            416_184_956.0,
            565_808_551.0,
            682_627_163.0,
            724_514_517.0,
            731_058_943.0,
            740_662_035.0,
            970_233_120.0,
            1_141_566_755.0,
            1_320_628_283.0,
            1_526_331_271.0,
            1_992_574_092.0,
        ],
        [
            1_992_574_092.0,
            1_526_331_271.0,
            1_320_628_283.0,
            1_141_566_755.0,
            970_233_120.0,
            740_662_035.0,
            731_058_943.0,
            724_514_517.0,
            682_627_163.0,
            565_808_551.0,
            416_184_956.0,
            413_483_290.0,
            398_522_837.0,
            371_854_441.0,
            327_661_307.0,
            314_270_929.0,
        ],
        [
            682_627_163.0,
            314_270_929.0,
            1_992_574_092.0,
            398_522_837.0,
            1_141_566_755.0,
            327_661_307.0,
            731_058_943.0,
            413_483_290.0,
            1_526_331_271.0,
            371_854_441.0,
            970_233_120.0,
            416_184_956.0,
            1_320_628_283.0,
            565_808_551.0,
            740_662_035.0,
            724_514_517.0,
        ],
    ];
    for recovered in samples {
        assert_sixteen_observation_pair_distance_contract(recovered);
        assert_sixteen_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
