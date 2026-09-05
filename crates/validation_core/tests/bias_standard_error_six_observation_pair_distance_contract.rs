use validation_core::bias_standard_error;

fn assert_six_observation_pair_distance_contract(recovered: [f64; 6]) {
    let truth = [0.0; 6];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x419c_057d_42fc_5857,
        "the exact six-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP high"
    );
}

#[test]
fn exact_six_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            1_120_315_269.0,
            1_513_609_015.0,
            1_569_037_659.0,
            1_789_057_504.0,
            1_807_936_669.0,
            1_914_796_738.0,
        ],
        [
            1_914_796_738.0,
            1_569_037_659.0,
            1_120_315_269.0,
            1_807_936_669.0,
            1_513_609_015.0,
            1_789_057_504.0,
        ],
        [
            1_569_037_659.0,
            1_120_315_269.0,
            1_914_796_738.0,
            1_513_609_015.0,
            1_789_057_504.0,
            1_807_936_669.0,
        ],
    ];
    for recovered in samples {
        assert_six_observation_pair_distance_contract(recovered);
        assert_six_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
