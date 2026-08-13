//! Unicode canonical identity for leakage-safe corpus splits.
//!
//! UAX #15 treats NFC and NFD forms of the same abstract character sequence
//! as equivalent. Independent split membership of those forms would invent
//! independence between identical evidence.

use crate::{CorpusSplitError, LeakageLink, LeakageLinkKind};
use std::collections::BTreeSet;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

/// NFC identity of one non-empty UTF-8 document body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalTextIdentity {
    nfc_text: String,
}

impl CanonicalTextIdentity {
    /// Collect the NFC form of `text` as the document's Unicode identity.
    ///
    /// # Errors
    ///
    /// Returns [`CorpusSplitError::EmptyCanonicalText`] when `text` is empty.
    pub fn from_text(text: &str) -> Result<Self, CorpusSplitError> {
        if text.is_empty() {
            return Err(CorpusSplitError::EmptyCanonicalText);
        }
        Ok(Self {
            nfc_text: text.nfc().collect(),
        })
    }

    /// Borrow the NFC document body.
    #[must_use]
    pub fn nfc_text(&self) -> &str {
        &self.nfc_text
    }
}

/// Return whether two non-empty texts are canonically equivalent under NFC.
///
/// # Errors
///
/// Returns [`CorpusSplitError::EmptyCanonicalText`] when either body is empty.
pub fn texts_are_canonically_equivalent(left: &str, right: &str) -> Result<bool, CorpusSplitError> {
    if left.is_empty() || right.is_empty() {
        return Err(CorpusSplitError::EmptyCanonicalText);
    }
    Ok(left.nfc().eq(right.nfc()))
}

/// Emit undirected [`LeakageLinkKind::CanonicalEquivalent`] links for NFC/NFD pairs.
///
/// # Errors
///
/// Returns [`CorpusSplitError::DuplicateDocumentIdentity`] for repeated identities
/// or [`CorpusSplitError::EmptyCanonicalText`] for an empty body.
pub fn canonical_equivalence_links(
    documents: &[(Uuid, &str)],
) -> Result<Vec<LeakageLink>, CorpusSplitError> {
    let mut seen = BTreeSet::new();
    for (document_id, text) in documents {
        if !seen.insert(*document_id) {
            return Err(CorpusSplitError::DuplicateDocumentIdentity);
        }
        if text.is_empty() {
            return Err(CorpusSplitError::EmptyCanonicalText);
        }
    }

    let mut links = Vec::new();
    for left_index in 0..documents.len() {
        for right_index in (left_index + 1)..documents.len() {
            let (left_id, left_text) = documents[left_index];
            let (right_id, right_text) = documents[right_index];
            if left_text.nfc().eq(right_text.nfc()) {
                links.push(LeakageLink {
                    left: left_id,
                    right: right_id,
                    kind: LeakageLinkKind::CanonicalEquivalent,
                });
            }
        }
    }
    Ok(links)
}

#[cfg(test)]
mod tests {
    use super::{
        CanonicalTextIdentity, canonical_equivalence_links, texts_are_canonically_equivalent,
    };
    use crate::{CorpusSplitError, LeakageLinkKind};
    use uuid::Uuid;

    #[test]
    fn triple_equivalent_bodies_emit_three_links() {
        let first = Uuid::now_v7();
        let second = Uuid::now_v7();
        let third = Uuid::now_v7();
        let nfc = "\u{00e9}";
        let nfd = "e\u{0301}";
        let links = canonical_equivalence_links(&[(first, nfc), (second, nfd), (third, nfc)])
            .expect("triple");
        assert_eq!(links.len(), 3);
        assert!(
            links
                .iter()
                .all(|link| link.kind == LeakageLinkKind::CanonicalEquivalent)
        );
        assert!(texts_are_canonically_equivalent(nfc, nfc).expect("self"));
        assert_eq!(
            CanonicalTextIdentity::from_text("").expect_err("empty"),
            CorpusSplitError::EmptyCanonicalText
        );
    }
}
