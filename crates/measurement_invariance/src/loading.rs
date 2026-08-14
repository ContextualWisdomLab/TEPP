//! Known-truth RMSE for multi-group loadings.

use crate::InvarianceError;

/// One group-specific loading used for invariance recovery.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupLoading {
    group_index: u32,
    loading: f64,
}

impl GroupLoading {
    /// Construct a finite group loading.
    ///
    /// Non-finite values are rejected later by
    /// [`loading_root_mean_square_error`]; this constructor keeps the record
    /// transparent so tests can compute the same residual.
    #[must_use]
    pub const fn new(group_index: u32, loading: f64) -> Self {
        Self {
            group_index,
            loading,
        }
    }

    /// Return the group index.
    #[must_use]
    pub const fn group_index(self) -> u32 {
        self.group_index
    }

    /// Return the loading value.
    #[must_use]
    pub const fn loading(self) -> f64 {
        self.loading
    }
}

/// RMSE of recovered loadings against known-truth loadings.
///
/// # Errors
///
/// Returns [`InvarianceError::InvalidLoadingPayload`] when either slice is
/// empty, the lengths differ, a group index mismatches, or a loading is
/// non-finite.
pub fn loading_root_mean_square_error(
    truth: &[GroupLoading],
    decided: &[GroupLoading],
) -> Result<f64, InvarianceError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(InvarianceError::InvalidLoadingPayload);
    }
    let mut sum_squares = 0.0_f64;
    for (truth_row, decided_row) in truth.iter().zip(decided) {
        if truth_row.group_index() != decided_row.group_index()
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
    fn mismatched_groups_and_nan_fail_closed() {
        let truth = [GroupLoading::new(0, 0.5)];
        let other_group = [GroupLoading::new(1, 0.5)];
        assert_eq!(
            loading_root_mean_square_error(&truth, &other_group),
            Err(InvarianceError::InvalidLoadingPayload)
        );
        let nan = [GroupLoading::new(0, f64::NAN)];
        assert_eq!(
            loading_root_mean_square_error(&truth, &nan),
            Err(InvarianceError::InvalidLoadingPayload)
        );
        assert_eq!(GroupLoading::new(2, 0.1).group_index(), 2);
    }
}
