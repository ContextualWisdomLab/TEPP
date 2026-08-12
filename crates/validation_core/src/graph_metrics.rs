//! Relation-graph recovery precision and recall.

use crate::ValidationError;
use std::collections::BTreeSet;

/// One undirected recovered/true edge identity pair.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EdgeIdentity {
    /// Lexicographically smaller endpoint label.
    pub left: u64,
    /// Lexicographically larger endpoint label.
    pub right: u64,
}

impl EdgeIdentity {
    /// Construct a normalized undirected edge identity.
    #[must_use]
    pub fn new(a: u64, b: u64) -> Self {
        if a <= b {
            Self { left: a, right: b }
        } else {
            Self { left: b, right: a }
        }
    }
}

/// Precision of recovered edges against the truth edge set.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when the recovered set is empty.
pub fn edge_precision(
    truth: &[EdgeIdentity],
    recovered: &[EdgeIdentity],
) -> Result<f64, ValidationError> {
    if recovered.is_empty() {
        return Err(ValidationError::InvalidInput);
    }
    let truth_set: BTreeSet<_> = truth.iter().copied().collect();
    let recovered_set: BTreeSet<_> = recovered.iter().copied().collect();
    let true_positive = recovered_set.intersection(&truth_set).count() as f64;
    Ok(true_positive / recovered_set.len() as f64)
}

/// Recall of recovered edges against the truth edge set.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when the truth set is empty.
pub fn edge_recall(
    truth: &[EdgeIdentity],
    recovered: &[EdgeIdentity],
) -> Result<f64, ValidationError> {
    if truth.is_empty() {
        return Err(ValidationError::InvalidInput);
    }
    let truth_set: BTreeSet<_> = truth.iter().copied().collect();
    let recovered_set: BTreeSet<_> = recovered.iter().copied().collect();
    let true_positive = recovered_set.intersection(&truth_set).count() as f64;
    Ok(true_positive / truth_set.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{EdgeIdentity, edge_precision, edge_recall};
    use crate::ValidationError;

    #[test]
    fn precision_recall_and_identity_normalization() {
        assert_eq!(EdgeIdentity::new(2, 1), EdgeIdentity::new(1, 2));
        let truth = [EdgeIdentity::new(1, 2), EdgeIdentity::new(2, 3)];
        let recovered = [
            EdgeIdentity::new(1, 2),
            EdgeIdentity::new(3, 4),
            EdgeIdentity::new(2, 1),
        ];
        // recovered unique: {1-2, 3-4}; TP=1 → precision 0.5; recall 1/2
        assert!((edge_precision(&truth, &recovered).expect("p") - 0.5).abs() < 1e-12);
        assert!((edge_recall(&truth, &recovered).expect("r") - 0.5).abs() < 1e-12);
        assert_eq!(
            edge_precision(&truth, &[]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            edge_recall(&[], &recovered),
            Err(ValidationError::InvalidInput)
        );
    }
}
