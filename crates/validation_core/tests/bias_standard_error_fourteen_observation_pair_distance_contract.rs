use validation_core::bias_standard_error;

fn assert_fourteen_observation_pair_distance_contract(recovered: [f64; 14]) {
    let truth = [0.0; 14];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x41a2_3df9_5954_fb0b,
        "the exact fourteen-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP low"
    );
}

#[test]
fn exact_fourteen_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            169_198_177.0,
            170_212_614.0,
            363_421_213.0,
            482_119_205.0,
            503_813_918.0,
            556_586_639.0,
            757_346_256.0,
            811_004_051.0,
            882_684_595.0,
            948_393_523.0,
            1_052_267_532.0,
            1_523_536_361.0,
            1_895_880_649.0,
            1_922_535_250.0,
        ],
        [
            1_922_535_250.0,
            1_895_880_649.0,
            1_523_536_361.0,
            1_052_267_532.0,
            948_393_523.0,
            882_684_595.0,
            811_004_051.0,
            757_346_256.0,
            556_586_639.0,
            503_813_918.0,
            482_119_205.0,
            363_421_213.0,
            170_212_614.0,
            169_198_177.0,
        ],
        [
            757_346_256.0,
            169_198_177.0,
            1_922_535_250.0,
            482_119_205.0,
            1_523_536_361.0,
            170_212_614.0,
            948_393_523.0,
            503_813_918.0,
            1_895_880_649.0,
            363_421_213.0,
            1_052_267_532.0,
            556_586_639.0,
            882_684_595.0,
            811_004_051.0,
        ],
    ];
    for recovered in samples {
        assert_fourteen_observation_pair_distance_contract(recovered);
        assert_fourteen_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
