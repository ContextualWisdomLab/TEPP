//! Size, depth, identity, and provenance gates for untrusted payloads.

use crate::PayloadBoundError;

/// Closed vocabulary of untrusted inbound payload kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayloadKind {
    /// External document bytes.
    Document,
    /// Serialized domain or wire record.
    SerializedRecord,
    /// Model checkpoint or artifact bytes.
    ModelCheckpoint,
    /// LLM or agent output.
    LlmOutput,
}

impl PayloadKind {
    /// Return the stable wire payload-kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::SerializedRecord => "serialized_record",
            Self::ModelCheckpoint => "model_checkpoint",
            Self::LlmOutput => "llm_output",
        }
    }

    /// Parse a stable wire payload-kind name.
    ///
    /// # Errors
    ///
    /// Returns [`PayloadBoundError::InvalidPayloadDecision`] for unrecognized
    /// names.
    pub fn from_wire_name(name: &str) -> Result<Self, PayloadBoundError> {
        match name {
            "document" => Ok(Self::Document),
            "serialized_record" => Ok(Self::SerializedRecord),
            "model_checkpoint" => Ok(Self::ModelCheckpoint),
            "llm_output" => Ok(Self::LlmOutput),
            _ => Err(PayloadBoundError::InvalidPayloadDecision),
        }
    }
}

/// Positive byte and nesting-depth limits for one untrusted payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PayloadBound {
    max_bytes: usize,
    max_depth: usize,
}

impl PayloadBound {
    /// Construct a positive payload bound.
    ///
    /// # Errors
    ///
    /// Returns [`PayloadBoundError::InvalidBound`] when either maximum is zero.
    pub const fn new(max_bytes: usize, max_depth: usize) -> Result<Self, PayloadBoundError> {
        if max_bytes == 0 || max_depth == 0 {
            return Err(PayloadBoundError::InvalidBound);
        }
        Ok(Self {
            max_bytes,
            max_depth,
        })
    }

    /// Return the maximum accepted payload size in bytes.
    #[must_use]
    pub const fn max_bytes(self) -> usize {
        self.max_bytes
    }

    /// Return the maximum accepted nesting depth.
    #[must_use]
    pub const fn max_depth(self) -> usize {
        self.max_depth
    }
}

/// Refuse an untrusted payload that lacks identity or provenance or exceeds
/// the configured size or depth bound.
///
/// # Errors
///
/// Returns a missing-identity, missing-provenance, size, or depth error.
pub fn refuse_untrusted_payload(
    kind: PayloadKind,
    identity: Option<&str>,
    provenance: Option<&str>,
    byte_len: usize,
    depth: usize,
    bound: PayloadBound,
) -> Result<(), PayloadBoundError> {
    let _ = kind.wire_name();
    match identity {
        Some(value) if !value.is_empty() => {}
        _ => return Err(PayloadBoundError::MissingIdentity),
    }
    match provenance {
        Some(value) if !value.is_empty() => {}
        _ => return Err(PayloadBoundError::MissingProvenance),
    }
    if byte_len > bound.max_bytes() {
        return Err(PayloadBoundError::PayloadTooLarge);
    }
    if depth > bound.max_depth() {
        return Err(PayloadBoundError::PayloadTooDeep);
    }
    Ok(())
}

/// Fraction of recovered accept/reject flags that match known truth.
///
/// # Errors
///
/// Returns [`PayloadBoundError::InvalidPayloadDecision`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(truth: &[bool], decided: &[bool]) -> Result<f64, PayloadBoundError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(PayloadBoundError::InvalidPayloadDecision);
    }
    let mut matches = 0_u32;
    for (truth_flag, decided_flag) in truth.iter().zip(decided) {
        if truth_flag == decided_flag {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{PayloadBound, PayloadKind, identity_recovery_rate, refuse_untrusted_payload};
    use crate::PayloadBoundError;

    #[test]
    fn local_branches_cover_kinds_bounds_and_payloads() {
        assert_eq!(
            PayloadBound::new(0, 1),
            Err(PayloadBoundError::InvalidBound)
        );
        assert_eq!(
            PayloadBound::new(1, 0),
            Err(PayloadBoundError::InvalidBound)
        );
        let bound = PayloadBound::new(8, 2).expect("bound");
        assert_eq!(bound.max_bytes(), 8);
        assert_eq!(bound.max_depth(), 2);
        assert_eq!(
            refuse_untrusted_payload(PayloadKind::LlmOutput, None, Some("p"), 1, 1, bound),
            Err(PayloadBoundError::MissingIdentity)
        );
        assert_eq!(
            refuse_untrusted_payload(PayloadKind::Document, Some(""), Some("p"), 1, 1, bound),
            Err(PayloadBoundError::MissingIdentity)
        );
        assert_eq!(
            refuse_untrusted_payload(PayloadKind::SerializedRecord, Some("id"), None, 1, 1, bound),
            Err(PayloadBoundError::MissingProvenance)
        );
        assert_eq!(
            refuse_untrusted_payload(
                PayloadKind::ModelCheckpoint,
                Some("id"),
                Some(""),
                1,
                1,
                bound
            ),
            Err(PayloadBoundError::MissingProvenance)
        );
        assert_eq!(
            refuse_untrusted_payload(
                PayloadKind::ModelCheckpoint,
                Some("id"),
                Some("p"),
                9,
                1,
                bound
            ),
            Err(PayloadBoundError::PayloadTooLarge)
        );
        assert_eq!(
            refuse_untrusted_payload(PayloadKind::Document, Some("id"), Some("p"), 1, 3, bound),
            Err(PayloadBoundError::PayloadTooDeep)
        );
        refuse_untrusted_payload(PayloadKind::Document, Some("id"), Some("p"), 8, 2, bound)
            .expect("ok");
        for kind in [
            PayloadKind::Document,
            PayloadKind::SerializedRecord,
            PayloadKind::ModelCheckpoint,
            PayloadKind::LlmOutput,
        ] {
            assert_eq!(
                PayloadKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            PayloadKind::from_wire_name("trusted"),
            Err(PayloadBoundError::InvalidPayloadDecision)
        );
        let matched = identity_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(PayloadBoundError::InvalidPayloadDecision)
        );
        assert_eq!(
            identity_recovery_rate(&[true], &[]),
            Err(PayloadBoundError::InvalidPayloadDecision)
        );
    }
}
