//! Explicit invariance status that may or may not license shared meaning.

use crate::InvarianceError;

/// Established invariance status for a multi-group comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvarianceLevel {
    /// Same factor structure only; loadings are not comparable.
    Configural,
    /// Equal loadings; factor variances/means remain group-specific.
    Metric,
    /// Equal loadings and intercepts.
    Scalar,
}

impl InvarianceLevel {
    /// Stable wire name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Configural => "configural",
            Self::Metric => "metric",
            Self::Scalar => "scalar",
        }
    }

    /// Parse a stable wire invariance-status name.
    ///
    /// # Errors
    ///
    /// Returns [`InvarianceError::UnknownInvarianceLevel`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, InvarianceError> {
        match name {
            "configural" => Ok(Self::Configural),
            "metric" => Ok(Self::Metric),
            "scalar" => Ok(Self::Scalar),
            _ => Err(InvarianceError::UnknownInvarianceLevel),
        }
    }

    /// Return whether this status licenses shared metric meaning.
    #[must_use]
    pub const fn licenses_shared_meaning(self) -> bool {
        matches!(self, Self::Metric | Self::Scalar)
    }
}

/// Refuse to treat a weaker invariance status as shared meaning.
///
/// # Errors
///
/// Returns [`InvarianceError::InvarianceTooWeakForSharedMeaning`] when the
/// status is only configural.
pub fn refuse_noninvariant_as_shared_meaning(
    level: InvarianceLevel,
) -> Result<(), InvarianceError> {
    if level.licenses_shared_meaning() {
        Ok(())
    } else {
        Err(InvarianceError::InvarianceTooWeakForSharedMeaning)
    }
}

#[cfg(test)]
mod tests {
    use super::InvarianceLevel;
    use crate::InvarianceError;

    #[test]
    fn wire_names_round_trip() {
        for level in [
            InvarianceLevel::Configural,
            InvarianceLevel::Metric,
            InvarianceLevel::Scalar,
        ] {
            assert_eq!(
                InvarianceLevel::from_wire_name(level.wire_name()).expect("round trip"),
                level
            );
        }
        assert_eq!(
            InvarianceLevel::from_wire_name("partial"),
            Err(InvarianceError::UnknownInvarianceLevel)
        );
    }
}
