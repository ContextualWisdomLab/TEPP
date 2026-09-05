use validation_core::bias_standard_error;

fn assert_twelve_observation_pair_distance_contract(recovered: [f64; 12]) {
    let truth = [0.0; 12];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x41a6_5ddb_5161_045f,
        "the exact twelve-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP high"
    );
}

#[test]
fn exact_twelve_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            18_775_780.0,
            73_991_125.0,
            198_689_967.0,
            631_050_858.0,
            778_682_730.0,
            826_435_964.0,
            853_584_967.0,
            1_530_809_509.0,
            1_562_270_376.0,
            1_586_067_346.0,
            1_682_017_356.0,
            1_750_122_820.0,
        ],
        [
            1_750_122_820.0,
            1_682_017_356.0,
            1_586_067_346.0,
            1_562_270_376.0,
            1_530_809_509.0,
            853_584_967.0,
            826_435_964.0,
            778_682_730.0,
            631_050_858.0,
            198_689_967.0,
            73_991_125.0,
            18_775_780.0,
        ],
        [
            826_435_964.0,
            18_775_780.0,
            1_750_122_820.0,
            198_689_967.0,
            1_682_017_356.0,
            73_991_125.0,
            1_530_809_509.0,
            778_682_730.0,
            1_586_067_346.0,
            631_050_858.0,
            1_562_270_376.0,
            853_584_967.0,
        ],
    ];
    for recovered in samples {
        assert_twelve_observation_pair_distance_contract(recovered);
        assert_twelve_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
