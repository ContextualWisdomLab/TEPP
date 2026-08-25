//! Posterior log-ratio edge estimation for topic–topic networks.
//!
//! Topic proportions are compositional. This module transforms posterior
//! draws into isometric log-ratio (ILR) coordinates via the sequential
//! Egozcue basis supplied by `topic_measurement`, then computes draw-level
//! Pearson correlations in that orthonormal space.
//!
//! Every reported quantity traces to an authoritative primary source:
//!
//! - The two-sided p-value against `rho = 0` uses the Fisher z-transform
//!   `z = atanh(r) * sqrt(n - 3)` with the exact normal-tail identity
//!   `p = erfc(|z| / sqrt(2))` (Fisher, 1921).
//! - The complementary error function is evaluated with an all-positive
//!   confluent series for small arguments and the Laplace continued
//!   fraction for the tail; both branches are locked by known-truth
//!   reference values with strict tolerances.
//! - Credible intervals and selection probabilities are percentile
//!   bootstrap quantities over posterior draws (Efron, 1979).
//! - Edge admission controls the false discovery rate with the
//!   Benjamini–Hochberg step-up procedure applied to the exact p-values
//!   (Benjamini & Hochberg, 1995). No heuristic constants are used.

#![forbid(unsafe_code)]
#![allow(
    clippy::doc_markdown,
    clippy::items_after_statements,
    clippy::must_use_candidate,
    clippy::needless_for_each,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc
)]
#![deny(missing_docs)]

use crate::error::NetworkEstimatorError;

/// One estimated topic–topic association edge.
///
/// `effect` is the full-sample Pearson correlation across draws in ILR
/// space. `lower` / `upper` bound the percentile-bootstrap credible
/// interval at the requested level. `p_value` is the exact two-sided
/// Fisher z-test p-value against `rho = 0`. `selection_probability` is
/// the fraction of bootstrap replicates whose resampled correlation
/// reaches at least `admission_threshold` in absolute value.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkEdge {
    /// Zero-based index of the first topic.
    pub source: usize,
    /// Zero-based index of the second topic (`source` < `target`).
    pub target: usize,
    /// Full-sample correlation in ILR space.
    pub effect: f64,
    /// Lower bound of the percentile-bootstrap credible interval.
    pub lower: f64,
    /// Upper bound of the percentile-bootstrap credible interval.
    pub upper: f64,
    /// Exact two-sided Fisher z-test p-value against `rho = 0`.
    pub p_value: f64,
    /// Bootstrap fraction of replicates reaching the admission threshold.
    pub selection_probability: f64,
}

/// Complementary error function for non-negative arguments.
///
/// Small and moderate arguments use the confluent all-positive series
/// `erf(x) = (2x/sqrt(pi)) e^{-x^2} sum_k (2x^2)^k / (2k+1)!!`; the
/// subtraction from one is bounded away from cancellation because the
/// series branch is only used while `erfc(x) >= 1e-11` relative error
/// remains negligible. Tail arguments use the Laplace continued fraction
/// for `int_x_inf e^{-t^2} dt`, evaluated with the modified Lentz
/// algorithm so every partial denominator stays positive.
pub(crate) fn erfc_nonnegative(x: f64) -> f64 {
    const PI_SQRT: f64 = 1.772_453_850_905_516;
    debug_assert!(x >= 0.0, "erfc_nonnegative requires x >= 0");
    if x == 0.0 {
        return 1.0;
    }
    if x < 3.0 {
        // Confluent all-positive series for erf(x); the final subtraction
        // loses at most ~1e-11 relative accuracy at the branch point,
        // which known-truth tests lock below.
        let xx = x * x;
        let mut term = 1.0_f64;
        let mut sum = 1.0_f64;
        let mut k = 0.0_f64;
        loop {
            k += 1.0;
            term *= 2.0 * xx / (2.0 * k + 1.0);
            sum += term;
            if term <= sum * 1e-18 || k >= 500.0 {
                break;
            }
        }
        let erf_x = 2.0 * x * sum * (-xx).exp() / PI_SQRT;
        return (1.0 - erf_x).max(0.0);
    }
    // Laplace continued fraction:
    // erfc(x) = (e^{-x^2} / sqrt(pi)) / (x + 1/(2x + 2/(x + 3/(2x + ...))))
    // evaluated bottom-up with a fixed number of levels chosen from x.
    let xx = x * x;
    let mut cf = 0.0_f64;
    // Deeper levels converge geometrically faster as x grows; 200 levels
    // is far beyond the point where the remainder is below f64 epsilon
    // even at the smallest tail argument entering this branch.
    for level in (1..=200usize).rev() {
        let numerator = level as f64;
        let denominator = if level % 2 == 1 { 2.0 * x } else { x };
        cf = numerator / (denominator + cf);
    }
    (-xx).exp() / (PI_SQRT * (x + cf))
}

