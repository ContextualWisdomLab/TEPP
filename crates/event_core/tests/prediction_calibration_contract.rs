//! CHRONOS occurrence forecasts stay hypothetical and recover Brier scores.

use event_core::{
    ChronosOccurrenceForecast, ChronosPredictionId, EventConfidence, EventError, OccurrenceTruth,
    chronos_prediction_brier_score, refuse_prediction_as_instance,
};

fn forecast(raw: u32, probability: f64) -> ChronosOccurrenceForecast {
    ChronosOccurrenceForecast::new(
        ChronosPredictionId::from_raw(raw),
        EventConfidence::new(probability).expect("probability"),
    )
}

#[test]
fn chronos_prediction_cannot_be_cast_to_an_instance() {
    let prediction = ChronosPredictionId::from_raw(7);
    assert_eq!(
        refuse_prediction_as_instance(prediction),
        Err(EventError::PredictionIsNotEventInstance)
    );
}

#[test]
fn perfect_occurrence_forecasts_recover_zero_brier() {
    let forecasts = [forecast(1, 1.0), forecast(2, 0.0), forecast(3, 1.0)];
    let outcomes = [
        OccurrenceTruth::Occurred,
        OccurrenceTruth::DidNotOccur,
        OccurrenceTruth::Occurred,
    ];
    let score = chronos_prediction_brier_score(&forecasts, &outcomes).expect("brier");
    assert!(score.abs() < 1e-15, "perfect Brier {score}");
}

#[test]
fn calibrated_forecasts_beat_overconfident_always_occur_and_mismatches_fail_closed() {
    let calibrated = [forecast(1, 0.8), forecast(2, 0.2), forecast(3, 0.7)];
    let overconfident = [forecast(1, 1.0), forecast(2, 1.0), forecast(3, 1.0)];
    let outcomes = [
        OccurrenceTruth::Occurred,
        OccurrenceTruth::DidNotOccur,
        OccurrenceTruth::Occurred,
    ];
    let calibrated_brier = chronos_prediction_brier_score(&calibrated, &outcomes).expect("cal");
    let naive_brier = chronos_prediction_brier_score(&overconfident, &outcomes).expect("naive");
    assert!(
        calibrated_brier < naive_brier,
        "calibrated Brier {calibrated_brier} must be below always-occur Brier {naive_brier}"
    );
    assert_eq!(
        chronos_prediction_brier_score(&calibrated, &[OccurrenceTruth::Occurred]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        chronos_prediction_brier_score(&[], &[]),
        Err(EventError::InvalidWirePayload)
    );
}
