//! Stable topic identity independent of activity state.

use crate::TopicLineageError;
use uuid::Uuid;

/// Opaque global topic identity (P0).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TopicIdentity(Uuid);

impl TopicIdentity {
    /// Reconstruct from a UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Borrow the UUID value.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

/// Fraction of recovered identities that match known-truth identities.
///
/// # Errors
///
/// Returns [`TopicLineageError::InvalidIdentityPayload`] when either slice is
/// empty or the lengths differ.
#[allow(clippy::cast_precision_loss)]
pub fn identity_recovery_rate(
    truth: &[TopicIdentity],
    decided: &[TopicIdentity],
) -> Result<f64, TopicLineageError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(TopicLineageError::InvalidIdentityPayload);
    }
    let mut matches = 0_usize;
    for (truth_id, decided_id) in truth.iter().zip(decided) {
        if truth_id == decided_id {
            matches += 1;
        }
    }
    Ok(matches as f64 / truth.len() as f64)
}

/// Explicit refusal to treat reactivation as a newly minted topic.
///
/// # Errors
///
/// Returns [`TopicLineageError::ReactivationIsNotNewTopic`] when the proposed
/// identity differs from the incumbent.
pub fn refuse_new_identity_on_reactivation(
    incumbent: TopicIdentity,
    proposed: TopicIdentity,
) -> Result<(), TopicLineageError> {
    if incumbent == proposed {
        Ok(())
    } else {
        Err(TopicLineageError::ReactivationIsNotNewTopic)
    }
}

#[cfg(test)]
mod tests {
    use super::{TopicIdentity, identity_recovery_rate, refuse_new_identity_on_reactivation};
    use crate::TopicLineageError;
    use uuid::Uuid;

    #[test]
    fn identity_helpers_cover_local_branches() {
        let identity = TopicIdentity::from_uuid(Uuid::from_u128(1));
        assert_eq!(identity.as_uuid(), Uuid::from_u128(1));
        assert_eq!(
            refuse_new_identity_on_reactivation(identity, identity),
            Ok(())
        );
        let other = TopicIdentity::from_uuid(Uuid::from_u128(2));
        assert_eq!(
            refuse_new_identity_on_reactivation(identity, other),
            Err(TopicLineageError::ReactivationIsNotNewTopic)
        );
        assert_eq!(
            identity_recovery_rate(&[identity], &[]),
            Err(TopicLineageError::InvalidIdentityPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[], &[identity]),
            Err(TopicLineageError::InvalidIdentityPayload)
        );
        assert_eq!(identity_recovery_rate(&[identity], &[identity]), Ok(1.0));
        assert_eq!(identity_recovery_rate(&[identity], &[other]), Ok(0.0));
    }
}
