//! Rolling-origin evaluation windows over cutoff snapshots.

use temporal_core::KnowledgeCutoff;

/// One rolling-origin evaluation origin with train and test cutoffs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RollingOriginWindow {
    /// Inclusive knowledge cutoff for training evidence.
    pub train_cutoff: KnowledgeCutoff,
    /// Inclusive knowledge cutoff for the test horizon.
    pub test_cutoff: KnowledgeCutoff,
}

/// Build contiguous rolling-origin windows from ordered cutoffs.
///
/// Each window uses cutoff `i` for training and cutoff `i + 1` for testing.
///
/// # Errors
///
/// Returns [`crate::CorpusSplitError::InvalidSplitConfiguration`] when fewer
/// than two cutoffs are provided or order is not strictly increasing.
pub fn rolling_origin_windows(
    ordered_cutoffs: &[KnowledgeCutoff],
) -> Result<Vec<RollingOriginWindow>, crate::CorpusSplitError> {
    if ordered_cutoffs.len() < 2 {
        return Err(crate::CorpusSplitError::InvalidSplitConfiguration);
    }
    for window in ordered_cutoffs.windows(2) {
        if window[0].instant() >= window[1].instant() {
            return Err(crate::CorpusSplitError::InvalidSplitConfiguration);
        }
    }
    Ok(ordered_cutoffs
        .windows(2)
        .map(|pair| RollingOriginWindow {
            train_cutoff: pair[0],
            test_cutoff: pair[1],
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::rolling_origin_windows;
    use crate::CorpusSplitError;
    use temporal_core::KnowledgeCutoff;

    #[test]
    fn windows_require_strictly_increasing_cutoffs() {
        let early = KnowledgeCutoff::parse_rfc3339("2026-01-01T00:00:00Z").expect("e");
        let mid = KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("m");
        let late = KnowledgeCutoff::parse_rfc3339("2026-03-01T00:00:00Z").expect("l");
        let windows = rolling_origin_windows(&[early, mid, late]).expect("windows");
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].train_cutoff, early);
        assert_eq!(windows[0].test_cutoff, mid);
        assert_eq!(
            rolling_origin_windows(&[early]),
            Err(CorpusSplitError::InvalidSplitConfiguration)
        );
        assert_eq!(
            rolling_origin_windows(&[mid, early]),
            Err(CorpusSplitError::InvalidSplitConfiguration)
        );
    }
}
