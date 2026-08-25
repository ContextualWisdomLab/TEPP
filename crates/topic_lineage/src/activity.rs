//! Activity states that cannot change topic identity.

use crate::{TopicIdentity, TopicLineageError};

/// Activity of one global topic identity over time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TopicActivity {
    /// The topic is currently expressed.
    Active,
    /// The topic is temporarily unexpressed without losing identity.
    Dormant,
    /// The topic has returned after dormancy under the same identity.
    Reactivated,
}

impl TopicActivity {
    /// Stable wire name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Dormant => "dormant",
            Self::Reactivated => "reactivated",
        }
    }
}

/// One topic identity together with its current activity state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TopicLineageRecord {
    identity: TopicIdentity,
    activity: TopicActivity,
}

impl TopicLineageRecord {
    /// Open an active topic identity.
    #[must_use]
    pub const fn active(identity: TopicIdentity) -> Self {
        Self {
            identity,
            activity: TopicActivity::Active,
        }
    }

    /// Return the durable topic identity.
    #[must_use]
    pub const fn identity(self) -> TopicIdentity {
        self.identity
    }

    /// Return the current activity state.
    #[must_use]
    pub const fn activity(self) -> TopicActivity {
        self.activity
    }

    /// Move an active or reactivated topic into dormancy.
    ///
    /// # Errors
    ///
    /// Returns [`TopicLineageError::InvalidActivityTransition`] when the topic
    /// is already dormant.
    pub fn make_dormant(self) -> Result<Self, TopicLineageError> {
        match self.activity {
            TopicActivity::Active | TopicActivity::Reactivated => Ok(Self {
                identity: self.identity,
                activity: TopicActivity::Dormant,
            }),
            TopicActivity::Dormant => Err(TopicLineageError::InvalidActivityTransition),
        }
    }

    /// Reactivate a dormant topic without changing its identity.
    ///
    /// # Errors
    ///
    /// Returns [`TopicLineageError::InvalidActivityTransition`] when the topic
    /// is not dormant.
    pub fn reactivate(self) -> Result<Self, TopicLineageError> {
        match self.activity {
            TopicActivity::Dormant => Ok(Self {
                identity: self.identity,
                activity: TopicActivity::Reactivated,
            }),
            TopicActivity::Active | TopicActivity::Reactivated => {
                Err(TopicLineageError::InvalidActivityTransition)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TopicActivity, TopicLineageRecord};
    use crate::{TopicIdentity, TopicLineageError};
    use uuid::Uuid;

    #[test]
    fn illegal_transitions_fail_closed() {
        let identity = TopicIdentity::from_uuid(Uuid::from_u128(5));
        let active = TopicLineageRecord::active(identity);
        assert_eq!(active.activity().wire_name(), "active");
        assert_eq!(
            active.reactivate(),
            Err(TopicLineageError::InvalidActivityTransition)
        );
        let dormant = active.make_dormant().expect("dormant");
        assert_eq!(
            dormant.make_dormant(),
            Err(TopicLineageError::InvalidActivityTransition)
        );
        let reactivated = dormant.reactivate().expect("reactivated");
        assert_eq!(reactivated.activity().wire_name(), "reactivated");
        assert_eq!(TopicActivity::Dormant.wire_name(), "dormant");
    }
}
