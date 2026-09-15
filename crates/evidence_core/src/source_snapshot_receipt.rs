//! Versioned immutable source-snapshot evidence bindings.

use crate::{ContentDigest, EvidenceError, EvidenceId, SourceArtifact};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use temporal_core::AvailableTime;

/// Versioned wire schema for an Evidence-owned source-snapshot receipt.
pub const SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION: &str = "tepp.evidence.source_snapshot_receipt.v1";
/// Maximum canonical JSON size accepted for one source-snapshot receipt.
pub const SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT: usize = 16 * 1024;
const MAX_SNAPSHOT_IDENTIFIER_BYTES: usize = 256;

/// Immutable Evidence-owned binding for one exact source snapshot.
///
/// The receipt binds a stable receipt identity, the owning immutable
/// [`SourceArtifact`] identity, logical snapshot identity, exact source-content
/// SHA-256, and the authoritative time at which that source snapshot became
/// available. Downstream analysis may retain the canonical receipt digest as an
/// opaque binding. This object does not assert source ownership, signature,
/// authorization, or chain of custody, and it does not substitute for a numeric
/// estimator-payload digest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSnapshotReceiptV1 {
    schema_version: String,
    receipt_id: String,
    source_artifact_id: String,
    snapshot_id: String,
    source_snapshot_sha256: String,
    available_at: String,
}

impl SourceSnapshotReceiptV1 {
    /// Construct one receipt from an already validated immutable source artifact.
    ///
    /// The content digest is derived from the artifact rather than accepted as a
    /// free caller-supplied authority. Identity and digest remain separate so
    /// equal bytes ingested into distinct Evidence records do not collapse their
    /// provenance context.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidWirePayload`] when the snapshot identity
    /// is empty, mutable, noncanonical, or otherwise outside the bounded
    /// Evidence contract.
    pub fn from_source_artifact(
        receipt_id: EvidenceId,
        snapshot_id: impl Into<String>,
        source_artifact: &SourceArtifact,
        available_at: AvailableTime,
    ) -> Result<Self, EvidenceError> {
        let receipt = Self {
            schema_version: SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: receipt_id.to_string(),
            source_artifact_id: source_artifact.id().to_string(),
            snapshot_id: snapshot_id.into(),
            source_snapshot_sha256: source_artifact.content_digest().to_string(),
            available_at: available_at.to_rfc3339(),
        };
        receipt.validate()?;
        Ok(receipt)
    }

    /// Parse and validate one bounded canonical JSON receipt.
    ///
    /// Parsing validates identity/digest/clock shape and canonical encoding. It
    /// does not authenticate an arbitrary wire sender; consumers still require
    /// their owning repository/service authority before accepting a receipt as
    /// trusted Evidence state.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidWirePayload`] for malformed, duplicate,
    /// unknown, noncanonical, or oversized receipt JSON, and
    /// [`EvidenceError::UnsupportedWireVersion`] for another schema version.
    pub fn from_json(payload: &str) -> Result<Self, EvidenceError> {
        if payload.len() > SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT {
            return Err(EvidenceError::InvalidWirePayload);
        }
        let receipt: Self =
            serde_json::from_str(payload).map_err(|_| EvidenceError::InvalidWirePayload)?;
        receipt.validate()?;
        Ok(receipt)
    }

    /// Serialize canonical validated receipt JSON.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidWirePayload`] when the receipt cannot be
    /// validated, serialized, or kept inside the bounded wire envelope.
    pub fn to_json(&self) -> Result<String, EvidenceError> {
        self.validate()?;
        let payload = serde_json::to_string(self).map_err(|_| EvidenceError::InvalidWirePayload)?;
        if payload.len() > SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT {
            return Err(EvidenceError::InvalidWirePayload);
        }
        Ok(payload)
    }

    /// Return the SHA-256 of canonical receipt JSON for opaque downstream binding.
    ///
    /// # Errors
    ///
    /// Returns an evidence validation error when canonical serialization fails.
    pub fn binding_sha256(&self) -> Result<ContentDigest, EvidenceError> {
        self.to_json()
            .map(|payload| ContentDigest::sha256(payload.as_bytes()))
    }

    /// Return the canonical RFC 9562 UUIDv7 receipt identity.
    #[must_use]
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    /// Return the owning immutable source-artifact identity.
    #[must_use]
    pub fn source_artifact_id(&self) -> &str {
        &self.source_artifact_id
    }

