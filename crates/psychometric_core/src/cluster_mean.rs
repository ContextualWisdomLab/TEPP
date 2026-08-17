//! Cluster-mean within/between OLS and Kish-weighted slopes.
//!
//! This is a two-level OLS decomposition after centering within cluster (CWC).
//! It is not DSEM, not RI-CLPM, and not a random-effects sampler.
//!
//! Enders and Tofighi (2007, Table 2, pp. 124–127) separate the
//! **within-cluster** slope, the **between-cluster** slope, and the
//! **contextual** effect. The CWC cluster-mean coefficient is the contextual
//! effect (`between − within`), not the between-cluster effect.

use std::collections::BTreeMap;

use crate::error::PsychometricError;
use crate::indicator::require_finite;
use crate::loading::ordinary_least_squares_slope;

/// One clustered predictor–outcome pair on already-mapped coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClusteredScore {
    /// Cluster identity (person, document family, or membership unit).
    pub cluster_key: u64,
    /// Already-mapped predictor coordinate.
    pub predictor: f64,
    /// Already-mapped outcome coordinate.
    pub outcome: f64,
}

/// Recovered within-cluster, between-cluster, and contextual OLS slopes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WithinBetweenSlopes {
    /// OLS slope of cluster-mean-centered outcomes on centered predictors.
    pub within_slope: f64,
    /// OLS slope of cluster-mean outcomes on cluster-mean predictors.
    pub between_slope: f64,
    /// CWC cluster-mean coefficient: `between_slope - within_slope`.
    pub contextual_effect: f64,
}

/// Recover within-cluster and between-cluster OLS slopes after CWC.
///
/// Between components use unweighted cluster means. Within components use the
/// stacked cluster-mean-centered residuals. A grand-mean pooled slope is not
/// returned because it confounds the two (Enders & Tofighi, 2007; Curran &
/// Bauer, 2011; Hamaker, Kuiper, & Grasman, 2015).
///
/// `contextual_effect` is `between_slope − within_slope`. Enders and Tofighi
/// (2007, Table 2, pp. 124–127) show that this is the cluster-mean coefficient
/// under CWC (`γ01` in their Equations 4–5), **not** the between-cluster
/// effect. The between-cluster effect is the cluster-mean-only slope (CGM
/// `γ01` in their Equations 7–8). Adding the CWC contextual coefficient to the
/// within slope recovers the between-cluster effect. This is two-level OLS, not
/// the multilevel maximum-likelihood model they estimate.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] for empty, singleton, or
/// non-finite rows, [`PsychometricError::InsufficientClusters`] when fewer than
/// two clusters are present, and [`PsychometricError::SingularDesign`] when
/// either the within or the between predictor has zero variance.
pub fn recover_cluster_mean_within_between_slopes(
    rows: &[ClusteredScore],
) -> Result<WithinBetweenSlopes, PsychometricError> {
    if rows.len() < 2 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut groups: BTreeMap<u64, Vec<(f64, f64)>> = BTreeMap::new();
    for row in rows {
        if !row.predictor.is_finite() || !row.outcome.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        groups
            .entry(row.cluster_key)
            .or_default()
            .push((row.predictor, row.outcome));
    }
    if groups.len() < 2 {
        return Err(PsychometricError::InsufficientClusters);
    }

    let mut within_predictors = Vec::new();
    let mut within_outcomes = Vec::new();
    let mut between_predictors = Vec::new();
    let mut between_outcomes = Vec::new();
    for pairs in groups.values() {
        let count = pairs.len() as f64;
        let mut pred_sum = 0.0_f64;
        let mut out_sum = 0.0_f64;
        for &(predictor, outcome) in pairs {
            pred_sum += predictor;
            out_sum += outcome;
        }
        let pred_mean = pred_sum / count;
        let out_mean = out_sum / count;
        between_predictors.push(pred_mean);
        between_outcomes.push(out_mean);
        for &(predictor, outcome) in pairs {
            within_predictors.push(predictor - pred_mean);
            within_outcomes.push(outcome - out_mean);
        }
    }

    let within_slope = ordinary_least_squares_slope(&within_predictors, &within_outcomes)?;
    let between_slope = ordinary_least_squares_slope(&between_predictors, &between_outcomes)?;
    let contextual_effect = contextual_effect_from_slopes(within_slope, between_slope)?;
    Ok(WithinBetweenSlopes {
        within_slope,
        between_slope,
        contextual_effect,
    })
}

/// Enders and Tofighi (2007, p. 127) identity: CWC `γ01 = β_between − β_within`.
///
/// This helper is crate-visible so overflow of the subtraction can be recovered
/// in unit tests. It is not a random-effects estimator.
pub(crate) fn contextual_effect_from_slopes(
    within_slope: f64,
    between_slope: f64,
) -> Result<f64, PsychometricError> {
    require_finite(between_slope - within_slope)
}

