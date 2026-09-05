use validation_core::bias_standard_error;

const EXPECTED_STANDARD_ERROR_BITS: u64 = 0x0009_3cd3_a2c8_198e;

fn represented_sample() -> [f64; 3] {
    [0.0, f64::MIN_POSITIVE, 2.0 * f64::MIN_POSITIVE]
}

#[test]
fn nonzero_three_level_standard_error_survives_square_underflow() {
    let [low, middle, high] = represented_sample();
    let truth = [0.0; 3];
    let permutations = [
        [low, middle, high],
        [low, high, middle],
        [middle, low, high],
        [middle, high, low],
        [high, low, middle],
        [high, middle, low],
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
