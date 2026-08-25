#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Prompt boilerplate is not unique latent content.
//!
//! Instruction and prompt text stays explicit method structure. It is not
//! unique document meaning and is not erased by a stopword list
//! (ADR 0004/0012).

mod error;
mod kind;

/// Fail-closed prompt-source errors.
pub use error::PromptSourceError;
/// Closed vocabulary of prompt-related token treatments.
pub use kind::PromptKind;
/// Fraction of recovered prompt kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat prompt boilerplate as stopword deletion.
pub use kind::refuse_prompt_as_stopword_deletion;
/// Refuse to treat prompt boilerplate as unique latent content.
pub use kind::refuse_prompt_as_unique_content;