/// Kish effective sample size `ESS = (Σ w)² / Σ w²` for non-negative weights.
///
/// This is the same Kish (1965) formula used by `membership_core`. It is
/// reimplemented here so `psychometric_core` stays standalone.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidWeight`] for empty, negative, non-finite,
/// or all-zero weights.
pub fn kish_effective_sample_size(weights: &[f64]) -> Result<f64, PsychometricError> {
    if weights.is_empty() {
        return Err(PsychometricError::InvalidWeight);
    }
    let mut sum = 0.0_f64;
    let mut sum_sq = 0.0_f64;
    for &weight in weights {
        if !weight.is_finite() || weight < 0.0 {
            return Err(PsychometricError::InvalidWeight);
        }
        sum += weight;
        sum_sq += weight * weight;
    }
    if sum <= 0.0 {
        return Err(PsychometricError::InvalidWeight);
    }
    require_finite((sum * sum) / sum_sq)
}

/// Weighted least-squares slope using Kish membership/survey weights.
///
/// The slope is the ordinary WLS estimator. Kish ESS is the information
/// diagnostic, not a second slope.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] for length or finiteness
/// failures, [`PsychometricError::InvalidWeight`] for invalid weights, and
/// [`PsychometricError::SingularDesign`] when the weighted predictor has zero
/// variance.
pub fn recover_kish_weighted_slope(
    predictor: &[f64],
    outcome: &[f64],
    weights: &[f64],
) -> Result<f64, PsychometricError> {
    if predictor.len() < 2 || predictor.len() != outcome.len() || predictor.len() != weights.len() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let _ess = kish_effective_sample_size(weights)?;
    let mut weight_sum = 0.0_f64;
    let mut pred_sum = 0.0_f64;
    let mut out_sum = 0.0_f64;
    for index in 0..predictor.len() {
        let pred = predictor[index];
        let out = outcome[index];
        let weight = weights[index];
        if !pred.is_finite() || !out.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        weight_sum += weight;
        pred_sum += weight * pred;
        out_sum += weight * out;
    }
    let pred_mean = pred_sum / weight_sum;
    let out_mean = out_sum / weight_sum;
    let mut cross = 0.0_f64;
    let mut pred_ss = 0.0_f64;
    for index in 0..predictor.len() {
        let pred_dev = predictor[index] - pred_mean;
        let out_dev = outcome[index] - out_mean;
        let weight = weights[index];
        cross += weight * pred_dev * out_dev;
        pred_ss += weight * pred_dev * pred_dev;
    }
    if pred_ss <= 0.0 {
        return Err(PsychometricError::SingularDesign);
    }
    require_finite(cross / pred_ss)
}

#[cfg(test)]
mod tests {
    use super::{
        ClusteredScore, contextual_effect_from_slopes, kish_effective_sample_size,
        recover_cluster_mean_within_between_slopes, recover_kish_weighted_slope,
    };
    use crate::error::PsychometricError;

    #[test]
    fn noiseless_cwc_recovers_distinct_within_between_and_contextual() {
        let rows = [
            ClusteredScore {
                cluster_key: 1,
                predictor: 0.0,
                outcome: 2.0,
            },
            ClusteredScore {
                cluster_key: 1,
                predictor: 2.0,
                outcome: 3.0,
            },
            ClusteredScore {
                cluster_key: 2,
                predictor: 4.0,
                outcome: 10.0,
            },
            ClusteredScore {
                cluster_key: 2,
                predictor: 6.0,
                outcome: 11.0,
            },
        ];
        // cluster 1 mean x=1 y=2.5; cluster 2 mean x=5 y=10.5 → between = 2
        // within: (-1,-0.5),(1,0.5) and (-1,-0.5),(1,0.5) → within = 0.5
        // contextual = 2 − 0.5 = 1.5 (CWC γ01; not the between slope)
        let recovered = recover_cluster_mean_within_between_slopes(&rows).expect("cwc");
        assert!((recovered.within_slope - 0.5).abs() < 1e-12);
        assert!((recovered.between_slope - 2.0).abs() < 1e-12);
        assert!((recovered.contextual_effect - 1.5).abs() < 1e-12);
        assert!((recovered.contextual_effect - recovered.between_slope).abs() > 1e-9);
        assert!(
            ((recovered.contextual_effect + recovered.within_slope) - recovered.between_slope)
                .abs()
                < 1e-15
        );
    }

    #[test]
    fn overflowing_contextual_subtraction_fails_closed() {
        assert_eq!(
            contextual_effect_from_slopes(-f64::MAX, f64::MAX),
            Err(PsychometricError::InvalidNumericInput)
        );
        let ok = contextual_effect_from_slopes(0.5, 2.0).expect("finite");
        assert!((ok - 1.5).abs() < 1e-15);
    }

