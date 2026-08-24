//! Label-invariant pair precision and recall for recovered clusters.

use crate::NetworkError;
use std::collections::HashMap;

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
    let mut truth_counts = HashMap::<ClusterLabel, u64>::new();
    let mut decided_counts = HashMap::<ClusterLabel, u64>::new();
    let mut joint_counts = HashMap::<(ClusterLabel, ClusterLabel), u64>::new();
    let mut truth_pairs = 0_u64;
    let mut decided_pairs = 0_u64;
    let mut joint_pairs = 0_u64;

    for (&truth_label, &decided_label) in truth.iter().zip(decided) {
        let truth_count = truth_counts.entry(truth_label).or_default();
        truth_pairs = truth_pairs
            .checked_add(*truth_count)
            .ok_or(NetworkError::InvalidClusterPayload)?;
        *truth_count = truth_count
            .checked_add(1)
            .ok_or(NetworkError::InvalidClusterPayload)?;

        let decided_count = decided_counts.entry(decided_label).or_default();
        decided_pairs = decided_pairs
            .checked_add(*decided_count)
            .ok_or(NetworkError::InvalidClusterPayload)?;
        *decided_count = decided_count
            .checked_add(1)
            .ok_or(NetworkError::InvalidClusterPayload)?;

        let joint_count = joint_counts
            .entry((truth_label, decided_label))
            .or_default();
        joint_pairs = joint_pairs
            .checked_add(*joint_count)
            .ok_or(NetworkError::InvalidClusterPayload)?;
        *joint_count = joint_count
            .checked_add(1)
            .ok_or(NetworkError::InvalidClusterPayload)?;
    }

    let (numerator, denominator) = match kind {
        RateKind::Precision => (joint_pairs, decided_pairs),
        RateKind::Recall => (joint_pairs, truth_pairs),
    };
    if denominator == 0 {
        return Err(NetworkError::InvalidClusterPayload);
    }
    // The public metric is f64 and valid pair counts may exceed u32.
    #[allow(clippy::cast_precision_loss)]
    let ratio = numerator as f64 / denominator as f64;
    Ok(ratio)
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

    #[test]
    fn pair_sizes_and_lengths_with_mixed_outcomes_cover_rate_branches() {
        let truth = [
            ClusterLabel::new(1),
            ClusterLabel::new(1),
            ClusterLabel::new(2),
        ];
        let decided = [
            ClusterLabel::new(1),
            ClusterLabel::new(2),
            ClusterLabel::new(2),
        ];

        let precision = cluster_pair_precision(&truth, &decided).expect("precision");
        let recall = cluster_pair_recall(&truth, &decided).expect("recall");

        assert!((precision - (0.0_f64 / 1.0_f64)).abs() < f64::EPSILON);
        assert!((recall - 0.0_f64).abs() < f64::EPSILON);

        let true_aligned = [
            ClusterLabel::new(3),
            ClusterLabel::new(3),
            ClusterLabel::new(2),
        ];
        let aligned_precision =
            cluster_pair_precision(&truth, &true_aligned).expect("precision_aligned");
        assert!((aligned_precision - 1.0_f64).abs() < f64::EPSILON);
    }

    #[test]
    fn recall_with_no_truth_pairs_is_invalid() {
        let truth = [
            ClusterLabel::new(1),
            ClusterLabel::new(2),
            ClusterLabel::new(3),
        ];
        let decided = [
            ClusterLabel::new(1),
            ClusterLabel::new(1),
            ClusterLabel::new(2),
        ];

        assert_eq!(
            cluster_pair_recall(&truth, &decided),
            Err(NetworkError::InvalidClusterPayload)
        );
    }

    #[test]
    fn payloads_with_mismatched_lengths_are_invalid() {
        let truth = [
            ClusterLabel::new(1),
            ClusterLabel::new(2),
            ClusterLabel::new(3),
        ];
        let decided = [ClusterLabel::new(1), ClusterLabel::new(1)];

        assert_eq!(
            cluster_pair_precision(&truth, &decided),
            Err(NetworkError::InvalidClusterPayload)
        );
        assert_eq!(
            cluster_pair_recall(&truth, &decided),
            Err(NetworkError::InvalidClusterPayload)
        );
    }
}
