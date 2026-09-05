use validation_core::bias_standard_error;

const EXPECTED_STANDARD_ERROR_BITS: u64 = 0x3fd1_7a99_c875_b980;

fn represented_sample() -> [f64; 3] {
    [
        f64::from_bits(0x3ff7_c8a6_308f_7624),
        f64::from_bits(0x3ff0_4284_fcf1_21a0),
        f64::from_bits(0x3fff_659d_6d25_7410),
    ]
}

#[test]
fn exact_translated_anchor_conditioning_preserves_correct_rounding() {
    let [middle, low, high] = represented_sample();
    let truth = [0.0; 3];
    let permutations = [
        [low, middle, high],
        [middle, low, high],
        [high, low, middle],
    ];

    for recovered in permutations {
        let standard_error = bias_standard_error(&truth, &recovered).expect("finite standard error");
        assert_eq!(standard_error.to_bits(), EXPECTED_STANDARD_ERROR_BITS);

        let mirrored = recovered.map(|value| -value);
        let mirrored_standard_error =
            bias_standard_error(&truth, &mirrored).expect("finite mirrored standard error");
        assert_eq!(mirrored_standard_error.to_bits(), EXPECTED_STANDARD_ERROR_BITS);
    }
}
