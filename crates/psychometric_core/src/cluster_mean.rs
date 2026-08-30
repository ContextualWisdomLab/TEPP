//! Cluster-mean within/between OLS and Kish-weighted slopes.
//!
//! This is a two-level OLS decomposition after centering within cluster (CWC).
//! It is not DSEM, not RI-CLPM, and not a random-effects sampler.
//!
//! Enders and Tofighi (2007, Table 2, pp. 124–127) separate the
//! **within-cluster** slope, the **between-cluster** slope, and the
//! **contextual** effect. The CWC cluster-mean coefficient is the contextual
//! effect (`between − within`), not the between-cluster effect. Kish-weighted
//! CWC uses weighted cluster means and cluster-total WLS between; that
//! `n_j`-weighted between is a different estimand from their unweighted
//! between when cluster sizes differ. Kish ESS is diagnostic, not a slope.
//! A cluster whose scaled total underflows to 0 after max-scale is
//! Kish-zero relative information and is omitted.

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
/// reimplemented here so `psychometric_core` stays standalone. ESS is
/// homogeneous of degree 0: a common positive scale on every weight does
/// not change the diagnostic. Weights are therefore divided by their
/// maximum before forming `(Σ w)² / Σ w²`, so a valid common scale of
/// `f64::MAX` or `f64::MIN_POSITIVE` does not overflow or underflow to
/// an all-zero sum.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidWeight`] for empty, negative, non-finite,
/// or all-zero weights.
pub fn kish_effective_sample_size(weights: &[f64]) -> Result<f64, PsychometricError> {
    let max_weight = require_max_positive_weight(weights)?;
    let mut sum = 0.0_f64;
    let mut sum_sq = 0.0_f64;
    for &weight in weights {
        let scaled = weight / max_weight;
        sum += scaled;
        sum_sq += scaled * scaled;
    }
    require_finite((sum * sum) / sum_sq)
}

/// Maximum strictly positive finite weight. Negative, non-finite, empty, or
/// all-zero series fail closed. Crate-visible so overflow-scale tests can
/// recover the same scale the estimators use.
pub(crate) fn require_max_positive_weight(weights: &[f64]) -> Result<f64, PsychometricError> {
    if weights.is_empty() {
        return Err(PsychometricError::InvalidWeight);
    }
    let mut max_weight = 0.0_f64;
    for &weight in weights {
        if !weight.is_finite() || weight < 0.0 {
            return Err(PsychometricError::InvalidWeight);
        }
        if weight > max_weight {
            max_weight = weight;
        }
    }
    if max_weight <= 0.0 {
        return Err(PsychometricError::InvalidWeight);
    }
    Ok(max_weight)
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
    let max_weight = require_max_positive_weight(weights)?;
    let mut weight_sum = 0.0_f64;
    let mut pred_sum = 0.0_f64;
    let mut out_sum = 0.0_f64;
    for index in 0..predictor.len() {
        let pred = predictor[index];
        let out = outcome[index];
        let weight = weights[index] / max_weight;
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
        let weight = weights[index] / max_weight;
        cross += weight * pred_dev * out_dev;
        pred_ss += weight * pred_dev * pred_dev;
    }
    if pred_ss <= 0.0 {
        return Err(PsychometricError::SingularDesign);
    }
    require_finite(cross / pred_ss)
}

/// Recovered Kish-weighted within-cluster, between-cluster, and contextual
/// slopes plus Kish ESS diagnostics.
///
/// `observation_effective_sample_size` and `cluster_effective_sample_size`
/// are Kish (1965) information diagnostics. They are not slopes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KishWeightedWithinBetweenSlopes {
    /// WLS slope of weighted-mean-centered outcomes on centered predictors.
    pub within_slope: f64,
    /// WLS slope of weighted cluster-mean outcomes on weighted cluster-mean
    /// predictors, with cluster totals `W_j = sum_i w_ij`.
    pub between_slope: f64,
    /// Kish-weighted CWC cluster-mean coefficient:
    /// `between_slope - within_slope`.
    pub contextual_effect: f64,
    /// Kish ESS of the observation weights. Not a slope.
    pub observation_effective_sample_size: f64,
    /// Kish ESS of the cluster totals `W_j`. Not a slope.
    pub cluster_effective_sample_size: f64,
}

