#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Logistic-normal and log-ratio coordinates for compositional topic proportions.
//!
//! Raw topic proportions are not Euclidean indicators. Downstream network and
//! psychometric analysis must use additive log-ratio (logistic-normal) maps
//! rather than TF-IDF, BM25, or keyword scores as inferential coordinates.

mod coordinates;
mod error;
mod lexical;

/// Additive log-ratio map from a simplex vector.
pub use coordinates::additive_log_ratio;
/// Inverse additive log-ratio map back to the simplex.
pub use coordinates::from_additive_log_ratio;
/// Fail-closed topic-coordinate errors.
pub use error::TopicMeasurementError;
/// Refuse lexical retrieval weights as inferential coordinates.
pub use lexical::refuse_lexical_inferential_weight;
