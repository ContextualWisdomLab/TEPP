use model_selection::selected_k_recovery_summary;

#[test]
fn failure_rate_monte_carlo_error_uses_every_attempted_replication() {
    let summary = selected_k_recovery_summary(&[Some(4), None, Some(4), None], 4)
        .expect("recovery summary");

    assert!((summary.failure_rate() - 0.5).abs() < f64::EPSILON);
    assert!(
        (summary.failure_rate_monte_carlo_standard_error() - 0.25).abs() < f64::EPSILON
    );
}

#[test]
fn failure_rate_monte_carlo_error_is_zero_when_every_attempt_succeeds() {
    let summary = selected_k_recovery_summary(&[Some(4), Some(4), Some(4)], 4)
        .expect("recovery summary");

    assert!(summary.failure_rate().abs() < f64::EPSILON);
    assert!(
        summary
            .failure_rate_monte_carlo_standard_error()
            .abs()
            < f64::EPSILON
    );
}
