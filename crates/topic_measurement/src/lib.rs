#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Logistic-normal and log-ratio coordinates for compositional topic proportions.
//!
//! Raw topic proportions are compositional rather than unconstrained Euclidean
//! indicators. ALR supplies a reference-dependent full-rank logistic-normal map
//! for regression and psychometric interfaces; it is not an orthonormal
//! Aitchison-distance isometry. Distance-based Aitchison geometry uses the
//! sequential Egozcue ILR basis. TF-IDF, BM25, and keyword scores remain
//! forbidden inferential coordinates.

mod coordinates;
mod error;
mod lexical;

/// Additive log-ratio map from a simplex vector.
pub use coordinates::additive_log_ratio;
/// Inverse additive log-ratio map back to the simplex.
pub use coordinates::from_additive_log_ratio;
/// Inverse isometric log-ratio map back to the simplex.
pub use coordinates::from_isometric_log_ratio;
/// Isometric log-ratio map from a simplex vector.
pub use coordinates::isometric_log_ratio;
/// Fail-closed topic-coordinate errors.
pub use error::TopicMeasurementError;
/// Refuse lexical retrieval weights as inferential coordinates.
pub use lexical::refuse_lexical_inferential_weight;
