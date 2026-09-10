use validation_core::{ValidationError, bias_standard_error};

#[test]
fn positive_subnormal_bias_standard_error_below_binary64_range_fails_closed() {
    let quantum = f64::from_bits(1);
    let twice_quantum = f64::from_bits(2);
    let truth = [0.0; 4];

    let recovered = [0.0, quantum, twice_quantum, twice_quantum];
    assert_eq!(
        bias_standard_error(&truth, &recovered),
        Err(ValidationError::InvalidInput)
    );

    let permuted = [twice_quantum, 0.0, twice_quantum, quantum];
    assert_eq!(
        bias_standard_error(&truth, &permuted),
        Err(ValidationError::InvalidInput)
    );
}
