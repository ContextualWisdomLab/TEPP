//! Mention confidence recovers known Brier scores against binary truth.

use event_core::{EventConfidence, EventError, mention_brier_score};

#[test]
fn perfectly_calibrated_forecasts_recover_zero_brier() {
    let forecasts = [
        EventConfidence::new(0.0).expect("0"),
        EventConfidence::new(1.0).expect("1"),
        EventConfidence::new(0.0).expect("0"),
        EventConfidence::new(1.0).expect("1"),
    ];
    let outcomes = [false, true, false, true];
    let score = mention_brier_score(&forecasts, &outcomes).expect("brier");
    assert!(score.abs() < 1e-15, "perfect Brier {score}");
}

#[test]
fn constant_half_recovers_quarter_and_mismatches_fail_closed() {
    let forecasts = [
        EventConfidence::new(0.5).expect("half"),
        EventConfidence::new(0.5).expect("half"),
    ];
    let outcomes = [false, true];
    let score = mention_brier_score(&forecasts, &outcomes).expect("half");
    let residual = score - 0.25;
    let rmse = (residual * residual).sqrt();
    assert!(rmse < 1e-15, "Brier RMSE {rmse}");
    assert_eq!(
        mention_brier_score(&forecasts, &[true]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        mention_brier_score(&[], &[]),
        Err(EventError::InvalidWirePayload)
    );
}
