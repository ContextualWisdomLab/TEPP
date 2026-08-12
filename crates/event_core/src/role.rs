//! Typed event roles and arguments.

use crate::EventError;
use serde::{Deserialize, Serialize};

/// A typed role attached to an event instance.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventRoleKind {
    /// Primary agent or actor.
    Agent,
    /// Patient or affected entity.
    Patient,
    /// Causal or contributing factor.
    Factor,
    /// Product or outcome.
    Product,
    /// Place or locus.
    Place,
    /// Instrument or means.
    Instrument,
    /// Beneficiary.
    Beneficiary,
}

impl EventRoleKind {
    /// Parse a stable wire role name.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::UnknownEventRole`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, EventError> {
        match name {
            "agent" => Ok(Self::Agent),
            "patient" => Ok(Self::Patient),
            "factor" => Ok(Self::Factor),
            "product" => Ok(Self::Product),
            "place" => Ok(Self::Place),
            "instrument" => Ok(Self::Instrument),
            "beneficiary" => Ok(Self::Beneficiary),
            _ => Err(EventError::UnknownEventRole),
        }
    }

    /// Return the stable wire role name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Patient => "patient",
            Self::Factor => "factor",
            Self::Product => "product",
            Self::Place => "place",
            Self::Instrument => "instrument",
            Self::Beneficiary => "beneficiary",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EventRoleKind;
    use crate::EventError;

    #[test]
    fn roles_round_trip_wire_names() {
        for role in [
            EventRoleKind::Agent,
            EventRoleKind::Patient,
            EventRoleKind::Factor,
            EventRoleKind::Product,
            EventRoleKind::Place,
            EventRoleKind::Instrument,
            EventRoleKind::Beneficiary,
        ] {
            assert_eq!(
                EventRoleKind::from_wire_name(role.wire_name()).expect("parse"),
                role
            );
        }
        assert_eq!(
            EventRoleKind::from_wire_name("villain"),
            Err(EventError::UnknownEventRole)
        );
    }
}
