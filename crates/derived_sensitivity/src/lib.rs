#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Derived outputs inherit source sensitivity.
//!
//! Topic, factor, and relation artifacts are not public merely because they
//! are derived. They inherit the source sensitivity class, and blanket PII
//! masking is not a declassification grant (ADR 0009).

mod classification;
mod error;

/// One derived topic, factor, or relation artifact with inherited sensitivity.
pub use classification::DerivedArtifact;
/// Closed sensitivity vocabulary for source and derived artifacts.
pub use classification::SensitivityClass;
/// Inherit the source sensitivity class onto a derived artifact.
pub use classification::inherit_sensitivity;
/// Refuse to treat a blanket PII mask as declassification authorization.
pub use classification::refuse_blanket_mask_as_declassification;
/// Refuse to treat derivation as declassification to public.
pub use classification::refuse_derivation_as_public;
/// Fraction of inherited classes that match known truth.
pub use classification::sensitivity_recovery_rate;
/// Fail-closed derived-sensitivity errors.
pub use error::DerivedSensitivityError;
