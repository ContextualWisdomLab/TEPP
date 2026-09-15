//! Versioned immutable source-snapshot evidence bindings.

use crate::{ContentDigest, EvidenceError, EvidenceId, SourceObservation};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use temporal_core::{AvailableTime, SystemTime};

/// Versioned wire schema for an Evidence-owned source-snapshot receipt.
pub const SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION: &str = "tepp.evidence.source_snapshot_receipt.v1";
/// Maximum canonical JSON size accepted for one source-snapshot receipt.
pub const SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT: usize = 16 * 1024;
const MAX_SNAPSHOT_IDENTIFIER_BYTES: usize = 256;

/// Immutable Evidence-owned binding for one exact source snapshot.
///
/// The receipt binds a stable receipt identity, the owning immutable source
/// artifact identity, logical snapshot identity, exact source-content SHA-256,
/// and the Evidence-owned source-ingress clocks from [`SourceObservation`].
/// Downstream analysis may retain the canonical receipt digest as an opaque
/// binding. This object does not assert source ownership, signature,
/// authorization, external authenticity, or chain of custody, and it does not
/// substitute for a numeric estimator-payload digest.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSnapshotReceiptV1 {
    schema_version: String,
    receipt_id: String,
    source_artifact_id: String,
    snapshot_id: String,
    source_snapshot_sha256: String,
    system_observed_at: String,
    available_at: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceSnapshotReceiptWireV1 {
    schema_version: String,
    receipt_id: String,
    source_artifact_id: String,
    snapshot_id: String,
    source_snapshot_sha256: String,
    system_observed_at: String,
    available_at: String,
}

impl SourceSnapshotReceiptV1 {
    /// Construct one receipt from an Evidence-owned source observation.
    ///
    /// Evidence mints the receipt identity internally. Source-artifact identity,
    /// source-content digest, system observation time, and availability time are
    /// copied from the owner-controlled observation rather than accepted as free
    /// constructor arguments. In this schema version, Evidence ingress makes the
    /// source available immediately, so the two nominal clocks must represent
    /// the same absolute instant.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidWirePayload`] when the snapshot identity
    /// is empty, mutable, noncanonical, or otherwise outside the bounded
    /// Evidence contract.
    pub fn from_source_observation(
        snapshot_id: impl Into<String>,
        observation: &SourceObservation,
    ) -> Result<Self, EvidenceError> {
        let receipt = Self {
            schema_version: SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: EvidenceId::new().to_string(),
            source_artifact_id: observation.source_artifact_id().to_string(),
            snapshot_id: snapshot_id.into(),
            source_snapshot_sha256: observation.source_snapshot_sha256().to_string(),
            system_observed_at: observation.system_observed_at().to_rfc3339(),
            available_at: observation.available_at().to_rfc3339(),
        };
        receipt.validate()?;
        Ok(receipt)
    }

    /// Parse and validate one bounded canonical JSON receipt.
    ///
    /// Parsing validates identity/digest/clock shape and canonical encoding. It
    /// does not authenticate an arbitrary wire sender; consumers still require
    /// their owning repository/service authority before accepting a parsed
    /// receipt as trusted Evidence state.
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
        let wire: SourceSnapshotReceiptWireV1 =
            serde_json::from_str(payload).map_err(|_| EvidenceError::InvalidWirePayload)?;
        let receipt = Self {
            schema_version: wire.schema_version,
            receipt_id: wire.receipt_id,
            source_artifact_id: wire.source_artifact_id,
            snapshot_id: wire.snapshot_id,
            source_snapshot_sha256: wire.source_snapshot_sha256,
            system_observed_at: wire.system_observed_at,
            available_at: wire.available_at,
        };
        receipt.validate()?;
        if serialize_bounded_receipt(&receipt)? != payload {
            return Err(EvidenceError::InvalidWirePayload);
        }
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
        serialize_bounded_receipt(self)
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

    /// Return the canonical RFC 3339 system clock for source observation.
    #[must_use]
    pub fn system_observed_at(&self) -> &str {
        &self.system_observed_at
    }

    /// Return the canonical RFC 3339 availability clock derived at Evidence ingress.
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
        let system_observed = SystemTime::parse_rfc3339(&self.system_observed_at)
            .map_err(|_| EvidenceError::InvalidWirePayload)?;
        let available = AvailableTime::parse_rfc3339(&self.available_at)
            .map_err(|_| EvidenceError::InvalidWirePayload)?;
        if system_observed.to_rfc3339() != self.system_observed_at
            || available.to_rfc3339() != self.available_at
            || system_observed.instant() != available.instant()
        {
            return Err(EvidenceError::InvalidWirePayload);
        }
        Ok(())
    }
}

