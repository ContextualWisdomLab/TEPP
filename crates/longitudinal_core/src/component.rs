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
    /// Construct a component record from its identity fields and raw value.
    ///
    /// The value is stored exactly as given, including non-finite values;
    /// this constructor performs no validation.
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

fn add_scaled_square(
    scale: &mut f64,
    scaled_sum_squares: &mut f64,
    residual_scale: f64,
    residual_ratio: f64,
) {
    if residual_scale == 0.0 || residual_ratio == 0.0 {
        return;
    }
    if *scale == 0.0 {
        *scale = residual_scale;
        *scaled_sum_squares = residual_ratio * residual_ratio;
    } else if residual_scale > *scale {
        let ratio = *scale / residual_scale;
        *scaled_sum_squares =
            *scaled_sum_squares * ratio * ratio + residual_ratio * residual_ratio;
        *scale = residual_scale;
    } else {
        let ratio = residual_scale / *scale;
        let normalized = ratio * residual_ratio;
        *scaled_sum_squares += normalized * normalized;
    }
}

/// RMSE of recovered components against known-truth components.
///
/// Direct finite residuals keep their own magnitude so small recovery errors
/// are not erased merely because an unrelated matched component is extreme.
/// If one finite endpoint subtraction overflows, that residual alone is
/// represented as `endpoint_scale × normalized_difference`, where the latter
/// is bounded by two. The root-mean-square accumulator then rescales those
/// representations without ever materializing a non-representable residual.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidComponentPayload`] when either slice is
/// empty, the lengths differ, a unit/occasion/level identity mismatches, an
/// input value is non-finite, or the final RMSE is not representable.
pub fn component_root_mean_square_error(
    truth: &[ComponentValue],
    decided: &[ComponentValue],
) -> Result<f64, LongitudinalError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(LongitudinalError::InvalidComponentPayload);
    }

    let mut scale = 0.0_f64;
    let mut scaled_sum_squares = 0.0_f64;
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
        if residual.is_finite() {
            add_scaled_square(
                &mut scale,
                &mut scaled_sum_squares,
                residual.abs(),
                1.0,
            );
        } else {
            let endpoint_scale = truth_row.value().abs().max(decided_row.value().abs());
            let normalized_residual =
                decided_row.value() / endpoint_scale - truth_row.value() / endpoint_scale;
            add_scaled_square(
                &mut scale,
                &mut scaled_sum_squares,
                endpoint_scale,
                normalized_residual.abs(),
            );
        }
    }

    if scale == 0.0 {
        return Ok(0.0);
    }
    let rmse = scale * (scaled_sum_squares / truth.len() as f64).sqrt();
    if rmse.is_finite() {
        Ok(rmse)
    } else {
        Err(LongitudinalError::InvalidComponentPayload)
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentValue, component_root_mean_square_error};
    use crate::{ComponentLevel, LongitudinalError};

    #[test]
    fn maximal_residuals_do_not_overflow() {
        let truth = [
            ComponentValue::new(0, 0, ComponentLevel::Between, 0.0),
            ComponentValue::new(1, 0, ComponentLevel::Between, 0.0),
        ];
        let maxed = [
            ComponentValue::new(0, 0, ComponentLevel::Between, f64::MAX),
            ComponentValue::new(1, 0, ComponentLevel::Between, f64::MAX),
        ];
        assert_eq!(
            component_root_mean_square_error(&truth, &maxed),
            Ok(f64::MAX)
        );
        let partial_extreme = [
            ComponentValue::new(0, 0, ComponentLevel::Between, f64::MAX),
            ComponentValue::new(1, 0, ComponentLevel::Between, 0.0),
        ];
        let expected = f64::MAX / f64::sqrt(2.0);
        let got = component_root_mean_square_error(&truth, &partial_extreme).expect("scaled rmse");
        assert!((got - expected).abs() <= expected * 4.0 * f64::EPSILON);
    }

    #[test]
    fn representable_rmse_survives_an_overflowing_individual_residual() {
        let truth = [
            ComponentValue::new(0, 0, ComponentLevel::Between, -f64::MAX),
            ComponentValue::new(1, 0, ComponentLevel::Between, 0.0),
            ComponentValue::new(2, 0, ComponentLevel::Between, 0.0),
            ComponentValue::new(3, 0, ComponentLevel::Between, 0.0),
        ];
        let decided = [
            ComponentValue::new(0, 0, ComponentLevel::Between, f64::MAX),
            ComponentValue::new(1, 0, ComponentLevel::Between, 0.0),
            ComponentValue::new(2, 0, ComponentLevel::Between, 0.0),
            ComponentValue::new(3, 0, ComponentLevel::Between, 0.0),
        ];
        assert_eq!(
            component_root_mean_square_error(&truth, &decided),
            Ok(f64::MAX)
        );
    }

    #[test]
    fn finite_residual_precision_is_not_lost_to_unrelated_endpoint_scale() {
        let truth = [
            ComponentValue::new(0, 0, ComponentLevel::Between, f64::MAX),
            ComponentValue::new(1, 0, ComponentLevel::Between, 0.0),
        ];
        let decided = [
            ComponentValue::new(0, 0, ComponentLevel::Between, f64::MAX),
            ComponentValue::new(1, 0, ComponentLevel::Between, 1.0),
        ];
        let expected = 1.0 / f64::sqrt(2.0);
        assert_eq!(
            component_root_mean_square_error(&truth, &decided),
            Ok(expected)
        );
    }

    #[test]
    fn overflowing_residual_fails_closed() {
        let truth = [ComponentValue::new(0, 0, ComponentLevel::Within, -f64::MAX)];
        let decided = [ComponentValue::new(0, 0, ComponentLevel::Within, f64::MAX)];
        assert_eq!(
            component_root_mean_square_error(&truth, &decided),
            Err(LongitudinalError::InvalidComponentPayload)
        );
    }

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
