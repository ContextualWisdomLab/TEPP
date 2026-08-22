//! Construct-class classification and interpretation gates.

use crate::error::PsychometricError;

/// Higher-order construct class before ESEM, composite, or network modeling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConstructClass {
    /// Reflective indicators of a common latent factor.
    Reflective,
    /// Formative or composite indicators that define the construct.
    Formative,
    /// Interacting indicators that belong in a network model.
    Network,
    /// Insufficient evidence to classify the construct.
    Unresolved,
}

impl ConstructClass {
    /// Stable wire name for the construct class.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reflective => "reflective",
            Self::Formative => "formative",
            Self::Network => "network",
            Self::Unresolved => "unresolved",
        }
    }

    /// Return whether reflective ESEM/set-ESEM is admissible.
    #[must_use]
    pub const fn admits_reflective_esem(self) -> bool {
        matches!(self, Self::Reflective)
    }
}

/// Interpret a classified construct as reflective.
///
/// A good global fit statistic is not authority to reinterpret a formative or
/// network structure as reflective (ADR 0005).
///
/// # Errors
///
/// Returns [`PsychometricError::FormativeReinterpretationForbidden`] for
/// formative or network classes and
/// [`PsychometricError::UnresolvedConstruct`] when the class is unresolved.
pub fn interpret_as_reflective(
    classified: ConstructClass,
    global_fit_acceptable: bool,
) -> Result<ConstructClass, PsychometricError> {
    match (classified, global_fit_acceptable) {
        (ConstructClass::Reflective, true | false) => Ok(ConstructClass::Reflective),
        (ConstructClass::Unresolved, true | false) => Err(PsychometricError::UnresolvedConstruct),
        (ConstructClass::Formative | ConstructClass::Network, true | false) => {
            Err(PsychometricError::FormativeReinterpretationForbidden)
        }
    }
}

/// Permit a latent-mean or path comparison only when invariance evidence is
/// already established for the claimed comparison.
///
/// # Errors
///
/// Returns [`PsychometricError::InvarianceRequired`] when the required
/// invariance level has not been met.
pub fn compare_latent_means(invariance_level_met: bool) -> Result<(), PsychometricError> {
    if invariance_level_met {
        Ok(())
    } else {
        Err(PsychometricError::InvarianceRequired)
    }
}

#[cfg(test)]
mod tests {
    use super::{ConstructClass, compare_latent_means, interpret_as_reflective};
    use crate::error::PsychometricError;

    #[test]
    fn reflective_only_admits_esem_and_invariance_is_required() {
        assert!(ConstructClass::Reflective.admits_reflective_esem());
        assert!(!ConstructClass::Formative.admits_reflective_esem());
        compare_latent_means(true).expect("ok");
        assert_eq!(
            interpret_as_reflective(ConstructClass::Reflective, true).expect("fit unused"),
            ConstructClass::Reflective
        );
        assert_eq!(
            interpret_as_reflective(ConstructClass::Network, false),
            Err(PsychometricError::FormativeReinterpretationForbidden)
        );
    }
}
