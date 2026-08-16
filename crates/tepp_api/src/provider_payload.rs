//! Purpose-bound provider payloads and separately authorized re-identification.

use crate::ApiError;
use crate::authorization::{
    AnalyticalPurpose, ExportAuthorizationRequest, authorize_export, require_export_allowed,
};
use crate::wire::require_nonempty;
use sha2::{Digest, Sha256};
use std::fmt;
use temporal_core::TemporalInstant;

/// Time-bounded purpose grant evaluated at a decision instant.
///
/// `valid_from` / `valid_to` are RFC 3339 UTC instants (`YYYY-MM-DDTHH:MM:SSZ`).
/// An omitted `valid_to` is an open-ended grant. The decision instant is the
/// authorization's available/system time and must not use future-available
/// evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurposeGrant {
    /// Opaque tenant/workspace identity bound to the grant.
    pub tenant_workspace_id: String,
    /// Opaque principal identity (not a password or token).
    pub principal_id: String,
    /// Declared analytical purpose.
    pub purpose: AnalyticalPurpose,
    /// Inclusive grant start instant.
    pub valid_from: String,
    /// Inclusive grant end instant, or `None` for an open-ended grant.
    pub valid_to: Option<String>,
    /// Whether a separate re-identification path is authorized.
    pub reidentification_authorized: bool,
}

/// Evidence offered to a model provider or modular CWL peer.
///
/// Direct identity mappings must stay empty on this path. Opaque analytical
/// identifiers and membership roles are scientific linkage and are not
/// blanket-masked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderEvidenceOffer {
    /// Tenant/workspace that owns the offered evidence.
    pub tenant_workspace_id: String,
    /// Opaque artifact identity.
    pub artifact_id: String,
    /// Opaque analytical identifier retained for multilevel membership.
    pub opaque_analytical_id: String,
    /// Optional free-text source body.
    pub source_text: Option<String>,
    /// Direct identity mapping; must be absent for provider disclosure.
    pub identity_mapping: Option<String>,
    /// Optional contextual membership role preserved as scientific linkage.
    pub membership_role: Option<String>,
}

/// Minimized payload that may be sent to a provider.
///
/// Construct only via [`minimize_provider_payload`]. The payload never carries
/// a direct identity mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinimizedProviderPayload {
    artifact_id: String,
    opaque_analytical_id: String,
    source_text: Option<String>,
    membership_role: Option<String>,
}

impl MinimizedProviderPayload {
    /// Opaque artifact identity.
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    /// Opaque analytical identifier retained for measurement.
    #[must_use]
    pub fn opaque_analytical_id(&self) -> &str {
        &self.opaque_analytical_id
    }

    /// Free-text source body when the purpose grant allows it.
    #[must_use]
    pub fn source_text(&self) -> Option<&str> {
        self.source_text.as_deref()
    }

    /// Contextual membership role, if offered.
    #[must_use]
    pub fn membership_role(&self) -> Option<&str> {
        self.membership_role.as_deref()
    }

    /// Direct identity mapping; always absent on the provider path.
    #[must_use]
    pub const fn identity_mapping(&self) -> Option<&str> {
        None
    }
}

/// Log-safe record of a provider disclosure decision.
///
/// Ordinary logs must not copy source text or direct identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderDisclosureLog {
    purpose: String,
    included_source_text: bool,
    included_identity_mapping: bool,
}

impl ProviderDisclosureLog {
    /// Stable purpose wire name recorded with the decision.
    #[must_use]
    pub fn purpose_wire_name(&self) -> &str {
        &self.purpose
    }

    /// Whether the minimized payload included a source body.
    #[must_use]
    pub const fn included_source_text(&self) -> bool {
        self.included_source_text
    }

    /// Whether a direct identity mapping was included; always false.
    #[must_use]
    pub const fn included_identity_mapping(&self) -> bool {
        self.included_identity_mapping
    }
}

