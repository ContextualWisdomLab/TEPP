//! Known-truth unsupported-claim rates for interpretation proposals.

use crate::InterpretationError;

/// Whether a claim is actually supported by the cited evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimSupport {
    /// The cited evidence supports the claim.
    Supported,
    /// The claim is unsupported or was promoted without evidence.
    Unsupported,
}

/// False-support rate: unsupported truth labeled supported, over unsupported truth.
///
/// # Errors
///
/// Returns [`InterpretationError::InvalidSupportPayload`] when either slice is
/// empty, the lengths differ, or the truth stream contains no unsupported
/// claim.
pub fn unsupported_claim_rate(
    truth: &[ClaimSupport],
    decided: &[ClaimSupport],
) -> Result<f64, InterpretationError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(InterpretationError::InvalidSupportPayload);
    }
    let mut unsupported_truth = 0_u32;
    let mut false_support = 0_u32;
    for (truth_label, decided_label) in truth.iter().zip(decided) {
        if *truth_label == ClaimSupport::Unsupported {
            unsupported_truth += 1;
            if *decided_label == ClaimSupport::Supported {
                false_support += 1;
            }
        }
    }
    if unsupported_truth == 0 {
        return Err(InterpretationError::InvalidSupportPayload);
    }
    Ok(f64::from(false_support) / f64::from(unsupported_truth))
}

#[cfg(test)]
mod tests {
    use super::{ClaimSupport, unsupported_claim_rate};
    use crate::InterpretationError;

    #[test]
    fn support_rate_rejects_empty_and_all_supported_truth() {
        assert_eq!(
            unsupported_claim_rate(&[], &[]),
            Err(InterpretationError::InvalidSupportPayload)
        );
        assert_eq!(
            unsupported_claim_rate(&[ClaimSupport::Supported], &[ClaimSupport::Supported]),
            Err(InterpretationError::InvalidSupportPayload)
        );
        assert_eq!(
            unsupported_claim_rate(
                &[ClaimSupport::Unsupported],
                &[ClaimSupport::Supported, ClaimSupport::Supported]
            ),
            Err(InterpretationError::InvalidSupportPayload)
        );
    }
}
