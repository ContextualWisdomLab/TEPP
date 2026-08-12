//! Reproducible truth manifests for recovery studies.

use crate::SimulationError;
use crate::document_process::SimulatedDocument;
use crate::latent_event::LatentEvent;
use crate::relation_process::{ObservedRelation, TrueRelation};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// Immutable known-truth corpus bound to an explicit seed and content digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TruthManifest {
    seed: u64,
    config_digest: String,
    content_digest: String,
    events: Vec<LatentEvent>,
    documents: Vec<SimulatedDocument>,
    true_relations: Vec<TrueRelation>,
    observed_relations: Vec<ObservedRelation>,
}

impl TruthManifest {
    /// Construct a truth manifest and compute its content digest.
    #[must_use]
    pub fn new(
        seed: u64,
        config_digest: String,
        events: Vec<LatentEvent>,
        documents: Vec<SimulatedDocument>,
        true_relations: Vec<TrueRelation>,
        observed_relations: Vec<ObservedRelation>,
    ) -> Self {
        let mut manifest = Self {
            seed,
            config_digest,
            content_digest: String::new(),
            events,
            documents,
            true_relations,
            observed_relations,
        };
        manifest.content_digest = manifest.compute_content_digest();
        manifest
    }

    /// Explicit scenario seed.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Digest of the configuration used to generate the corpus.
    #[must_use]
    pub fn config_digest(&self) -> &str {
        &self.config_digest
    }

