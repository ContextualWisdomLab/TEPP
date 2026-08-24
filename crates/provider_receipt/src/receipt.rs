//! Purpose-bound field-code receipts for provider disclosure.

use crate::ProviderReceiptError;

/// One provider-disclosure receipt of field codes under a purpose.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderReceipt {
    purpose_code: u16,
    field_codes: Vec<u16>,
}

impl ProviderReceipt {
    /// Record the field codes sent to a provider under one purpose.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderReceiptError::InvalidReceiptPayload`] when no field
    /// codes are supplied.
    pub fn new(purpose_code: u16, field_codes: &[u16]) -> Result<Self, ProviderReceiptError> {
        if field_codes.is_empty() {
            return Err(ProviderReceiptError::InvalidReceiptPayload);
        }
        Ok(Self {
            purpose_code,
            field_codes: field_codes.to_vec(),
        })
    }

    /// Purpose bound to the disclosure.
    #[must_use]
    pub const fn purpose_code(&self) -> u16 {
        self.purpose_code
    }

    /// Field codes sent, never source text.
    #[must_use]
    pub fn field_codes(&self) -> &[u16] {
        &self.field_codes
    }
}

/// Refuse to place raw source text in a provider receipt.
///
/// # Errors
///
/// Always returns [`ProviderReceiptError::SourceTextNotDisclosable`].
pub fn refuse_source_text_in_receipt() -> Result<(), ProviderReceiptError> {
    Err(ProviderReceiptError::SourceTextNotDisclosable)
}

/// Refuse to place source identity in a provider receipt.
///
/// # Errors
///
/// Always returns [`ProviderReceiptError::SourceIdentityNotDisclosable`].
pub fn refuse_source_identity_in_receipt() -> Result<(), ProviderReceiptError> {
    Err(ProviderReceiptError::SourceIdentityNotDisclosable)
}

/// Refuse to treat a blanket PII mask as provider-disclosure authorization.
///
/// # Errors
///
/// Always returns [`ProviderReceiptError::BlanketMaskIsNotAuthorization`].
pub fn refuse_blanket_mask_as_disclosure() -> Result<(), ProviderReceiptError> {
    Err(ProviderReceiptError::BlanketMaskIsNotAuthorization)
}

/// Fraction of recovered field codes that match known truth.
///
/// # Errors
///
/// Returns [`ProviderReceiptError::InvalidReceiptPayload`] when the field-code
/// lengths differ.
pub fn receipt_recovery_rate(
    truth: &ProviderReceipt,
    decided: &ProviderReceipt,
) -> Result<f64, ProviderReceiptError> {
    if truth.field_codes.len() != decided.field_codes.len() {
        return Err(ProviderReceiptError::InvalidReceiptPayload);
    }
    let mut matches = 0_u32;
    for (truth_field, decided_field) in truth.field_codes.iter().zip(&decided.field_codes) {
        if truth_field == decided_field {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.field_codes.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        ProviderReceipt, receipt_recovery_rate, refuse_blanket_mask_as_disclosure,
        refuse_source_identity_in_receipt, refuse_source_text_in_receipt,
    };
    use crate::ProviderReceiptError;

    #[test]
    fn local_branches_cover_construct_and_fail_closed_paths() {
        let receipt = ProviderReceipt::new(7, &[1, 2]).expect("receipt");
        assert_eq!(receipt.purpose_code(), 7);
        assert_eq!(receipt.field_codes(), &[1, 2]);
        let matched = receipt_recovery_rate(&receipt, &receipt).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            ProviderReceipt::new(7, &[]),
            Err(ProviderReceiptError::InvalidReceiptPayload)
        );
        let short = ProviderReceipt::new(7, &[1]).expect("short");
        assert_eq!(
            receipt_recovery_rate(&receipt, &short),
            Err(ProviderReceiptError::InvalidReceiptPayload)
        );
        assert_eq!(
            refuse_source_text_in_receipt(),
            Err(ProviderReceiptError::SourceTextNotDisclosable)
        );
        assert_eq!(
            refuse_source_identity_in_receipt(),
            Err(ProviderReceiptError::SourceIdentityNotDisclosable)
        );
        assert_eq!(
            refuse_blanket_mask_as_disclosure(),
            Err(ProviderReceiptError::BlanketMaskIsNotAuthorization)
        );
    }
}
