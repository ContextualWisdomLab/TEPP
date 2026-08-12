//! Contextual membership roles (not permanent entity classes).

use crate::MembershipError;
use serde::{Deserialize, Serialize};

/// A contextual role used by multilevel and multiple-membership models.
///
/// Customer, partner, and competitor are roles that can change over time for the
/// same organization without rewriting entity identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipRole {
    /// Document or passage author.
    Author,
    /// Organizational department or team.
    Department,
    /// Stable organization identity under role assignments.
    Organization,
    /// Customer role in a commercial context.
    Customer,
    /// Partner role in a commercial context.
    Partner,
    /// Competitor role in a commercial context.
    Competitor,
    /// Project or program membership.
    Project,
    /// Opportunity pool or deal context.
    OpportunityPool,
    /// Document or prompt template family.
    Template,
    /// Language community or locale channel.
    Language,
    /// Geographic or market location.
    Location,
    /// Temporal episode or campaign.
    Episode,
}

impl MembershipRole {
    /// Parse a stable wire role name.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError::UnknownMembershipRole`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, MembershipError> {
        match name {
            "author" => Ok(Self::Author),
            "department" => Ok(Self::Department),
            "organization" => Ok(Self::Organization),
            "customer" => Ok(Self::Customer),
            "partner" => Ok(Self::Partner),
            "competitor" => Ok(Self::Competitor),
            "project" => Ok(Self::Project),
            "opportunity_pool" => Ok(Self::OpportunityPool),
            "template" => Ok(Self::Template),
            "language" => Ok(Self::Language),
            "location" => Ok(Self::Location),
            "episode" => Ok(Self::Episode),
            _ => Err(MembershipError::UnknownMembershipRole),
        }
    }

    /// Return the stable wire role name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Author => "author",
            Self::Department => "department",
            Self::Organization => "organization",
            Self::Customer => "customer",
            Self::Partner => "partner",
            Self::Competitor => "competitor",
            Self::Project => "project",
            Self::OpportunityPool => "opportunity_pool",
            Self::Template => "template",
            Self::Language => "language",
            Self::Location => "location",
            Self::Episode => "episode",
        }
    }
}