    /// Digest of the generated truth rows.
    #[must_use]
    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }

    /// Latent events in ordinal order.
    #[must_use]
    pub fn events(&self) -> &[LatentEvent] {
        &self.events
    }

    /// Simulated documents.
    #[must_use]
    pub fn documents(&self) -> &[SimulatedDocument] {
        &self.documents
    }

    /// True generative relations.
    #[must_use]
    pub fn true_relations(&self) -> &[TrueRelation] {
        &self.true_relations
    }

    /// Observed relations after noise.
    #[must_use]
    pub fn observed_relations(&self) -> &[ObservedRelation] {
        &self.observed_relations
    }

    /// Number of latent events.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Number of documents.
    #[must_use]
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Verify scientific invariants required of every truth corpus.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::ManifestInvariantViolation`] when temporal
    /// order, membership multiplicity, parent linkage, or digest integrity fails.
    pub fn verify_invariants(&self) -> Result<(), SimulationError> {
        if self.content_digest != self.compute_content_digest() {
            return Err(SimulationError::ManifestInvariantViolation);
        }
        if self.events.is_empty() {
            return Err(SimulationError::ManifestInvariantViolation);
        }
        if self.documents.is_empty() {
            return Err(SimulationError::ManifestInvariantViolation);
        }
        for window in self.events.windows(2) {
            if window[0].ordinal() >= window[1].ordinal() {
                return Err(SimulationError::ManifestInvariantViolation);
            }
            if window[0].event_time().instant() > window[1].event_time().instant() {
                return Err(SimulationError::ManifestInvariantViolation);
            }
        }
        let event_ids: BTreeSet<_> = self.events.iter().map(LatentEvent::event_id).collect();
        let document_ids: BTreeSet<_> = self
            .documents
            .iter()
            .map(SimulatedDocument::document_id)
            .collect();
        if document_ids.len() != self.documents.len() {
            return Err(SimulationError::ManifestInvariantViolation);
        }
        for document in &self.documents {
            if !event_ids.contains(&document.event_id()) {
                return Err(SimulationError::ManifestInvariantViolation);
            }
            if document.memberships().is_empty() {
                return Err(SimulationError::ManifestInvariantViolation);
            }
            let roles: BTreeSet<_> = document
                .memberships()
                .iter()
                .map(crate::document_process::SimulatedMembership::role_label)
                .collect();
            if roles.len() != document.memberships().len() {
                return Err(SimulationError::ManifestInvariantViolation);
            }
            if let Some(parent) = document.parent_document_id()
                && !document_ids.contains(&parent)
            {
                return Err(SimulationError::ManifestInvariantViolation);
            }
        }
        Ok(())
    }

    #[cfg(test)]
    fn corrupt_content_digest(&mut self) {
        self.content_digest = "00".into();
    }

    fn compute_content_digest(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.seed.to_le_bytes());
        hasher.update(self.config_digest.as_bytes());
        for event in &self.events {
            hasher.update(event.event_id().as_bytes());
            hasher.update(event.ordinal().to_le_bytes());
            hasher.update(event.event_time().to_rfc3339().as_bytes());
            hasher.update(event.state().wire_name().as_bytes());
        }
        for document in &self.documents {
            hasher.update(document.document_id().as_bytes());
            hasher.update(document.event_id().as_bytes());
            hasher.update(document.document_time().to_rfc3339().as_bytes());
            hasher.update(document.available_time().to_rfc3339().as_bytes());
            hasher.update(document.method_effect().wire_name().as_bytes());
            if let Some(parent) = document.parent_document_id() {
                hasher.update(parent.as_bytes());
            }
            if let Some(observed) = document.observed_event_time() {
                hasher.update(observed.to_rfc3339().as_bytes());
            } else {
                hasher.update(b"missing");
            }
            for membership in document.memberships() {
                hasher.update(membership.group_id().as_bytes());
                hasher.update(membership.role_label().as_bytes());
                hasher.update(membership.weight_bps().to_le_bytes());
            }
        }
        for relation in &self.true_relations {
            hasher.update(relation.relation_id().as_bytes());
            hasher.update(relation.kind().wire_name().as_bytes());
            hasher.update(relation.source_id().as_bytes());
            hasher.update(relation.target_id().as_bytes());
        }
        for relation in &self.observed_relations {
            hasher.update(relation.relation_id().as_bytes());
            hasher.update(relation.kind().wire_name().as_bytes());
            hasher.update(relation.source_id().as_bytes());
            hasher.update(relation.target_id().as_bytes());
            hasher.update([u8::from(relation.is_true_positive())]);
        }
        hex_encode(&hasher.finalize())
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Digest a configuration fingerprint string.
#[must_use]
pub fn digest_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_encode(&hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{TruthManifest, digest_bytes};
    use crate::SimulationError;
    use crate::document_process::{DocumentMethodEffect, SimulatedDocument, SimulatedMembership};
    use crate::latent_event::{LatentEvent, LatentEventState};
    use crate::relation_process::{ObservedRelation, SimulatedRelationKind, TrueRelation};
    use temporal_core::{AvailableTime, DocumentTime, EventTime};
    use uuid::Uuid;

    fn event(id: u128, stamp: &str, ordinal: u32) -> LatentEvent {
        LatentEvent::new(
            Uuid::from_u128(id),
            EventTime::parse_rfc3339(stamp).expect("event"),
            ordinal,
            LatentEventState::Occurred,
        )
    }

    fn original_doc(
        doc_id: u128,
        event_id: u128,
        memberships: Vec<SimulatedMembership>,
        observed: Option<EventTime>,
    ) -> SimulatedDocument {
        SimulatedDocument::new(
            Uuid::from_u128(doc_id),
            Uuid::from_u128(event_id),
            DocumentTime::parse_rfc3339("2026-01-03T00:00:00Z").expect("d"),
            AvailableTime::parse_rfc3339("2026-01-04T00:00:00Z").expect("a"),
            DocumentMethodEffect::Original,
            None,
            observed,
            memberships,
        )
        .expect("doc")
    }

    fn sample_manifest() -> TruthManifest {
        let event_time = EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("e");
        let events = vec![
            event(1, "2026-01-01T00:00:00Z", 0),
            event(2, "2026-01-02T00:00:00Z", 1),
        ];
        let documents = vec![original_doc(
            3,
            1,
            vec![SimulatedMembership::new(Uuid::from_u128(9), "author", 1)],
            Some(event_time),
        )];
        let true_relations = vec![TrueRelation::new(
            Uuid::from_u128(4),
            SimulatedRelationKind::TransitionsTo,
            Uuid::from_u128(1),
            Uuid::from_u128(2),
        )];
        let observed_relations = vec![ObservedRelation::new(
            Uuid::from_u128(4),
            SimulatedRelationKind::TransitionsTo,
            Uuid::from_u128(1),
            Uuid::from_u128(2),
            true,
        )];
        TruthManifest::new(
            7,
            digest_bytes(b"cfg"),
            events,
            documents,
            true_relations,
            observed_relations,
        )
    }

    #[test]
    fn valid_manifest_passes_and_exposes_counts() {
        let manifest = sample_manifest();
        manifest.verify_invariants().expect("ok");
        assert_eq!(manifest.seed(), 7);
        assert!(!manifest.config_digest().is_empty());
        assert!(!manifest.content_digest().is_empty());
        assert_eq!(manifest.event_count(), 2);
        assert_eq!(manifest.document_count(), 1);
        assert_eq!(manifest.events().len(), 2);
        assert_eq!(manifest.documents().len(), 1);
        assert_eq!(manifest.true_relations().len(), 1);
        assert_eq!(manifest.observed_relations().len(), 1);
        assert_eq!(digest_bytes(b"a").len(), 64);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn invariant_failures_cover_each_contract() {
        let mut corrupted = sample_manifest();
        corrupted.corrupt_content_digest();
        assert_eq!(
            corrupted.verify_invariants(),
            Err(SimulationError::ManifestInvariantViolation)
        );

        let empty_events = TruthManifest::new(
            1,
            "c".into(),
            Vec::new(),
            sample_manifest().documents().to_vec(),
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            empty_events.verify_invariants(),
            Err(SimulationError::ManifestInvariantViolation)
        );

        let empty_docs = TruthManifest::new(
            1,
            "c".into(),
            sample_manifest().events().to_vec(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            empty_docs.verify_invariants(),
            Err(SimulationError::ManifestInvariantViolation)
        );

        let bad_ordinal = TruthManifest::new(
            1,
            "c".into(),
            vec![
                event(1, "2026-01-01T00:00:00Z", 1),
                event(2, "2026-01-02T00:00:00Z", 0),
            ],
            sample_manifest().documents().to_vec(),
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            bad_ordinal.verify_invariants(),
            Err(SimulationError::ManifestInvariantViolation)
        );

        let bad_time_order = TruthManifest::new(
            1,
            "c".into(),
            vec![
                event(1, "2026-01-02T00:00:00Z", 0),
                event(2, "2026-01-01T00:00:00Z", 1),
            ],
            sample_manifest().documents().to_vec(),
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            bad_time_order.verify_invariants(),
            Err(SimulationError::ManifestInvariantViolation)
        );

        let doc = original_doc(
            3,
            1,
            vec![SimulatedMembership::new(Uuid::from_u128(9), "author", 1)],
            None,
        );
        let duplicate_docs = TruthManifest::new(
            1,
            "c".into(),
            vec![event(1, "2026-01-01T00:00:00Z", 0)],
            vec![doc.clone(), doc],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            duplicate_docs.verify_invariants(),
            Err(SimulationError::ManifestInvariantViolation)
        );

        let unknown_event = TruthManifest::new(
            1,
            "c".into(),
            vec![event(1, "2026-01-01T00:00:00Z", 0)],
            vec![original_doc(
                3,
                99,
                vec![SimulatedMembership::new(Uuid::from_u128(9), "author", 1)],
                None,
            )],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            unknown_event.verify_invariants(),
            Err(SimulationError::ManifestInvariantViolation)
        );

        let empty_memberships = TruthManifest::new(
            1,
            "c".into(),
            vec![event(1, "2026-01-01T00:00:00Z", 0)],
            vec![original_doc(3, 1, Vec::new(), None)],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            empty_memberships.verify_invariants(),
            Err(SimulationError::ManifestInvariantViolation)
        );

        let duplicate_roles = TruthManifest::new(
            1,
            "c".into(),
            vec![event(1, "2026-01-01T00:00:00Z", 0)],
            vec![original_doc(
                3,
                1,
                vec![
                    SimulatedMembership::new(Uuid::from_u128(9), "author", 1),
                    SimulatedMembership::new(Uuid::from_u128(10), "author", 1),
                ],
                None,
            )],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            duplicate_roles.verify_invariants(),
            Err(SimulationError::ManifestInvariantViolation)
        );

        let orphan_parent = TruthManifest::new(
            1,
            "c".into(),
            vec![event(1, "2026-01-01T00:00:00Z", 0)],
            vec![
                SimulatedDocument::new(
                    Uuid::from_u128(3),
                    Uuid::from_u128(1),
                    DocumentTime::parse_rfc3339("2026-01-03T00:00:00Z").expect("d"),
                    AvailableTime::parse_rfc3339("2026-01-04T00:00:00Z").expect("a"),
                    DocumentMethodEffect::Revision,
                    Some(Uuid::from_u128(999)),
                    None,
                    vec![SimulatedMembership::new(Uuid::from_u128(9), "author", 1)],
                )
                .expect("revision"),
            ],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            orphan_parent.verify_invariants(),
            Err(SimulationError::ManifestInvariantViolation)
        );
    }
}
