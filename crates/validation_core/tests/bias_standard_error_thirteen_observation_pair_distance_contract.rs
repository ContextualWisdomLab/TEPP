use validation_core::bias_standard_error;

fn assert_thirteen_observation_pair_distance_contract(recovered: [f64; 13]) {
    let truth = [0.0; 13];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x41a2_9a8e_6db8_cb76,
        "the exact thirteen-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP high"
    );
}

#[test]
fn exact_thirteen_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            13_412_968.0,
            42_445_497.0,
            117_340_356.0,
            152_587_301.0,
            309_740_336.0,
            359_871_277.0,
            717_207_453.0,
            811_347_466.0,
            1_016_388_094.0,
            1_092_140_579.0,
            1_412_658_032.0,
            1_429_960_424.0,
            1_525_741_984.0,
        ],
        [
            1_525_741_984.0,
            1_429_960_424.0,
            1_412_658_032.0,
            1_092_140_579.0,
            1_016_388_094.0,
            811_347_466.0,
            717_207_453.0,
            359_871_277.0,
            309_740_336.0,
            152_587_301.0,
            117_340_356.0,
            42_445_497.0,
            13_412_968.0,
        ],
        [
            717_207_453.0,
            13_412_968.0,
            1_525_741_984.0,
            152_587_301.0,
            1_412_658_032.0,
            42_445_497.0,
            1_092_140_579.0,
            309_740_336.0,
            1_429_960_424.0,
            117_340_356.0,
            1_016_388_094.0,
            359_871_277.0,
            811_347_466.0,
        ],
    ];
    for recovered in samples {
        assert_thirteen_observation_pair_distance_contract(recovered);
        assert_thirteen_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
