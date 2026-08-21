//! Known-truth RMSE for within/between components.

use crate::{ComponentLevel, LongitudinalError};

/// One unit-specific within or between component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComponentValue {
    unit_index: u32,
    occasion_index: u32,
    level: ComponentLevel,
    value: f64,
}

impl ComponentValue {
    /// Construct a finite component record.
    ///
    /// Non-finite values are rejected later by
    /// [`component_root_mean_square_error`]; this constructor keeps the record
    /// transparent so tests can compute the same residual.
    #[must_use]
    pub const fn new(
        unit_index: u32,
        occasion_index: u32,
        level: ComponentLevel,
        value: f64,
    ) -> Self {
        Self {
            unit_index,
            occasion_index,
            level,
            value,
        }
    }

    /// Return the unit index.
    #[must_use]
    pub const fn unit_index(self) -> u32 {
        self.unit_index
    }

    /// Return the occasion index.
    #[must_use]
    pub const fn occasion_index(self) -> u32 {
        self.occasion_index
    }

    /// Return the component level.
    #[must_use]
    pub const fn level(self) -> ComponentLevel {
        self.level
    }

    /// Return the component value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }
}

/// RMSE of recovered components against known-truth components.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidComponentPayload`] when either slice is
/// empty, the lengths differ, a unit/occasion/level identity mismatches, or a
/// value is non-finite.
pub fn component_root_mean_square_error(
    truth: &[ComponentValue],
    decided: &[ComponentValue],
) -> Result<f64, LongitudinalError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(LongitudinalError::InvalidComponentPayload);
    }
    let mut sum_squares = 0.0_f64;
    for (truth_row, decided_row) in truth.iter().zip(decided) {
        if truth_row.unit_index() != decided_row.unit_index()
            || truth_row.occasion_index() != decided_row.occasion_index()
            || truth_row.level() != decided_row.level()
            || !truth_row.value().is_finite()
            || !decided_row.value().is_finite()
        {
            return Err(LongitudinalError::InvalidComponentPayload);
        }
        let residual = decided_row.value() - truth_row.value();
        sum_squares += residual * residual;
    }
    Ok((sum_squares / truth.len() as f64).sqrt())
}

#[cfg(test)]
mod tests {
    use super::{ComponentValue, component_root_mean_square_error};
    use crate::{ComponentLevel, LongitudinalError};

    #[test]
    fn mismatched_identity_and_nan_fail_closed() {
        let truth = [ComponentValue::new(0, 0, ComponentLevel::Between, 0.5)];
        let other_unit = [ComponentValue::new(1, 0, ComponentLevel::Between, 0.5)];
        assert_eq!(
            component_root_mean_square_error(&truth, &other_unit),
            Err(LongitudinalError::InvalidComponentPayload)
        );
        let other_level = [ComponentValue::new(0, 0, ComponentLevel::Within, 0.5)];
        assert_eq!(
            component_root_mean_square_error(&truth, &other_level),
            Err(LongitudinalError::InvalidComponentPayload)
        );
        let other_occasion = [ComponentValue::new(0, 1, ComponentLevel::Between, 0.5)];
        assert_eq!(
            component_root_mean_square_error(&truth, &other_occasion),
            Err(LongitudinalError::InvalidComponentPayload)
        );
        let nan = [ComponentValue::new(0, 0, ComponentLevel::Between, f64::NAN)];
        assert_eq!(
            component_root_mean_square_error(&truth, &nan),
            Err(LongitudinalError::InvalidComponentPayload)
        );
        let nan_truth = [ComponentValue::new(0, 0, ComponentLevel::Between, f64::NAN)];
        assert_eq!(
            component_root_mean_square_error(&nan_truth, &truth),
            Err(LongitudinalError::InvalidComponentPayload)
        );
        assert_eq!(
            component_root_mean_square_error(&truth, &[]),
            Err(LongitudinalError::InvalidComponentPayload)
        );
        let valid = [ComponentValue::new(0, 0, ComponentLevel::Between, 0.5)];
        assert_eq!(component_root_mean_square_error(&truth, &valid), Ok(0.0));
        assert_eq!(
            component_root_mean_square_error(&[], &valid),
            Err(LongitudinalError::InvalidComponentPayload)
        );
        assert_eq!(
            ComponentValue::new(2, 3, ComponentLevel::Within, 0.1).occasion_index(),
            3
        );
    }
}
