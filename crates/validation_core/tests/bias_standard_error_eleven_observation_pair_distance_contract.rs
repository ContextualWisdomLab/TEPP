use validation_core::bias_standard_error;

fn assert_eleven_observation_pair_distance_contract(recovered: [f64; 11]) {
    let truth = [0.0; 11];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x41a4_47fc_a451_7b3f,
        "the exact eleven-observation pair-distance ratio must determine the represented-input SE; the translated floating moment/sqrt fallback rounds one ULP low"
    );
}

#[test]
fn exact_eleven_observation_dispersion_uses_pair_distance_midpoint_proof() {
    let samples = [
        [
            50_511_426.0,
            167_164_486.0,
            318_141_475.0,
            357_712_576.0,
            407_960_427.0,
            441_767_841.0,
            691_573_103.0,
            733_495_428.0,
            1_082_192_974.0,
            1_543_970_183.0,
            1_797_594_737.0,
        ],
        [
            1_797_594_737.0,
            1_543_970_183.0,
            1_082_192_974.0,
            733_495_428.0,
            691_573_103.0,
            441_767_841.0,
            407_960_427.0,
            357_712_576.0,
            318_141_475.0,
            167_164_486.0,
            50_511_426.0,
        ],
        [
            691_573_103.0,
            50_511_426.0,
            1_797_594_737.0,
            318_141_475.0,
            1_543_970_183.0,
            167_164_486.0,
            1_082_192_974.0,
            407_960_427.0,
            733_495_428.0,
            357_712_576.0,
            441_767_841.0,
        ],
    ];
    for recovered in samples {
        assert_eleven_observation_pair_distance_contract(recovered);
        assert_eleven_observation_pair_distance_contract(recovered.map(|value| -value));
    }
}
