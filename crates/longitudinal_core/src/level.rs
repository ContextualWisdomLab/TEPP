//! Explicit within/between component status.

use crate::LongitudinalError;

/// Established longitudinal component level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentLevel {
    /// Stable between-unit component.
    Between,
    /// Occasion-specific within-unit residual.
    Within,
}

impl ComponentLevel {
    /// Stable wire name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Between => "between",
            Self::Within => "within",
        }
    }

    /// Parse a stable wire component-level name.
    ///
    /// # Errors
    ///
    /// Returns [`LongitudinalError::UnknownComponentLevel`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, LongitudinalError> {
        match name {
            "between" => Ok(Self::Between),
            "within" => Ok(Self::Within),
            _ => Err(LongitudinalError::UnknownComponentLevel),
        }
    }

    /// Return whether this level is within-unit change.
    #[must_use]
    pub const fn is_within_change(self) -> bool {
        matches!(self, Self::Within)
    }
}

/// Refuse to treat a between-unit component as within-unit change.
///
/// # Errors
///
/// Returns [`LongitudinalError::BetweenIsNotWithinChange`] when the component
/// is between-unit.
pub fn refuse_between_as_within_change(level: ComponentLevel) -> Result<(), LongitudinalError> {
    if level.is_within_change() {
        Ok(())
    } else {
        Err(LongitudinalError::BetweenIsNotWithinChange)
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentLevel;
    use crate::LongitudinalError;

    #[test]
    fn wire_names_round_trip() {
        for level in [ComponentLevel::Between, ComponentLevel::Within] {
            assert_eq!(
                ComponentLevel::from_wire_name(level.wire_name()).expect("round trip"),
                level
            );
        }
        assert_eq!(
            ComponentLevel::from_wire_name("pooled"),
            Err(LongitudinalError::UnknownComponentLevel)
        );
    }
}
