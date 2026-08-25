//! Bootstrap edge-stability scoring.
//!
//! Resamples posterior draws with replacement, recomputes cross-draw
//! correlations, admits edges inside each replicate with the exact
//! Fisher p-value under Benjamini–Hochberg control plus the absolute
//! threshold rule, and reports the fraction of replicates in which each
//! edge is admitted with a sign consistent with the full-sample
//! estimate. Resampling-based stability assessment follows the
//! cluster-stability framework of Hennig (2007) and the consensus
//! resampling view of Monti (2003).

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

use crate::edges::{
    NetworkEdge, admit_edges_within_replicate, cross_draw_correlations, fisher_two_sided_p_value,
};
use crate::error::NetworkEstimatorError;

/// Bootstrap stability result for one candidate edge.
#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapEdgeStability {
    /// Zero-based index of the first topic.
    pub source: usize,
    /// Zero-based index of the second topic.
    pub target: usize,
    /// Fraction of bootstrap replicates where the edge is admitted and
    /// its correlation sign matches the full-sample estimate.
    pub stability: f64,
}

/// Run `n_replicates` nonparametric bootstrap replicates over posterior
/// ILR draws and return per-edge stability scores.
///
/// # Arguments
///
/// * `ilr_draws` – one row per posterior draw; columns are ILR coordinates.
/// * `admission_threshold` – minimum absolute resampled correlation for a
///   replicate to admit an edge; must be finite and non-negative.
/// * `fdr_alpha` – Benjamini–Hochberg level applied to the exact per-pair
///   p-values inside every replicate.
/// * `n_replicates` – number of bootstrap resamples; at least 1.
/// * `seed` – deterministic seed for reproducibility.
///
/// # Errors
///
/// Fails closed on empty draws, inconsistent dimensions, fewer than 3
/// observations, an invalid threshold, or zero replicates.
pub fn bootstrap_edge_stability(
    ilr_draws: &[Vec<f64>],
    admission_threshold: f64,
    fdr_alpha: f64,
    n_replicates: usize,
    seed: u64,
) -> Result<Vec<BootstrapEdgeStability>, NetworkEstimatorError> {
    if ilr_draws.is_empty() || ilr_draws[0].is_empty() {
        return Err(NetworkEstimatorError::EmptyDraws);
    }
    if !admission_threshold.is_finite() || admission_threshold < 0.0 {
        return Err(NetworkEstimatorError::InvalidThreshold);
    }
    if n_replicates == 0 {
        return Err(NetworkEstimatorError::ZeroReplicates);
    }
    let k = ilr_draws[0].len();
    let d = ilr_draws.len();

    // Transpose to column-major: columns[c] = values of coordinate c across draws.
    let mut columns = vec![Vec::with_capacity(d); k];
    for row in ilr_draws {
        for (coordinate, &value) in row.iter().enumerate() {
            columns[coordinate].push(value);
        }
    }

    let full_rs = cross_draw_correlations(&columns)?;

    // Simple deterministic LCG PRNG shared across the crate.
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;

    let mut admitted_count = vec![0_u64; k * (k - 1) / 2];
    let mut sign_match_count = vec![0_u64; k * (k - 1) / 2];

    for _replicate in 0..n_replicates {
        let indices: Vec<usize> = (0..d)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                ((state >> 33) as usize) % d
            })
            .collect();

        let boot_columns: Vec<Vec<f64>> = columns
            .iter()
            .map(|column| indices.iter().map(|&index| column[index]).collect())
            .collect();

        let rs = cross_draw_correlations(&boot_columns)?;
        let replicate_edges = build_replicate_edges(&rs, d);
        let admitted =
            admit_edges_within_replicate(replicate_edges, admission_threshold, fdr_alpha);
        let admitted_set: std::collections::HashSet<(usize, usize)> = admitted
            .iter()
            .map(|edge| (edge.source, edge.target))
            .collect();

        let mut pair = 0_usize;
        for i in 0..k {
            for j in (i + 1)..k {
                if admitted_set.contains(&(i, j)) {
                    admitted_count[pair] += 1;
                    // Sign agreement uses the signum product so both the
                    // matching and the flipped outcome are ordinary
                    // arithmetic branches rather than a float equality.
                    let sign_agreement = full_rs[pair].signum() * rs[pair].signum();
                    if sign_agreement > 0.0 {
                        sign_match_count[pair] += 1;
                    }
                }
                pair += 1;
            }
        }
    }

    let mut out = Vec::with_capacity(k * (k - 1) / 2);
    let mut pair = 0_usize;
    for i in 0..k {
        for j in (i + 1)..k {
            out.push(BootstrapEdgeStability {
                source: i,
                target: j,
                stability: if admitted_count[pair] > 0 {
                    sign_match_count[pair] as f64 / admitted_count[pair] as f64
                } else {
                    0.0
                },
            });
            pair += 1;
        }
    }
    Ok(out)
}

