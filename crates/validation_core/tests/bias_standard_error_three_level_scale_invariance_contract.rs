use validation_core::bias_standard_error;

const EXPECTED_STANDARD_ERROR_BITS: u64 = 0x64f9_5555_5555_5555;

fn represented_sample() -> [f64; 3] {
    [
        0.0,
        f64::from_bits(0x64f4_0000_0000_0000),
        f64::from_bits(0x6515_0000_0000_0000),
    ]
}

#[test]
fn exact_three_level_rational_scale_is_invariant_under_exact_power_of_two_scaling() {
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
