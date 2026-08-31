//! Leiden community detection for admitted positive topic–topic edges.
//!
//! Replaces the greedy union-find stand-in. Each replicate partition is
//! Traag, Waltman, and van Eck (2019): fast local moving of nodes under
//! Newman–Girvan modularity (γ = 1), refinement that keeps communities
//! internally connected, and aggregation. Louvain is rejected because it
//! can emit internally disconnected communities. Union-find is rejected
//! because it merges every surviving edge into one component and makes
//! no modularity claim.
//!
//! Isolated topics remain singleton labels. The consensus layer still
//! maps topics that never appear in the perturbation graph to
//! unclustered. This module does not claim a causal cluster, a graphical
//! lasso, or an end-to-end export workflow.

#![allow(
    clippy::doc_markdown,
    clippy::items_after_statements,
    clippy::must_use_candidate,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc,
    clippy::too_many_lines
)]

use crate::edges::NetworkEdge;

/// Deterministic LCG shared with the consensus perturbation stream.
pub(crate) fn step_rng(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn shuffle<T>(state: &mut u64, items: &mut [T]) {
    for index in (1..items.len()).rev() {
        let bound = (index + 1) as u64;
        let other = (step_rng(state) % bound) as usize;
        items.swap(index, other);
    }
}

struct Graph {
    node_count: usize,
    adjacency: Vec<Vec<(usize, f64)>>,
    strength: Vec<f64>,
    total_weight: f64,
}

impl Graph {
    fn from_edges(edges: &[&NetworkEdge], node_count: usize) -> Self {
        let mut adjacency = vec![Vec::new(); node_count];
        let mut strength = vec![0.0_f64; node_count];
        for edge in edges {
            if edge.source >= node_count || edge.target >= node_count {
                continue;
            }
            if edge.source == edge.target || edge.effect <= 0.0 {
                continue;
            }
            adjacency[edge.source].push((edge.target, edge.effect));
            adjacency[edge.target].push((edge.source, edge.effect));
            strength[edge.source] += edge.effect;
            strength[edge.target] += edge.effect;
        }
        for neighbors in &mut adjacency {
            neighbors.sort_by(|left, right| left.0.cmp(&right.0).then(right.1.total_cmp(&left.1)));
            neighbors.dedup_by(|later, kept| {
                if later.0 == kept.0 {
                    kept.1 += later.1;
                    true
                } else {
                    false
                }
            });
        }
        let total_weight = strength.iter().sum::<f64>() / 2.0;
        Self {
            node_count,
            adjacency,
            strength,
            total_weight,
        }
    }

    fn weight_to_community(&self, node: usize, membership: &[usize], community: usize) -> f64 {
        self.adjacency[node]
            .iter()
            .filter(|(neighbor, _)| membership[*neighbor] == community)
            .map(|(_, weight)| *weight)
            .sum()
    }
}

/// Partition `k` topics with Leiden modularity on the surviving edges.
pub(crate) fn leiden_partition(
    edges: &[&NetworkEdge],
    topic_count: usize,
    rng: &mut u64,
) -> Vec<usize> {
    if topic_count == 0 {
        return Vec::new();
    }
    let mut graph = Graph::from_edges(edges, topic_count);
    let mut membership: Vec<usize> = (0..topic_count).collect();
    if graph.total_weight <= 0.0 {
        return membership;
    }

    let mut leaf_members: Vec<Vec<usize>> = (0..topic_count).map(|node| vec![node]).collect();
    for _ in 0..topic_count {
        let moved = move_nodes_fast(&graph, &mut membership, rng);
        split_disconnected(&graph, &mut membership);
        let refined = refine_partition(&graph, &membership, rng);
        let (aggregate, next_membership, next_leaves) =
            aggregate_graph(&graph, &membership, &refined, &leaf_members);
        if aggregate.node_count == graph.node_count && !moved {
            break;
        }
        if aggregate.node_count <= 1 {
            membership = next_membership;
            leaf_members = next_leaves;
            break;
        }
        graph = aggregate;
        membership = next_membership;
        leaf_members = next_leaves;
    }

    project_original_labels(&leaf_members, &membership, topic_count)
}

fn project_original_labels(
    leaf_members: &[Vec<usize>],
    membership: &[usize],
    topic_count: usize,
) -> Vec<usize> {
    let mut labels = vec![0_usize; topic_count];
    for (supernode, members) in leaf_members.iter().enumerate() {
        let community = membership[supernode];
        for &node in members {
            labels[node] = community;
        }
    }
    dense_labels(&labels)
}

fn dense_labels(membership: &[usize]) -> Vec<usize> {
    let mut remap = vec![
        None;
        membership
            .iter()
            .copied()
            .max()
            .map_or(0, |value| value + 1)
    ];
    let mut next = 0_usize;
    membership
        .iter()
        .map(|&community| {
            *remap[community].get_or_insert_with(|| {
                let label = next;
                next += 1;
                label
            })
        })
        .collect()
}

fn community_totals(graph: &Graph, membership: &[usize]) -> Vec<f64> {
    let community_count = membership
        .iter()
        .copied()
        .max()
        .map_or(0, |value| value + 1);
    let mut totals = vec![0.0_f64; community_count];
    for (node, &community) in membership.iter().enumerate() {
        totals[community] += graph.strength[node];
    }
    totals
}

fn modularity_delta(
    strength: f64,
    weight_from: f64,
    weight_to: f64,
    total_from: f64,
    total_to: f64,
    total_weight: f64,
) -> f64 {
    // Moving node i from community c to d, γ = 1:
    // ΔQ = (k_{i→d} − k_{i→c}) / m − k_i (Σ_d − Σ_c + k_i) / (2 m²)
    // `total_from` still includes `strength`.
    let two_m = 2.0 * total_weight;
    (weight_to - weight_from) / total_weight
        - strength * (total_to - total_from + strength) / (two_m * total_weight)
}

fn move_nodes_fast(graph: &Graph, membership: &mut [usize], rng: &mut u64) -> bool {
    if graph.node_count <= 1 {
        return false;
    }
    if graph.total_weight <= 0.0 {
        return false;
    }
    let mut totals = community_totals(graph, membership);
    let mut queue: Vec<usize> = (0..graph.node_count).collect();
    shuffle(rng, &mut queue);
    let mut queued = vec![true; graph.node_count];
    let mut any_move = false;
    let mut cursor = 0_usize;
    while cursor < queue.len() {
        let node = queue[cursor];
        cursor += 1;
        queued[node] = false;
        let from = membership[node];
        let weight_from = graph.weight_to_community(node, membership, from);
        let mut best_community = from;
        let mut best_delta = 0.0_f64;
        let mut seen = Vec::new();
        for &(neighbor, _) in &graph.adjacency[node] {
            let candidate = membership[neighbor];
            if candidate == from || seen.contains(&candidate) {
                continue;
            }
            seen.push(candidate);
            let weight_to = graph.weight_to_community(node, membership, candidate);
            let delta = modularity_delta(
                graph.strength[node],
                weight_from,
                weight_to,
                totals[from],
                totals[candidate],
                graph.total_weight,
            );
            if delta > best_delta {
                best_delta = delta;
                best_community = candidate;
            }
        }
        if best_community == from || best_delta <= 0.0 {
            continue;
        }
        membership[node] = best_community;
        totals[from] -= graph.strength[node];
        totals[best_community] += graph.strength[node];
        any_move = true;
        for &(neighbor, _) in &graph.adjacency[node] {
            if membership[neighbor] != best_community && !queued[neighbor] {
                queued[neighbor] = true;
                queue.push(neighbor);
            }
        }
    }
    any_move
}

fn split_disconnected(graph: &Graph, membership: &mut [usize]) {
    let community_count = membership
        .iter()
        .copied()
        .max()
        .map_or(0, |value| value + 1);
    let mut next_label = community_count;
    for community in 0..community_count {
        let members: Vec<usize> = membership
            .iter()
            .enumerate()
            .filter_map(|(node, &label)| (label == community).then_some(node))
            .collect();
        if members.len() <= 1 {
            continue;
        }
        let mut index_of = vec![0_usize; graph.node_count];
        for (index, &node) in members.iter().enumerate() {
            index_of[node] = index;
        }
        let mut visited = vec![false; members.len()];
        let mut first_component = true;
        for start_index in 0..members.len() {
            if visited[start_index] {
                continue;
            }
            let mut stack = vec![start_index];
            visited[start_index] = true;
            let mut component = Vec::new();
            while let Some(index) = stack.pop() {
                component.push(members[index]);
                for &(neighbor, _) in &graph.adjacency[members[index]] {
                    if membership[neighbor] != community {
                        continue;
                    }
                    let neighbor_index = index_of[neighbor];
                    if !visited[neighbor_index] {
                        visited[neighbor_index] = true;
                        stack.push(neighbor_index);
                    }
                }
            }
            if first_component {
                first_component = false;
                continue;
            }
            for node in component {
                membership[node] = next_label;
            }
            next_label += 1;
        }
    }
}

fn refine_partition(graph: &Graph, membership: &[usize], rng: &mut u64) -> Vec<usize> {
    let mut refined: Vec<usize> = (0..graph.node_count).collect();
    let community_count = membership
        .iter()
        .copied()
        .max()
        .map_or(0, |value| value + 1);
    for community in 0..community_count {
        let members: Vec<usize> = membership
            .iter()
            .enumerate()
            .filter_map(|(node, &label)| (label == community).then_some(node))
            .collect();
        if members.len() <= 1 {
            continue;
        }
        let mut sub_membership: Vec<usize> = (0..members.len()).collect();
        let mut sub_adj = vec![Vec::new(); members.len()];
        let mut sub_strength = vec![0.0_f64; members.len()];
        let mut index_of = vec![usize::MAX; graph.node_count];
        for (index, &node) in members.iter().enumerate() {
            index_of[node] = index;
        }
        for (index, &node) in members.iter().enumerate() {
            for &(neighbor, weight) in &graph.adjacency[node] {
                if membership[neighbor] != community {
                    continue;
                }
                let neighbor_index = index_of[neighbor];
                sub_adj[index].push((neighbor_index, weight));
                sub_strength[index] += weight;
            }
        }
        let sub_total = sub_strength.iter().sum::<f64>() / 2.0;
        if sub_total <= 0.0 {
            continue;
        }
        let subgraph = Graph {
            node_count: members.len(),
            adjacency: sub_adj,
            strength: sub_strength,
            total_weight: sub_total,
        };
        move_nodes_fast(&subgraph, &mut sub_membership, rng);
        split_disconnected(&subgraph, &mut sub_membership);
        let offset = refined.iter().copied().max().map_or(0, |value| value + 1);
        for (index, &node) in members.iter().enumerate() {
            refined[node] = offset + sub_membership[index];
        }
    }
    dense_labels(&refined)
}

fn aggregate_graph(
    graph: &Graph,
    parent: &[usize],
    refined: &[usize],
    leaf_members: &[Vec<usize>],
) -> (Graph, Vec<usize>, Vec<Vec<usize>>) {
    let refined_count = refined.iter().copied().max().map_or(0, |value| value + 1);
    let mut next_leaves = vec![Vec::new(); refined_count];
    for node in 0..graph.node_count {
        next_leaves[refined[node]].extend_from_slice(&leaf_members[node]);
    }
    let mut adjacency = vec![Vec::new(); refined_count];
    let mut strength = vec![0.0_f64; refined_count];
    for node in 0..graph.node_count {
        strength[refined[node]] += graph.strength[node];
        let source = refined[node];
        for &(neighbor, weight) in &graph.adjacency[node] {
            let target = refined[neighbor];
            if source < target {
                adjacency[source].push((target, weight));
                adjacency[target].push((source, weight));
            }
        }
    }
    for neighbors in &mut adjacency {
        neighbors.sort_by_key(|left| left.0);
        neighbors.dedup_by(|later, kept| {
            if later.0 == kept.0 {
                kept.1 += later.1;
                true
            } else {
                false
            }
        });
    }
    let aggregate = Graph {
        node_count: refined_count,
        adjacency,
        strength,
        total_weight: graph.total_weight,
    };
    let mut parent_of_refined = vec![0_usize; refined_count];
    for node in 0..graph.node_count {
        parent_of_refined[refined[node]] = parent[node];
    }
    let next_membership = dense_labels(&parent_of_refined);
    (aggregate, next_membership, next_leaves)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edges::NetworkEdge;

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

    fn refs(owned: &[NetworkEdge]) -> Vec<&NetworkEdge> {
        owned.iter().collect()
    }

    fn communities_connected(edges: &[&NetworkEdge], labels: &[usize]) -> bool {
        let graph = Graph::from_edges(edges, labels.len());
        let community_count = labels.iter().copied().max().map_or(0, |value| value + 1);
        for community in 0..community_count {
            let members: Vec<usize> = labels
                .iter()
                .enumerate()
                .filter_map(|(node, &label)| (label == community).then_some(node))
                .collect();
            if members.len() <= 1 {
                continue;
            }
            let mut index_of = vec![0_usize; labels.len()];
            for (index, &node) in members.iter().enumerate() {
                index_of[node] = index;
            }
            let mut seen = vec![false; members.len()];
            let mut stack = vec![0_usize];
            seen[0] = true;
            let mut visited = 1_usize;
            while let Some(index) = stack.pop() {
                for &(neighbor, _) in &graph.adjacency[members[index]] {
                    if labels[neighbor] != community {
                        continue;
                    }
                    let neighbor_index = index_of[neighbor];
                    if !seen[neighbor_index] {
                        seen[neighbor_index] = true;
                        visited += 1;
                        stack.push(neighbor_index);
                    }
                }
            }
            if visited != members.len() {
                return false;
            }
        }
        true
    }

    #[test]
    fn empty_and_edgeless_graphs_stay_singletons() {
        let mut rng = 1_u64;
        assert!(leiden_partition(&[], 0, &mut rng).is_empty());
        let labels = leiden_partition(&[], 4, &mut rng);
        assert_eq!(labels, vec![0, 1, 2, 3]);
        assert!(dense_labels(&[]).is_empty());
        let totals = community_totals(&Graph::from_edges(&[], 0), &[]);
        assert!(totals.is_empty());
    }

    #[test]
    fn out_of_range_self_and_nonpositive_edges_are_ignored() {
        let owned = vec![
            edge(0, 9, 1.0),
            edge(1, 1, 1.0),
            edge(0, 1, 0.0),
            edge(0, 1, -0.4),
        ];
        let edges = refs(&owned);
        let mut rng = 2_u64;
        let labels = leiden_partition(&edges, 2, &mut rng);
        assert_eq!(labels, vec![0, 1]);
    }

    #[test]
    fn two_nodes_with_a_positive_edge_share_a_community() {
        let owned = vec![edge(0, 1, 0.9)];
        let edges = refs(&owned);
        let mut rng = 3_u64;
        let labels = leiden_partition(&edges, 2, &mut rng);
        assert_eq!(labels[0], labels[1]);
        assert!(communities_connected(&edges, &labels));
    }

    #[test]
    fn two_cliques_joined_by_a_weak_bridge_stay_apart() {
        // Union-find would glue both triangles into one component. Leiden
        // modularity recovers the planted pair of communities.
        let owned = vec![
            edge(0, 1, 1.0),
            edge(1, 2, 1.0),
            edge(0, 2, 1.0),
            edge(3, 4, 1.0),
            edge(4, 5, 1.0),
            edge(3, 5, 1.0),
            edge(2, 3, 0.01),
        ];
        let edges = refs(&owned);
        let mut rng = 11_u64;
        let labels = leiden_partition(&edges, 6, &mut rng);
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[1], labels[2]);
        assert_eq!(labels[3], labels[4]);
        assert_eq!(labels[4], labels[5]);
        assert_ne!(labels[0], labels[3]);
        assert!(communities_connected(&edges, &labels));
    }

    #[test]
    fn duplicate_edges_are_summed_and_remain_connected() {
        let owned = vec![edge(0, 1, 0.4), edge(0, 1, 0.4), edge(1, 2, 0.8)];
        let edges = refs(&owned);
        let mut rng = 7_u64;
        let labels = leiden_partition(&edges, 3, &mut rng);
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[1], labels[2]);
        assert!(communities_connected(&edges, &labels));
    }

    #[test]
    fn identical_seed_reproduces_the_partition() {
        let owned = vec![
            edge(0, 1, 0.9),
            edge(1, 2, 0.85),
            edge(3, 4, 0.88),
            edge(2, 3, 0.2),
        ];
        let edges = refs(&owned);
        let mut first_rng = 21_u64;
        let mut second_rng = 21_u64;
        let first = leiden_partition(&edges, 5, &mut first_rng);
        let second = leiden_partition(&edges, 5, &mut second_rng);
        assert_eq!(first, second);
        assert_eq!(first.len(), 5);
        assert_ne!(first[4], first[0]);
        assert!(communities_connected(&edges, &first));
    }

    #[test]
    fn isolated_topic_keeps_a_distinct_label() {
        let owned = vec![edge(0, 1, 0.95)];
        let edges = refs(&owned);
        let mut rng = 5_u64;
        let labels = leiden_partition(&edges, 3, &mut rng);
        assert_eq!(labels[0], labels[1]);
        assert_ne!(labels[2], labels[0]);
    }

    #[test]
    fn disconnected_membership_is_split_into_connected_components() {
        let owned = vec![edge(0, 1, 1.0), edge(2, 3, 1.0)];
        let edges = refs(&owned);
        let graph = Graph::from_edges(&edges, 4);
        let mut membership = vec![0, 0, 0, 0];
        split_disconnected(&graph, &mut membership);
        assert_eq!(membership[0], membership[1]);
        assert_eq!(membership[2], membership[3]);
        assert_ne!(membership[0], membership[2]);
    }

    #[test]
    fn single_node_graph_does_not_move() {
        let graph = Graph {
            node_count: 1,
            adjacency: vec![Vec::new()],
            strength: vec![1.0],
            total_weight: 0.5,
        };
        let mut membership = vec![0];
        let mut rng = 9_u64;
        assert!(!move_nodes_fast(&graph, &mut membership, &mut rng));
        assert_eq!(membership, vec![0]);
    }

    #[test]
    fn edgeless_multi_node_graph_does_not_move() {
        let graph = Graph::from_edges(&[], 3);
        let mut membership = vec![0, 1, 2];
        let mut rng = 4_u64;
        assert!(!move_nodes_fast(&graph, &mut membership, &mut rng));
        assert_eq!(membership, vec![0, 1, 2]);
    }

    #[test]
    fn refine_skips_edgeless_communities_and_merges_a_triangle() {
        let owned = vec![edge(0, 1, 1.0), edge(1, 2, 1.0), edge(0, 2, 1.0)];
        let edges = refs(&owned);
        let graph = Graph::from_edges(&edges, 4);
        let membership = vec![0, 0, 0, 1];
        let mut rng = 8_u64;
        let refined = refine_partition(&graph, &membership, &mut rng);
        assert_eq!(refined[0], refined[1]);
        assert_eq!(refined[1], refined[2]);
        assert_ne!(refined[3], refined[0]);

        let empty_pair = Graph::from_edges(&[], 2);
        let pair_membership = vec![0, 0];
        let skipped = refine_partition(&empty_pair, &pair_membership, &mut rng);
        assert_eq!(skipped.len(), 2);
    }

    #[test]
    fn modularity_delta_is_negative_when_leaving_a_stronger_community() {
        let delta = modularity_delta(1.0, 2.0, 0.01, 3.0, 2.0, 4.0);
        assert!(delta < 0.0);
    }

    #[test]
    fn shuffle_of_a_single_item_is_a_no_op() {
        let mut rng = 13_u64;
        let mut items = vec![7_usize];
        shuffle(&mut rng, &mut items);
        assert_eq!(items, vec![7]);
        let mut empty: Vec<usize> = Vec::new();
        shuffle(&mut rng, &mut empty);
        assert!(empty.is_empty());
    }
}
