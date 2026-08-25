//! Versioned corpus-split leakage-audit wire contract.
//!
//! Buyers and modular consumers receive inclusion/exclusion counts, relation
//! co-partition identity, and a canonical `SHA-256` digest that binds to the
//! `corpus_split_manifest` row in migration `0003`. Source text is never
//! exported.

use crate::ApiError;
use crate::wire::{from_json, require_contract_version, require_nonempty, to_json};
use corpus_split::{
    CorpusDocument, CorpusSnapshot, LeakageLink, LeakageLinkKind, assert_no_group_leakage,
    build_connected_groups, cutoff_eligible,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use temporal_core::KnowledgeCutoff;
use uuid::Uuid;

/// Supported corpus-split leakage-audit contract version.
pub const CORPUS_SPLIT_MANIFEST_CONTRACT_VERSION: u16 = 1;

const SHA256_HEX_LENGTH: usize = 64;
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Train, validation, and test document identities for one split.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusSplitPartitions {
    /// Documents assigned to the training partition.
    pub train_document_ids: Vec<Uuid>,
    /// Documents assigned to the validation partition.
    pub validation_document_ids: Vec<Uuid>,
    /// Documents assigned to the test partition.
    pub test_document_ids: Vec<Uuid>,
}

/// Leakage-audit summary a consumer can verify without reading TEPP tables.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusSplitManifest {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Opaque split-manifest identity.
    pub manifest_id: String,
    /// Knowledge cutoff applied when the snapshot was materialised.
    pub knowledge_cutoff: String,
    /// Versioned split-policy identity.
    pub split_policy_version: String,
    /// Digest of relation-connected group membership.
    pub relation_component_digest: String,
    /// Digest of the training partition identities.
    pub train_partition_digest: String,
    /// Digest of the validation partition identities.
    pub validation_partition_digest: String,
    /// Digest of the test partition identities.
    pub test_partition_digest: String,
    /// Documents eligible at the cutoff and assigned to a partition.
    pub included_document_count: u64,
    /// Candidate documents unavailable at the cutoff.
    pub excluded_unavailable_at_cutoff_count: u64,
    /// Relation-connected groups in the eligible universe, including isolates.
    pub connected_group_count: u64,
    /// Distinct governed link kinds present among eligible documents.
    pub governed_link_kinds: Vec<String>,
    /// Canonical digest of the other fields; matches migration `0003`.
    pub split_manifest_digest_sha256: String,
}

impl CorpusSplitManifest {
    /// Build a validated manifest from a cutoff-filtered domain snapshot.
    ///
    /// Late-available candidates increment
    /// [`CorpusSplitManifest::excluded_unavailable_at_cutoff_count`] and never
    /// enter a partition. Linked eligible documents must share one partition.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when identities are empty, the
    /// eligible snapshot is empty, partitions leak, overlap, or omit an
    /// eligible document, or a digest cannot be bound.
    pub fn from_domain(
        manifest_id: impl Into<String>,
        knowledge_cutoff: &KnowledgeCutoff,
        split_policy_version: impl Into<String>,
        candidates: &[CorpusDocument],
        links: &[LeakageLink],
        partitions: &CorpusSplitPartitions,
    ) -> Result<Self, ApiError> {
        let manifest_id = manifest_id.into();
        let split_policy_version = split_policy_version.into();
        require_nonempty(&manifest_id)?;
        require_nonempty(&split_policy_version)?;

        let mut excluded = 0_u64;
        let mut snapshot = CorpusSnapshot::new();
        for document in candidates {
            if cutoff_eligible(&document.available_time, knowledge_cutoff) {
                snapshot
                    .insert_if_eligible(document.clone(), knowledge_cutoff)
                    .map_err(|_| ApiError::InvalidWirePayload)?;
            } else {
                excluded = excluded.saturating_add(1);
            }
        }
        if snapshot.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }

        let universe: Vec<Uuid> = snapshot.document_ids().collect();
        let assigned = assigned_partitions(partitions)?;
        if assigned.len() != universe.len()
            || universe
                .iter()
                .any(|identity| !assigned.contains_key(identity))
        {
            return Err(ApiError::InvalidWirePayload);
        }

        let groups = build_connected_groups(&universe, links);
        assert_no_group_leakage(&groups, &assigned).map_err(|_| ApiError::InvalidWirePayload)?;

        let mut kinds = BTreeSet::new();
        let eligible: BTreeSet<Uuid> = universe.iter().copied().collect();
        for link in links {
            if eligible.contains(&link.left) && eligible.contains(&link.right) {
                kinds.insert(link_kind_wire(link.kind).to_owned());
            }
        }

