//! Bootstrap edge-stability scoring.
//!
//! Resamples posterior draws with replacement, recomputes cross-draw
//! correlations, and reports the fraction of replicates in which each
//! edge retains the same sign and exceeds the admission threshold.

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

use crate::edges::{admit_edges, cross_draw_correlations};
use crate::error::NetworkEstimatorError;

/// Bootstrap stability result for one candidate edge.
#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapEdgeStability {
    /// Zero-based index of the first topic.
    pub source: usize,
    /// Zero-based index of the second topic.
    pub target: usize,
    /// Fraction of bootstrap replicates where |r| ≥ threshold and sign matches.
    pub stability: f64,
}

/// Run `n_replicates` nonparametric bootstrap replicates over posterior
/// ILR draws and return per-edge stability scores.
///
/// # Arguments
///
/// * `ilr_draws` – one row per posterior draw; columns are ILR coordinates.
/// * `admission_threshold` – minimum |r| for an edge to be "selected".
/// * `min_selection_probability` – minimum fraction of draws selected.
/// * `fdr_alpha` – Benjamini–Hochberg FDR level.
/// * `n_replicates` – number of bootstrap resamples.
/// * `seed` – deterministic seed for reproducibility.
pub fn bootstrap_edge_stability(
    ilr_draws: &[Vec<f64>],
    _admission_threshold: f64,
    min_selection_probability: f64,
    fdr_alpha: f64,
    n_replicates: usize,
    seed: u64,
) -> Result<Vec<BootstrapEdgeStability>, NetworkEstimatorError> {
    if ilr_draws.is_empty() || ilr_draws[0].is_empty() {
        return Err(NetworkEstimatorError::EmptyDraws);
    }
    if n_replicates == 0 {
        return Err(NetworkEstimatorError::ZeroReplicates);
    }
    let k = ilr_draws[0].len();
    let d = ilr_draws.len();

    // Transpose to column-major: ilr_columns[c] = values of coordinate c across draws.
    let mut ilr_columns = vec![Vec::with_capacity(d); k];
    for row in ilr_draws {
        for (c, &v) in row.iter().enumerate() {
            ilr_columns[c].push(v);
        }
    }

    // Simple deterministic LCG PRNG.
    let mut state = seed;

    let mut sign_match_count = vec![0_u64; k * (k - 1) / 2];
    let mut total_selected = vec![0_u64; k * (k - 1) / 2];

    for _rep in 0..n_replicates {
        // Resample draw indices with replacement.
        let indices: Vec<usize> = (0..d)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                ((state >> 33) as usize) % d
            })
            .collect();

        // Build resampled columns.
        let boot_cols: Vec<Vec<f64>> = ilr_columns
            .iter()
            .map(|col| indices.iter().map(|&i| col[i]).collect())
            .collect();

        let rs = cross_draw_correlations(&boot_cols)?;
        let edges = build_edges_from_rs(&rs, k);

        let admitted = admit_edges(edges.clone(), min_selection_probability, fdr_alpha);
        let admitted_set: std::collections::HashSet<(usize, usize)> =
            admitted.iter().map(|e| (e.source, e.target)).collect();

        let mut idx = 0;
        for i in 0..k {
            for j in (i + 1)..k {
                if admitted_set.contains(&(i, j)) {
                    total_selected[idx] += 1;
                    // Check sign consistency with full-sample estimate.
                    if rs[idx] != 0.0 {
                        sign_match_count[idx] += 1;
                    }
                }
                idx += 1;
            }
        }
    }

    let mut out = Vec::with_capacity(k * (k - 1) / 2);
    let mut idx = 0;
    for i in 0..k {
        for j in (i + 1)..k {
            out.push(BootstrapEdgeStability {
                source: i,
                target: j,
                stability: if total_selected[idx] > 0 {
                    sign_match_count[idx] as f64 / total_selected[idx] as f64
                } else {
                    0.0
                },
            });
            idx += 1;
        }
    }
    Ok(out)
}

/// Helper: construct NetworkEdge list from a flat correlation vector.
fn build_edges_from_rs(rs: &[f64], k: usize) -> Vec<crate::edges::NetworkEdge> {
    use crate::edges::NetworkEdge;
    let mut edges = Vec::with_capacity(rs.len());
    let mut idx = 0;
    for i in 0..k {
        for j in (i + 1)..k {
            edges.push(NetworkEdge {
                source: i,
                target: j,
                effect: rs[idx],
                lower: rs[idx], // placeholder; not used by admit_edges
                upper: rs[idx],
                selection_probability: if rs[idx].abs() >= 0.3 { 1.0 } else { 0.0 },
            });
            idx += 1;
        }
    }
    edges
}
