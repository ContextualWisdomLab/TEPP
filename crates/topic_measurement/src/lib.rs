#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Logistic-normal and log-ratio coordinates for compositional topic proportions.
//!
//! Raw topic proportions are compositional rather than unconstrained Euclidean
//! indicators. ALR supplies a reference-dependent full-rank logistic-normal map
//! for regression and psychometric interfaces; it is not an orthonormal
//! Aitchison-distance isometry. Distance-based Aitchison geometry uses the
//! sequential Egozcue ILR basis, whose pairwise Euclidean distance recovers
//! CLR Aitchison distance. TF-IDF, BM25, and keyword scores remain
//! forbidden inferential coordinates.

mod coordinates;
mod error;
mod lexical;
mod posterior_draw;
mod reference;
mod sparse;

/// Additive log-ratio map from a simplex vector.
pub use coordinates::additive_log_ratio;
/// Aitchison distance between two simplex vectors.
pub use coordinates::aitchison_distance;
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
/// Stable counter-based draw algorithm identity.
pub use posterior_draw::JOINT_POSTERIOR_DRAW_ALGORITHM_VERSION;
/// Versioned deterministic joint Gaussian plausible-value draw set.
pub use posterior_draw::JointPosteriorDrawSet;
/// One exact fit-bound plausible value before artifact provenance binding.
pub use posterior_draw::JointPosteriorPlausibleValue;
/// Identified joint precision in document-major ALR coordinate order.
pub use reference::JointCoordinatePrecision;
/// Posterior uncertainty representation retained by a fitted reference model.
pub use reference::PosteriorApproximation;
/// One admitted structural prevalence feature.
pub use reference::PrevalenceFeature;
/// Validated input for the CPU `f64` reference estimator.
pub use reference::ReferenceTopicInput;
/// A converged topic-model result with uncertainty and lineage counts.
pub use reference::ReferenceTopicModel;
/// Bounded deterministic reference-estimator configuration.
pub use reference::ReferenceTopicModelConfig;
/// One inferred predecessor/successor association within a fitted topic.
pub use reference::TopicSequenceEdge;
/// Fit the bounded deterministic CPU `f64` TRSL-TM reference estimator.
pub use reference::fit_reference_topic_model;
/// Validated compressed sparse numeric matrix.
pub use sparse::SparseMatrix;
/// Whether compressed values are grouped by row or by column.
pub use sparse::SparseOrientation;
