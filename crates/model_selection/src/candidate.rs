//! Candidate topic counts with statistical diagnostics.

use crate::ModelSelectionError;

/// One candidate `K` together with the diagnostics that may admit it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelCandidate {
    candidate_k: u32,
    schwarz_score: Option<f64>,
    complexity: Option<f64>,
    llm_vote_only: bool,
}

impl ModelCandidate {
    /// Construct a statistically supported candidate.
    ///
    /// `schwarz_score` is a higher-is-better model-selection score such as
    /// `ℓ - (p ln N)/2`; it is not a held-out predictive likelihood.
    ///
    /// # Errors
    ///
    /// Returns [`ModelSelectionError::NonPositiveCandidateK`] when `candidate_k`
    /// is less than two, or [`ModelSelectionError::InvalidDiagnostic`] when a
    /// diagnostic is non-finite.
    pub fn statistical(
        candidate_k: u32,
        schwarz_score: f64,
        complexity: f64,
    ) -> Result<Self, ModelSelectionError> {
        if candidate_k < 2 {
            return Err(ModelSelectionError::NonPositiveCandidateK);
        }
        if !schwarz_score.is_finite() || !complexity.is_finite() || complexity < 0.0 {
            return Err(ModelSelectionError::InvalidDiagnostic);
        }
        Ok(Self {
            candidate_k,
            schwarz_score: Some(schwarz_score),
            complexity: Some(complexity),
            llm_vote_only: false,
        })
    }

    /// Construct a candidate whose only support is an LLM vote.
    ///
    /// The vote may later recommend among statistically admissible candidates.
    /// It cannot itself define the numerical optimum.
    ///
    /// # Errors
    ///
    /// Returns [`ModelSelectionError::NonPositiveCandidateK`] when `candidate_k`
    /// is less than two.
    pub fn llm_vote_only(candidate_k: u32) -> Result<Self, ModelSelectionError> {
        if candidate_k < 2 {
            return Err(ModelSelectionError::NonPositiveCandidateK);
        }
        Ok(Self {
            candidate_k,
            schwarz_score: None,
            complexity: None,
            llm_vote_only: true,
        })
    }

    /// Return the candidate topic count.
    #[must_use]
    pub const fn candidate_k(self) -> u32 {
        self.candidate_k
    }

    /// Return whether this candidate carries finite statistical diagnostics.
    #[must_use]
    pub const fn is_statistically_supported(self) -> bool {
        match (self.schwarz_score, self.complexity) {
            (Some(_), Some(_)) => !self.llm_vote_only,
            _ => false,
        }
    }

    /// Return the higher-is-better Schwarz model-selection score.
    #[must_use]
    pub const fn schwarz_score(self) -> Option<f64> {
        self.schwarz_score
    }

    /// Complexity penalty (larger is worse) when statistically supported.
    #[must_use]
    pub const fn complexity(self) -> Option<f64> {
        self.complexity
    }

    /// Return whether the candidate is an LLM vote without statistical support.
    #[must_use]
    pub const fn is_llm_vote_only(self) -> bool {
        self.llm_vote_only
    }

    /// Return whether `self` Pareto-dominates `other` on score and complexity.
    #[must_use]
    pub fn dominates(self, other: Self) -> bool {
        let (Some(self_score), Some(self_complexity), Some(other_score), Some(other_complexity)) = (
            self.schwarz_score,
            self.complexity,
            other.schwarz_score,
            other.complexity,
        ) else {
            return false;
        };
        let no_worse = self_score >= other_score && self_complexity <= other_complexity;
        let strictly_better = self_score > other_score || self_complexity < other_complexity;
        no_worse && strictly_better
    }
}

#[cfg(test)]
mod tests {
    use super::ModelCandidate;
    use crate::ModelSelectionError;

    #[test]
    fn statistical_candidate_accessors_and_dominance_cover_branches() {
        let better = ModelCandidate::statistical(4, -10.0, 5.0).expect("better");
        let worse = ModelCandidate::statistical(8, -20.0, 9.0).expect("worse");
        assert_eq!(better.candidate_k(), 4);
        assert_eq!(better.schwarz_score(), Some(-10.0));
        assert_eq!(better.complexity(), Some(5.0));
        assert!(better.is_statistically_supported());
        assert!(!better.is_llm_vote_only());
        assert!(better.dominates(worse));
        assert!(!worse.dominates(better));
        assert!(!better.dominates(better));

        let llm = ModelCandidate::llm_vote_only(3).expect("valid llm candidate");
        assert!(llm.is_llm_vote_only());
        assert!(!llm.is_statistically_supported());
        assert!(!llm.dominates(better));
        assert!(!better.dominates(llm));
        assert_eq!(
            ModelCandidate::statistical(0, -1.0, 1.0),
            Err(ModelSelectionError::NonPositiveCandidateK)
        );
        assert_eq!(
            ModelCandidate::statistical(2, -1.0, -0.1),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert_eq!(
            ModelCandidate::statistical(2, f64::NAN, 1.0),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert_eq!(
            ModelCandidate::statistical(2, -1.0, f64::INFINITY),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert_eq!(
            ModelCandidate::llm_vote_only(1),
            Err(ModelSelectionError::NonPositiveCandidateK)
        );
    }
}