impl fmt::Display for ProviderDisclosureLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "provider_disclosure purpose={} source={} mapping={}",
            self.purpose, self.included_source_text, self.included_identity_mapping
        )
    }
}

/// Separately protected identity mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityMappingRecord {
    /// Tenant/workspace that owns the mapping.
    pub tenant_workspace_id: String,
    /// Opaque analytical identifier.
    pub opaque_analytical_id: String,
    /// Direct identity string; never copied onto a provider payload.
    pub direct_identity: String,
}

/// Result of an elevated re-identification disclosure.
///
/// Construct only via [`disclose_identity_mapping`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisclosedIdentityMapping {
    opaque_analytical_id: String,
    direct_identity: String,
}

impl DisclosedIdentityMapping {
    /// Opaque analytical identifier that was resolved.
    #[must_use]
    pub fn opaque_analytical_id(&self) -> &str {
        &self.opaque_analytical_id
    }

    /// Direct identity released on the elevated path only.
    #[must_use]
    pub fn direct_identity(&self) -> &str {
        &self.direct_identity
    }
}

/// Redacted outcome of an elevated re-identification decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReidentificationAuditOutcome {
    /// The protected mapping was released after audit append succeeded.
    Allowed,
    /// A well-formed request was denied by purpose, tenant, lifetime, or role policy.
    Denied,
}

impl ReidentificationAuditOutcome {
    /// Stable wire name for append-only audit persistence.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
        }
    }
}

/// Redacted append-only evidence for an elevated re-identification decision.
///
/// Direct identity is deliberately absent. The digest identifies the governed
/// decision input without copying protected source or mapping content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReidentificationAuditRecord {
    tenant_workspace_id: String,
    principal_id: String,
    purpose_wire_name: String,
    action_code: &'static str,
    opaque_analytical_id: String,
    decision_time: String,
    outcome: ReidentificationAuditOutcome,
    decision_digest: String,
}

impl ReidentificationAuditRecord {
    /// Tenant/workspace in which the decision occurred.
    #[must_use]
    pub fn tenant_workspace_id(&self) -> &str {
        &self.tenant_workspace_id
    }

    /// Opaque principal that requested disclosure.
    #[must_use]
    pub fn principal_id(&self) -> &str {
        &self.principal_id
    }

    /// Purpose wire name evaluated by policy.
    #[must_use]
    pub fn purpose_wire_name(&self) -> &str {
        &self.purpose_wire_name
    }

    /// Stable elevated action code.
    #[must_use]
    pub const fn action_code(&self) -> &'static str {
        self.action_code
    }

    /// Opaque analytical identity involved in the decision.
    #[must_use]
    pub fn opaque_analytical_id(&self) -> &str {
        &self.opaque_analytical_id
    }

    /// Canonical UTC decision instant.
    #[must_use]
    pub fn decision_time(&self) -> &str {
        &self.decision_time
    }

    /// Allowed or denied decision outcome.
    #[must_use]
    pub const fn outcome(&self) -> ReidentificationAuditOutcome {
        self.outcome
    }

    /// Canonical SHA-256 digest of the governed decision input.
    #[must_use]
    pub fn decision_digest(&self) -> &str {
        &self.decision_digest
    }
}

/// Append-only persistence port for elevated re-identification audit evidence.
///
/// Implementations must append an immutable row or event and must never copy
/// the disclosed direct identity into ordinary audit storage.
pub trait ReidentificationAuditSink {
    /// Append one redacted decision record before disclosure or denial returns.
    ///
    /// # Errors
    ///
    /// Returns a redacted [`ApiError`] when append-only persistence fails. The
    /// disclosure then fails closed and no direct identity is returned.
    fn append_reidentification_audit(
        &mut self,
        record: &ReidentificationAuditRecord,
    ) -> Result<(), ApiError>;
}

