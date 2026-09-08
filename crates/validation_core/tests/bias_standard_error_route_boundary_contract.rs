use validation_core::{ValidationError, bias_standard_error};

#[test]
fn admitted_exact_route_refuses_unrepresentable_quarter_subnormal_standard_error() {
    let minimum_subnormal = f64::from_bits(1);
    let truth = [0.0; 4];
    let recovered = [minimum_subnormal, 0.0, 0.0, 0.0];

    assert_eq!(
        bias_standard_error(&truth, &recovered),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn generic_route_past_exact_budget_refuses_unrepresentable_subnormal_standard_error() {
    const SAMPLE_COUNT: usize = 17;
    let minimum_subnormal = f64::from_bits(1);
    let truth = [0.0; SAMPLE_COUNT];
    let mut recovered = [0.0; SAMPLE_COUNT];
    recovered[0] = minimum_subnormal;

    assert_eq!(
        bias_standard_error(&truth, &recovered),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn admitted_exact_route_preserves_zero_dispersion_for_minimum_subnormal_translation() {
    const SAMPLE_COUNT: usize = 16;
    let minimum_subnormal = f64::from_bits(1);
    let truth = [0.0; SAMPLE_COUNT];
    let recovered = [minimum_subnormal; SAMPLE_COUNT];

    assert_eq!(bias_standard_error(&truth, &recovered), Ok(0.0));
}
