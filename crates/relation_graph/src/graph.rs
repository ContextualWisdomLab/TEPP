//! In-memory typed relation graph with forward-transition invariants.

use crate::{RelationEdge, RelationEdgeId, RelationEndpointId, RelationError};
use std::collections::{HashMap, HashSet};

/// In-memory graph of typed document/event/entity relations.
///
/// The transition subgraph admits only forward-order edges and rejects cycles
/// so state-transition paths remain a DAG in event identity space.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RelationGraph {
    edges: HashMap<RelationEdgeId, RelationEdge>,
    transition_adjacency: HashMap<RelationEndpointId, Vec<RelationEndpointId>>,
}

impl RelationGraph {
    /// Create an empty relation graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a validated edge, enforcing transition acyclicity.
    ///
    /// # Errors
    ///
    /// Returns duplicate-identity or transition-cycle errors.
    pub fn insert(&mut self, edge: RelationEdge) -> Result<(), RelationError> {
        if self.edges.contains_key(&edge.edge_id()) {
            return Err(RelationError::DuplicateRelationEdge);
        }
        if edge.is_transition_edge() && self.transition_path_exists(edge.target(), edge.source()) {
            return Err(RelationError::TransitionCycle);
        }
        if edge.is_transition_edge() {
            self.transition_adjacency
                .entry(edge.source())
                .or_default()
                .push(edge.target());
        }
        self.edges.insert(edge.edge_id(), edge);
        Ok(())
    }

    /// Return the number of stored edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Borrow one edge by identity.
    #[must_use]
    pub fn edge(&self, edge_id: RelationEdgeId) -> Option<&RelationEdge> {
        self.edges.get(&edge_id)
    }

    /// Iterate all edges in arbitrary hash order.
    pub fn edges(&self) -> impl Iterator<Item = &RelationEdge> {
        self.edges.values()
    }

    /// Count forward-transition edges only.
    #[must_use]
    pub fn transition_edge_count(&self) -> usize {
        self.edges
            .values()
            .filter(|edge| edge.is_transition_edge())
            .count()
    }

    fn transition_path_exists(&self, from: RelationEndpointId, to: RelationEndpointId) -> bool {
        let mut stack = vec![from];
        let mut seen = HashSet::new();
        while let Some(node) = stack.pop() {
            if !seen.insert(node) {
                continue;
            }
            if node == to {
                return true;
            }
            if let Some(neighbors) = self.transition_adjacency.get(&node) {
                stack.extend(neighbors.iter().copied());
            }
        }
        false
    }
}