/// Minimize evidence for a model provider without blanket PII masking.
///
/// Opaque analytical identifiers and membership roles are preserved. Source
/// text follows [`crate::authorize_export`]. Direct identity mappings are
/// refused on this path even when re-identification is separately authorized.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for empty identities, empty
/// optional strings, inverted grant windows, or non-canonical RFC 3339 UTC
/// instants. Returns [`ApiError::AuthorizationDenied`] for expired or
/// not-yet-valid grants, cross-tenant offers, attached identity mappings, or
/// purpose-denied source text.
pub fn minimize_provider_payload(
    grant: &PurposeGrant,
    offer: &ProviderEvidenceOffer,
    decision_time: &str,
) -> Result<(MinimizedProviderPayload, ProviderDisclosureLog), ApiError> {
    validate_grant(grant)?;
    require_nonempty(&offer.tenant_workspace_id)?;
    require_nonempty(&offer.artifact_id)?;
    require_nonempty(&offer.opaque_analytical_id)?;
    require_optional_nonempty(offer.source_text.as_deref())?;
    require_optional_nonempty(offer.membership_role.as_deref())?;
    require_optional_nonempty(offer.identity_mapping.as_deref())?;
    require_rfc3339_utc(decision_time)?;
    if !grant_covers(grant, decision_time) {
        return Err(ApiError::AuthorizationDenied);
    }
    if offer.tenant_workspace_id != grant.tenant_workspace_id {
        return Err(ApiError::AuthorizationDenied);
    }
    if offer.identity_mapping.is_some() {
        return Err(ApiError::AuthorizationDenied);
    }
    let decision = authorize_export(&ExportAuthorizationRequest {
        tenant_workspace_id: grant.tenant_workspace_id.clone(),
        principal_id: grant.principal_id.clone(),
        purpose: grant.purpose,
        artifact_id: offer.artifact_id.clone(),
        includes_source_text: offer.source_text.is_some(),
    })?;
    require_export_allowed(&decision)?;
    let log = ProviderDisclosureLog {
        purpose: grant.purpose.wire_name().into(),
        included_source_text: offer.source_text.is_some(),
        included_identity_mapping: false,
    };
    Ok((
        MinimizedProviderPayload {
            artifact_id: offer.artifact_id.clone(),
            opaque_analytical_id: offer.opaque_analytical_id.clone(),
            source_text: offer.source_text.clone(),
            membership_role: offer.membership_role.clone(),
        },
        log,
    ))
}

/// Disclose a direct identity mapping on the elevated scientific path.
///
/// This is not a provider payload. Modular consumers, operational monitoring,
/// and partner disclosure cannot receive the mapping even when the grant flag
/// is set.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for empty identities, inverted
/// windows, or non-canonical instants. Returns
/// [`ApiError::AuthorizationDenied`] when the grant is expired, not yet
/// valid, cross-tenant, missing the elevated flag, or not
/// [`AnalyticalPurpose::ScientificValidation`].
pub fn disclose_identity_mapping<S: ReidentificationAuditSink>(
    grant: &PurposeGrant,
    mapping: &IdentityMappingRecord,
    decision_time: &str,
    audit_sink: &mut S,
) -> Result<(DisclosedIdentityMapping, ReidentificationAuditRecord), ApiError> {
    validate_grant(grant)?;
    require_nonempty(&mapping.tenant_workspace_id)?;
    require_nonempty(&mapping.opaque_analytical_id)?;
    require_nonempty(&mapping.direct_identity)?;
    require_rfc3339_utc(decision_time)?;

    let allowed = grant_covers(grant, decision_time)
        && mapping.tenant_workspace_id == grant.tenant_workspace_id
        && grant.reidentification_authorized
        && grant.purpose == AnalyticalPurpose::ScientificValidation;
    let outcome = if allowed {
        ReidentificationAuditOutcome::Allowed
    } else {
        ReidentificationAuditOutcome::Denied
    };
    let audit_record = ReidentificationAuditRecord {
        tenant_workspace_id: grant.tenant_workspace_id.clone(),
        principal_id: grant.principal_id.clone(),
        purpose_wire_name: grant.purpose.wire_name().into(),
        action_code: "reidentify_identity_mapping",
        opaque_analytical_id: mapping.opaque_analytical_id.clone(),
        decision_time: decision_time.into(),
        outcome,
        decision_digest: reidentification_decision_digest(grant, mapping, decision_time, outcome)?,
    };
    audit_sink.append_reidentification_audit(&audit_record)?;
    if !allowed {
        return Err(ApiError::AuthorizationDenied);
    }
    Ok((
        DisclosedIdentityMapping {
            opaque_analytical_id: mapping.opaque_analytical_id.clone(),
            direct_identity: mapping.direct_identity.clone(),
        },
        audit_record,
    ))
}

