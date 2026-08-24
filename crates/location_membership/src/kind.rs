//! Location membership versus entity identity and language.

use crate::LocationMembershipError;

/// Closed vocabulary of location-related membership treatments.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocationKind {
    /// Time-varying market or place membership.
    Location,
    /// Permanent entity identity under role assignments.
    EntityIdentity,
    /// Language community or locale channel.
    LanguageChannel,
}

impl LocationKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Location => "location",
            Self::EntityIdentity => "entity_identity",
            Self::LanguageChannel => "language_channel",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`LocationMembershipError::InvalidLocationPayload`] for
    /// unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, LocationMembershipError> {
        match name {
            "location" => Ok(Self::Location),
            "entity_identity" => Ok(Self::EntityIdentity),
            "language_channel" => Ok(Self::LanguageChannel),
            _ => Err(LocationMembershipError::InvalidLocationPayload),
        }
    }
}

/// Refuse to treat location membership as permanent entity identity.
///
/// # Errors
///
/// Returns [`LocationMembershipError::LocationIsNotEntityIdentity`] when
/// `kind` is [`LocationKind::Location`].
pub fn refuse_location_as_entity_identity(
    kind: LocationKind,
) -> Result<(), LocationMembershipError> {
    match kind {
        LocationKind::Location => Err(LocationMembershipError::LocationIsNotEntityIdentity),
        LocationKind::EntityIdentity | LocationKind::LanguageChannel => Ok(()),
    }
}

/// Refuse to treat location membership as a language channel.
///
/// # Errors
///
/// Returns [`LocationMembershipError::LocationIsNotLanguageChannel`] when
/// `kind` is [`LocationKind::Location`].
pub fn refuse_location_as_language_channel(
    kind: LocationKind,
) -> Result<(), LocationMembershipError> {
    match kind {
        LocationKind::Location => Err(LocationMembershipError::LocationIsNotLanguageChannel),
        LocationKind::EntityIdentity | LocationKind::LanguageChannel => Ok(()),
    }
}

/// Fraction of recovered location kinds that match known truth.
///
/// # Errors
///
/// Returns [`LocationMembershipError::InvalidLocationPayload`] when either
/// slice is empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[LocationKind],
    decided: &[LocationKind],
) -> Result<f64, LocationMembershipError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(LocationMembershipError::InvalidLocationPayload);
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
    use super::{
        LocationKind, identity_recovery_rate, refuse_location_as_entity_identity,
        refuse_location_as_language_channel,
    };
    use crate::LocationMembershipError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_location_as_entity_identity(LocationKind::Location),
            Err(LocationMembershipError::LocationIsNotEntityIdentity)
        );
        assert_eq!(
            refuse_location_as_language_channel(LocationKind::Location),
            Err(LocationMembershipError::LocationIsNotLanguageChannel)
        );
        refuse_location_as_entity_identity(LocationKind::EntityIdentity).expect("entity");
        refuse_location_as_entity_identity(LocationKind::LanguageChannel).expect("language");
        refuse_location_as_language_channel(LocationKind::EntityIdentity).expect("entity");
        refuse_location_as_language_channel(LocationKind::LanguageChannel).expect("language");
        for kind in [
            LocationKind::Location,
            LocationKind::EntityIdentity,
            LocationKind::LanguageChannel,
        ] {
            assert_eq!(
                LocationKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            LocationKind::from_wire_name("project"),
            Err(LocationMembershipError::InvalidLocationPayload)
        );
        let matched = identity_recovery_rate(&[LocationKind::Location], &[LocationKind::Location])
            .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(LocationMembershipError::InvalidLocationPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[LocationKind::Location], &[]),
            Err(LocationMembershipError::InvalidLocationPayload)
        );
    }
}