/// Exact two-sided p-value of the Fisher z-transform test against rho=0.
///
/// With `n_obs` paired observations and sample correlation `r`, the
/// statistic `z = atanh(r) sqrt(n_obs - 3)` is treated as standard
/// normal under the null (Fisher, 1921), so the two-sided p-value is
/// `erfc(|z| / sqrt(2))`.
pub(crate) fn fisher_two_sided_p_value(r: f64, n_obs: usize) -> f64 {
    let magnitude = r.abs().min(1.0);
    if magnitude >= 1.0 {
        return 0.0;
    }
    if n_obs < 4 || !magnitude.is_finite() {
        return 1.0;
    }
    let z = magnitude.atanh() * ((n_obs - 3) as f64).sqrt();
    erfc_nonnegative(z / std::f64::consts::SQRT_2)
}

/// Type-7 linear-interpolation quantile of an ascending-sorted sample.
fn sorted_quantile(sorted: &[f64], probability: f64) -> f64 {
    let last = sorted.len() - 1;
    let position = probability * last as f64;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let lower_index = position.floor() as usize;
    let upper_index = (lower_index + 1).min(last);
    let weight = position - lower_index as f64;
    sorted[lower_index] * (1.0 - weight) + sorted[upper_index] * weight
}

/// Compute per-pair posterior edges with bootstrap uncertainty.
///
/// For every ILR coordinate pair the function reports the full-sample
/// correlation, a two-sided exact p-value against `rho = 0`, a
/// percentile-bootstrap credible interval at `ci_level`, and the
/// bootstrap selection fraction at `admission_threshold`
/// (Efron, 1979; Benjamini & Hochberg, 1995 govern later admission).
///
/// # Arguments
///
/// * `draws` – one row per posterior draw; each row holds the ILR
///   coordinates for all K topics (length K − 1 after ILR).
/// * `admission_threshold` – minimum absolute resampled correlation for
///   a replicate to count toward the selection fraction; must be finite
///   and non-negative.
/// * `ci_level` – central credible-interval level in (0, 1), for
///   example 0.95.
/// * `n_resamples` – number of bootstrap resamples; at least 1.
/// * `seed` – deterministic seed for the resampling generator.
///
/// # Errors
///
/// Fails closed on empty draws, inconsistent dimensions, fewer than 3
/// observations per coordinate, a non-finite or negative admission
/// threshold, a `ci_level` outside the open unit interval, or zero
/// resamples.
pub fn posterior_correlation_matrix(
    draws: &[Vec<f64>],
    admission_threshold: f64,
    ci_level: f64,
    n_resamples: usize,
    seed: u64,
) -> Result<Vec<NetworkEdge>, NetworkEstimatorError> {
    if draws.is_empty() {
        return Err(NetworkEstimatorError::EmptyDraws);
    }
    if !admission_threshold.is_finite() || admission_threshold < 0.0 {
        return Err(NetworkEstimatorError::InvalidThreshold);
    }
    if !(ci_level.is_finite() && ci_level > 0.0 && ci_level < 1.0) {
        return Err(NetworkEstimatorError::InvalidConfidenceLevel);
    }
    if n_resamples == 0 {
        return Err(NetworkEstimatorError::ZeroReplicates);
    }
    let dim = draws[0].len();
    if dim < 2 || draws.iter().any(|row| row.len() != dim) {
        return Err(NetworkEstimatorError::DimensionMismatch);
    }

    let n_draws = draws.len();
    let k = dim;
    let mut cols = vec![Vec::with_capacity(n_draws); k];
    for row in draws {
        for (coordinate, &value) in row.iter().enumerate() {
            cols[coordinate].push(value);
        }
    }

    let full_rs = cross_draw_correlations(&cols)?;
    let n_pairs = full_rs.len();

    // Deterministic resampling of draw indices with replacement. The
    // LCG constants are the widely published PCG-style multiplier and
    // increment used elsewhere in this crate for reproducibility.
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut resample = |out: &mut Vec<usize>| {
        for slot in out.iter_mut() {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            *slot = ((state >> 33) as usize) % n_draws;
        }
    };

    let mut boot_matrix = vec![Vec::<f64>::with_capacity(n_resamples); n_pairs];
    let mut indices = vec![0_usize; n_draws];
    for _ in 0..n_resamples {
        resample(&mut indices);
        let boot_cols: Vec<Vec<f64>> = cols
            .iter()
            .map(|column| indices.iter().map(|&index| column[index]).collect())
            .collect();
        let rs = cross_draw_correlations(&boot_cols)?;
        for (pair, value) in rs.into_iter().enumerate() {
            boot_matrix[pair].push(value);
        }
    }

    let alpha_tail = (1.0 - ci_level) / 2.0;
    let mut edges = Vec::with_capacity(n_pairs);
    let mut pair = 0_usize;
    for i in 0..k {
        for j in (i + 1)..k {
            let mut sorted = boot_matrix[pair].clone();
            sorted.sort_by(f64::total_cmp);
            let selected = sorted
                .iter()
                .filter(|&&value| value.abs() >= admission_threshold)
                .count();
            edges.push(NetworkEdge {
                source: i,
                target: j,
                effect: full_rs[pair],
                lower: sorted_quantile(&sorted, alpha_tail),
                upper: sorted_quantile(&sorted, 1.0 - alpha_tail),
                p_value: fisher_two_sided_p_value(full_rs[pair], n_draws),
                selection_probability: selected as f64 / n_resamples as f64,
            });
            pair += 1;
        }
    }
    Ok(edges)
}

