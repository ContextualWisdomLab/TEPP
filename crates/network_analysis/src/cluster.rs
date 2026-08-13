//! Label-invariant pair precision and recall for recovered clusters.

use crate::NetworkError;

/// An opaque cluster identity used only for equality.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ClusterLabel(u32);

impl ClusterLabel {
    /// Construct a cluster label.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Return the numeric label.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Pair precision: same-decided pairs that are also same-truth, over decided pairs.
///
/// # Errors
///
/// Returns [`NetworkError::InvalidClusterPayload`] when the slices are empty,
/// have fewer than two members, differ in length, or the decided stream has
/// no same-cluster pair.
pub fn cluster_pair_precision(
    truth: &[ClusterLabel],
    decided: &[ClusterLabel],
) -> Result<f64, NetworkError> {
    pair_rate(truth, decided, RateKind::Precision)
}

/// Pair recall: same-truth pairs that are also same-decided, over truth pairs.
///
/// # Errors
///
/// Returns [`NetworkError::InvalidClusterPayload`] when the slices are empty,
/// have fewer than two members, differ in length, or the truth stream has no
/// same-cluster pair.
pub fn cluster_pair_recall(
    truth: &[ClusterLabel],
    decided: &[ClusterLabel],
) -> Result<f64, NetworkError> {
    pair_rate(truth, decided, RateKind::Recall)
}

#[derive(Clone, Copy)]
enum RateKind {
    Precision,
    Recall,
}

fn pair_rate(
    truth: &[ClusterLabel],
    decided: &[ClusterLabel],
    kind: RateKind,
) -> Result<f64, NetworkError> {
    if truth.len() < 2 || truth.len() != decided.len() {
        return Err(NetworkError::InvalidClusterPayload);
    }
    let mut denominator = 0_u32;
    let mut numerator = 0_u32;
    for left in 0..truth.len() {
        for right in (left + 1)..truth.len() {
            let truth_same = truth[left] == truth[right];
            let decided_same = decided[left] == decided[right];
            let counted = match kind {
                RateKind::Precision => decided_same,
                RateKind::Recall => truth_same,
            };
            if counted {
                denominator += 1;
                if truth_same && decided_same {
                    numerator += 1;
                }
            }
        }
    }
    if denominator == 0 {
        return Err(NetworkError::InvalidClusterPayload);
    }
    Ok(f64::from(numerator) / f64::from(denominator))
}

#[cfg(test)]
mod tests {
    use super::{ClusterLabel, cluster_pair_precision, cluster_pair_recall};
    use crate::NetworkError;

    #[test]
    fn singleton_and_all_singletons_fail_closed() {
        let one = [ClusterLabel::new(1)];
        assert_eq!(
            cluster_pair_recall(&one, &one),
            Err(NetworkError::InvalidClusterPayload)
        );
        let all_unique_truth = [ClusterLabel::new(1), ClusterLabel::new(2)];
        let all_unique_decided = [ClusterLabel::new(3), ClusterLabel::new(4)];
        assert_eq!(
            cluster_pair_precision(&all_unique_truth, &all_unique_decided),
            Err(NetworkError::InvalidClusterPayload)
        );
        assert_eq!(ClusterLabel::new(9).value(), 9);
    }
}
