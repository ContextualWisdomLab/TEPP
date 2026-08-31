#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Compositional network, clustering, and posterior-association analysis
//! for TRSL-TM outputs.
//!
//! Raw topic proportions are not ordinary Euclidean coordinates. Cluster
//! recovery is scored with label-invariant pair precision and recall against
//! known truth (ADR 0005/0012). The posterior estimator transforms draws into
//! isometric log-ratio coordinates, estimates topic–topic associations with
//! uncertainty and bootstrap stability, applies multiplicity-corrected edge
//! admission, and derives co-assignment consensus clusters.

mod cluster;
mod consensus;
mod edges;
mod error;
mod leiden;
mod simplex;
mod stability;

/// Pair precision of recovered clusters.
pub use cluster::cluster_pair_precision;
/// Pair recall of recovered clusters.
pub use cluster::cluster_pair_recall;
/// Opaque cluster identity.
pub use cluster::ClusterLabel;
/// Derive consensus clusters from repeated partitions.
pub use consensus::consensus_clusters;
/// Consensus clustering output.
pub use consensus::ConsensusClusterOutput;
/// Apply multiplicity-corrected edge-admission policy.
pub use edges::admit_edges;
/// Apply the per-replicate admission rule used inside stability scoring.
pub use edges::admit_edges_within_replicate;
/// Compute posterior correlation matrix from ILR draws.
pub use edges::posterior_correlation_matrix;
/// One estimated topic–topic association edge.
pub use edges::NetworkEdge;
/// Fail-closed network-analysis errors.
pub use error::NetworkError;
/// Fail-closed posterior-network-estimator errors.
pub use error::NetworkEstimatorError;
/// Refuse raw simplex proportions as Euclidean coordinates.
pub use simplex::refuse_raw_simplex_as_euclidean;
/// Run bootstrap replicates and return stability scores.
pub use stability::bootstrap_edge_stability;
/// Per-edge bootstrap stability score.
pub use stability::BootstrapEdgeStability;
