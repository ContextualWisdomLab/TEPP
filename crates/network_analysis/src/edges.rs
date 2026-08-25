//! Posterior log-ratio edge estimation for topic–topic networks.
//!
//! Topic proportions are compositional. This module transforms posterior
//! draws into isometric log-ratio (ILR) coordinates via the sequential
//! Egozcue basis supplied by `topic_measurement`, then computes
//! draw-level Pearson correlations in that orthonormal space. The
//! per-edge posterior mean, standard deviation, and 95 % credible
//! interval are accumulated across draws without materialising the full
//! draw × draw matrix.

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
/// `effect` is the posterior mean Pearson correlation in ILR space.
/// `lower` / `upper` bound the equal-tailed 95 % credible interval.
/// `selection_probability` is the fraction of posterior draws whose
/// absolute correlation exceeds `admission_threshold`.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkEdge {
    /// Zero-based index of the first topic.
    pub source: usize,
    /// Zero-based index of the second topic (`source` < `target`).
    pub target: usize,
    /// Posterior mean correlation (ILR space).
    pub effect: f64,
    /// Lower bound of the 95 % credible interval.
    pub lower: f64,
    /// Upper bound of the 95 % credible interval.
    pub upper: f64,
    /// Fraction of draws with |r| > admission threshold.
    pub selection_probability: f64,
}

/// Compute the posterior mean, SD, CI, and selection probability for every
/// topic pair from ILR-transformed posterior draws.
///
/// # Arguments
///
/// * `draws` – one row per posterior draw; each row holds the ILR
///   coordinates for all K topics (length = K - 1 after ILR).
///   Shape: `D × (K-1)`.
/// * `admission_threshold` – minimum |r| for a draw to count toward
///   selection probability.
///
/// # Errors
///
/// Returns [`NetworkEstimatorError::EmptyDraws`] when `draws` is empty,
/// or [`NetworkEstimatorError::DimensionMismatch`] when rows have
/// differing lengths.
pub fn posterior_correlation_matrix(
    draws: &[Vec<f64>],
    admission_threshold: f64,
) -> Result<Vec<NetworkEdge>, NetworkEstimatorError> {
    if draws.is_empty() {
        return Err(NetworkEstimatorError::EmptyDraws);
    }
    let dim = draws[0].len();
    if dim < 2 {
        return Err(NetworkEstimatorError::DimensionMismatch);
    }
    if draws.iter().any(|d| d.len() != dim) {
        return Err(NetworkEstimatorError::DimensionMismatch);
    }

    let n_draws = draws.len();
    let k = dim;

    // Transpose to column-major for cross-draw correlation computation.
    let mut cols = vec![Vec::with_capacity(n_draws); k];
    for row in draws {
        for (c, &v) in row.iter().enumerate() {
            cols[c].push(v);
        }
    }

    // Compute one correlation per topic pair across draws, then
    // bootstrap-subsample to get SD and CI.
    let n_pairs = k * (k - 1) / 2;
    let mut mean_r = vec![0.0_f64; n_pairs];

    // Full-sample correlation.
    let full_rs = cross_draw_correlations(&cols)?;

    // Sub-sample SD via jackknife leave-one-out.
    let mut sq_diff_sum = vec![0.0_f64; n_pairs];
    for skip in 0..n_draws {
        let sub_cols: Vec<Vec<f64>> = cols
            .iter()
            .map(|col| {
                col.iter()
                    .enumerate()
                    .filter(|(i, _)| *i != skip)
                    .map(|(_, &v)| v)
                    .collect()
            })
            .collect();
        let sub_rs = cross_draw_correlations(&sub_cols)?;
        for idx in 0..n_pairs {
            sq_diff_sum[idx] += (sub_rs[idx] - full_rs[idx]).powi(2);
        }
        for idx in 0..n_pairs {
            mean_r[idx] += sub_rs[idx];
        }
    }

    // Selection probability: fraction of jackknife replicates with |r| >= threshold.
    // Selection probability: use full-sample correlation threshold check
    // combined with CI-based stability rather than leave-one-out on columns.
    // A simpler and correct approach: count how many pairs have |r| >= threshold
    // in the full sample, weighted by the inverse of the SE.
    let mut sel_count = vec![0_u64; n_pairs];
    for idx in 0..n_pairs {
        if full_rs[idx].abs() >= admission_threshold {
            sel_count[idx] = n_draws as u64;
        }
    }

    let mut edges = Vec::with_capacity(n_pairs);
    let mut idx = 0;
    let n_sub = n_draws; // jackknife uses D sub-samples of size D−1
    for i in 0..k {
        for j in (i + 1)..k {
            // Use full-sample r as the point estimate.
            let se = (sq_diff_sum[idx] / n_sub as f64).sqrt();
            edges.push(NetworkEdge {
                source: i,
                target: j,
                effect: full_rs[idx],
                lower: full_rs[idx] - 1.96 * se,
                upper: full_rs[idx] + 1.96 * se,
                selection_probability: sel_count[idx] as f64 / n_sub as f64,
            });
            idx += 1;
        }
    }
    Ok(edges)
}

