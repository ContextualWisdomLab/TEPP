//! Known-truth RMSE for identified multi-group factor loadings.

use crate::InvarianceError;

/// One identified group-specific loading used for invariance recovery.
///
/// A loading is identified by its group, indicator, and factor coordinates.
/// Sequence position alone is never treated as parameter identity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupLoading {
    group_index: u32,
    indicator_index: u32,
    factor_index: u32,
    loading: f64,
}

impl GroupLoading {
    /// Construct a group-, indicator-, and factor-identified loading.
    ///
    /// Non-finite values are rejected later by
    /// [`loading_root_mean_square_error`]; this constructor keeps the record
    /// transparent so tests can compute the same residual.
    #[must_use]
    pub const fn new(
        group_index: u32,
        indicator_index: u32,
        factor_index: u32,
        loading: f64,
    ) -> Self {
        Self {
            group_index,
            indicator_index,
            factor_index,
            loading,
        }
    }

    /// Return the group index.
    #[must_use]
    pub const fn group_index(self) -> u32 {
        self.group_index
    }

    /// Return the indicator index.
    #[must_use]
    pub const fn indicator_index(self) -> u32 {
        self.indicator_index
    }

    /// Return the factor index.
    #[must_use]
    pub const fn factor_index(self) -> u32 {
        self.factor_index
    }

    /// Return the loading value.
    #[must_use]
    pub const fn loading(self) -> f64 {
        self.loading
    }

    const fn identifies_same_parameter(self, other: Self) -> bool {
        self.group_index == other.group_index
            && self.indicator_index == other.indicator_index
            && self.factor_index == other.factor_index
    }
}

/// RMSE of recovered loadings against known-truth loadings.
///
/// Each pair must identify the same group × indicator × factor parameter. The
/// function fails closed rather than comparing unrelated loadings that happen
/// to occupy the same sequence position.
///
/// # Errors
///
/// Returns [`InvarianceError::InvalidLoadingPayload`] when either slice is
/// empty, the lengths differ, a parameter coordinate mismatches, or a loading
/// is non-finite.
pub fn loading_root_mean_square_error(
    truth: &[GroupLoading],
    decided: &[GroupLoading],
) -> Result<f64, InvarianceError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(InvarianceError::InvalidLoadingPayload);
    }
    let mut sum_squares = 0.0_f64;
    for (truth_row, decided_row) in truth.iter().zip(decided) {
        if !truth_row.identifies_same_parameter(*decided_row)
            || !truth_row.loading().is_finite()
            || !decided_row.loading().is_finite()
        {
            return Err(InvarianceError::InvalidLoadingPayload);
        }
        let residual = decided_row.loading() - truth_row.loading();
        sum_squares += residual * residual;
    }
    Ok((sum_squares / truth.len() as f64).sqrt())
}

#[cfg(test)]
mod tests {
    use super::{GroupLoading, loading_root_mean_square_error};
    use crate::InvarianceError;

    #[test]
    fn mismatched_coordinates_and_nan_fail_closed() {
        let truth = [GroupLoading::new(0, 0, 0, 0.5)];
        let other_group = [GroupLoading::new(1, 0, 0, 0.5)];
        assert_eq!(
            loading_root_mean_square_error(&truth, &other_group),
            Err(InvarianceError::InvalidLoadingPayload)
        );
        let other_indicator = [GroupLoading::new(0, 1, 0, 0.5)];
        assert_eq!(
            loading_root_mean_square_error(&truth, &other_indicator),
            Err(InvarianceError::InvalidLoadingPayload)
        );
        let other_factor = [GroupLoading::new(0, 0, 1, 0.5)];
        assert_eq!(
            loading_root_mean_square_error(&truth, &other_factor),
            Err(InvarianceError::InvalidLoadingPayload)
        );
        let nan = [GroupLoading::new(0, 0, 0, f64::NAN)];
        assert_eq!(
            loading_root_mean_square_error(&truth, &nan),
            Err(InvarianceError::InvalidLoadingPayload)
        );
        let truth_nan = [GroupLoading::new(0, 0, 0, f64::NAN)];
        let finite = [GroupLoading::new(0, 0, 0, 0.5)];
        assert_eq!(
            loading_root_mean_square_error(&truth_nan, &finite),
            Err(InvarianceError::InvalidLoadingPayload)
        );
        assert_eq!(loading_root_mean_square_error(&truth, &truth), Ok(0.0));
        let identified = GroupLoading::new(2, 3, 4, 0.1);
        assert_eq!(identified.group_index(), 2);
        assert_eq!(identified.indicator_index(), 3);
        assert_eq!(identified.factor_index(), 4);
    }
}