        let mut manifest = Self {
            contract_version: CORPUS_SPLIT_MANIFEST_CONTRACT_VERSION,
            manifest_id,
            knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
            split_policy_version,
            relation_component_digest: relation_component_digest(&groups),
            train_partition_digest: partition_digest(&partitions.train_document_ids),
            validation_partition_digest: partition_digest(&partitions.validation_document_ids),
            test_partition_digest: partition_digest(&partitions.test_document_ids),
            included_document_count: u64::try_from(snapshot.len())
                .map_err(|_| ApiError::InvalidWirePayload)?,
            excluded_unavailable_at_cutoff_count: excluded,
            connected_group_count: u64::try_from(groups.len())
                .map_err(|_| ApiError::InvalidWirePayload)?,
            governed_link_kinds: kinds.into_iter().collect(),
            split_manifest_digest_sha256: String::new(),
        };
        manifest.split_manifest_digest_sha256 = canonical_digest(&manifest);
        manifest.validate()?;
        Ok(manifest)
    }

    /// Parse and validate a JSON leakage-audit manifest.
    ///
    /// # Errors
    ///
    /// Returns wire, version, digest-mismatch, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        let manifest: Self = from_json(payload)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Serialize this manifest to JSON after validation.
    ///
    /// # Errors
    ///
    /// Returns field-validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json(self)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(
            self.contract_version,
            CORPUS_SPLIT_MANIFEST_CONTRACT_VERSION,
        )?;
        require_nonempty(&self.manifest_id)?;
        require_nonempty(&self.knowledge_cutoff)?;
        require_nonempty(&self.split_policy_version)?;
        require_sha256_hex(&self.relation_component_digest)?;
        require_sha256_hex(&self.train_partition_digest)?;
        require_sha256_hex(&self.validation_partition_digest)?;
        require_sha256_hex(&self.test_partition_digest)?;
        require_sha256_hex(&self.split_manifest_digest_sha256)?;
        for kind in &self.governed_link_kinds {
            if !matches!(
                kind.as_str(),
                "revision"
                    | "translation"
                    | "copied_variant"
                    | "same_episode"
                    | "canonical_equivalent"
            ) {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        if self
            .governed_link_kinds
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            return Err(ApiError::InvalidWirePayload);
        }
        if self.split_manifest_digest_sha256 != canonical_digest(self) {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }
}

fn assigned_partitions(partitions: &CorpusSplitPartitions) -> Result<BTreeMap<Uuid, u8>, ApiError> {
    let mut assigned = BTreeMap::new();
    for (code, identities) in [
        (0_u8, &partitions.train_document_ids),
        (1_u8, &partitions.validation_document_ids),
        (2_u8, &partitions.test_document_ids),
    ] {
        for identity in identities {
            if assigned.insert(*identity, code).is_some() {
                return Err(ApiError::InvalidWirePayload);
            }
        }
    }
    Ok(assigned)
}

fn link_kind_wire(kind: LeakageLinkKind) -> &'static str {
    match kind {
        LeakageLinkKind::Revision => "revision",
        LeakageLinkKind::Translation => "translation",
        LeakageLinkKind::CopiedVariant => "copied_variant",
        LeakageLinkKind::SameEpisode => "same_episode",
        LeakageLinkKind::CanonicalEquivalent => "canonical_equivalent",
    }
}

fn partition_digest(identities: &[Uuid]) -> String {
    let mut ordered = identities.to_vec();
    ordered.sort_unstable();
    let mut hasher = Sha256::new();
    for identity in ordered {
        hasher.update(identity.as_bytes());
    }
    hex_encode(&hasher.finalize())
}

fn relation_component_digest(groups: &[corpus_split::ConnectedGroup]) -> String {
    let mut hasher = Sha256::new();
    for group in groups {
        let mut members: Vec<Uuid> = group.members().iter().copied().collect();
        members.sort_unstable();
        for member in members {
            hasher.update(member.as_bytes());
        }
        hasher.update([0xff]);
    }
    hex_encode(&hasher.finalize())
}

fn canonical_digest(manifest: &CorpusSplitManifest) -> String {
    let mut hasher = Sha256::new();
    hasher.update(manifest.knowledge_cutoff.as_bytes());
    hasher.update([0]);
    hasher.update(manifest.split_policy_version.as_bytes());
    hasher.update([0]);
    hasher.update(manifest.relation_component_digest.as_bytes());
    hasher.update(manifest.train_partition_digest.as_bytes());
    hasher.update(manifest.validation_partition_digest.as_bytes());
    hasher.update(manifest.test_partition_digest.as_bytes());
    hasher.update(manifest.included_document_count.to_le_bytes());
    hasher.update(manifest.excluded_unavailable_at_cutoff_count.to_le_bytes());
    hasher.update(manifest.connected_group_count.to_le_bytes());
    for kind in &manifest.governed_link_kinds {
        hasher.update(kind.as_bytes());
        hasher.update([0]);
    }
    hex_encode(&hasher.finalize())
}

