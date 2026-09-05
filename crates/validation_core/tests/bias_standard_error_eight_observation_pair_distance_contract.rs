use validation_core::bias_standard_error;

fn assert_eight_observation_pair_distance_contract(recovered: [f64; 8]) {
    let truth = [0.0; 8];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x41ac_8601_97ac_cd4c,
        "the exact eight-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP high"
    );
}

#[test]
fn exact_eight_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            121_838_114.0,
            741_324_193.0,
            994_684_355.0,
            1_673_429_661.0,
            1_824_103_795.0,
            1_861_311_798.0,
            1_872_117_478.0,
            1_936_395_613.0,
        ],
        [
            1_936_395_613.0,
            1_872_117_478.0,
            1_861_311_798.0,
            1_824_103_795.0,
            1_673_429_661.0,
            994_684_355.0,
            741_324_193.0,
            121_838_114.0,
        ],
        [
            1_824_103_795.0,
            121_838_114.0,
            1_936_395_613.0,
            994_684_355.0,
            1_872_117_478.0,
            741_324_193.0,
            1_861_311_798.0,
            1_673_429_661.0,
        ],
    ];
    for recovered in samples {
        assert_eight_observation_pair_distance_contract(recovered);
        assert_eight_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