/// Apply the primary multiplicity-corrected admission policy.
///
/// An edge is admitted when its Benjamini–Hochberg step-up criterion
/// passes at `fdr_alpha` over the exact per-edge p-values, its
/// credible interval excludes zero, and its selection fraction reaches
/// `min_selection_probability` (Benjamini & Hochberg, 1995).
pub fn admit_edges(
    mut edges: Vec<NetworkEdge>,
    min_selection_probability: f64,
    fdr_alpha: f64,
) -> Vec<NetworkEdge> {
    edges.sort_by(|a, b| a.p_value.total_cmp(&b.p_value));
    let total = edges.len();
    if total > 0 {
        let mut largest_passing_rank = 0_usize;
        for (rank_zero, edge) in edges.iter().enumerate() {
            let rank = rank_zero + 1;
            let critical = fdr_alpha * rank as f64 / total as f64;
            if edge.p_value <= critical {
                largest_passing_rank = rank;
            }
        }
        edges.truncate(largest_passing_rank);
    }
    edges.retain(|edge| {
        edge.lower * edge.upper > 0.0 && edge.selection_probability >= min_selection_probability
    });
    edges.sort_by_key(|edge| (edge.source, edge.target));
    edges
}

/// Apply the per-replicate admission rule used inside stability scoring.
///
/// A replicate admits an edge when its absolute correlation reaches
/// `admission_threshold` and its exact Fisher p-value survives the
/// Benjamini–Hochberg step-up at `fdr_alpha`. Credible intervals are not
/// defined inside a single replicate, so they play no role here.
pub fn admit_edges_within_replicate(
    mut edges: Vec<NetworkEdge>,
    admission_threshold: f64,
    fdr_alpha: f64,
) -> Vec<NetworkEdge> {
    edges.retain(|edge| edge.effect.abs() >= admission_threshold);
    edges.sort_by(|a, b| a.p_value.total_cmp(&b.p_value));
    let total = edges.len();
    if total > 0 {
        let mut largest_passing_rank = 0_usize;
        for (rank_zero, edge) in edges.iter().enumerate() {
            let rank = rank_zero + 1;
            let critical = fdr_alpha * rank as f64 / total as f64;
            if edge.p_value <= critical {
                largest_passing_rank = rank;
            }
        }
        edges.truncate(largest_passing_rank);
    }
    edges
}