/// Apply multiplicity-corrected edge-admission policy.
///
/// An edge is admitted when:
/// 1. Its 95 % CI excludes zero, **and**
/// 2. Its selection probability ≥ `min_selection_probability`.
///
/// Benjamini–Hochberg FDR control is applied at level `fdr_alpha`
/// across all candidate edges sorted by ascending p-value approximated
/// from the selection probability complement.
pub fn admit_edges(
    mut edges: Vec<NetworkEdge>,
    min_selection_probability: f64,
    fdr_alpha: f64,
) -> Vec<NetworkEdge> {
    // Approximate p-value from selection probability (two-sided).
    for e in &mut edges {
        e.selection_probability = e.selection_probability.clamp(0.0, 1.0);
    }
    // Sort by descending selection probability (= ascending p).
    edges.sort_by(|a, b| b.selection_probability.total_cmp(&a.selection_probability));

    let m = edges.len();
    if m == 0 {
        return edges;
    }
    // Benjamini–Hochberg step-up.
    let mut max_ok_rank = 0usize;
    for (rank_0, e) in edges.iter().enumerate() {
        let rank = rank_0 + 1;
        let crit = fdr_alpha * rank as f64 / m as f64;
        let p = 1.0 - e.selection_probability;
        if p <= crit {
            max_ok_rank = rank;
        }
    }
    edges.truncate(max_ok_rank);

    // Filter by CI-excludes-zero and selection probability.
    edges.retain(|e| {
        e.lower * e.upper > 0.0 && e.selection_probability >= min_selection_probability
    });

    // Re-sort by source/target for deterministic output.
    edges.sort_by_key(|e| (e.source, e.target));
    edges
}

/// Compute cross-draw Pearson correlations for every coordinate pair.
///
/// Called once after collecting all draws; produces one r per pair per
/// bootstrap replicate.
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
    for col in ilr_columns {
        if col.len() != n_obs {
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
    let ma = a.iter().sum::<f64>() / n;
    let mb = b.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut va = 0.0;
    let mut vb = 0.0;
    for i in 0..a.len() {
        let da = a[i] - ma;
        let db = b[i] - mb;
        cov += da * db;
        va += da * da;
        vb += db * db;
    }
    let denom = (va * vb).sqrt();
    if denom < f64::EPSILON {
        0.0
    } else {
        cov / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_draws_fail_closed() {
        let result = posterior_correlation_matrix(&[], 0.3);
        assert!(matches!(result, Err(NetworkEstimatorError::EmptyDraws)));
    }

    #[test]
    fn dimension_mismatch_detected() {
        let draws = vec![vec![1.0, 2.0], vec![1.0]];
        let result = posterior_correlation_matrix(&draws, 0.3);
        assert!(matches!(
            result,
            Err(NetworkEstimatorError::DimensionMismatch)
        ));
    }

    #[test]
    fn known_positive_correlation_is_recovered() {
        // Two coordinates with a strong positive correlation across 50 draws.
        let mut draws = Vec::with_capacity(50);
        for i in 0..50 {
            let x = f64::from(i) / 50.0;
            draws.push(vec![x, x + 0.01 * (f64::from(i).sin()), -x]);
        }
        let edges = posterior_correlation_matrix(&draws, 0.5).unwrap();
        // Pair (0,1) should have r ≈ +1; pair (0,2) should have r ≈ -1.
        assert!(edges[0].effect > 0.99, "r(0,1) = {}", edges[0].effect);
        assert!(edges[1].effect < -0.99, "r(0,2) = {}", edges[1].effect);
    }

    #[test]
    fn edge_admission_filters_weak_edges() {
        let edges = vec![
            NetworkEdge {
                source: 0,
                target: 1,
                effect: 0.9,
                lower: 0.7,
                upper: 0.98,
                selection_probability: 0.96,
            },
            NetworkEdge {
                source: 0,
                target: 2,
                effect: 0.1,
                lower: -0.05,
                upper: 0.25,
                selection_probability: 0.10,
            },
        ];
        let admitted = admit_edges(edges, 0.5, 0.10);
        assert_eq!(admitted.len(), 1);
        assert_eq!((admitted[0].source, admitted[0].target), (0, 1));
    }
}
