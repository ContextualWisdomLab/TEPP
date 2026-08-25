//! Co-assignment consensus clustering from repeated label-invariant partitions.
//!
//! Runs multiple rounds of greedy modularity partitioning over admitted
//! positive edges (a lightweight stand-in for Leiden until a vetted crate
//! is adopted), builds a co-assignment matrix across replicates, and
//! derives consensus clusters by thresholding that matrix. Topics whose
//! co-assignment frequency falls below `consensus_threshold` remain
//! unclustered.

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

/// Derive consensus clusters from repeated partitions of the admitted edge set.
///
/// # Arguments
///
/// * `edges` – admitted positive edges (source, target, effect).
/// * `k_topics` – total number of topics.
/// * `n_replicates` – number of perturbed partitions to generate.
/// * `consensus_threshold` – minimum co-assignment fraction for same-cluster.
/// * `seed` – deterministic seed.
pub fn consensus_clusters(
    edges: &[NetworkEdge],
    k_topics: usize,
    n_replicates: usize,
    consensus_threshold: f64,
    seed: u64,
) -> Result<ConsensusClusterOutput, NetworkEstimatorError> {
    if k_topics == 0 {
        return Err(NetworkEstimatorError::DimensionMismatch);
    }
    if n_replicates == 0 {
        return Err(NetworkEstimatorError::ZeroReplicates);
    }

    // Build adjacency from positive edges only.
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for e in edges {
        if e.effect > 0.0 {
            adj.entry(e.source).or_default().push(e.target);
            adj.entry(e.target).or_default().push(e.source);
        }
    }

    // Deterministic LCG.
    let mut state = seed;

    // Accumulate co-assignment counts.
    let mut co_count = vec![vec![0_u64; k_topics]; k_topics];

    for _ in 0..n_replicates {
        // Perturb: randomly drop each edge with p = 0.1.
        let perturbed: Vec<&NetworkEdge> = edges
            .iter()
            .filter(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                ((state >> 33) as f64 / (u64::MAX >> 33) as f64) > 0.1
            })
            .collect();

        let partition = greedy_modularity_partition(&perturbed, k_topics);

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
    let mut assignments = vec![None; k_topics];
    let mut next_cluster = 0_usize;

    for i in 0..k_topics {
        if assignments[i].is_some() {
            continue;
        }
        // Check if topic i has sufficient co-assignment with itself (always true).
        // Find all j ≥ i with co-assignment ≥ threshold.
        let members: Vec<usize> = (i..k_topics)
            .filter(|&j| {
                assignments[j].is_none()
                    && co_count[i][j] as f64 / n_replicates as f64 >= consensus_threshold
            })
            .collect();

        if members.len() < 2 {
            // Singletons stay unclustered unless they pair strongly.
            if !members.is_empty()
                && co_count[i][i] as f64 / n_replicates as f64 >= consensus_threshold
                && adj.contains_key(&i)
            {
                assignments[i] = Some(next_cluster);
                next_cluster += 1;
            }
            continue;
        }

        for &m in &members {
            assignments[m] = Some(next_cluster);
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

/// Greedy modularity partition (label-invariant, single pass).
fn greedy_modularity_partition(edges: &[&NetworkEdge], k: usize) -> Vec<usize> {
    let mut parent: Vec<usize> = (0..k).collect();

    fn find(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }

    fn union(p: &mut [usize], a: usize, b: usize) {
        let ra = find(p, a);
        let rb = find(p, b);
        if ra != rb {
            p[rb] = ra;
        }
    }

    // Sort edges by descending effect.
    let mut sorted: Vec<&NetworkEdge> = edges.to_vec();
    sorted.sort_by(|a, b| b.effect.total_cmp(&a.effect));

    for e in &sorted {
        union(&mut parent, e.source, e.target);
    }

    // Normalise labels to 0..n_clusters.
    let mut label_map: HashMap<usize, usize> = HashMap::new();
    parent
        .iter()
        .map(|&root| {
            let len = label_map.len();
            *label_map.entry(root).or_insert(len)
        })
        .collect()
}
