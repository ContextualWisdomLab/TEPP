use validation_core::bias_standard_error;

fn assert_ten_observation_pair_distance_contract(recovered: [f64; 10]) {
    let truth = [0.0; 10];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x41a5_e8a1_0795_bf6c,
        "the exact ten-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP low"
    );
}

#[test]
fn exact_ten_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            418_906_532.0,
            431_535_003.0,
            554_793_949.0,
            905_115_878.0,
            1_079_195_036.0,
            1_340_223_962.0,
            1_591_821_052.0,
            1_592_008_872.0,
            1_807_262_929.0,
            1_996_099_302.0,
        ],
        [
            1_996_099_302.0,
            1_807_262_929.0,
            1_592_008_872.0,
            1_591_821_052.0,
            1_340_223_962.0,
            1_079_195_036.0,
            905_115_878.0,
            554_793_949.0,
            431_535_003.0,
            418_906_532.0,
        ],
        [
            1_079_195_036.0,
            418_906_532.0,
            1_996_099_302.0,
            554_793_949.0,
            1_807_262_929.0,
            431_535_003.0,
            1_592_008_872.0,
            905_115_878.0,
            1_591_821_052.0,
            1_340_223_962.0,
        ],
    ];
    for recovered in samples {
        assert_ten_observation_pair_distance_contract(recovered);
        assert_ten_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