/// Recover Kish-weighted within-cluster and between-cluster slopes after CWC.
///
/// Cluster means are weighted: `x_j_bar = sum_i w_ij x_ij / sum_i w_ij`.
/// Within WLS uses the stacked weighted-mean-centered residuals
/// with the observation weights. Between WLS uses those cluster means with
/// cluster totals `W_j = sum_i w_ij`. The contextual effect is
/// weighted between minus weighted within (Enders & Tofighi, 2007, Table 2
/// identity). Equal cluster sizes and equal observation weights recover the
/// unweighted CWC slopes. A common positive scale on every observation
/// weight is the same estimand (Kish ESS and WLS are homogeneous of
/// degree 0); weights are divided by their maximum so `f64::MAX` or
/// `f64::MIN_POSITIVE` common scales recover that estimand. A cluster
/// whose original weights are all zero has no weighted mean and fails
/// closed. A cluster whose scaled total underflows to 0 after that
/// max-scale has Kish-zero relative information (Kish, 1965) and is
/// omitted; if fewer than two clusters remain, the design is
/// [`PsychometricError::InsufficientClusters`]. Unequal `n_j` even with
/// equal observation weights makes the `n_j`-weighted between a different
/// estimand from Enders and Tofighi's unweighted between. Pooled Kish WLS
/// of the raw scores is not the weighted within slope. Kish ESS is
/// reported as the observation and cluster diagnostics and is not a slope.
///
/// This is two-level WLS, not DSEM, not RI-CLPM, and not their multilevel
/// maximum-likelihood model.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] for empty, singleton,
/// length-mismatched, or non-finite rows,
/// [`PsychometricError::InvalidWeight`] for invalid weights or a
/// zero-weight cluster, [`PsychometricError::InsufficientClusters`] when
/// fewer than two clusters remain after omitting Kish-zero relative
/// information, and [`PsychometricError::SingularDesign`] when either the
/// weighted within or the weighted between predictor has zero variance.
pub fn recover_kish_weighted_cluster_mean_within_between_slopes(
    rows: &[ClusteredScore],
    weights: &[f64],
) -> Result<KishWeightedWithinBetweenSlopes, PsychometricError> {
    if rows.len() < 2 || rows.len() != weights.len() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let max_weight = require_max_positive_weight(weights)?;
    let observation_effective_sample_size = kish_effective_sample_size(weights)?;
    let mut groups: BTreeMap<u64, Vec<(f64, f64, f64)>> = BTreeMap::new();
    for (row, &weight) in rows.iter().zip(weights.iter()) {
        if !row.predictor.is_finite() || !row.outcome.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        groups
            .entry(row.cluster_key)
            .or_default()
            .push((row.predictor, row.outcome, weight));
    }
    if groups.len() < 2 {
        return Err(PsychometricError::InsufficientClusters);
    }

    let mut within_predictors = Vec::new();
    let mut within_outcomes = Vec::new();
    let mut within_weights = Vec::new();
    let mut between_predictors = Vec::new();
    let mut between_outcomes = Vec::new();
    let mut cluster_weights = Vec::new();
    for pairs in groups.values() {
        let mut original_positive = false;
        let mut weight_sum = 0.0_f64;
        let mut pred_sum = 0.0_f64;
        let mut out_sum = 0.0_f64;
        for &(predictor, outcome, weight) in pairs {
            if weight > 0.0 {
                original_positive = true;
            }
            let scaled = weight / max_weight;
            weight_sum += scaled;
            pred_sum += scaled * predictor;
            out_sum += scaled * outcome;
        }
        if !original_positive {
            return Err(PsychometricError::InvalidWeight);
        }
        if weight_sum <= 0.0 {
            continue;
        }
        let pred_mean = require_finite(pred_sum / weight_sum)?;
        let out_mean = require_finite(out_sum / weight_sum)?;
        between_predictors.push(pred_mean);
        between_outcomes.push(out_mean);
        cluster_weights.push(weight_sum);
        for &(predictor, outcome, weight) in pairs {
            within_predictors.push(predictor - pred_mean);
            within_outcomes.push(outcome - out_mean);
            within_weights.push(weight / max_weight);
        }
    }
    if between_predictors.len() < 2 {
        return Err(PsychometricError::InsufficientClusters);
    }

    let within_slope =
        recover_kish_weighted_slope(&within_predictors, &within_outcomes, &within_weights)?;
    let between_slope =
        recover_kish_weighted_slope(&between_predictors, &between_outcomes, &cluster_weights)?;
    let contextual_effect = contextual_effect_from_slopes(within_slope, between_slope)?;
    let cluster_effective_sample_size = kish_effective_sample_size(&cluster_weights)?;
    Ok(KishWeightedWithinBetweenSlopes {
        within_slope,
        between_slope,
        contextual_effect,
        observation_effective_sample_size,
        cluster_effective_sample_size,
    })
}

