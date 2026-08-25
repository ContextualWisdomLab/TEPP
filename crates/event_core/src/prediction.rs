//! CHRONOS occurrence forecasts stay hypothetical until later evidence.

use crate::{EventConfidence, EventError, EventInstanceId};

/// Opaque CHRONOS occurrence-prediction identity.
///
/// A forecast is hypothesized future or schema-completion evidence. It is
/// never a promoted event instance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ChronosPredictionId(u32);

impl ChronosPredictionId {
    /// Reconstruct a prediction identity from a raw fixture or estimator label.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Return the raw prediction label.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Later-observed occurrence truth for a CHRONOS forecast.
///
/// Truth is recovered from later evidence. It does not rewrite the forecast
/// into an event instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OccurrenceTruth {
    /// Later evidence established that the predicted event occurred.
    Occurred,
    /// Later evidence established that the predicted event did not occur.
    DidNotOccur,
}

impl OccurrenceTruth {
    /// Return the stable wire label name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Occurred => "occurred",
            Self::DidNotOccur => "did_not_occur",
        }
    }

    /// Parse a stable wire occurrence-truth label.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::UnknownOccurrenceTruth`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, EventError> {
        match name {
            "occurred" => Ok(Self::Occurred),
            "did_not_occur" => Ok(Self::DidNotOccur),
            _ => Err(EventError::UnknownOccurrenceTruth),
        }
    }

    /// Return whether later evidence established occurrence.
    #[must_use]
    pub const fn occurred(self) -> bool {
        matches!(self, Self::Occurred)
    }

    /// Return the binary probability target used for Brier scoring.
    ///
    /// Occurred truth is `1.0`; non-occurrence is `0.0`.
    #[must_use]
    pub const fn as_probability_target(self) -> f64 {
        match self {
            Self::Occurred => 1.0,
            Self::DidNotOccur => 0.0,
        }
    }
}

/// One CHRONOS occurrence forecast that remains hypothetical.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChronosOccurrenceForecast {
    prediction_id: ChronosPredictionId,
    probability: EventConfidence,
}

impl ChronosOccurrenceForecast {
    /// Bind a prediction identity to an occurrence probability.
    #[must_use]
    pub const fn new(prediction_id: ChronosPredictionId, probability: EventConfidence) -> Self {
        Self {
            prediction_id,
            probability,
        }
    }

    /// Return the prediction identity.
    #[must_use]
    pub const fn prediction_id(self) -> ChronosPredictionId {
        self.prediction_id
    }

    /// Return the hypothesized occurrence probability.
    #[must_use]
    pub const fn probability(self) -> EventConfidence {
        self.probability
    }
}

/// Explicit refusal to treat a CHRONOS occurrence prediction as an event instance.
///
/// # Errors
///
/// Always returns [`EventError::PredictionIsNotEventInstance`].
pub fn refuse_prediction_as_instance(
    _prediction: ChronosPredictionId,
) -> Result<EventInstanceId, EventError> {
    Err(EventError::PredictionIsNotEventInstance)
}

/// Mean squared error of CHRONOS occurrence probabilities against later truth.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when the slices are empty or
/// have unequal length.
pub fn chronos_prediction_brier_score(
    forecasts: &[ChronosOccurrenceForecast],
    outcomes: &[OccurrenceTruth],
) -> Result<f64, EventError> {
    if forecasts.is_empty() || forecasts.len() != outcomes.len() {
        return Err(EventError::InvalidWirePayload);
    }
    let mut square_sum = 0.0_f64;
    for (forecast, outcome) in forecasts.iter().zip(outcomes) {
        let residual = forecast.probability().value() - outcome.as_probability_target();
        square_sum += residual * residual;
    }
    mean_square(square_sum, forecasts.len())
}

fn mean_square(square_sum: f64, count: usize) -> Result<f64, EventError> {
    let n = u32::try_from(count).map_err(|_| EventError::InvalidWirePayload)?;
    if n == 0 {
        return Err(EventError::InvalidWirePayload);
    }
    Ok(square_sum / f64::from(n))
}

#[cfg(test)]
mod tests {
    use super::{
        ChronosOccurrenceForecast, ChronosPredictionId, OccurrenceTruth,
        chronos_prediction_brier_score, refuse_prediction_as_instance,
    };
    use crate::{EventConfidence, EventError};

    #[test]
    fn prediction_helpers_cover_local_branches() {
        let prediction = ChronosPredictionId::from_raw(9);
        assert_eq!(prediction.raw(), 9);
        assert_eq!(
            refuse_prediction_as_instance(prediction),
            Err(EventError::PredictionIsNotEventInstance)
        );
        assert_eq!(OccurrenceTruth::Occurred.wire_name(), "occurred");
        assert_eq!(OccurrenceTruth::DidNotOccur.wire_name(), "did_not_occur");
        assert_eq!(
            OccurrenceTruth::from_wire_name("occurred").expect("parse"),
            OccurrenceTruth::Occurred
        );
        assert_eq!(
            OccurrenceTruth::from_wire_name("did_not_occur").expect("parse"),
            OccurrenceTruth::DidNotOccur
        );
        assert_eq!(
            OccurrenceTruth::from_wire_name("maybe"),
            Err(EventError::UnknownOccurrenceTruth)
        );
        assert!(OccurrenceTruth::Occurred.occurred());
        assert!(!OccurrenceTruth::DidNotOccur.occurred());
        assert!((OccurrenceTruth::Occurred.as_probability_target() - 1.0).abs() < f64::EPSILON);
        assert!((OccurrenceTruth::DidNotOccur.as_probability_target() - 0.0).abs() < f64::EPSILON);

        let forecast = ChronosOccurrenceForecast::new(
            prediction,
            EventConfidence::new(0.25).expect("probability"),
        );
        assert_eq!(forecast.prediction_id(), prediction);
        assert!((forecast.probability().value() - 0.25).abs() < f64::EPSILON);
        let miss = chronos_prediction_brier_score(&[forecast], &[OccurrenceTruth::Occurred])
            .expect("miss");
        assert!((miss - 0.5625).abs() < 1e-15);
        assert_eq!(
            chronos_prediction_brier_score(&[], &[OccurrenceTruth::Occurred]),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            super::mean_square(0.0, 0),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            super::mean_square(1.0, usize::MAX),
            Err(EventError::InvalidWirePayload)
        );
        assert!((super::mean_square(1.0, 2).expect("half") - 0.5).abs() < f64::EPSILON);
    }
}
