//! Duplicate-aware effective sample size and group-normalized weights.

use crate::ConnectedGroup;

/// Compute Kish effective sample size for non-negative weights.
///
/// Returns `0.0` when the weight vector is empty or all zeros.
#[must_use]
pub fn effective_sample_size(weights: &[f64]) -> f64 {
    let sum: f64 = weights.iter().copied().sum();
    let sum_sq: f64 = weights.iter().map(|weight| weight * weight).sum();
    if sum_sq == 0.0 {
        return 0.0;
    }
    (sum * sum) / sum_sq
}

/// Normalize member weights within each connected group to sum to one.
///
/// Groups with no positive mass are skipped. Members absent from `weights`
/// are treated as zero and omitted from the output map.
#[must_use]
pub fn group_normalized_weights(
    groups: &[ConnectedGroup],
    weights: &[(uuid::Uuid, f64)],
) -> Vec<(uuid::Uuid, f64)> {
    let weight_map: std::collections::BTreeMap<uuid::Uuid, f64> = weights
        .iter()
        .copied()
        .filter(|(_, weight)| *weight > 0.0)
        .collect();
    let mut normalized = Vec::new();
    for group in groups {
        let total: f64 = group
            .members()
            .iter()
            .filter_map(|member| weight_map.get(member).copied())
            .sum();
        if total <= 0.0 {
            continue;
        }
        for member in group.members() {
            if let Some(weight) = weight_map.get(member) {
                normalized.push((*member, weight / total));
            }
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::{effective_sample_size, group_normalized_weights};
    use crate::connected_group::{LeakageLink, LeakageLinkKind, build_connected_groups};
    use uuid::Uuid;

    #[test]
    fn ess_and_group_normalization_contracts() {
        assert!((effective_sample_size(&[]) - 0.0).abs() < 1e-12);
        assert!((effective_sample_size(&[0.0, 0.0]) - 0.0).abs() < 1e-12);
        let ess = effective_sample_size(&[1.0, 1.0, 1.0, 1.0]);
        assert!((ess - 4.0).abs() < 1e-12);
        let unequal = effective_sample_size(&[1.0, 0.0, 0.0, 0.0]);
        assert!((unequal - 1.0).abs() < 1e-12);

        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        let groups = build_connected_groups(
            &[a, b],
            &[LeakageLink {
                left: a,
                right: b,
                kind: LeakageLinkKind::SameEpisode,
            }],
        );
        let normalized = group_normalized_weights(&groups, &[(a, 1.0), (b, 3.0)]);
        let sum: f64 = normalized.iter().map(|(_, weight)| weight).sum();
        assert!((sum - 1.0).abs() < 1e-12);
        let empty = group_normalized_weights(&groups, &[(a, 0.0), (b, 0.0)]);
        assert!(empty.is_empty());
        // Partial weight maps omit members without positive mass.
        let partial = group_normalized_weights(&groups, &[(a, 2.0)]);
        assert_eq!(partial.len(), 1);
        assert!((partial[0].1 - 1.0).abs() < 1e-12);
    }
}