/// Refuse treating pooled Kish WLS of raw scores as the weighted within
/// slope.
///
/// Pooled WLS confounds within-cluster and between-cluster association
/// (Enders & Tofighi, 2007; Curran & Bauer, 2011; Hamaker, Kuiper, &
/// Grasman, 2015). Weighted CWC within is WLS of the weighted-mean-centered
/// residuals.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PooledKishSlopeIsNotWeightedWithinSlope`].
pub fn refuse_pooled_kish_slope_as_weighted_within_slope(
    pooled_slope: f64,
    weighted_within_slope: f64,
) -> Result<f64, PsychometricError> {
    let _ = (pooled_slope, weighted_within_slope);
    Err(PsychometricError::PooledKishSlopeIsNotWeightedWithinSlope)
}

/// Refuse treating unweighted between as Kish-weighted between.
///
/// Enders and Tofighi (2007, Table 2) form the between slope from
/// unweighted cluster means. Kish-weighted CWC forms WLS of those means
/// with cluster totals `W_j`. Unequal `n_j` makes the estimands
/// differ even when every observation weight equals 1.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::UnweightedBetweenSlopeIsNotKishWeightedBetweenSlope`].
pub fn refuse_unweighted_between_slope_as_kish_weighted_between_slope(
    unweighted_between_slope: f64,
    kish_weighted_between_slope: f64,
) -> Result<f64, PsychometricError> {
    let _ = (unweighted_between_slope, kish_weighted_between_slope);
    Err(PsychometricError::UnweightedBetweenSlopeIsNotKishWeightedBetweenSlope)
}

/// Refuse treating Kish ESS as a slope.
///
/// Kish (1965) \(\mathrm{ESS}=(\sum w)^{2}/\sum w^{2}\) is the information
/// diagnostic. WLS uses the weights in the slope; ESS is not a second
/// slope.
///
/// # Errors
///
/// Always returns [`PsychometricError::KishEffectiveSampleSizeIsNotASlope`].
pub fn refuse_kish_effective_sample_size_as_slope(
    effective_sample_size: f64,
) -> Result<f64, PsychometricError> {
    let _ = effective_sample_size;
    Err(PsychometricError::KishEffectiveSampleSizeIsNotASlope)
}

#[cfg(test)]
mod tests {
    use super::{
        contextual_effect_from_slopes, kish_effective_sample_size,
        recover_cluster_mean_within_between_slopes,
        recover_kish_weighted_cluster_mean_within_between_slopes, recover_kish_weighted_slope,
        refuse_kish_effective_sample_size_as_slope,
        refuse_pooled_kish_slope_as_weighted_within_slope,
        refuse_unweighted_between_slope_as_kish_weighted_between_slope, ClusteredScore,
    };
    use crate::error::PsychometricError;

    fn equal_n_cwc_rows() -> [ClusteredScore; 4] {
        [
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
        ]
    }

