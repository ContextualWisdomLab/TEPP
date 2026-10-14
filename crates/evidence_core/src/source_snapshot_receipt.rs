//! Versioned immutable source-snapshot evidence bindings.

use crate::{ContentDigest, EvidenceError, EvidenceId, SourceAvailability};
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
/// artifact identity, exact Evidence-owned observation and availability record
/// identities, logical snapshot identity, exact source-content SHA-256, and the
/// Evidence-owned observation/availability clocks from [`SourceAvailability`].
/// Downstream analysis may retain the canonical receipt digest as an opaque
/// binding. This object does not assert source ownership, signature,
/// authorization, external authenticity, or chain of custody, and it does not
/// substitute for a numeric estimator-payload digest.
///
/// Arbitrary JSON cannot construct this owner-issued type. Persisted/external
/// JSON is parsed only as [`ValidatedSourceSnapshotReceiptWireV1`]; repository or
/// service authentication is a separate boundary before any later code may
/// treat that wire record as trusted Evidence state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSnapshotReceiptV1 {
    schema_version: String,
    receipt_id: String,
    source_artifact_id: String,
    source_observation_id: String,
    source_availability_id: String,
    snapshot_id: String,
    source_snapshot_sha256: String,
    system_observed_at: String,
    available_at: String,
}

/// Canonical, structurally validated source-snapshot receipt wire record.
///
/// This type proves only that one JSON payload is canonical and satisfies the
/// receipt field invariants. It is deliberately distinct from
/// [`SourceSnapshotReceiptV1`]: parsing bytes does not prove that the Evidence
/// owner issued, stored, or authenticated the record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatedSourceSnapshotReceiptWireV1 {
    schema_version: String,
    receipt_id: String,
    source_artifact_id: String,
    source_observation_id: String,
    source_availability_id: String,
    snapshot_id: String,
    source_snapshot_sha256: String,
    system_observed_at: String,
    available_at: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceSnapshotReceiptWireDtoV1 {
    schema_version: String,
    receipt_id: String,
    source_artifact_id: String,
    source_observation_id: String,
    source_availability_id: String,
    snapshot_id: String,
    source_snapshot_sha256: String,
    system_observed_at: String,
    available_at: String,
}