    #[test]
    fn empty_or_one_cluster_or_nonfinite_rows_fail_closed() {
        assert_eq!(
            recover_cluster_mean_within_between_slopes(&[]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_cluster_mean_within_between_slopes(&[ClusteredScore {
                cluster_key: 1,
                predictor: 0.0,
                outcome: 1.0,
            }]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_cluster_mean_within_between_slopes(&[
                ClusteredScore {
                    cluster_key: 1,
                    predictor: 0.0,
                    outcome: 1.0,
                },
                ClusteredScore {
                    cluster_key: 1,
                    predictor: 1.0,
                    outcome: 2.0,
                },
            ]),
            Err(PsychometricError::InsufficientClusters)
        );
        assert_eq!(
            recover_cluster_mean_within_between_slopes(&[
                ClusteredScore {
                    cluster_key: 1,
                    predictor: f64::NAN,
                    outcome: 1.0,
                },
                ClusteredScore {
                    cluster_key: 2,
                    predictor: 1.0,
                    outcome: 2.0,
                },
            ]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_cluster_mean_within_between_slopes(&[
                ClusteredScore {
                    cluster_key: 1,
                    predictor: 0.0,
                    outcome: f64::INFINITY,
                },
                ClusteredScore {
                    cluster_key: 2,
                    predictor: 1.0,
                    outcome: 2.0,
                },
            ]),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn singular_within_or_between_predictor_fails() {
        let no_within = [
            ClusteredScore {
                cluster_key: 1,
                predictor: 0.0,
                outcome: 1.0,
            },
            ClusteredScore {
                cluster_key: 1,
                predictor: 0.0,
                outcome: 2.0,
            },
            ClusteredScore {
                cluster_key: 2,
                predictor: 1.0,
                outcome: 3.0,
            },
            ClusteredScore {
                cluster_key: 2,
                predictor: 1.0,
                outcome: 4.0,
            },
        ];
        assert_eq!(
            recover_cluster_mean_within_between_slopes(&no_within),
            Err(PsychometricError::SingularDesign)
        );
        let no_between = [
            ClusteredScore {
                cluster_key: 1,
                predictor: 0.0,
                outcome: 1.0,
            },
            ClusteredScore {
                cluster_key: 1,
                predictor: 2.0,
                outcome: 2.0,
            },
            ClusteredScore {
                cluster_key: 2,
                predictor: 0.0,
                outcome: 3.0,
            },
            ClusteredScore {
                cluster_key: 2,
                predictor: 2.0,
                outcome: 4.0,
            },
        ];
        assert_eq!(
            recover_cluster_mean_within_between_slopes(&no_between),
            Err(PsychometricError::SingularDesign)
        );
    }

    #[test]
    fn kish_ess_and_weighted_slope_oracles() {
        let ess = kish_effective_sample_size(&[1.0, 1.0, 1.0, 1.0]).expect("eq");
        assert!((ess - 4.0).abs() < 1e-12);
        let unequal = kish_effective_sample_size(&[1.0, 0.0, 0.0, 0.0]).expect("one");
        assert!((unequal - 1.0).abs() < 1e-12);
        let slope =
            recover_kish_weighted_slope(&[0.0, 1.0, 2.0], &[0.0, 2.0, 4.0], &[1.0, 1.0, 1.0])
                .expect("wls");
        assert!((slope - 2.0).abs() < 1e-12);
        assert_eq!(
            kish_effective_sample_size(&[]),
            Err(PsychometricError::InvalidWeight)
        );
        assert_eq!(
            kish_effective_sample_size(&[-0.1]),
            Err(PsychometricError::InvalidWeight)
        );
        assert_eq!(
            kish_effective_sample_size(&[f64::NAN]),
            Err(PsychometricError::InvalidWeight)
        );
        assert_eq!(
            kish_effective_sample_size(&[0.0, 0.0]),
            Err(PsychometricError::InvalidWeight)
        );
        assert_eq!(
            recover_kish_weighted_slope(&[0.0], &[1.0], &[1.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_kish_weighted_slope(&[0.0, 1.0], &[1.0], &[1.0, 1.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_kish_weighted_slope(&[0.0, 1.0], &[1.0, 2.0], &[1.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_kish_weighted_slope(&[0.0, f64::NAN], &[1.0, 2.0], &[1.0, 1.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_kish_weighted_slope(&[0.0, 1.0], &[1.0, f64::INFINITY], &[1.0, 1.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_kish_weighted_slope(&[0.0, 1.0], &[1.0, 2.0], &[-1.0, 1.0]),
            Err(PsychometricError::InvalidWeight)
        );
        assert_eq!(
            recover_kish_weighted_slope(&[1.0, 1.0], &[2.0, 3.0], &[1.0, 1.0]),
            Err(PsychometricError::SingularDesign)
        );
    }
}
