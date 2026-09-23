//! Affine EventTime-basis alignment for prevalence recovery.
//!
//! A linear prevalence trajectory is invariant to centering/scaling of its time
//! coordinate, but its intercept and slope are not. Scientific recovery must
//! therefore compare coefficients only after both are expressed in one declared
//! time basis. This module performs that validation-coordinate reparameterization;
//! it does not alter estimator state or authenticate EventTime provenance.

use crate::ValidationError;

/// One affine EventTime coordinate expressed against a caller-declared common origin.
///
/// Physical time in seconds is represented as `x = (t - center) / scale`.
/// `center_seconds` may be negative; `scale_seconds` is finite and strictly positive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearTimeBasis {
    center_seconds: f64,
    scale_seconds: f64,
}

impl LinearTimeBasis {
    /// Construct a finite affine time basis.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when the center is non-finite or
    /// the scale is non-finite/non-positive.
    pub fn new(center_seconds: f64, scale_seconds: f64) -> Result<Self, ValidationError> {
        if !center_seconds.is_finite() || !scale_seconds.is_finite() || scale_seconds <= 0.0 {
            return Err(ValidationError::InvalidInput);
        }
        Ok(Self {
            center_seconds,
            scale_seconds,
        })
    }

    /// Return the basis center in seconds from the caller-declared common origin.
    #[must_use]
    pub const fn center_seconds(self) -> f64 {
        self.center_seconds
    }

    /// Return the strictly positive basis scale in seconds.
    #[must_use]
    pub const fn scale_seconds(self) -> f64 {
        self.scale_seconds
    }
}

/// Prevalence intercepts and EventTime slopes expressed in one [`LinearTimeBasis`].
#[derive(Clone, Debug, PartialEq)]
pub struct LinearPrevalenceTimeCoefficients {
    intercepts: Vec<f64>,
    event_time_slopes: Vec<f64>,
}

impl LinearPrevalenceTimeCoefficients {
    /// Return ALR-coordinate intercepts in the target time basis.
    #[must_use]
    pub fn intercepts(&self) -> &[f64] {
        &self.intercepts
    }

    /// Return ALR-coordinate EventTime slopes in the target time basis.
    #[must_use]
    pub fn event_time_slopes(&self) -> &[f64] {
        &self.event_time_slopes
    }
}

/// Re-express linear prevalence coefficients from one affine EventTime basis to another.
///
/// For source coordinate `x_s = (t - c_s) / s_s` and target coordinate
/// `x_t = (t - c_t) / s_t`, a trajectory `a_s + b_s x_s` is represented exactly
/// in the target basis by
///
/// `a_t = a_s + b_s (c_t - c_s) / s_s`
///
/// and
///
/// `b_t = b_s s_t / s_s`.
///
/// Source and target centers must be measured in seconds from the same physical
/// origin. This transformation is orthogonal to topic-label/ALR-basis alignment:
/// it changes the prevalence feature axis, not the topic coordinate axis.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for empty/mismatched coefficient
/// vectors, non-finite coefficients, or a non-finite transformed result.
pub fn reexpress_linear_prevalence_time_basis(
    intercepts: &[f64],
    event_time_slopes: &[f64],
    source_basis: LinearTimeBasis,
    target_basis: LinearTimeBasis,
) -> Result<LinearPrevalenceTimeCoefficients, ValidationError> {
    if intercepts.is_empty()
        || intercepts.len() != event_time_slopes.len()
        || intercepts
            .iter()
            .chain(event_time_slopes)
            .any(|value| !value.is_finite())
    {
        return Err(ValidationError::InvalidInput);
    }

    let center_shift =
        (target_basis.center_seconds - source_basis.center_seconds) / source_basis.scale_seconds;
    let scale_ratio = target_basis.scale_seconds / source_basis.scale_seconds;
    if !center_shift.is_finite() || !scale_ratio.is_finite() {
        return Err(ValidationError::InvalidInput);
    }

    let target_intercepts: Vec<f64> = intercepts
        .iter()
        .zip(event_time_slopes)
        .map(|(intercept, slope)| intercept + slope * center_shift)
        .collect();
    let target_slopes: Vec<f64> = event_time_slopes
        .iter()
        .map(|slope| slope * scale_ratio)
        .collect();
    if target_intercepts
        .iter()
        .chain(&target_slopes)
        .any(|value| !value.is_finite())
    {
        return Err(ValidationError::InvalidInput);
    }

    Ok(LinearPrevalenceTimeCoefficients {
        intercepts: target_intercepts,
        event_time_slopes: target_slopes,
    })
}