fn serialize_bounded_receipt<T: Serialize>(value: &T) -> Result<String, EvidenceError> {
    let payload = serde_json::to_string(value).map_err(|_| EvidenceError::InvalidWirePayload)?;
    if payload.len() > SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT {
        return Err(EvidenceError::InvalidWirePayload);
    }
    Ok(payload)
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
    use super::{
        MAX_SNAPSHOT_IDENTIFIER_BYTES, SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT,
        SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION, SourceSnapshotReceiptV1,
        serialize_bounded_receipt, valid_snapshot_id,
    };
    use crate::{EvidenceError, SourceArtifact, SourceObservation};
    use serde::Serialize;
    use serde::ser::Serializer;

    struct SerializationFailure;

    impl Serialize for SerializationFailure {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom("intentional test failure"))
        }
    }

    fn receipt() -> SourceSnapshotReceiptV1 {
        let source_artifact = SourceArtifact::from_bytes(b"canonical snapshot").expect("artifact");
        let observation = SourceObservation::observe(&source_artifact).expect("observation");
        SourceSnapshotReceiptV1::from_source_observation(
            "snapshot-rubin-loading",
            &observation,
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
    fn mutable_and_malformed_snapshot_identifiers_fail_closed() {
        let source_artifact = SourceArtifact::from_bytes(b"source").expect("artifact");
        let observation = SourceObservation::observe(&source_artifact).expect("observation");
        for alias in [
            "main",
            "master",
            "latest",
            "latest-release",
            "release-latest",
            "pr-123",
            "pull-123",
            "issue-42",
        ] {
            assert_eq!(
                SourceSnapshotReceiptV1::from_source_observation(alias, &observation),
                Err(EvidenceError::InvalidWirePayload)
            );
        }
        assert!(!valid_snapshot_id(""));
        assert!(!valid_snapshot_id("snapshot/branch"));
        assert!(!valid_snapshot_id(&"a".repeat(MAX_SNAPSHOT_IDENTIFIER_BYTES + 1)));
        assert!(valid_snapshot_id("snapshot:v1_2026.09-15"));
    }

    #[test]
    fn private_validation_branches_refuse_noncanonical_fields() {
        let canonical = receipt();
        let mut unsupported = canonical.clone();
        unsupported.schema_version = "tepp.evidence.source_snapshot_receipt.v2".into();
        assert_eq!(
            unsupported.to_json(),
            Err(EvidenceError::UnsupportedWireVersion)
        );

        let mut bad_receipt_id = canonical.clone();
        bad_receipt_id.receipt_id = "not-a-uuid".into();
        assert_eq!(
            bad_receipt_id.to_json(),
            Err(EvidenceError::InvalidWirePayload)
        );

        let mut uppercase_receipt_id = canonical.clone();
        uppercase_receipt_id.receipt_id = uppercase_receipt_id.receipt_id.to_ascii_uppercase();
        assert_eq!(
            uppercase_receipt_id.to_json(),
            Err(EvidenceError::InvalidWirePayload)
        );

        let mut bad_source_id = canonical.clone();
        bad_source_id.source_artifact_id = "not-a-uuid".into();
        assert_eq!(
            bad_source_id.to_json(),
            Err(EvidenceError::InvalidWirePayload)
        );

        let mut uppercase_source_id = canonical.clone();
        uppercase_source_id.source_artifact_id =
            uppercase_source_id.source_artifact_id.to_ascii_uppercase();
        assert_eq!(
            uppercase_source_id.to_json(),
            Err(EvidenceError::InvalidWirePayload)
        );

        let mut bad_digest = canonical.clone();
        bad_digest.source_snapshot_sha256 = "not-a-digest".into();
        assert_eq!(
            bad_digest.to_json(),
            Err(EvidenceError::InvalidContentDigest)
        );

        let mut uppercase_digest = canonical.clone();
        uppercase_digest.source_snapshot_sha256 =
            uppercase_digest.source_snapshot_sha256.to_ascii_uppercase();
        assert_eq!(
            uppercase_digest.to_json(),
            Err(EvidenceError::InvalidContentDigest)
        );

        let mut bad_system_time = canonical.clone();
        bad_system_time.system_observed_at = "not-a-time".into();
        assert_eq!(
            bad_system_time.to_json(),
            Err(EvidenceError::InvalidWirePayload)
        );

        let mut bad_available_time = canonical.clone();
        bad_available_time.available_at = "not-a-time".into();
        assert_eq!(
            bad_available_time.to_json(),
            Err(EvidenceError::InvalidWirePayload)
        );

        let mut offset_time = canonical.clone();
        offset_time.available_at = "2026-08-01T08:59:59+09:00".into();
        assert_eq!(
            offset_time.to_json(),
            Err(EvidenceError::InvalidWirePayload)
        );

        let mut mismatched_clocks = canonical;
        mismatched_clocks.available_at = "2000-01-01T00:00:00Z".into();
        assert_eq!(
            mismatched_clocks.to_json(),
            Err(EvidenceError::InvalidWirePayload)
        );
    }

    #[test]
    fn bounded_serializer_covers_failure_and_size_paths() {
        assert_eq!(
            serialize_bounded_receipt(&SerializationFailure),
            Err(EvidenceError::InvalidWirePayload)
        );
        let oversized = "x".repeat(SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT + 1);
        assert_eq!(
            serialize_bounded_receipt(&oversized),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            serialize_bounded_receipt(&"ok").expect("bounded json"),
            "\"ok\""
        );
    }
}
