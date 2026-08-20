//! Fail-closed selective-disclosure errors.

use std::fmt;

/// A fail-closed selective-disclosure error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SelectiveDisclosureError {
    /// A requested field was not present on the source artifact.
    MissingSourceField,
    /// Direct identity or source text was requested without re-identification.
    UnauthorizedField,
    /// A scientific purpose omitted required authorship, time, or membership.
    BlanketMaskDestroysMeasurement,
    /// Field or purpose slices were empty, duplicated, or unknown.
    InvalidDisclosurePayload,
}

impl fmt::Display for SelectiveDisclosureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingSourceField => "requested field is absent from the source artifact",
            Self::UnauthorizedField => {
                "direct identity and source text require re-identification purpose"
            }
            Self::BlanketMaskDestroysMeasurement => {
                "blanket PII masking would destroy scientific linkage"
            }
            Self::InvalidDisclosurePayload => "invalid selective-disclosure payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SelectiveDisclosureError {}

#[cfg(test)]
mod tests {
    use super::SelectiveDisclosureError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                SelectiveDisclosureError::MissingSourceField,
                "requested field is absent from the source artifact",
            ),
            (
                SelectiveDisclosureError::UnauthorizedField,
                "direct identity and source text require re-identification purpose",
            ),
            (
                SelectiveDisclosureError::BlanketMaskDestroysMeasurement,
                "blanket PII masking would destroy scientific linkage",
            ),
            (
                SelectiveDisclosureError::InvalidDisclosurePayload,
                "invalid selective-disclosure payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