impl SourceSnapshotReceiptV1 {
    /// Construct one receipt from Evidence-owned source availability.
    ///
    /// Evidence mints the receipt identity internally. Source-artifact,
    /// source-observation, and source-availability identities, source-content
    /// digest, system observation time, and availability time are copied from
    /// the owner-controlled availability record rather than accepted as free
    /// constructor arguments. Availability must not precede source observation,
    /// but the two clocks remain distinct and need not be equal.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidWirePayload`] when the snapshot identity
    /// is empty, mutable, noncanonical, or otherwise outside the bounded
    /// Evidence contract.
    pub fn from_source_availability(
        snapshot_id: impl Into<String>,
        availability: &SourceAvailability,
    ) -> Result<Self, EvidenceError> {
        let receipt = Self {
            schema_version: SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: EvidenceId::new().to_string(),
            source_artifact_id: availability.source_artifact_id().to_string(),
            source_observation_id: availability.source_observation_id().to_string(),
            source_availability_id: availability.availability_id().to_string(),
            snapshot_id: snapshot_id.into(),
            source_snapshot_sha256: availability.source_snapshot_sha256().to_string(),
            system_observed_at: availability.system_observed_at().to_rfc3339(),
            available_at: availability.available_at().to_rfc3339(),
        };
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

    /// Return the exact Evidence-owned source-observation record identity.
    #[must_use]
    pub fn source_observation_id(&self) -> &str {
        &self.source_observation_id
    }

    /// Return the exact Evidence-owned source-availability record identity.
    #[must_use]
    pub fn source_availability_id(&self) -> &str {
        &self.source_availability_id
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

    /// Return the canonical RFC 3339 Evidence-owned availability clock.
    #[must_use]
    pub fn available_at(&self) -> &str {
        &self.available_at
    }

    /// Recheck all owner-issued receipt invariants before serialization/use.
    fn validate(&self) -> Result<(), EvidenceError> {
        validate_receipt_fields(
            &self.schema_version,
            &self.receipt_id,
            &self.source_artifact_id,
            &self.source_observation_id,
            &self.source_availability_id,
            &self.snapshot_id,
            &self.source_snapshot_sha256,
            &self.system_observed_at,
            &self.available_at,
        )
    }
}

impl ValidatedSourceSnapshotReceiptWireV1 {
    /// Parse and structurally validate one bounded canonical JSON receipt.
    ///
    /// Successful parsing does not authenticate the sender or promote this wire
    /// record into [`SourceSnapshotReceiptV1`]. A repository/service owner must
    /// separately authenticate persisted state before trusted scientific use.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidWirePayload`] for malformed, duplicate,
    /// unknown, noncanonical, backdated, or oversized receipt JSON, and
    /// [`EvidenceError::UnsupportedWireVersion`] for another schema version.
    pub fn from_json(payload: &str) -> Result<Self, EvidenceError> {
        if payload.len() > SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT {
            return Err(EvidenceError::InvalidWirePayload);
        }
        let wire: SourceSnapshotReceiptWireDtoV1 =
            serde_json::from_str(payload).map_err(|_| EvidenceError::InvalidWirePayload)?;
        let validated = Self {
            schema_version: wire.schema_version,
            receipt_id: wire.receipt_id,
            source_artifact_id: wire.source_artifact_id,
            source_observation_id: wire.source_observation_id,
            source_availability_id: wire.source_availability_id,
            snapshot_id: wire.snapshot_id,
            source_snapshot_sha256: wire.source_snapshot_sha256,
            system_observed_at: wire.system_observed_at,
            available_at: wire.available_at,
        };
        validated.validate()?;
        if serialize_bounded_receipt(&validated)? != payload {
            return Err(EvidenceError::InvalidWirePayload);
        }
        Ok(validated)
    }

    /// Return the canonical RFC 9562 UUIDv7 receipt identity from the wire record.
    #[must_use]
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    /// Return the immutable source-artifact identity from the wire record.
    #[must_use]
    pub fn source_artifact_id(&self) -> &str {
        &self.source_artifact_id
    }

    /// Return the source-observation record identity from the wire record.
    #[must_use]
    pub fn source_observation_id(&self) -> &str {
        &self.source_observation_id
    }

    /// Return the source-availability record identity from the wire record.
    #[must_use]
    pub fn source_availability_id(&self) -> &str {
        &self.source_availability_id
    }

    /// Return the logical source-snapshot identity from the wire record.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Return the canonical lowercase source-content SHA-256 from the wire record.
    #[must_use]
    pub fn source_snapshot_sha256(&self) -> &str {
        &self.source_snapshot_sha256
    }

    /// Return the canonical RFC 3339 source-observation system clock.
    #[must_use]
    pub fn system_observed_at(&self) -> &str {
        &self.system_observed_at
    }

    /// Return the canonical RFC 3339 availability clock from the wire record.
    #[must_use]
    pub fn available_at(&self) -> &str {
        &self.available_at
    }

    /// Recheck structural wire invariants without promoting trust state.
    fn validate(&self) -> Result<(), EvidenceError> {
        validate_receipt_fields(
            &self.schema_version,
            &self.receipt_id,
            &self.source_artifact_id,
            &self.source_observation_id,
            &self.source_availability_id,
            &self.snapshot_id,
            &self.source_snapshot_sha256,
            &self.system_observed_at,
            &self.available_at,
        )
    }
}

/// Validate the shared identity, digest, and temporal invariants for owner and wire records.
fn validate_receipt_fields(
    schema_version: &str,
    receipt_id: &str,
    source_artifact_id: &str,
    source_observation_id: &str,
    source_availability_id: &str,
    snapshot_id: &str,
    source_snapshot_sha256: &str,
    system_observed_at: &str,
    available_at: &str,
) -> Result<(), EvidenceError> {
    if schema_version != SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION {
        return Err(EvidenceError::UnsupportedWireVersion);
    }
    let parsed_receipt_id =
        EvidenceId::from_str(receipt_id).map_err(|_| EvidenceError::InvalidWirePayload)?;
    let parsed_source_artifact_id =
        EvidenceId::from_str(source_artifact_id).map_err(|_| EvidenceError::InvalidWirePayload)?;
    let parsed_source_observation_id = EvidenceId::from_str(source_observation_id)
        .map_err(|_| EvidenceError::InvalidWirePayload)?;
    let parsed_source_availability_id = EvidenceId::from_str(source_availability_id)
        .map_err(|_| EvidenceError::InvalidWirePayload)?;
    if parsed_receipt_id.to_string() != receipt_id
        || parsed_source_artifact_id.to_string() != source_artifact_id
        || parsed_source_observation_id.to_string() != source_observation_id
        || parsed_source_availability_id.to_string() != source_availability_id
        || !valid_snapshot_id(snapshot_id)
    {
        return Err(EvidenceError::InvalidWirePayload);
    }
    let digest = ContentDigest::from_str(source_snapshot_sha256)
        .map_err(|_| EvidenceError::InvalidContentDigest)?;
    if digest.to_string() != source_snapshot_sha256 {
        return Err(EvidenceError::InvalidContentDigest);
    }
    let system_observed = SystemTime::parse_rfc3339(system_observed_at)
        .map_err(|_| EvidenceError::InvalidWirePayload)?;
    let available = AvailableTime::parse_rfc3339(available_at)
        .map_err(|_| EvidenceError::InvalidWirePayload)?;
    if system_observed.to_rfc3339() != system_observed_at
        || available.to_rfc3339() != available_at
        || available.instant() < system_observed.instant()
    {
        return Err(EvidenceError::InvalidWirePayload);
    }
    Ok(())
}

/// Serialize one receipt-shaped record before applying the public wire byte ceiling.
fn serialize_bounded_receipt<T: Serialize>(value: &T) -> Result<String, EvidenceError> {
    let payload = serde_json::to_string(value).map_err(|_| EvidenceError::InvalidWirePayload)?;
    enforce_receipt_byte_limit(payload)
}

/// Enforce the receipt byte ceiling after serialization, independent of serializer type.
fn enforce_receipt_byte_limit(payload: String) -> Result<String, EvidenceError> {
    if payload.len() > SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT {
        return Err(EvidenceError::InvalidWirePayload);
    }
    Ok(payload)
}

/// Accept only bounded immutable snapshot identifiers and reject mutable locator aliases.
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
        "head" | "latest" | "main" | "master" | "latest-release" | "release-latest"
    ) && !numeric_alias
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_SNAPSHOT_IDENTIFIER_BYTES, SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT,
        SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION, SourceSnapshotReceiptV1,
        ValidatedSourceSnapshotReceiptWireV1, enforce_receipt_byte_limit,
        serialize_bounded_receipt, valid_snapshot_id,
    };
    use crate::{EvidenceError, SourceArtifact, SourceAvailability, SourceObservation};
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
        let availability = SourceAvailability::make_available(&observation).expect("availability");
        SourceSnapshotReceiptV1::from_source_availability("snapshot-rubin-loading", &availability)
            .expect("receipt")
    }

    #[test]
    fn canonical_wire_round_trip_preserves_fields_without_trust_promotion() {
        let receipt = receipt();
        let json = receipt.to_json().expect("json");
        assert!(json.contains(SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION));
        let parsed =
            ValidatedSourceSnapshotReceiptWireV1::from_json(&json).expect("validated wire");
        assert_eq!(parsed.receipt_id(), receipt.receipt_id());
        assert_eq!(parsed.source_artifact_id(), receipt.source_artifact_id());
        assert_eq!(
            parsed.source_observation_id(),
            receipt.source_observation_id()
        );
        assert_eq!(
            parsed.source_availability_id(),
            receipt.source_availability_id()
        );
        assert_eq!(parsed.snapshot_id(), receipt.snapshot_id());
        assert_eq!(
            parsed.source_snapshot_sha256(),
            receipt.source_snapshot_sha256()
        );
        assert_eq!(parsed.system_observed_at(), receipt.system_observed_at());
        assert_eq!(parsed.available_at(), receipt.available_at());
        assert_eq!(
            receipt.binding_sha256().expect("binding").to_string().len(),
            64
        );
    }

    #[test]
    fn mutable_and_malformed_snapshot_identifiers_fail_closed() {
        let source_artifact = SourceArtifact::from_bytes(b"source").expect("artifact");
        let observation = SourceObservation::observe(&source_artifact).expect("observation");
        let availability = SourceAvailability::make_available(&observation).expect("availability");
        for alias in [
            "HEAD",
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
                SourceSnapshotReceiptV1::from_source_availability(alias, &availability),
                Err(EvidenceError::InvalidWirePayload)
            );
        }
        assert!(!valid_snapshot_id(""));
        assert!(!valid_snapshot_id("snapshot/branch"));
        assert!(!valid_snapshot_id(
            &"a".repeat(MAX_SNAPSHOT_IDENTIFIER_BYTES + 1)
        ));
        assert!(valid_snapshot_id("snapshot:v1_2026.09-15"));
        assert!(valid_snapshot_id("pr-release-v1"));
    }

    #[test]
    fn private_owner_validation_branches_refuse_noncanonical_fields() {
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

        let mut bad_observation_id = canonical.clone();
        bad_observation_id.source_observation_id = "not-a-uuid".into();
        assert_eq!(
            bad_observation_id.to_json(),
            Err(EvidenceError::InvalidWirePayload)
        );

        let mut uppercase_observation_id = canonical.clone();
        uppercase_observation_id.source_observation_id = uppercase_observation_id
            .source_observation_id
            .to_ascii_uppercase();
        assert_eq!(
            uppercase_observation_id.to_json(),
            Err(EvidenceError::InvalidWirePayload)
        );

        let mut bad_availability_id = canonical.clone();
        bad_availability_id.source_availability_id = "not-a-uuid".into();
        assert_eq!(
            bad_availability_id.to_json(),
            Err(EvidenceError::InvalidWirePayload)
        );

        let mut uppercase_availability_id = canonical.clone();
        uppercase_availability_id.source_availability_id = uppercase_availability_id
            .source_availability_id
            .to_ascii_uppercase();
        assert_eq!(
            uppercase_availability_id.to_json(),
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

        let mut offset_system_time = canonical.clone();
        offset_system_time.system_observed_at =
            offset_system_time.system_observed_at.replace('Z', "+00:00");
        assert_eq!(
            offset_system_time.to_json(),
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

        let mut backdated_availability = canonical;
        backdated_availability.available_at = "2000-01-01T00:00:00Z".into();
        assert_eq!(
            backdated_availability.to_json(),
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

    #[test]
    fn byte_limit_is_a_non_generic_post_serialization_boundary() {
        let oversized = "x".repeat(SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT + 1);
        assert_eq!(
            enforce_receipt_byte_limit(oversized),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            enforce_receipt_byte_limit("\"ok\"".into()).expect("bounded payload"),
            "\"ok\""
        );
    }
}
