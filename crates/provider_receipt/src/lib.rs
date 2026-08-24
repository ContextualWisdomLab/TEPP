#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Provider-disclosure receipts that refuse source text and identity.
//!
//! A receipt records which field codes were sent to a model provider under one
//! purpose. It cannot carry source text or source identity, and blanket PII
//! masking is not a disclosure grant (ADR 0009).

mod error;
mod receipt;

/// Fail-closed provider-receipt errors.
pub use error::ProviderReceiptError;
/// One provider-disclosure receipt of field codes under a purpose.
pub use receipt::ProviderReceipt;
/// Fraction of recovered field codes that match known truth.
pub use receipt::receipt_recovery_rate;
/// Refuse to treat a blanket PII mask as provider-disclosure authorization.
pub use receipt::refuse_blanket_mask_as_disclosure;
/// Refuse to place source identity in a provider receipt.
pub use receipt::refuse_source_identity_in_receipt;
/// Refuse to place raw source text in a provider receipt.
pub use receipt::refuse_source_text_in_receipt;
