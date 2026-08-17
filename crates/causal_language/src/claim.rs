//! Claim kinds that require identification before causal language.

use crate::CausalLanguageError;

/// Closed vocabulary of association versus identified causal claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimKind {
    /// Co-occurrence or statistical association without identification.
    Association,
    /// Earlier-later order without an identified causal design.
    TemporalPrecedence,
    /// A citation, hyperlink, or membership link without identification.
    DocumentLink,
    /// Randomized or otherwise experimental identification.
    IdentifiedExperimental,
    /// Quasi-experimental identification with a stated design.
    IdentifiedQuasiExperimental,
    /// Defensible observational identification with a stated design.
    IdentifiedObservational,
}

impl ClaimKind {
    /// Return the stable wire claim name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Association => "association",
            Self::TemporalPrecedence => "temporal_precedence",
            Self::DocumentLink => "document_link",
            Self::IdentifiedExperimental => "identified_experimental",
            Self::IdentifiedQuasiExperimental => "identified_quasi_experimental",
            Self::IdentifiedObservational => "identified_observational",
        }
    }

    /// Parse a stable wire claim name.
    ///
    /// # Errors
    ///
    /// Returns [`CausalLanguageError::InvalidClaimPayload`] for unrecognized
    /// names, including bare `causal` or `causes`.
    pub fn from_wire_name(name: &str) -> Result<Self, CausalLanguageError> {
        match name {
            "association" => Ok(Self::Association),
            "temporal_precedence" => Ok(Self::TemporalPrecedence),
            "document_link" => Ok(Self::DocumentLink),
            "identified_experimental" => Ok(Self::IdentifiedExperimental),
            "identified_quasi_experimental" => Ok(Self::IdentifiedQuasiExperimental),
            "identified_observational" => Ok(Self::IdentifiedObservational),
            _ => Err(CausalLanguageError::InvalidClaimPayload),
        }
    }

    /// Return whether this kind is an identified causal claim.
    #[must_use]
    pub const fn is_identified_causal(self) -> bool {
        match self {
            Self::Association | Self::TemporalPrecedence | Self::DocumentLink => false,
            Self::IdentifiedExperimental
            | Self::IdentifiedQuasiExperimental
            | Self::IdentifiedObservational => true,
        }
    }
}

/// Refuse to treat an unidentified claim as causal language.
///
/// Identified experimental, quasi-experimental, and defensible observational
/// designs are causal-eligible. They are not association, so this gate lets
/// them through.
///
/// # Errors
///
/// Returns [`CausalLanguageError::UnidentifiedIsNotCausal`] when `kind` is
/// [`ClaimKind::Association`], [`ClaimKind::TemporalPrecedence`], or
/// [`ClaimKind::DocumentLink`].
pub fn refuse_unidentified_as_causal(kind: ClaimKind) -> Result<(), CausalLanguageError> {
    match kind {
        ClaimKind::Association | ClaimKind::TemporalPrecedence | ClaimKind::DocumentLink => {
            Err(CausalLanguageError::UnidentifiedIsNotCausal)
        }
        ClaimKind::IdentifiedExperimental
        | ClaimKind::IdentifiedQuasiExperimental
        | ClaimKind::IdentifiedObservational => Ok(()),
    }
}

/// Fraction of recovered claim kinds that match known truth.
///
/// # Errors
///
/// Returns [`CausalLanguageError::InvalidClaimPayload`] when either slice is
/// empty or the lengths differ.
pub fn claim_kind_recovery_rate(
    truth: &[ClaimKind],
    decided: &[ClaimKind],
) -> Result<f64, CausalLanguageError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(CausalLanguageError::InvalidClaimPayload);
    }
    let mut matches = 0_u32;
    for (truth_kind, decided_kind) in truth.iter().zip(decided) {
        if truth_kind == decided_kind {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{ClaimKind, claim_kind_recovery_rate, refuse_unidentified_as_causal};
    use crate::CausalLanguageError;

    #[test]
    fn local_branches_cover_claims_payloads_and_wire_names() {
        assert_eq!(
            refuse_unidentified_as_causal(ClaimKind::Association),
            Err(CausalLanguageError::UnidentifiedIsNotCausal)
        );
        assert_eq!(
            refuse_unidentified_as_causal(ClaimKind::TemporalPrecedence),
            Err(CausalLanguageError::UnidentifiedIsNotCausal)
        );
        assert_eq!(
            refuse_unidentified_as_causal(ClaimKind::DocumentLink),
            Err(CausalLanguageError::UnidentifiedIsNotCausal)
        );
        refuse_unidentified_as_causal(ClaimKind::IdentifiedExperimental).expect("experimental");
        refuse_unidentified_as_causal(ClaimKind::IdentifiedQuasiExperimental)
            .expect("quasi-experimental");
        refuse_unidentified_as_causal(ClaimKind::IdentifiedObservational).expect("observational");
        for kind in [
            ClaimKind::Association,
            ClaimKind::TemporalPrecedence,
            ClaimKind::DocumentLink,
            ClaimKind::IdentifiedExperimental,
            ClaimKind::IdentifiedQuasiExperimental,
            ClaimKind::IdentifiedObservational,
        ] {
            assert_eq!(
                ClaimKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
            assert_eq!(
                kind.is_identified_causal(),
                matches!(
                    kind,
                    ClaimKind::IdentifiedExperimental
                        | ClaimKind::IdentifiedQuasiExperimental
                        | ClaimKind::IdentifiedObservational
                )
            );
        }
        assert_eq!(
            ClaimKind::from_wire_name("causes"),
            Err(CausalLanguageError::InvalidClaimPayload)
        );
        let truth = [ClaimKind::Association, ClaimKind::IdentifiedExperimental];
        let matched = claim_kind_recovery_rate(&truth, &truth).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        let partial =
            claim_kind_recovery_rate(&truth, &[ClaimKind::Association, ClaimKind::Association])
                .expect("partial");
        assert!((partial - 0.5).abs() < f64::EPSILON);
        assert_eq!(
            claim_kind_recovery_rate(&[], &[]),
            Err(CausalLanguageError::InvalidClaimPayload)
        );
        assert_eq!(
            claim_kind_recovery_rate(&truth, &[]),
            Err(CausalLanguageError::InvalidClaimPayload)
        );
    }
}