fn require_sha256_hex(value: &str) -> Result<(), ApiError> {
    if value.len() != SHA256_HEX_LENGTH
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{
        CORPUS_SPLIT_MANIFEST_CONTRACT_VERSION, CorpusSplitManifest, CorpusSplitPartitions,
        canonical_digest, hex_encode, link_kind_wire, require_sha256_hex,
    };
    use crate::ApiError;
    use corpus_split::{CorpusDocument, LeakageLink, LeakageLinkKind};
    use temporal_core::{AvailableTime, KnowledgeCutoff};
    use uuid::Uuid;

    fn document(id: u128, stamp: &str) -> CorpusDocument {
        CorpusDocument::new(
            Uuid::from_u128(id),
            AvailableTime::parse_rfc3339(stamp).expect("available"),
        )
    }

    fn cutoff() -> KnowledgeCutoff {
        KnowledgeCutoff::parse_rfc3339("2026-03-01T00:00:00Z").expect("cutoff")
    }

    fn partitions(train: &[u128], validation: &[u128], test: &[u128]) -> CorpusSplitPartitions {
        CorpusSplitPartitions {
            train_document_ids: train.iter().copied().map(Uuid::from_u128).collect(),
            validation_document_ids: validation.iter().copied().map(Uuid::from_u128).collect(),
            test_document_ids: test.iter().copied().map(Uuid::from_u128).collect(),
        }
    }

    fn valid_manifest() -> CorpusSplitManifest {
        CorpusSplitManifest::from_domain(
            "split-unit",
            &cutoff(),
            "relation-aware-v1",
            &[
                document(1, "2026-01-01T00:00:00Z"),
                document(2, "2026-01-02T00:00:00Z"),
                document(3, "2026-08-01T00:00:00Z"),
            ],
            &[LeakageLink {
                left: Uuid::from_u128(1),
                right: Uuid::from_u128(2),
                kind: LeakageLinkKind::Revision,
            }],
            &partitions(&[1, 2], &[], &[]),
        )
        .expect("valid")
    }

    fn assert_invalid_domain(
        manifest_id: &str,
        policy: &str,
        candidates: &[CorpusDocument],
        partitions: &CorpusSplitPartitions,
    ) {
        assert_eq!(
            CorpusSplitManifest::from_domain(
                manifest_id,
                &cutoff(),
                policy,
                candidates,
                &[],
                partitions,
            ),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn domain_adapter_rejects_empty_universe_and_broken_partitions() {
        let early = document(1, "2026-01-01T00:00:00Z");
        let late = document(1, "2026-08-01T00:00:00Z");
        let covered = partitions(&[1], &[], &[]);
        let early_only = std::slice::from_ref(&early);
        assert_invalid_domain(" ", "policy", early_only, &covered);
        assert_invalid_domain("id", "", early_only, &covered);
        assert_invalid_domain(
            "id",
            "policy",
            std::slice::from_ref(&late),
            &partitions(&[], &[], &[]),
        );
        assert_invalid_domain(
            "id",
            "policy",
            &[early.clone(), document(1, "2026-01-02T00:00:00Z")],
            &covered,
        );
        assert_invalid_domain("id", "policy", early_only, &partitions(&[1, 1], &[], &[]));
        assert_invalid_domain("id", "policy", early_only, &partitions(&[1], &[1], &[]));
        assert_invalid_domain("id", "policy", early_only, &partitions(&[], &[], &[]));
        assert_invalid_domain("id", "policy", early_only, &partitions(&[9], &[], &[]));
    }

    #[test]
    fn validation_rejects_corrupt_manifest_fields() {
        let mut unsupported = valid_manifest();
        unsupported.contract_version = 2;
        assert_eq!(
            unsupported.to_json(),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            CorpusSplitManifest::from_json(r#"{"contract_version":2}"#),
            Err(ApiError::InvalidWirePayload)
        );

        let mut empty_id = valid_manifest();
        empty_id.manifest_id.clear();
        assert_eq!(empty_id.to_json(), Err(ApiError::InvalidWirePayload));
        let mut empty_cutoff = valid_manifest();
        empty_cutoff.knowledge_cutoff.clear();
        assert_eq!(empty_cutoff.to_json(), Err(ApiError::InvalidWirePayload));
        let mut empty_policy = valid_manifest();
        empty_policy.split_policy_version.clear();
        assert_eq!(empty_policy.to_json(), Err(ApiError::InvalidWirePayload));
        let mut bad_relation = valid_manifest();
        bad_relation.relation_component_digest = "short".into();
        assert_eq!(bad_relation.to_json(), Err(ApiError::InvalidWirePayload));
        let mut bad_train = valid_manifest();
        bad_train.train_partition_digest = "short".into();
        assert_eq!(bad_train.to_json(), Err(ApiError::InvalidWirePayload));
        let mut bad_validation = valid_manifest();
        bad_validation.validation_partition_digest = "short".into();
        assert_eq!(bad_validation.to_json(), Err(ApiError::InvalidWirePayload));
        let mut bad_test = valid_manifest();
        bad_test.test_partition_digest = "short".into();
        assert_eq!(bad_test.to_json(), Err(ApiError::InvalidWirePayload));

        let mut bad_kind = valid_manifest();
        bad_kind.governed_link_kinds = vec!["tfidf".into()];
        bad_kind.split_manifest_digest_sha256 = canonical_digest(&bad_kind);
        assert_eq!(bad_kind.to_json(), Err(ApiError::InvalidWirePayload));

        let mut noncanonical_kinds = valid_manifest();
        noncanonical_kinds.governed_link_kinds = vec!["translation".into(), "revision".into()];
        noncanonical_kinds.split_manifest_digest_sha256 = canonical_digest(&noncanonical_kinds);
        let payload = serde_json::to_string(&noncanonical_kinds).expect("payload");
        assert_eq!(
            CorpusSplitManifest::from_json(&payload),
            Err(ApiError::InvalidWirePayload)
        );

        noncanonical_kinds.governed_link_kinds = vec!["revision".into(), "revision".into()];
        noncanonical_kinds.split_manifest_digest_sha256 = canonical_digest(&noncanonical_kinds);
        let payload = serde_json::to_string(&noncanonical_kinds).expect("duplicate payload");
        assert_eq!(
            CorpusSplitManifest::from_json(&payload),
            Err(ApiError::InvalidWirePayload)
        );

        assert_eq!(
            require_sha256_hex("short"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            require_sha256_hex(&"AB".repeat(32)),
            Err(ApiError::InvalidWirePayload)
        );
        require_sha256_hex(&"ab".repeat(32)).expect("hex");
        assert_eq!(hex_encode(&[0x0f, 0xa0]), "0fa0");
        assert_eq!(link_kind_wire(LeakageLinkKind::Revision), "revision");
        assert_eq!(link_kind_wire(LeakageLinkKind::Translation), "translation");
        assert_eq!(
            link_kind_wire(LeakageLinkKind::CopiedVariant),
            "copied_variant"
        );
        assert_eq!(link_kind_wire(LeakageLinkKind::SameEpisode), "same_episode");
        assert_eq!(
            link_kind_wire(LeakageLinkKind::CanonicalEquivalent),
            "canonical_equivalent"
        );
    }

    #[test]
    fn all_governed_link_kinds_are_exported_when_eligible() {
        let links = [
            LeakageLink {
                left: Uuid::from_u128(1),
                right: Uuid::from_u128(2),
                kind: LeakageLinkKind::Revision,
            },
            LeakageLink {
                left: Uuid::from_u128(2),
                right: Uuid::from_u128(3),
                kind: LeakageLinkKind::CopiedVariant,
            },
            LeakageLink {
                left: Uuid::from_u128(3),
                right: Uuid::from_u128(4),
                kind: LeakageLinkKind::SameEpisode,
            },
            LeakageLink {
                left: Uuid::from_u128(4),
                right: Uuid::from_u128(1),
                kind: LeakageLinkKind::Translation,
            },
            LeakageLink {
                left: Uuid::from_u128(1),
                right: Uuid::from_u128(4),
                kind: LeakageLinkKind::CanonicalEquivalent,
            },
            LeakageLink {
                left: Uuid::from_u128(9),
                right: Uuid::from_u128(1),
                kind: LeakageLinkKind::Revision,
            },
        ];
        let manifest = CorpusSplitManifest::from_domain(
            "kinds",
            &cutoff(),
            "relation-aware-v1",
            &[
                document(1, "2026-01-01T00:00:00Z"),
                document(2, "2026-01-01T00:00:00Z"),
                document(3, "2026-01-01T00:00:00Z"),
                document(4, "2026-01-01T00:00:00Z"),
                document(9, "2026-08-01T00:00:00Z"),
            ],
            &links,
            &partitions(&[1, 2, 3, 4], &[], &[]),
        )
        .expect("kinds");
        assert_eq!(
            manifest.governed_link_kinds,
            vec![
                "canonical_equivalent".to_owned(),
                "copied_variant".to_owned(),
                "revision".to_owned(),
                "same_episode".to_owned(),
                "translation".to_owned()
            ]
        );
        assert_eq!(manifest.excluded_unavailable_at_cutoff_count, 1);
        assert_eq!(
            manifest.contract_version,
            CORPUS_SPLIT_MANIFEST_CONTRACT_VERSION
        );
    }
}
