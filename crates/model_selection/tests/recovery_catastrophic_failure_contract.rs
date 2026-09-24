use model_selection::selected_k_recovery_summary;

#[test]
fn all_failed_recovery_still_preserves_the_unconditional_denominator() {
    let summary = selected_k_recovery_summary(&[None, None, None, None], 4)
        .expect("catastrophic recovery failure is itself scientific evidence");

    assert_eq!(summary.replication_count(), 4);
    assert_eq!(summary.success_count(), 0);
    assert_eq!(summary.failure_count(), 4);
    assert!((summary.failure_rate() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn one_success_does_not_erase_the_remaining_failed_attempts() {
    let summary = selected_k_recovery_summary(&[None, Some(5), None], 4)
        .expect("near-catastrophic recovery retains its denominator");

    assert_eq!(summary.replication_count(), 3);
    assert_eq!(summary.success_count(), 1);
    assert_eq!(summary.failure_count(), 2);
    assert!((summary.failure_rate() - (2.0 / 3.0)).abs() < f64::EPSILON);
}