const REIDENTIFICATION_AUDIT_DIGEST_VERSION: &str = "tepp.reidentification.audit.v1";

fn reidentification_decision_digest(
    grant: &PurposeGrant,
    mapping: &IdentityMappingRecord,
    decision_time: &str,
    outcome: ReidentificationAuditOutcome,
) -> Result<String, ApiError> {
    let mut hasher = Sha256::new();
    for value in [
        REIDENTIFICATION_AUDIT_DIGEST_VERSION,
        "reidentify_identity_mapping",
        &grant.tenant_workspace_id,
        &grant.principal_id,
        grant.purpose.wire_name(),
        &grant.valid_from,
        if grant.valid_to.is_some() { "1" } else { "0" },
        grant.valid_to.as_deref().unwrap_or(""),
        if grant.reidentification_authorized {
            "1"
        } else {
            "0"
        },
        &mapping.tenant_workspace_id,
        &mapping.opaque_analytical_id,
        &mapping.direct_identity,
        decision_time,
        outcome.wire_name(),
    ] {
        update_audit_digest_field(&mut hasher, value)?;
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn update_audit_digest_field(hasher: &mut Sha256, value: &str) -> Result<(), ApiError> {
    let length = u64::try_from(value.len()).map_err(|_| ApiError::LimitExceeded)?;
    hasher.update(length.to_be_bytes());
    hasher.update(value.as_bytes());
    Ok(())
}

// Lexicographic comparisons in `validate_grant` and `grant_covers` are valid
// only after fixed-width UTC RFC 3339 validation; `TemporalInstant` also
// rejects impossible calendar dates and leap seconds before comparison.
fn validate_grant(grant: &PurposeGrant) -> Result<(), ApiError> {
    require_nonempty(&grant.tenant_workspace_id)?;
    require_nonempty(&grant.principal_id)?;
    require_rfc3339_utc(&grant.valid_from)?;
    if let Some(until) = &grant.valid_to {
        require_rfc3339_utc(until)?;
        if until.as_str() < grant.valid_from.as_str() {
            return Err(ApiError::InvalidWirePayload);
        }
    }
    Ok(())
}

fn grant_covers(grant: &PurposeGrant, decision_time: &str) -> bool {
    if decision_time < grant.valid_from.as_str() {
        return false;
    }
    if let Some(until) = &grant.valid_to
        && decision_time > until.as_str()
    {
        return false;
    }
    true
}

fn require_optional_nonempty(value: Option<&str>) -> Result<(), ApiError> {
    match value {
        Some(text) => require_nonempty(text),
        None => Ok(()),
    }
}

fn require_rfc3339_utc(value: &str) -> Result<(), ApiError> {
    if is_rfc3339_utc(value) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

fn is_rfc3339_utc(value: &str) -> bool {
    // Exact second-resolution UTC (`YYYY-MM-DDTHH:MM:SSZ`). Calendar/syntax is
    // owned by `TemporalInstant`; keep length + trailing `Z` only so offsets and
    // fractional seconds stay out of grants without long `&&` branch chains.
    if value.len() != 20 {
        return false;
    }
    if value.as_bytes()[19] != b'Z' {
        return false;
    }
    TemporalInstant::parse_rfc3339(value).is_ok()
}

#[cfg(test)]
mod tests {
    use super::{
        DisclosedIdentityMapping, IdentityMappingRecord, MinimizedProviderPayload,
        ProviderDisclosureLog, ProviderEvidenceOffer, PurposeGrant, ReidentificationAuditOutcome,
        ReidentificationAuditRecord, ReidentificationAuditSink,
        disclose_identity_mapping as disclose_identity_mapping_with_audit, is_rfc3339_utc,
        minimize_provider_payload, reidentification_decision_digest,
    };
    use crate::ApiError;
    use crate::authorization::AnalyticalPurpose;

    fn grant(purpose: AnalyticalPurpose, reidentification: bool) -> PurposeGrant {
        PurposeGrant {
            tenant_workspace_id: "tenant-ws-1".into(),
            principal_id: "principal-analyst-1".into(),
            purpose,
            valid_from: "2026-01-01T00:00:00Z".into(),
            valid_to: Some("2026-12-31T23:59:59Z".into()),
            reidentification_authorized: reidentification,
        }
    }

    fn offer(source: Option<&str>) -> ProviderEvidenceOffer {
        ProviderEvidenceOffer {
            tenant_workspace_id: "tenant-ws-1".into(),
            artifact_id: "artifact-1".into(),
            opaque_analytical_id: "entity-1".into(),
            source_text: source.map(str::to_owned),
            identity_mapping: None,
            membership_role: None,
        }
    }

    fn mapping() -> IdentityMappingRecord {
        IdentityMappingRecord {
            tenant_workspace_id: "tenant-ws-1".into(),
            opaque_analytical_id: "entity-1".into(),
            direct_identity: "Pat Lee".into(),
        }
    }

    #[derive(Default)]
    struct RecordingAuditSink {
        records: Vec<ReidentificationAuditRecord>,
    }

    impl ReidentificationAuditSink for RecordingAuditSink {
        fn append_reidentification_audit(
            &mut self,
            record: &ReidentificationAuditRecord,
        ) -> Result<(), ApiError> {
            self.records.push(record.clone());
            Ok(())
        }
    }

    struct FailingSink;
    impl ReidentificationAuditSink for FailingSink {
        fn append_reidentification_audit(
            &mut self,
            _record: &ReidentificationAuditRecord,
        ) -> Result<(), ApiError> {
            Err(ApiError::AuthorizationDenied)
        }
    }

    fn disclose(
        grant: &PurposeGrant,
        mapping: &IdentityMappingRecord,
        decision_time: &str,
    ) -> Result<(DisclosedIdentityMapping, ReidentificationAuditRecord), ApiError> {
        let mut audit_sink = RecordingAuditSink::default();
        disclose_identity_mapping_with_audit(grant, mapping, decision_time, &mut audit_sink)
    }

    #[test]
    fn rfc3339_utc_is_strict_and_windows_are_inclusive() {
        assert!(is_rfc3339_utc("2026-01-01T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01T00:00:00+00:00"));
        assert!(!is_rfc3339_utc("2026-01-01 00:00:00Z"));
        assert!(!is_rfc3339_utc("20260101T000000Z"));
        assert!(!is_rfc3339_utc("xxxx-01-01T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-xx-01T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-xxT00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01Txx:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01T00:xx:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01T00:00:xxZ"));
        assert!(!is_rfc3339_utc("2026X01-01T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01X01T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01X00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01T00X00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01T00:00X00Z"));
        assert!(!is_rfc3339_utc("2026-01-01T00:00:00X"));

        let at_start = minimize_provider_payload(
            &grant(AnalyticalPurpose::ScientificValidation, false),
            &offer(None),
            "2026-01-01T00:00:00Z",
        )
        .expect("start");
        assert!(!at_start.1.included_source_text());
        let at_end = minimize_provider_payload(
            &grant(AnalyticalPurpose::ScientificValidation, false),
            &offer(None),
            "2026-12-31T23:59:59Z",
        )
        .expect("end");
        assert!(at_end.0.identity_mapping().is_none());
        assert_eq!(
            at_end.1.to_string(),
            "provider_disclosure purpose=scientific_validation source=false mapping=false"
        );
    }

    #[test]
    fn minimize_denies_expired_foreign_and_identity_mapping_offers() {
        let expired = PurposeGrant {
            valid_to: Some("2026-03-01T00:00:00Z".into()),
            ..grant(AnalyticalPurpose::ScientificValidation, false)
        };
        assert_eq!(
            minimize_provider_payload(&expired, &offer(None), "2026-06-15T12:00:00Z"),
            Err(ApiError::AuthorizationDenied)
        );
        let not_yet = PurposeGrant {
            valid_from: "2026-07-01T00:00:00Z".into(),
            ..grant(AnalyticalPurpose::ScientificValidation, false)
        };
        assert_eq!(
            minimize_provider_payload(&not_yet, &offer(None), "2026-06-15T12:00:00Z"),
            Err(ApiError::AuthorizationDenied)
        );
        let foreign = ProviderEvidenceOffer {
            tenant_workspace_id: "other-tenant".into(),
            ..offer(None)
        };
        assert_eq!(
            minimize_provider_payload(
                &grant(AnalyticalPurpose::ScientificValidation, false),
                &foreign,
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::AuthorizationDenied)
        );
        let mut mapped = offer(None);
        mapped.identity_mapping = Some("secret-name".into());
        assert_eq!(
            minimize_provider_payload(
                &grant(AnalyticalPurpose::ScientificValidation, false),
                &mapped,
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::AuthorizationDenied)
        );
    }

    #[test]
    fn partner_and_ops_follow_export_source_rules() {
        assert_eq!(
            minimize_provider_payload(
                &grant(AnalyticalPurpose::PartnerDisclosure, false),
                &offer(Some("body")),
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::AuthorizationDenied)
        );
        let partner = minimize_provider_payload(
            &grant(AnalyticalPurpose::PartnerDisclosure, false),
            &offer(None),
            "2026-06-15T12:00:00Z",
        )
        .expect("partner derived");
        assert!(partner.0.source_text().is_none());
        let ops = minimize_provider_payload(
            &grant(AnalyticalPurpose::OperationalMonitoring, false),
            &offer(None),
            "2026-06-15T12:00:00Z",
        )
        .expect("ops derived");
        assert_eq!(ops.1.purpose_wire_name(), "operational_monitoring");
    }

    #[test]
    fn empty_optionals_and_grant_fields_fail_closed() {
        let mut empty_principal = grant(AnalyticalPurpose::ScientificValidation, false);
        empty_principal.principal_id.clear();
        assert_eq!(
            minimize_provider_payload(&empty_principal, &offer(None), "2026-06-15T12:00:00Z"),
            Err(ApiError::InvalidWirePayload)
        );
        let mut empty_source = offer(Some(""));
        empty_source.source_text = Some(String::new());
        assert_eq!(
            minimize_provider_payload(
                &grant(AnalyticalPurpose::ScientificValidation, false),
                &empty_source,
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        let mut empty_role = offer(None);
        empty_role.membership_role = Some(String::new());
        assert_eq!(
            minimize_provider_payload(
                &grant(AnalyticalPurpose::ScientificValidation, false),
                &empty_role,
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        let mut empty_mapping_text = offer(None);
        empty_mapping_text.identity_mapping = Some(String::new());
        assert_eq!(
            minimize_provider_payload(
                &grant(AnalyticalPurpose::ScientificValidation, false),
                &empty_mapping_text,
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        let mut empty_artifact = offer(None);
        empty_artifact.artifact_id.clear();
        assert_eq!(
            minimize_provider_payload(
                &grant(AnalyticalPurpose::ScientificValidation, false),
                &empty_artifact,
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        let mut empty_opaque = offer(None);
        empty_opaque.opaque_analytical_id.clear();
        assert_eq!(
            minimize_provider_payload(
                &grant(AnalyticalPurpose::ScientificValidation, false),
                &empty_opaque,
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn disclose_covers_remaining_fail_closed_branches() {
        assert_eq!(
            disclose(
                &grant(AnalyticalPurpose::PartnerDisclosure, true),
                &mapping(),
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::AuthorizationDenied)
        );
        let expired = PurposeGrant {
            valid_to: Some("2026-02-01T00:00:00Z".into()),
            ..grant(AnalyticalPurpose::ScientificValidation, true)
        };
        assert_eq!(
            disclose(&expired, &mapping(), "2026-06-15T12:00:00Z"),
            Err(ApiError::AuthorizationDenied)
        );
        let inverted = PurposeGrant {
            valid_from: "2026-12-31T00:00:00Z".into(),
            valid_to: Some("2026-01-01T00:00:00Z".into()),
            ..grant(AnalyticalPurpose::ScientificValidation, true)
        };
        assert_eq!(
            disclose(&inverted, &mapping(), "2026-06-15T12:00:00Z"),
            Err(ApiError::InvalidWirePayload)
        );
        let foreign = IdentityMappingRecord {
            tenant_workspace_id: "other-tenant".into(),
            ..mapping()
        };
        assert_eq!(
            disclose(
                &grant(AnalyticalPurpose::ScientificValidation, true),
                &foreign,
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::AuthorizationDenied)
        );
        let mut empty = mapping();
        empty.direct_identity.clear();
        assert_eq!(
            disclose(
                &grant(AnalyticalPurpose::ScientificValidation, true),
                &empty,
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            disclose(
                &grant(AnalyticalPurpose::ScientificValidation, true),
                &mapping(),
                "bad",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        let mut empty_opaque = mapping();
        empty_opaque.opaque_analytical_id.clear();
        assert_eq!(
            disclose(
                &grant(AnalyticalPurpose::ScientificValidation, true),
                &empty_opaque,
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        let mut empty_tenant = mapping();
        empty_tenant.tenant_workspace_id.clear();
        assert_eq!(
            disclose(
                &grant(AnalyticalPurpose::ScientificValidation, true),
                &empty_tenant,
                "2026-06-15T12:00:00Z",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        let disclosed = DisclosedIdentityMapping {
            opaque_analytical_id: "entity-1".into(),
            direct_identity: "Pat Lee".into(),
        };
        assert_eq!(disclosed.opaque_analytical_id(), "entity-1");
        assert_eq!(disclosed.direct_identity(), "Pat Lee");
        let payload = MinimizedProviderPayload {
            artifact_id: "a".into(),
            opaque_analytical_id: "e".into(),
            source_text: None,
            membership_role: None,
        };
        assert_eq!(payload.artifact_id(), "a");
        let log = ProviderDisclosureLog {
            purpose: "scientific_validation".into(),
            included_source_text: false,
            included_identity_mapping: false,
        };
        assert!(!log.included_identity_mapping());
    }

    #[test]
    fn audit_digest_binds_outcome_when_other_canonical_fields_are_held_fixed() {
        let allowed = reidentification_decision_digest(
            &grant(AnalyticalPurpose::ScientificValidation, true),
            &mapping(),
            "2026-06-15T12:00:00Z",
            ReidentificationAuditOutcome::Allowed,
        )
        .expect("allowed digest");
        let denied = reidentification_decision_digest(
            &grant(AnalyticalPurpose::ScientificValidation, true),
            &mapping(),
            "2026-06-15T12:00:00Z",
            ReidentificationAuditOutcome::Denied,
        )
        .expect("denied digest");
        assert_ne!(
            allowed, denied,
            "outcome wire name must change the digest when grant, mapping, and time stay fixed"
        );
    }

    #[test]
    fn rfc3339_rejects_impossible_calendar_and_malformed_separators() {
        // Fixed-width UTC shape but impossible calendar → TemporalInstant fails closed.
        assert!(!is_rfc3339_utc("2026-02-30T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-13-01T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-00-01T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-00T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-32T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01T24:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01T00:60:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01T00:00:60Z"));
        // Separator / terminator failures that still keep length 20.
        assert!(!is_rfc3339_utc("2026/01/01T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01 00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-01-01T00-00-00Z"));
        assert!(!is_rfc3339_utc("2026-01-01T00:00:00z"));
        assert!(!is_rfc3339_utc("2026-01-01T00:00:0aZ"));
        assert_eq!(
            minimize_provider_payload(
                &grant(AnalyticalPurpose::ScientificValidation, false),
                &offer(None),
                "2026-02-30T00:00:00Z",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        let bad_from = PurposeGrant {
            valid_from: "2026-02-30T00:00:00Z".into(),
            ..grant(AnalyticalPurpose::ScientificValidation, false)
        };
        assert_eq!(
            minimize_provider_payload(&bad_from, &offer(None), "2026-06-15T12:00:00Z"),
            Err(ApiError::InvalidWirePayload)
        );
        let bad_to = PurposeGrant {
            valid_to: Some("2026-13-01T00:00:00Z".into()),
            ..grant(AnalyticalPurpose::ScientificValidation, false)
        };
        assert_eq!(
            minimize_provider_payload(&bad_to, &offer(None), "2026-06-15T12:00:00Z"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn digest_and_audit_cover_open_ended_grant_and_sink_failure() {
        let open = PurposeGrant {
            valid_to: None,
            reidentification_authorized: true,
            ..grant(AnalyticalPurpose::ScientificValidation, true)
        };
        let open_digest = reidentification_decision_digest(
            &open,
            &mapping(),
            "2026-06-15T12:00:00Z",
            ReidentificationAuditOutcome::Allowed,
        )
        .expect("open digest");
        let closed_digest = reidentification_decision_digest(
            &grant(AnalyticalPurpose::ScientificValidation, true),
            &mapping(),
            "2026-06-15T12:00:00Z",
            ReidentificationAuditOutcome::Allowed,
        )
        .expect("closed digest");
        assert_ne!(
            open_digest, closed_digest,
            "open-ended valid_to must change the canonical audit digest"
        );
        let unauthorized = reidentification_decision_digest(
            &grant(AnalyticalPurpose::ScientificValidation, false),
            &mapping(),
            "2026-06-15T12:00:00Z",
            ReidentificationAuditOutcome::Denied,
        )
        .expect("unauthorized digest");
        assert_ne!(closed_digest, unauthorized);

        let mut sink = FailingSink;
        assert_eq!(
            disclose_identity_mapping_with_audit(
                &grant(AnalyticalPurpose::ScientificValidation, true),
                &mapping(),
                "2026-06-15T12:00:00Z",
                &mut sink,
            ),
            Err(ApiError::AuthorizationDenied)
        );

        let mut sink = RecordingAuditSink::default();
        let (disclosed, audit) = disclose_identity_mapping_with_audit(
            &grant(AnalyticalPurpose::ScientificValidation, true),
            &mapping(),
            "2026-06-15T12:00:00Z",
            &mut sink,
        )
        .expect("allowed with audit");
        assert_eq!(disclosed.direct_identity(), "Pat Lee");
        assert_eq!(audit.tenant_workspace_id(), "tenant-ws-1");
        assert_eq!(audit.principal_id(), "principal-analyst-1");
        assert_eq!(audit.purpose_wire_name(), "scientific_validation");
        assert_eq!(audit.action_code(), "reidentify_identity_mapping");
        assert_eq!(audit.opaque_analytical_id(), "entity-1");
        assert_eq!(audit.decision_time(), "2026-06-15T12:00:00Z");
        assert_eq!(audit.outcome(), ReidentificationAuditOutcome::Allowed);
        assert!(audit.decision_digest().starts_with("sha256:"));
        assert_eq!(audit.outcome().wire_name(), "allowed");
        assert_eq!(ReidentificationAuditOutcome::Denied.wire_name(), "denied");
        assert_eq!(sink.records.len(), 1);
        assert_eq!(sink.records[0].decision_digest(), audit.decision_digest());
    }
}
