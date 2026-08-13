#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Explicit method sources for template, section, copy, style, and modality.
//!
//! These sources are modeled as method/background structure. They cannot be
//! used as inferential topic weights (ADR 0004/0012).

mod error;
mod source;

/// Fail-closed method-effect errors.
pub use error::MethodEffectsError;
/// Closed method-source vocabulary.
pub use source::MethodSourceKind;
/// Refuse to treat a method source as an inferential weight.
pub use source::refuse_method_source_as_inferential_weight;
/// Fraction of recovered method sources that match known truth.
pub use source::source_recovery_rate;
