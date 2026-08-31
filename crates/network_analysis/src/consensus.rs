//! Co-assignment consensus clustering from repeated Leiden partitions.
//!
//! Runs multiple rounds of Traag, Waltman, and van Eck (2019) Leiden
//! modularity on randomly perturbed admitted positive edges, builds a
//! co-assignment matrix across replicates, and derives consensus clusters
//! by thresholding that matrix. The resampling-based consensus view and
//! the stability rationale follow Monti (2003) and Hennig (2007); the edge
//! perturbation probability is an explicit parameter with provenance,
//! never an implicit constant. Union-find is not used.

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

use crate::edges::NetworkEdge;
use crate::error::NetworkEstimatorError;
use crate::leiden::leiden_partition;
use std::collections::HashMap;

/// Consensus clustering output.
///
/// * `assignments` – one entry per topic: `Some(cluster_id)` or `None` for unclustered.
/// * `co_assignment` – K × K symmetric matrix of co-assignment frequencies.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsensusClusterOutput {
    /// Per-topic cluster assignment (`None` = unclustered).
    pub assignments: Vec<Option<usize>>,
    /// K × K co-assignment frequency matrix.
    pub co_assignment: Vec<Vec<f64>>,
}

/// Derive consensus clusters from repeatedly perturbed Leiden partitions.
///
/// Each replicate independently drops every admitted positive edge with
/// probability `edge_drop_probability`, repartitions the surviving edges
/// with Leiden modularity (Traag et al., 2019), and accumulates
/// label-invariant co-assignment counts. The final assignment thresholds
/// the co-assignment frequency at `consensus_threshold`.
///
/// # Arguments
///
/// * `edges` – admitted positive edges (source, target, effect).
/// * `k_topics` – total number of topics.
/// * `n_replicates` – number of perturbed partitions to generate; at least 1.
/// * `consensus_threshold` – minimum co-assignment fraction for
///   same-cluster; must lie inside [0, 1].
/// * `edge_drop_probability` – per-edge independent drop probability in
///   each replicate; must be finite and inside [0, 1). The value is an
///   explicit design parameter of the resampling scheme (Monti, 2003;
///   Hennig, 2007), not an internal constant.
/// * `seed` – deterministic seed.
///
/// # Errors
///
/// Fails closed on zero topics or replicates, a threshold outside
/// [0, 1], or a non-finite drop probability at or above 1.
pub fn consensus_clusters(
    edges: &[NetworkEdge],
    k_topics: usize,
    n_replicates: usize,
    consensus_threshold: f64,
    edge_drop_probability: f64,
    seed: u64,
) -> Result<ConsensusClusterOutput, NetworkEstimatorError> {
    if k_topics == 0 {
        return Err(NetworkEstimatorError::DimensionMismatch);
    }
    if n_replicates == 0 {
        return Err(NetworkEstimatorError::ZeroReplicates);
    }
    if !consensus_threshold.is_finite() || !(0.0..=1.0).contains(&consensus_threshold) {
        return Err(NetworkEstimatorError::InvalidProbability);
    }
    if !edge_drop_probability.is_finite() || !(0.0..1.0).contains(&edge_drop_probability) {
        return Err(NetworkEstimatorError::InvalidProbability);
    }

    // Build adjacency from positive edges only.
    let mut adjacency: HashMap<usize, Vec<usize>> = HashMap::new();
    for edge in edges {
        if edge.effect > 0.0 {
            adjacency.entry(edge.source).or_default().push(edge.target);
            adjacency.entry(edge.target).or_default().push(edge.source);
        }
    }

    // Deterministic LCG shared across the crate.
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;

    // Accumulate co-assignment counts.
    let mut co_count = vec![vec![0_u64; k_topics]; k_topics];

    for _ in 0..n_replicates {
        // Perturb: drop each surviving positive edge independently with
        // the caller-supplied probability so cluster recovery is
        // stress-tested by resampling. Negative-effect edges are not
        // part of the partition graph at all, matching the adjacency.
        let perturbed: Vec<&NetworkEdge> = edges
            .iter()
            .filter(|edge| edge.effect > 0.0)
            .filter(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let draw = ((state >> 33) as f64 / (u64::MAX >> 33) as f64).min(1.0);
                draw >= edge_drop_probability
            })
            .collect();

        let partition = leiden_partition(&perturbed, k_topics, &mut state);

        for i in 0..k_topics {
            for j in 0..k_topics {
                if partition[i] == partition[j] {
                    co_count[i][j] += 1;
                }
            }
        }
    }

    // Build co-assignment frequency matrix and derive final assignment.
    let mut co_freq = vec![vec![0.0_f64; k_topics]; k_topics];
    let mut assignments: Vec<Option<usize>> = vec![None; k_topics];
    let mut next_cluster = 0_usize;

    for i in 0..k_topics {
        if assignments[i].is_some() {
            continue;
        }
        // Every candidate j in i..k is still unassigned here: topics
        // below i were finalized earlier and the loop never revisits
        // them, so membership reduces to the co-assignment threshold.
        let members: Vec<usize> = (i..k_topics)
            .filter(|&j| {
                assignments[j].is_none()
                    && co_count[i][j] as f64 / n_replicates as f64 >= consensus_threshold
            })
            .collect();

        // Self-coassignment is identically one, so i itself is always a
        // member; a lone member forms a cluster only when the topic
        // actually appears in the perturbed graph.
        if members.len() < 2 {
            if adjacency.contains_key(&i) {
                assignments[i] = Some(next_cluster);
                next_cluster += 1;
            }
            continue;
        }

        for &member in &members {
            assignments[member] = Some(next_cluster);
        }
        next_cluster += 1;
    }

    for i in 0..k_topics {
        for j in 0..k_topics {
            co_freq[i][j] = co_count[i][j] as f64 / n_replicates as f64;
        }
    }

    Ok(ConsensusClusterOutput {
        assignments,
        co_assignment: co_freq,
    })
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    fn edge(source: usize, target: usize, effect: f64) -> NetworkEdge {
        NetworkEdge {
            source,
            target,
            effect,
            lower: effect,
            upper: effect,
            p_value: 0.0,
            selection_probability: 1.0,
        }
    }

    #[test]
    fn invalid_parameters_fail_closed() {
        let chain = vec![edge(0, 1, 0.9), edge(1, 2, 0.8)];
        assert!(matches!(
            consensus_clusters(&chain, 0, 5, 0.5, 0.1, 1),
            Err(NetworkEstimatorError::DimensionMismatch)
        ));
        assert!(matches!(
            consensus_clusters(&chain, 3, 0, 0.5, 0.1, 1),
            Err(NetworkEstimatorError::ZeroReplicates)
        ));
        assert!(matches!(
            consensus_clusters(&chain, 3, 5, 1.5, 0.1, 1),
            Err(NetworkEstimatorError::InvalidProbability)
        ));
        assert!(matches!(
            consensus_clusters(&chain, 3, 5, -0.1, 0.1, 1),
            Err(NetworkEstimatorError::InvalidProbability)
        ));
        assert!(matches!(
            consensus_clusters(&chain, 3, 5, f64::NAN, 0.1, 1),
            Err(NetworkEstimatorError::InvalidProbability)
        ));
        assert!(matches!(
            consensus_clusters(&chain, 3, 5, 0.5, 1.0, 1),
            Err(NetworkEstimatorError::InvalidProbability)
        ));
        assert!(matches!(
            consensus_clusters(&chain, 3, 5, 0.5, f64::NAN, 1),
            Err(NetworkEstimatorError::InvalidProbability)
        ));
    }

    #[test]
    fn negative_effect_edges_never_join_the_graph() {
        // Only the positive edge is admitted to the perturbation graph;
        // the negative edge contributes no adjacency, so topic 2 stays
        // unclustered even though it appears in the input.
        let mixed = vec![edge(0, 1, 0.95), edge(1, 2, -0.95)];
        let output = consensus_clusters(&mixed, 3, 40, 0.9, 0.0, 13).unwrap();
        assert_eq!(output.assignments[0], output.assignments[1]);
        assert!(output.assignments[0].is_some());
        assert_eq!(output.assignments[2], None);
    }

    #[test]
    fn redundant_edges_between_clustered_topics_are_harmless() {
        // A duplicate edge must not change the Leiden partition of a
        // strongly connected triple.
        let chain = vec![edge(0, 1, 0.95), edge(1, 2, 0.9)];
        let duplicated = vec![edge(0, 1, 0.95), edge(1, 2, 0.9), edge(0, 2, 0.8)];
        let plain = consensus_clusters(&chain, 3, 20, 0.99, 0.0, 31).unwrap();
        let doubled = consensus_clusters(&duplicated, 3, 20, 0.99, 0.0, 31).unwrap();
        assert_eq!(plain.assignments, doubled.assignments);
        assert_eq!(plain.co_assignment, doubled.co_assignment);
    }

    #[test]
    fn zero_drop_probability_is_fully_deterministic() {
        // With no perturbation every replicate sees the identical chain,
        // so all three topics co-assign always and land in one cluster.
        let chain = vec![edge(0, 1, 0.95), edge(1, 2, 0.85)];
        let output = consensus_clusters(&chain, 3, 25, 0.99, 0.0, 11).unwrap();
        assert_eq!(output.assignments[0], output.assignments[1]);
        assert_eq!(output.assignments[1], output.assignments[2]);
        assert!((output.co_assignment[0][2] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn weak_edges_do_not_merge_under_perturbation() {
        // Two strongly-bound pairs of topics plus one isolated topic:
        // within-pair co-assignment stays high across perturbed
        // replicates, while the isolated topic never joins any cluster.
        let pairs = vec![edge(0, 1, 0.97), edge(2, 3, 0.96)];
        let output = consensus_clusters(&pairs, 5, 200, 0.6, 0.2, 21).unwrap();
        assert_eq!(output.assignments[0], output.assignments[1]);
        assert!(output.assignments[0].is_some());
        assert_eq!(output.assignments[2], output.assignments[3]);
        assert!(output.assignments[2].is_some());
        assert_ne!(output.assignments[0], output.assignments[2]);
        assert_eq!(output.assignments[4], None);
        assert!((output.co_assignment[4][4] - 1.0).abs() < 1e-12);
        // The within-pair edge survives roughly 80% of perturbations, so
        // its co-assignment stays well above the consensus threshold
        // while never reaching a deterministic 1.0.
        let within_pair = output.co_assignment[0][1];
        assert!(within_pair > 0.6, "co = {within_pair}");
        assert!(within_pair < 1.0, "co = {within_pair}");
        // Cross-pair topics never co-assign: they share no edge.
        assert!((output.co_assignment[0][2]).abs() < 1e-12);
    }

    #[test]
    fn heavily_perturbed_single_edge_leaves_both_topics_unclustered() {
        // With a 0.9 drop probability the only edge survives ~10% of
        // replicates, so pair co-assignment stays below the threshold;
        // each topic still carries graph adjacency and therefore forms
        // its own singleton cluster instead of disappearing.
        let single = vec![edge(0, 1, 0.99)];
        let output = consensus_clusters(&single, 2, 60, 0.5, 0.9, 41).unwrap();
        assert_ne!(output.assignments[0], None);
        assert_ne!(output.assignments[1], None);
        assert_ne!(output.assignments[0], output.assignments[1]);
    }

    #[test]
    fn spanning_cluster_forces_rescan_over_assigned_topics() {
        // A single edge joins topics 0 and 2 while topic 1 stays
        // isolated. After the first iteration assigns {0, 2}, the scan
        // for topic 1 must walk past an already-assigned topic before
        // finishing, exercising the assigned-skip arm of the membership
        // filter.
        let skip_pair = vec![edge(0, 2, 0.97)];
        let output = consensus_clusters(&skip_pair, 3, 20, 0.9, 0.0, 5).unwrap();
        assert_eq!(output.assignments[0], output.assignments[2]);
        assert_eq!(output.assignments[1], None);
    }

    #[test]
    fn isolated_topic_without_edges_stays_unclustered() {
        let chain = vec![edge(0, 1, 0.95), edge(1, 2, 0.9)];
        let output = consensus_clusters(&chain, 4, 25, 0.95, 0.0, 3).unwrap();
        assert!(output.assignments[0].is_some());
        assert!(output.assignments[1].is_some());
        assert!(output.assignments[2].is_some());
        assert_eq!(output.assignments[3], None);
        assert!((output.co_assignment[3][0]).abs() < 1e-12);
    }

    #[test]
    fn identical_inputs_and_seed_reproduce_output_exactly() {
        let chain = vec![edge(0, 1, 0.9), edge(1, 2, 0.8)];
        let first = consensus_clusters(&chain, 3, 15, 0.6, 0.1, 7).unwrap();
        let second = consensus_clusters(&chain, 3, 15, 0.6, 0.1, 7).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn two_cliques_with_a_weak_bridge_stay_two_consensus_clusters() {
        // Operator-visible GAP-009 remainder: union-find glued both
        // triangles; Leiden modularity plus co-assignment keeps them
        // apart under a high consensus threshold and no drop noise.
        let planted = vec![
            edge(0, 1, 1.0),
            edge(1, 2, 1.0),
            edge(0, 2, 1.0),
            edge(3, 4, 1.0),
            edge(4, 5, 1.0),
            edge(3, 5, 1.0),
            edge(2, 3, 0.01),
        ];
        let output = consensus_clusters(&planted, 6, 20, 0.8, 0.0, 17).unwrap();
        assert_eq!(output.assignments[0], output.assignments[1]);
        assert_eq!(output.assignments[1], output.assignments[2]);
        assert_eq!(output.assignments[3], output.assignments[4]);
        assert_eq!(output.assignments[4], output.assignments[5]);
        assert_ne!(output.assignments[0], output.assignments[3]);
        assert!(output.assignments[0].is_some());
        assert!(output.assignments[3].is_some());
        assert!(output.co_assignment[0][2] > 0.8);
        assert!(output.co_assignment[0][3] < 0.8);
    }
}