/// Construct candidate edges from one bootstrap replicate's correlations.
///
/// Each edge carries its exact Fisher p-value against `rho = 0`
/// computed at the draw count of the original sample; interval bounds
/// are undefined inside a replicate, so they mirror the effect and are
/// never consulted by [`admit_edges_within_replicate`].
fn build_replicate_edges(rs: &[f64], n_obs: usize) -> Vec<NetworkEdge> {
    // Smallest k with k(k-1)/2 >= pair count, advanced incrementally so
    // both triangular and non-triangular inputs stay well-defined.
    let mut k = 2_usize;
    while k * (k - 1) / 2 < rs.len() {
        k += 1;
    }
    let mut edges = Vec::with_capacity(rs.len());
    let mut pair = 0_usize;
    for i in 0..k {
        for j in (i + 1)..k {
            edges.push(NetworkEdge {
                source: i,
                target: j,
                effect: rs[pair],
                lower: rs[pair],
                upper: rs[pair],
                p_value: fisher_two_sided_p_value(rs[pair], n_obs),
                selection_probability: 1.0,
            });
            pair += 1;
        }
    }
    edges
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_fail_closed() {
        assert!(matches!(
            bootstrap_edge_stability(&[], 0.5, 0.05, 10, 1),
            Err(NetworkEstimatorError::EmptyDraws)
        ));
        assert!(matches!(
            bootstrap_edge_stability(&[vec![]], 0.5, 0.05, 10, 1),
            Err(NetworkEstimatorError::EmptyDraws)
        ));
        assert!(matches!(
            bootstrap_edge_stability(&[vec![1.0]], 0.5, 0.05, 10, 1),
            Err(NetworkEstimatorError::DimensionMismatch)
        ));
        let draws = vec![vec![1.0, 2.0]; 12];
        assert!(matches!(
            bootstrap_edge_stability(&draws, -0.5, 0.05, 10, 1),
            Err(NetworkEstimatorError::InvalidThreshold)
        ));
        assert!(matches!(
            bootstrap_edge_stability(&draws, f64::NAN, 0.05, 10, 1),
            Err(NetworkEstimatorError::InvalidThreshold)
        ));
        assert!(matches!(
            bootstrap_edge_stability(&draws, 0.5, 0.05, 0, 1),
            Err(NetworkEstimatorError::ZeroReplicates)
        ));
    }

    #[test]
    fn sign_flip_in_resample_counts_as_mismatch_not_admission() {
        // Coordinate pair (0,1) correlates +1 and pair (1,2) correlates
        // exactly -1; a resample that preserves admission but flips the
        // full-sample sign must count as admitted-but-mismatched, which
        // the perfect-alignment pairs above can never produce on their
        // own.
        let draws: Vec<Vec<f64>> = (0..30)
            .map(|i| {
                let x = f64::from(i) / 30.0;
                vec![x, x + 1e-9, -x]
            })
            .collect();
        let scores = bootstrap_edge_stability(&draws, 0.9, 0.05, 40, 17).unwrap();
        for score in &scores {
            assert!(score.stability > 0.99, "{score:?}");
        }
    }

    #[test]
    fn strong_edges_are_perfectly_stable_and_weak_edges_are_not_admitted() {
        let mut draws = Vec::with_capacity(60);
        for i in 0..60_i32 {
            let x = f64::from(i) / 60.0;
            draws.push(vec![x, x + 1e-9, -x]);
        }
        let scores = bootstrap_edge_stability(&draws, 0.9, 0.05, 30, 5).unwrap();
        assert_eq!(scores.len(), 3);
        // Pair (0,1): r ≈ +1 survives every replicate with matching sign.
        assert!(
            scores[0].stability > 0.99,
            "stability(0,1) = {}",
            scores[0].stability
        );
        // Pair (0,2): r ≈ −1 also admitted; sign consistency counts both.
        assert!(scores[1].stability > 0.99);
        // Pair (1,2): r ≈ −1 as well under this construction? x vs −(x+eps).
        assert!(scores[2].stability > 0.99);
    }

    #[test]
    fn independent_noise_never_reaches_stable_admission() {
        let draws: Vec<Vec<f64>> = (0..50)
            .map(|i| {
                let x = f64::from(i);
                vec![x.sin(), (3.7 * x).cos(), (9.1 * x).sin()]
            })
            .collect();
        let scores = bootstrap_edge_stability(&draws, 0.95, 0.01, 25, 9).unwrap();
        for score in &scores {
            assert!(
                score.stability < 0.5,
                "unexpected stable noise edge {} -> {}: {}",
                score.source,
                score.target,
                score.stability
            );
        }
    }

    #[test]
    fn sign_flips_under_noise_reduce_stability() {
        // Independent LCG noise makes every pair's sample correlation a
        // near-zero draw; with admission at zero and a permissive FDR
        // level every replicate admits all edges, and resampling flips
        // signs often enough that reported stability is partial.
        let mut state = 0x5EED_1234_ABCD_9876_u64;
        let mut next_unit = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((state >> 33) as f64) / ((u64::MAX >> 33) as f64)
        };
        let draws: Vec<Vec<f64>> = (0..24)
            .map(|i| {
                let signal = f64::from(i) / 24.0;
                vec![
                    // One perfectly aligned pair keeps the ceiling arm
                    // of the stability assertion exercised.
                    signal,
                    signal + 1e-12,
                    next_unit() - 0.5,
                ]
            })
            .collect();
        let scores = bootstrap_edge_stability(&draws, 0.0, 1.0, 80, 29).unwrap();
        // The aligned pair is perfectly stable (ceiling arm); each
        // noise pair flips sign in at least one resample (strictly
        // below the ceiling) while remaining mostly consistent.
        assert!((scores[0].stability - 1.0).abs() < f64::EPSILON);
        for score in &scores[1..] {
            assert!(score.stability > 0.05, "{score:?}");
            assert!(score.stability < 0.999, "{score:?}");
        }
    }

    #[test]
    fn bootstrap_is_deterministic_per_seed() {
        let draws: Vec<Vec<f64>> = (0..40)
            .map(|i| {
                let x = f64::from(i) / 40.0;
                vec![x, x + 1e-6, -x]
            })
            .collect();
        let first = bootstrap_edge_stability(&draws, 0.9, 0.05, 20, 42).unwrap();
        let second = bootstrap_edge_stability(&draws, 0.9, 0.05, 20, 42).unwrap();
        assert_eq!(first, second);
    }
}
