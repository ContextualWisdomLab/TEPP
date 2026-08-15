//! Explicit invariance status for shared metric interpretation.

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
    pub const fn licenses_shared_metric_meaning(self) -> bool {
        matches!(self, Self::Metric | Self::Scalar)
    }
}

/// Require an invariance status strong enough for shared metric meaning.
///
/// # Errors
///
/// Returns [`InvarianceError::InvarianceTooWeakForSharedMetricMeaning`] when
/// the status is only configural.
pub fn require_shared_metric_meaning(level: InvarianceLevel) -> Result<(), InvarianceError> {
    if level.licenses_shared_metric_meaning() {
        Ok(())
    } else {
        Err(InvarianceError::InvarianceTooWeakForSharedMetricMeaning)
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

    #[test]
    fn only_metric_or_scalar_status_licenses_shared_metric_meaning() {
        assert!(!InvarianceLevel::Configural.licenses_shared_metric_meaning());
        assert!(InvarianceLevel::Metric.licenses_shared_metric_meaning());
        assert!(InvarianceLevel::Scalar.licenses_shared_metric_meaning());
    }
}
