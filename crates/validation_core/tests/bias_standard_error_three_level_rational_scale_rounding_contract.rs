use validation_core::bias_standard_error;

const EXPECTED_STANDARD_ERROR_BITS: u64 = 0x3f79_5555_5555_5555;

fn represented_sample() -> [f64; 3] {
    [
        0.0,
        f64::from_bits(0x3f74_0000_0000_0000),
        f64::from_bits(0x3f95_0000_0000_0000),
    ]
}

#[test]
fn exact_three_level_rational_scale_preserves_correct_rounding() {
    let [low, middle, high] = represented_sample();
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
