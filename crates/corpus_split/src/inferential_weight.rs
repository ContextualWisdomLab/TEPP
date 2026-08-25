//! Retrieval scores and stopword deletion are not inferential split weights.

use crate::CorpusSplitError;

/// Proposed document or term scoring identity for a split or estimator input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WeightingScheme {
    /// Kish / group-normalized observation weights.
    GroupNormalizedEss,
    /// Uniform observation weights.
    Uniform,
    /// TF-IDF retrieval ranking score.
    TfIdf,
    /// BM25 retrieval ranking score.
    Bm25,
}

impl WeightingScheme {
    /// Return whether this scheme may enter a statistical estimator as a weight.
    #[must_use]
    pub const fn is_inferential_weight(self) -> bool {
        matches!(self, Self::GroupNormalizedEss | Self::Uniform)
    }

    /// Return the stable wire name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::GroupNormalizedEss => "group_normalized_ess",
            Self::Uniform => "uniform",
            Self::TfIdf => "tf_idf",
            Self::Bm25 => "bm25",
        }
    }
}

/// Proposed token-deletion rule applied before estimation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TokenDeletionRule {
    /// Keep tokens and model template, section, copied, and style as method structure.
    PreserveAndModelBackground,
    /// Delete tokens that appear on a global stopword list.
    GlobalStopwordList,
}

impl TokenDeletionRule {
    /// Return whether this rule is allowed as the default preprocessing policy.
    #[must_use]
    pub const fn is_default_allowed(self) -> bool {
        matches!(self, Self::PreserveAndModelBackground)
    }

    /// Return the stable wire name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::PreserveAndModelBackground => "preserve_and_model_background",
            Self::GlobalStopwordList => "global_stopword_list",
        }
    }
}

/// Refuse TF-IDF and BM25 as inferential estimator weights.
///
/// # Errors
///
/// Returns [`CorpusSplitError::InferentialRetrievalWeight`] unless `scheme` is
/// [`WeightingScheme::GroupNormalizedEss`] or [`WeightingScheme::Uniform`].
pub fn refuse_inferential_retrieval_weight(
    scheme: WeightingScheme,
) -> Result<(), CorpusSplitError> {
    if scheme.is_inferential_weight() {
        Ok(())
    } else {
        Err(CorpusSplitError::InferentialRetrievalWeight)
    }
}

/// Refuse global stopword deletion as the default preprocessing rule.
///
/// # Errors
///
/// Returns [`CorpusSplitError::DefaultStopwordDeletion`] unless `rule` is
/// [`TokenDeletionRule::PreserveAndModelBackground`].
pub fn refuse_default_stopword_deletion(rule: TokenDeletionRule) -> Result<(), CorpusSplitError> {
    if rule.is_default_allowed() {
        Ok(())
    } else {
        Err(CorpusSplitError::DefaultStopwordDeletion)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TokenDeletionRule, WeightingScheme, refuse_default_stopword_deletion,
        refuse_inferential_retrieval_weight,
    };
    use crate::CorpusSplitError;

    #[test]
    fn predicates_export_stable_wire_names_and_gates() {
        assert!(WeightingScheme::GroupNormalizedEss.is_inferential_weight());
        assert!(WeightingScheme::Uniform.is_inferential_weight());
        assert!(!WeightingScheme::TfIdf.is_inferential_weight());
        assert!(!WeightingScheme::Bm25.is_inferential_weight());
        assert_eq!(
            WeightingScheme::GroupNormalizedEss.wire_name(),
            "group_normalized_ess"
        );
        assert_eq!(WeightingScheme::Uniform.wire_name(), "uniform");
        assert_eq!(WeightingScheme::TfIdf.wire_name(), "tf_idf");
        assert_eq!(WeightingScheme::Bm25.wire_name(), "bm25");
        refuse_inferential_retrieval_weight(WeightingScheme::GroupNormalizedEss).expect("ess");
        refuse_inferential_retrieval_weight(WeightingScheme::Uniform).expect("uniform");
        assert_eq!(
            refuse_inferential_retrieval_weight(WeightingScheme::TfIdf),
            Err(CorpusSplitError::InferentialRetrievalWeight)
        );
        assert_eq!(
            refuse_inferential_retrieval_weight(WeightingScheme::Bm25),
            Err(CorpusSplitError::InferentialRetrievalWeight)
        );

        assert!(TokenDeletionRule::PreserveAndModelBackground.is_default_allowed());
        assert!(!TokenDeletionRule::GlobalStopwordList.is_default_allowed());
        assert_eq!(
            TokenDeletionRule::PreserveAndModelBackground.wire_name(),
            "preserve_and_model_background"
        );
        assert_eq!(
            TokenDeletionRule::GlobalStopwordList.wire_name(),
            "global_stopword_list"
        );
        refuse_default_stopword_deletion(TokenDeletionRule::PreserveAndModelBackground)
            .expect("preserve");
        assert_eq!(
            refuse_default_stopword_deletion(TokenDeletionRule::GlobalStopwordList),
            Err(CorpusSplitError::DefaultStopwordDeletion)
        );
    }
}