    /// Return the immutable logical source-snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Return the canonical lowercase source-content SHA-256.
    #[must_use]
    pub fn source_snapshot_sha256(&self) -> &str {
        &self.source_snapshot_sha256
    }

    /// Return the canonical RFC 3339 availability clock.
    #[must_use]
    pub fn available_at(&self) -> &str {
        &self.available_at
    }

    fn validate(&self) -> Result<(), EvidenceError> {
        if self.schema_version != SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION {
            return Err(EvidenceError::UnsupportedWireVersion);
        }
        let parsed_receipt_id =
            EvidenceId::from_str(&self.receipt_id).map_err(|_| EvidenceError::InvalidWirePayload)?;
        let parsed_source_artifact_id = EvidenceId::from_str(&self.source_artifact_id)
            .map_err(|_| EvidenceError::InvalidWirePayload)?;
        if parsed_receipt_id.to_string() != self.receipt_id
            || parsed_source_artifact_id.to_string() != self.source_artifact_id
            || !valid_snapshot_id(&self.snapshot_id)
        {
            return Err(EvidenceError::InvalidWirePayload);
        }
        let digest = ContentDigest::from_str(&self.source_snapshot_sha256)
            .map_err(|_| EvidenceError::InvalidContentDigest)?;
        if digest.to_string() != self.source_snapshot_sha256 {
            return Err(EvidenceError::InvalidContentDigest);
        }
        let available = AvailableTime::parse_rfc3339(&self.available_at)
            .map_err(|_| EvidenceError::InvalidWirePayload)?;
        if available.to_rfc3339() != self.available_at {
            return Err(EvidenceError::InvalidWirePayload);
        }
        Ok(())
    }
}

fn valid_snapshot_id(value: &str) -> bool {
    if value.is_empty()
        || value.len() > MAX_SNAPSHOT_IDENTIFIER_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return false;
    }
    let normalized = value.to_ascii_lowercase();
    let numeric_alias = ["pr-", "pull-", "issue-"]
        .into_iter()
        .filter_map(|prefix| normalized.strip_prefix(prefix))
        .any(|suffix| !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit()));
    !matches!(
        normalized.as_str(),
        "latest" | "main" | "master" | "latest-release" | "release-latest"
    ) && !numeric_alias
}

#[cfg(test)]
mod tests {
    use super::{SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION, SourceSnapshotReceiptV1};
    use crate::{EvidenceError, EvidenceId, SourceArtifact};
    use std::str::FromStr;
    use temporal_core::AvailableTime;

    fn receipt() -> SourceSnapshotReceiptV1 {
        let source_artifact = SourceArtifact::from_bytes(b"canonical snapshot").expect("artifact");
        SourceSnapshotReceiptV1::from_source_artifact(
            EvidenceId::from_str("018f1f6b-7c2a-7abc-8def-0123456789ab").expect("uuidv7"),
            "snapshot-rubin-loading",
            &source_artifact,
            AvailableTime::parse_rfc3339("2026-07-31T23:59:59Z").expect("availability"),
        )
        .expect("receipt")
    }

    #[test]
    fn canonical_round_trip_preserves_binding() {
        let receipt = receipt();
        let json = receipt.to_json().expect("json");
        assert!(json.contains(SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION));
        assert_eq!(
            SourceSnapshotReceiptV1::from_json(&json),
            Ok(receipt.clone())
        );
        assert_eq!(
            receipt.binding_sha256().expect("binding").to_string().len(),
            64
        );
    }

    #[test]
    fn mutable_snapshot_aliases_fail_closed() {
        let id = EvidenceId::from_str("018f1f6b-7c2a-7abc-8def-0123456789ab").expect("uuidv7");
        let source_artifact = SourceArtifact::from_bytes(b"source").expect("artifact");
        let available =
            AvailableTime::parse_rfc3339("2026-07-31T23:59:59Z").expect("availability");
        for alias in ["main", "latest", "pr-123", "issue-42"] {
            assert_eq!(
                SourceSnapshotReceiptV1::from_source_artifact(
                    id,
                    alias,
                    &source_artifact,
                    available,
                ),
                Err(EvidenceError::InvalidWirePayload)
            );
        }
    }
}