    fn unequal_n_cwc_rows() -> Vec<ClusteredScore> {
        let mut rows = vec![
            ClusteredScore {
                cluster_key: 1,
                predictor: -1.0,
                outcome: -0.5,
            },
            ClusteredScore {
                cluster_key: 1,
                predictor: 1.0,
                outcome: 0.5,
            },
            ClusteredScore {
                cluster_key: 2,
                predictor: 0.0,
                outcome: -0.5,
            },
            ClusteredScore {
                cluster_key: 2,
                predictor: 2.0,
                outcome: 0.5,
            },
        ];
        let within_x = [-2.5_f64, -1.5, -0.5, 0.5, 1.5, 2.5];
        for deviation in within_x {
            rows.push(ClusteredScore {
                cluster_key: 3,
                predictor: 2.0 + deviation,
                outcome: 4.0 + 0.5 * deviation,
            });
        }
        rows
    }

    #[test]
    fn noiseless_cwc_recovers_distinct_within_between_and_contextual() {
        let rows = equal_n_cwc_rows();
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
        let huge_ess = kish_effective_sample_size(&[f64::MAX, f64::MAX]).expect("huge");
        assert!((huge_ess - 2.0).abs() < 1e-12);
        let tiny_ess =
            kish_effective_sample_size(&[f64::MIN_POSITIVE, f64::MIN_POSITIVE, f64::MIN_POSITIVE])
                .expect("tiny");
        assert!((tiny_ess - 3.0).abs() < 1e-12);
        let unscaled = kish_effective_sample_size(&[1.0, 2.0, 3.0]).expect("unit");
        let scaled = kish_effective_sample_size(&[10.0, 20.0, 30.0]).expect("scaled");
        assert!((unscaled - scaled).abs() < 1e-12);
        let uneven = kish_effective_sample_size(&[1e-8, 1.0, 1e8]).expect("uneven");
        let scaled_uneven = kish_effective_sample_size(&[1e-9, 0.1, 1e7]).expect("scaled uneven");
        assert!((uneven - scaled_uneven).abs() < 1e-12);
        assert!((1.0 - 1e-12..=3.0 + 1e-12).contains(&uneven));
        assert!((1.0 - 1e-12..=2.0 + 1e-12).contains(&huge_ess));
        assert!((1.0 - 1e-12..=3.0 + 1e-12).contains(&tiny_ess));
        let uneven_slope =
            recover_kish_weighted_slope(&[0.0, 1.0, 2.0], &[0.0, 2.0, 4.0], &[1e-8, 1.0, 1e8])
                .expect("uneven wls");
        let scaled_uneven_slope =
            recover_kish_weighted_slope(&[0.0, 1.0, 2.0], &[0.0, 2.0, 4.0], &[1e-9, 0.1, 1e7])
                .expect("scaled uneven wls");
        assert!((uneven_slope - 2.0).abs() < 1e-12);
        assert!((scaled_uneven_slope - uneven_slope).abs() < 1e-12);
        let huge_slope = recover_kish_weighted_slope(
            &[0.0, 1.0, 2.0],
            &[0.0, 2.0, 4.0],
            &[f64::MAX, f64::MAX, f64::MAX],
        )
        .expect("huge wls");
        assert!((huge_slope - 2.0).abs() < 1e-12);
        let tiny_slope = recover_kish_weighted_slope(
            &[0.0, 1.0, 2.0],
            &[0.0, 2.0, 4.0],
            &[f64::MIN_POSITIVE, f64::MIN_POSITIVE, f64::MIN_POSITIVE],
        )
        .expect("tiny wls");
        assert!((tiny_slope - 2.0).abs() < 1e-12);
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
        assert_eq!(
            recover_kish_weighted_slope(&[0.0, f64::MAX], &[0.0, f64::MAX], &[1.0, 1.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn equal_n_equal_weights_recover_unweighted_cwc() {
        let rows = equal_n_cwc_rows();
        let weights = [1.0_f64, 1.0, 1.0, 1.0];
        let unweighted = recover_cluster_mean_within_between_slopes(&rows).expect("ols");
        let weighted =
            recover_kish_weighted_cluster_mean_within_between_slopes(&rows, &weights).expect("wls");
        assert!((weighted.within_slope - unweighted.within_slope).abs() < 1e-12);
        assert!((weighted.between_slope - unweighted.between_slope).abs() < 1e-12);
        assert!((weighted.contextual_effect - unweighted.contextual_effect).abs() < 1e-12);
        assert!((weighted.observation_effective_sample_size - 4.0).abs() < 1e-12);
        assert!((weighted.cluster_effective_sample_size - 2.0).abs() < 1e-12);
        for scale in [f64::MAX, f64::MIN_POSITIVE] {
            let scaled_weights = [scale, scale, scale, scale];
            let scaled =
                recover_kish_weighted_cluster_mean_within_between_slopes(&rows, &scaled_weights)
                    .expect("scaled wls");
            assert!((scaled.within_slope - unweighted.within_slope).abs() < 1e-12);
            assert!((scaled.between_slope - unweighted.between_slope).abs() < 1e-12);
            assert!((scaled.contextual_effect - unweighted.contextual_effect).abs() < 1e-12);
            assert!((scaled.observation_effective_sample_size - 4.0).abs() < 1e-12);
            assert!((scaled.cluster_effective_sample_size - 2.0).abs() < 1e-12);
        }
    }

    #[test]
    fn unequal_n_nj_weighted_between_differs_from_unweighted() {
        let rows = unequal_n_cwc_rows();
        let weights = vec![1.0_f64; rows.len()];
        let unweighted = recover_cluster_mean_within_between_slopes(&rows).expect("ols");
        let weighted =
            recover_kish_weighted_cluster_mean_within_between_slopes(&rows, &weights).expect("wls");
        assert!((weighted.within_slope - 0.5).abs() < 1e-12);
        assert!((unweighted.within_slope - 0.5).abs() < 1e-12);
        assert!((unweighted.between_slope - 2.0).abs() < 1e-12);
        assert!((weighted.between_slope - 2.25).abs() < 1e-12);
        assert!((weighted.between_slope - unweighted.between_slope).abs() > 1e-9);
        assert!((weighted.contextual_effect - 1.75).abs() < 1e-12);
        assert!(
            ((weighted.contextual_effect + weighted.within_slope) - weighted.between_slope).abs()
                < 1e-15
        );
        assert!((weighted.observation_effective_sample_size - 10.0).abs() < 1e-12);
        assert!((weighted.cluster_effective_sample_size - 100.0 / 44.0).abs() < 1e-12);
        let predictors: Vec<f64> = rows.iter().map(|row| row.predictor).collect();
        let outcomes: Vec<f64> = rows.iter().map(|row| row.outcome).collect();
        let pooled = recover_kish_weighted_slope(&predictors, &outcomes, &weights).expect("pooled");
        assert!((pooled - weighted.within_slope).abs() > 1e-9);
        assert_eq!(
            refuse_pooled_kish_slope_as_weighted_within_slope(pooled, weighted.within_slope),
            Err(PsychometricError::PooledKishSlopeIsNotWeightedWithinSlope)
        );
        assert_eq!(
            refuse_unweighted_between_slope_as_kish_weighted_between_slope(
                unweighted.between_slope,
                weighted.between_slope,
            ),
            Err(PsychometricError::UnweightedBetweenSlopeIsNotKishWeightedBetweenSlope)
        );
        assert_eq!(
            refuse_kish_effective_sample_size_as_slope(weighted.observation_effective_sample_size),
            Err(PsychometricError::KishEffectiveSampleSizeIsNotASlope)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn kish_weighted_cwc_fails_closed_on_length_weight_cluster_and_singular() {
        let rows = equal_n_cwc_rows();
        assert_eq!(
            recover_kish_weighted_cluster_mean_within_between_slopes(&[], &[]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_kish_weighted_cluster_mean_within_between_slopes(&rows, &[1.0, 1.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_kish_weighted_cluster_mean_within_between_slopes(&rows[..1], &[1.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_kish_weighted_cluster_mean_within_between_slopes(&rows, &[1.0, 1.0, 1.0, -0.1]),
            Err(PsychometricError::InvalidWeight)
        );
        let zero_cluster_weights = [1.0_f64, 1.0, 0.0, 0.0];
        assert_eq!(
            recover_kish_weighted_cluster_mean_within_between_slopes(&rows, &zero_cluster_weights),
            Err(PsychometricError::InvalidWeight)
        );
        assert_eq!(
            recover_kish_weighted_cluster_mean_within_between_slopes(
                &[
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
                ],
                &[1.0, 1.0],
            ),
            Err(PsychometricError::InsufficientClusters)
        );
        let nan_rows = [
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
        ];
        assert_eq!(
            recover_kish_weighted_cluster_mean_within_between_slopes(&nan_rows, &[1.0, 1.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
        let inf_rows = [
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
        ];
        assert_eq!(
            recover_kish_weighted_cluster_mean_within_between_slopes(&inf_rows, &[1.0, 1.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
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
            recover_kish_weighted_cluster_mean_within_between_slopes(
                &no_within,
                &[1.0, 1.0, 1.0, 1.0],
            ),
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
            recover_kish_weighted_cluster_mean_within_between_slopes(
                &no_between,
                &[1.0, 1.0, 1.0, 1.0],
            ),
            Err(PsychometricError::SingularDesign)
        );
        let overflowing_means = [
            ClusteredScore {
                cluster_key: 1,
                predictor: f64::MAX,
                outcome: f64::MAX,
            },
            ClusteredScore {
                cluster_key: 1,
                predictor: f64::MAX,
                outcome: f64::MAX,
            },
            ClusteredScore {
                cluster_key: 2,
                predictor: 0.0,
                outcome: 0.0,
            },
            ClusteredScore {
                cluster_key: 2,
                predictor: 1.0,
                outcome: 1.0,
            },
        ];
        assert_eq!(
            recover_kish_weighted_cluster_mean_within_between_slopes(
                &overflowing_means,
                &[1.0, 1.0, 1.0, 1.0],
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn kish_zero_underflow_cluster_is_omitted_not_invalid_weight() {
        let mixed_ess =
            kish_effective_sample_size(&[f64::MAX, f64::MIN_POSITIVE]).expect("mixed ess");
        assert!((mixed_ess - 1.0).abs() < 1e-12);
        let mixed_slope = recover_kish_weighted_slope(
            &[0.0, 1.0, 2.0],
            &[0.0, 2.0, 4.0],
            &[f64::MAX, f64::MAX, f64::MIN_POSITIVE],
        )
        .expect("mixed wls");
        assert!((mixed_slope - 2.0).abs() < 1e-12);
        let two_cluster = equal_n_cwc_rows();
        let two_cluster_weights = [f64::MAX, f64::MAX, f64::MAX, f64::MAX];
        let expected = recover_kish_weighted_cluster_mean_within_between_slopes(
            &two_cluster,
            &two_cluster_weights,
        )
        .expect("two-cluster max");
        let mut three_cluster = two_cluster.to_vec();
        three_cluster.push(ClusteredScore {
            cluster_key: 3,
            predictor: 100.0,
            outcome: 100.0,
        });
        three_cluster.push(ClusteredScore {
            cluster_key: 3,
            predictor: 200.0,
            outcome: 200.0,
        });
        let three_cluster_weights = [
            f64::MAX,
            f64::MAX,
            f64::MAX,
            f64::MAX,
            f64::MIN_POSITIVE,
            f64::MIN_POSITIVE,
        ];
        let recovered = recover_kish_weighted_cluster_mean_within_between_slopes(
            &three_cluster,
            &three_cluster_weights,
        )
        .expect("omit kish-zero");
        assert!((recovered.within_slope - expected.within_slope).abs() < 1e-12);
        assert!((recovered.between_slope - expected.between_slope).abs() < 1e-12);
        assert!((recovered.contextual_effect - expected.contextual_effect).abs() < 1e-12);
        assert!(
            (recovered.cluster_effective_sample_size - expected.cluster_effective_sample_size)
                .abs()
                < 1e-12
        );
        assert!(
            (recovered.observation_effective_sample_size
                - expected.observation_effective_sample_size)
                .abs()
                < 1e-12
        );
        let mixed_two = [f64::MAX, f64::MAX, f64::MIN_POSITIVE, f64::MIN_POSITIVE];
        assert_eq!(
            recover_kish_weighted_cluster_mean_within_between_slopes(&two_cluster, &mixed_two),
            Err(PsychometricError::InsufficientClusters)
        );
    }
}
