//! Effective sample size and design-effect helpers for multiple membership.

use crate::MembershipError;

/// Kish effective sample size for a set of non-negative finite weights.
///
/// `ESS = (Σ w)² / Σ w²`. Empty input fails closed. A single positive weight
/// yields ESS `1.0`. Zero total weight fails closed (no information).
///
/// # Errors
///
/// Returns [`MembershipError::InvalidMembershipWeight`] for empty input, any
/// non-finite or negative weight, or a zero total weight.
pub fn kish_effective_sample_size(weights: &[f64]) -> Result<f64, MembershipError> {
    if weights.is_empty() {
        return Err(MembershipError::InvalidMembershipWeight);
    }
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    for &weight in weights {
        let finite = weight.is_finite();
        let non_negative = weight >= 0.0;
        if !finite {
            return Err(MembershipError::InvalidMembershipWeight);
        }
        if !non_negative {
            return Err(MembershipError::InvalidMembershipWeight);
        }
        sum += weight;
        sum_sq += weight * weight;
    }
    if sum <= 0.0 {
        return Err(MembershipError::InvalidMembershipWeight);
    }
    // With finite non-negative weights and positive sum, sum_sq is positive.
    Ok((sum * sum) / sum_sq)
}

/// Design effect `n / ESS` for the same weight vector.
///
/// Values above `1.0` indicate inflation of variance relative to an equal-weight
/// sample of size `n = weights.len()`.
///
/// # Errors
///
/// Propagates [`kish_effective_sample_size`] failures.
pub fn design_effect(weights: &[f64]) -> Result<f64, MembershipError> {
    let ess = kish_effective_sample_size(weights)?;
    #[allow(clippy::cast_precision_loss)]
    let n = weights.len() as f64;
    Ok(n / ess)
}

/// Group-normalize weights so each group's weights sum to one, then return Kish
/// ESS over the concatenated normalized weights.
///
/// Used when co-partitioned groups must not dominate recovery or split ESS by
/// raw headcount.
///
/// # Errors
///
/// Returns [`MembershipError::InvalidMembershipWeight`] when any group is empty,
/// contains invalid weights, or has zero total weight.
pub fn group_normalized_kish_ess(groups: &[Vec<f64>]) -> Result<f64, MembershipError> {
    if groups.is_empty() {
        return Err(MembershipError::InvalidMembershipWeight);
    }
    let mut normalized = Vec::new();
    for group in groups {
        if group.is_empty() {
            return Err(MembershipError::InvalidMembershipWeight);
        }
        let mut sum = 0.0;
        for &weight in group {
            let finite = weight.is_finite();
            let non_negative = weight >= 0.0;
            if !finite {
                return Err(MembershipError::InvalidMembershipWeight);
            }
            if !non_negative {
                return Err(MembershipError::InvalidMembershipWeight);
            }
            sum += weight;
        }
        if sum <= 0.0 {
            return Err(MembershipError::InvalidMembershipWeight);
        }
        for &weight in group {
            normalized.push(weight / sum);
        }
    }
    kish_effective_sample_size(&normalized)
}

#[cfg(test)]
mod tests {
    use super::{design_effect, group_normalized_kish_ess, kish_effective_sample_size};
    use crate::MembershipError;

    #[test]
    fn kish_ess_and_design_effect_oracle_cases() {
        assert!(
            (kish_effective_sample_size(&[1.0, 1.0, 1.0, 1.0]).expect("eq") - 4.0).abs() < 1e-12
        );
        assert!((design_effect(&[1.0, 1.0, 1.0, 1.0]).expect("de") - 1.0).abs() < 1e-12);
        let unequal = kish_effective_sample_size(&[1.0, 0.0, 0.0, 0.0]).expect("one");
        assert!((unequal - 1.0).abs() < 1e-12);
        assert!(design_effect(&[1.0, 0.0, 0.0, 0.0]).expect("de2") > 1.0);

        assert_eq!(
            kish_effective_sample_size(&[]),
            Err(MembershipError::InvalidMembershipWeight)
        );
        assert_eq!(
            kish_effective_sample_size(&[f64::NAN]),
            Err(MembershipError::InvalidMembershipWeight)
        );
        assert_eq!(
            kish_effective_sample_size(&[-0.1]),
            Err(MembershipError::InvalidMembershipWeight)
        );
        assert_eq!(
            kish_effective_sample_size(&[0.0, 0.0]),
            Err(MembershipError::InvalidMembershipWeight)
        );
        assert_eq!(
            design_effect(&[f64::INFINITY]),
            Err(MembershipError::InvalidMembershipWeight)
        );

        let groups = vec![vec![2.0, 2.0], vec![1.0]];
        let ess = group_normalized_kish_ess(&groups).expect("g");
        // normalized: 0.5,0.5,1.0 → sum=2, sum_sq=0.25+0.25+1=1.5 → ess=4/1.5
        assert!((ess - (4.0 / 1.5)).abs() < 1e-12);
        assert_eq!(
            group_normalized_kish_ess(&[]),
            Err(MembershipError::InvalidMembershipWeight)
        );
        assert_eq!(
            group_normalized_kish_ess(&[vec![]]),
            Err(MembershipError::InvalidMembershipWeight)
        );
        assert_eq!(
            group_normalized_kish_ess(&[vec![0.0, 0.0]]),
            Err(MembershipError::InvalidMembershipWeight)
        );
        assert_eq!(
            group_normalized_kish_ess(&[vec![-1.0]]),
            Err(MembershipError::InvalidMembershipWeight)
        );
        assert_eq!(
            group_normalized_kish_ess(&[vec![f64::NAN]]),
            Err(MembershipError::InvalidMembershipWeight)
        );
    }
}
