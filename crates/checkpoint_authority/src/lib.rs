#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! A model checkpoint is not the CPU `f64` estimator.
//!
//! Checkpoints stay untrusted run artifacts until identity, digest, and
//! model-run provenance validate. They cannot replace the reference
//! estimator or promote a scientific claim (ADR 0001/0014).

mod authority;
mod error;

/// Closed vocabulary of scientific-authority roles for a run artifact.
pub use authority::ArtifactRole;
/// Identity, digest, and run provenance for one checkpoint offer.
pub use authority::CheckpointOffer;
/// Accept a checkpoint only as a validated run artifact.
pub use authority::accept_checkpoint_artifact;
/// Fraction of recovered artifact roles that match known truth.
pub use authority::authority_recovery_rate;
/// Refuse to treat a checkpoint as the CPU `f64` estimator.
pub use authority::refuse_checkpoint_as_estimator;
/// Fail-closed checkpoint-authority errors.
pub use error::CheckpointAuthorityError;