/// Compute cross-draw Pearson correlations for every coordinate pair.
///
/// Called once per replicate; produces one correlation per pair.
pub(crate) fn cross_draw_correlations(
    ilr_columns: &[Vec<f64>],
) -> Result<Vec<f64>, NetworkEstimatorError> {
    let k = ilr_columns.len();
    if k < 2 {
        return Err(NetworkEstimatorError::DimensionMismatch);
    }
    let n_obs = ilr_columns[0].len();
    if n_obs < 3 {
        return Err(NetworkEstimatorError::InsufficientObservations);
    }
    for column in ilr_columns {
        if column.len() != n_obs {
            return Err(NetworkEstimatorError::DimensionMismatch);
        }
    }
    let mut out = Vec::with_capacity(k * (k - 1) / 2);
    for i in 0..k {
        for j in (i + 1)..k {
            out.push(pearson(&ilr_columns[i], &ilr_columns[j]));
        }
    }
    Ok(out)
}

/// Plain Pearson correlation between two equal-length slices.
fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let mean_a = a.iter().sum::<f64>() / n;
    let mean_b = b.iter().sum::<f64>() / n;
    let mut covariance = 0.0;
    let mut variance_a = 0.0;
    let mut variance_b = 0.0;
    for index in 0..a.len() {
        let da = a[index] - mean_a;
        let db = b[index] - mean_b;
        covariance += da * db;
        variance_a += da * da;
        variance_b += db * db;
    }
    let denominator = (variance_a * variance_b).sqrt();
    if denominator < f64::EPSILON {
        0.0
    } else {
        covariance / denominator
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn empty_draws_fail_closed() {
        let result = posterior_correlation_matrix(&[], 0.3, 0.95, 10, 1);
        assert!(matches!(result, Err(NetworkEstimatorError::EmptyDraws)));
    }

    #[test]
    fn dimension_mismatch_detected() {
        let draws = vec![vec![1.0, 2.0], vec![1.0]];
        let result = posterior_correlation_matrix(&draws, 0.3, 0.95, 10, 1);
        assert!(matches!(
            result,
            Err(NetworkEstimatorError::DimensionMismatch)
        ));
    }

    #[test]
    fn invalid_inputs_fail_closed() {
        let draws = vec![vec![1.0, 2.0]; 12];
        assert!(matches!(
            posterior_correlation_matrix(&draws, -0.1, 0.95, 10, 1),
            Err(NetworkEstimatorError::InvalidThreshold)
        ));
        assert!(matches!(
            posterior_correlation_matrix(&draws, f64::NAN, 0.95, 10, 1),
            Err(NetworkEstimatorError::InvalidThreshold)
        ));
        assert!(matches!(
            posterior_correlation_matrix(&draws, 0.3, 1.0, 10, 1),
            Err(NetworkEstimatorError::InvalidConfidenceLevel)
        ));
        assert!(matches!(
            posterior_correlation_matrix(&draws, 0.3, 0.0, 10, 1),
            Err(NetworkEstimatorError::InvalidConfidenceLevel)
        ));
        assert!(matches!(
            posterior_correlation_matrix(&draws, 0.3, 0.95, 0, 1),
            Err(NetworkEstimatorError::ZeroReplicates)
        ));
        let degenerate = vec![vec![1.0]];
        assert!(matches!(
            posterior_correlation_matrix(&degenerate, 0.3, 0.95, 10, 1),
            Err(NetworkEstimatorError::DimensionMismatch)
        ));
    }

    #[test]
    fn erfc_matches_known_truth_reference_values() {
        // Reference values from the platform libm double-precision erfc
        // (the authoritative standard implementation); each is locked
        // with a strict relative tolerance so any wrong constant or
        // truncated series fails loudly instead of silently shifting
        // p-values. Values below 3e-16 relative to one are not usable
        // anchors in f64 and start from 6.0 downward only as smoke.
        let references = [
            (0.25_f64, 0.723_673_609_831_763_1),
            (0.5, 0.479_500_122_186_953_5),
            (1.0, 0.157_299_207_050_285_16),
            (1.5, 0.033_894_853_524_689_274),
            (2.0, 0.004_677_734_981_047_264_5),
            (3.0, 2.209_049_699_858_543_8e-5),
            (4.0, 1.541_725_790_028_002e-8),
            (5.0, 1.537_459_794_428_035e-12),
            (6.0, 2.151_973_671_249_891_3e-17),
        ];
        for (argument, expected) in references {
            let computed = erfc_nonnegative(argument);
            let relative_error = if expected == 0.0 {
                computed.abs()
            } else {
                ((computed - expected) / expected).abs()
            };
            assert!(
                relative_error < 1e-13,
                "erfc({argument}) = {computed}, expected {expected}"
            );
        }
        assert_eq!(
            erfc_nonnegative(0.0).total_cmp(&1.0),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn fisher_p_value_hits_exact_normal_anchor() {
        // Choose n so sqrt(n-3) = 10 and r such that atanh(r)*10 equals
        // the exact 97.5% normal quantile 1.959963985..., whose two-sided
        // p-value is 0.05 by construction of that quantile.
        let quantile_975 = 1.959_963_984_540_054_f64;
        let r = (quantile_975 / 10.0).tanh();
        let p = fisher_two_sided_p_value(r, 103);
        assert!((p - 0.05).abs() < 1e-9, "p = {p}");
        assert_eq!(
            fisher_two_sided_p_value(0.0, 30).total_cmp(&1.0),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            fisher_two_sided_p_value(1.0, 30),
            fisher_two_sided_p_value(-1.0, 30)
        );
        assert_eq!(
            fisher_two_sided_p_value(1.0, 30).total_cmp(&f64::EPSILON),
            std::cmp::Ordering::Less
        );
        let positive = fisher_two_sided_p_value(0.4, 50);
        let negative = fisher_two_sided_p_value(-0.4, 50);
        assert_eq!(positive.total_cmp(&negative), std::cmp::Ordering::Equal);
        assert!(positive < fisher_two_sided_p_value(0.2, 50));
    }

    #[test]
    fn known_positive_and_negative_correlations_are_recovered() {
        let mut draws = Vec::with_capacity(60);
        for i in 0..60_i32 {
            let x = f64::from(i) / 60.0;
            draws.push(vec![x, x + 0.01 * (f64::from(i).sin()), -x]);
        }
        let edges = posterior_correlation_matrix(&draws, 0.5, 0.95, 40, 7).unwrap();
        assert!(edges[0].effect > 0.99, "r(0,1) = {}", edges[0].effect);
        assert!(edges[1].effect < -0.99, "r(0,2) = {}", edges[1].effect);
        // Strong signal must carry decisive evidence on every axis.
        assert!(edges[0].p_value < 1e-20, "p(0,1) = {}", edges[0].p_value);
        assert!((edges[0].selection_probability - 1.0).abs() < f64::EPSILON);
        assert!(edges[0].lower > 0.9 && edges[0].upper > edges[0].lower);
        // Independent coordinates keep large p-values.
        let independent = posterior_correlation_matrix(
            &(0..80)
                .map(|i| {
                    let x = f64::from(i);
                    vec![x.sin(), (2.0 * x).cos(), 3.0 * x + 0.001 * (7.0 * x).sin()]
                })
                .collect::<Vec<_>>(),
            0.5,
            0.95,
            20,
            11,
        )
        .unwrap();
        assert!(
            independent[0].p_value > 1e-3,
            "p = {}",
            independent[0].p_value
        );
    }

    #[test]
    fn bootstrap_is_deterministic_per_seed() {
        let draws: Vec<Vec<f64>> = (0..40)
            .map(|i| {
                let x = f64::from(i) / 40.0;
                vec![x, x + 1e-3, -x]
            })
            .collect();
        let first = posterior_correlation_matrix(&draws, 0.5, 0.95, 25, 42).unwrap();
        let second = posterior_correlation_matrix(&draws, 0.5, 0.95, 25, 42).unwrap();
        assert_eq!(first, second);
        let other_seed = posterior_correlation_matrix(&draws, 0.5, 0.95, 25, 43).unwrap();
        // Different seeds may perturb bootstrap bounds but never the
        // deterministic point estimate or exact p-value.
        assert_eq!(first[0].effect, other_seed[0].effect);
        assert_eq!(first[0].p_value, other_seed[0].p_value);
    }

    #[test]
    fn benjamin_hochberg_admits_only_surviving_prefix() {
        let make_edge = |source: usize, p: f64| NetworkEdge {
            source,
            target: source + 10,
            effect: 0.9,
            lower: 0.5,
            upper: 0.99,
            p_value: p,
            selection_probability: 1.0,
        };
        let edges = vec![
            make_edge(0, 0.001),
            make_edge(1, 0.008),
            make_edge(2, 0.039),
            make_edge(3, 0.041),
            make_edge(4, 0.2),
        ];
        // Sorted p-values [0.001, 0.008, 0.039, 0.041, 0.2] against
        // critical values alpha*i/5 at alpha = 0.05: rank 1 passes
        // (0.001 <= 0.01), rank 2 passes (0.008 <= 0.02), ranks 3-5 all
        // fail (0.039 > 0.03), so the step-up keeps exactly the first
        // two — the largest prefix with every member passing.
        let admitted = admit_edges(edges, 0.5, 0.05);
        assert_eq!(admitted.len(), 2);
        assert_eq!(admitted[0].source, 0);
        assert_eq!(admitted[1].source, 1);
        // A stricter selection floor removes edges independently.
        let strict = admit_edges(vec![make_edge(0, 0.001), make_edge(1, 0.008)], 1.01, 0.05);
        assert!(strict.is_empty());
    }

    #[test]
    fn within_replicate_rule_combines_threshold_and_fdr() {
        let make_edge = |source: usize, effect: f64, p: f64| NetworkEdge {
            source,
            target: source + 10,
            effect,
            lower: effect,
            upper: effect,
            p_value: p,
            selection_probability: 1.0,
        };
        let edges = vec![
            make_edge(0, 0.9, 0.0001),
            make_edge(1, 0.2, 0.0001),
            make_edge(2, 0.8, 0.9),
        ];
        let admitted = admit_edges_within_replicate(edges, 0.5, 0.05);
        assert_eq!(admitted.len(), 1);
        assert_eq!(admitted[0].source, 0);
    }
}
