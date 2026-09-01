//! CWC-then-irregular residual log-rate is not raw-process drift.

use longitudinal_core::{
    EventTimedObservation, LaggedWithinResidual, LongitudinalError, center_within_unit_event_lags,
    recover_centered_irregular_residual_log_rate, recover_within_unit_irregular_residual_log_rate,
    refuse_cwc_residual_log_rate_as_raw_process_drift,
};

fn timed(unit: u32, event_time: f64, score: f64) -> EventTimedObservation {
    EventTimedObservation::new(unit, event_time, score)
}

#[test]
fn already_centered_unit_interval_recovers_ln_half() {
    let pair = LaggedWithinResidual::new(
        1.0,
        0.5,
        longitudinal_core::EventTimeInterval::new(1.0).expect("event time"),
    );
    let recovered = recover_centered_irregular_residual_log_rate(&[pair]).expect("ln(0.5)");
    assert!((recovered - 0.5_f64.ln()).abs() < f64::EPSILON);
}

#[test]
fn cwc_mean_is_deterministic_under_input_row_permutation() {
    let canonical = [
        timed(1, 0.0, 1.0e16),
        timed(1, 1.0, -1.0e16),
        timed(1, 2.0, 1.0),
        timed(2, 0.0, 2.0),
        timed(2, 1.0, 1.0),
    ];
    let shuffled = [
        timed(1, 0.0, 1.0e16),
        timed(1, 2.0, 1.0),
        timed(1, 1.0, -1.0e16),
        timed(2, 1.0, 1.0),
        timed(2, 0.0, 2.0),
    ];

    let expected = center_within_unit_event_lags(&canonical).expect("canonical ordering");
    let reordered = center_within_unit_event_lags(&shuffled).expect("shuffled ordering");
    assert_eq!(reordered, expected, "CWC output must not depend on input row order");
}

#[test]
fn cwc_of_raw_autoregressive_path_is_not_process_drift() {
    let drift = (0.5_f64).ln();
    let rows = [
        timed(0, 0.0, 8.0 + 1.0),
        timed(0, 1.0, 8.0 + drift.exp()),
        timed(0, 2.5, 8.0 + (drift * 2.5).exp()),
        timed(1, 0.0, 3.0 + 1.0),
        timed(1, 0.8, 3.0 + (drift * 0.8).exp()),
        timed(1, 2.0, 3.0 + (drift * 2.0).exp()),
    ];
    let composed = recover_within_unit_irregular_residual_log_rate(&rows).expect("cwc pairwise");
    assert!(
        (composed - drift).abs() > 1e-4,
        "Curran & Bauer: CWC of a time-related AR path must not recover raw drift"
    );
    assert_eq!(
        refuse_cwc_residual_log_rate_as_raw_process_drift(composed, drift),
        Err(LongitudinalError::CwcResidualLogRateIsNotRawProcessDrift)
    );
    let extracted = center_within_unit_event_lags(&rows).expect("pairs");
    assert!(extracted.len() >= 2);
}
