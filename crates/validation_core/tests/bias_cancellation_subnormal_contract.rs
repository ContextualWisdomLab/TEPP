use validation_core::mean_bias;

#[test]
fn extreme_cancellation_preserves_representable_subnormal_bias() {
    let minimum_subnormal = f64::from_bits(1);
    let twice_minimum_subnormal = f64::from_bits(2);
    let truth = [0.0; 4];

    let positive_bias = [
        f64::MAX,
        twice_minimum_subnormal,
        twice_minimum_subnormal,
        -f64::MAX,
    ];
    let recovered_positive = mean_bias(&truth, &positive_bias).expect("representable positive bias");
    assert_eq!(recovered_positive.to_bits(), minimum_subnormal.to_bits());

    let negative_bias = [
        -f64::MAX,
        -twice_minimum_subnormal,
        -twice_minimum_subnormal,
        f64::MAX,
    ];
    let recovered_negative = mean_bias(&truth, &negative_bias).expect("representable negative bias");
    assert_eq!(
        recovered_negative.to_bits(),
        (-minimum_subnormal).to_bits()
    );
}

#[test]
fn mixed_sign_bias_is_canonical_under_transport_permutation() {
    let truth = [0.0; 3];
    let first = [3.0, -1.0, -1.0];
    let permuted = [-1.0, 3.0, -1.0];
    let expected = 1.0 / 3.0;

    let first_bias = mean_bias(&truth, &first).expect("first bias");
    let permuted_bias = mean_bias(&truth, &permuted).expect("permuted bias");
    assert_eq!(first_bias.to_bits(), expected.to_bits());
    assert_eq!(permuted_bias.to_bits(), first_bias.to_bits());

    let negative = [-3.0, 1.0, 1.0];
    let negative_bias = mean_bias(&truth, &negative).expect("negative bias");
    assert_eq!(negative_bias.to_bits(), (-expected).to_bits());
}

#[test]
fn full_range_exact_cancellation_remains_exact_zero() {
    let minimum_subnormal = f64::from_bits(1);
    let truth = [0.0; 4];
    let recovered = [
        f64::MAX,
        minimum_subnormal,
        -f64::MAX,
        -minimum_subnormal,
    ];

    assert_eq!(mean_bias(&truth, &recovered), Ok(0.0));
}
