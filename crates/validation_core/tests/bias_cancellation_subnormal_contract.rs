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
