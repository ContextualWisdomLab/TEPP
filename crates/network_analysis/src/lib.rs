#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Compositional network and clustering gates for TRSL-TM outputs.
//!
//! Raw topic proportions are not ordinary Euclidean coordinates. Cluster
//! recovery is scored with label-invariant pair precision and recall against
//! known truth (ADR 0005/0012).

mod cluster;
mod error;
mod simplex;

/// Opaque cluster identity.
pub use cluster::ClusterLabel;
/// Pair precision of recovered clusters.
pub use cluster::cluster_pair_precision;
/// Pair recall of recovered clusters.
pub use cluster::cluster_pair_recall;
/// Fail-closed network-analysis errors.
pub use error::NetworkError;
/// Refuse raw simplex proportions as Euclidean coordinates.
pub use simplex::refuse_raw_simplex_as_euclidean;
