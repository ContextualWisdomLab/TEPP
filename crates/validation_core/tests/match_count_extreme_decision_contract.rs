use validation_core::{ValidationError, absolute_residuals, match_count};

#[test]
fn tolerance_match_decision_does_not_require_an_unrepresentable_residual() {
    let truth = [f64::MAX];
    let recovered = [-f64::MAX];

    assert_eq!(
        absolute_residuals(&truth, &recovered),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(match_count(&truth, &recovered, f64::MAX), Ok(0));
    assert_eq!(match_count(&truth, &recovered, 0.0), Ok(0));

    assert_eq!(match_count(&truth, &truth, 0.0), Ok(1));
}
