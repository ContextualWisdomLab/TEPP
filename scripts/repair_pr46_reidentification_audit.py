"""Apply PR 46 append-only re-identification audit and documentation repairs."""

from pathlib import Path


DECISION_DIGEST = "sha256:" + "a" * 64


def replace_once(text: str, old: str, new: str, label: str) -> str:
    """Replace exactly one fragment or fail closed."""
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)


def update_provider_module() -> None:
    """Add redacted audit types, sink, digest validation, and audited disclosure."""
    path = Path("crates/tepp_api/src/provider_payload.rs")
    text = path.read_text(encoding="utf-8")

    disclosed_impl = """impl DisclosedIdentityMapping {
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
"""
    audit_types = disclosed_impl + """
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
"""
    text = replace_once(text, disclosed_impl, audit_types, "audit type insertion")

    old_function = """pub fn disclose_identity_mapping(
    grant: &PurposeGrant,
    mapping: &IdentityMappingRecord,
    decision_time: &str,
) -> Result<DisclosedIdentityMapping, ApiError> {
    validate_grant(grant)?;
    require_nonempty(&mapping.tenant_workspace_id)?;
    require_nonempty(&mapping.opaque_analytical_id)?;
    require_nonempty(&mapping.direct_identity)?;
    require_rfc3339_utc(decision_time)?;
    if !grant_covers(grant, decision_time) {
        return Err(ApiError::AuthorizationDenied);
    }
    if mapping.tenant_workspace_id != grant.tenant_workspace_id {
        return Err(ApiError::AuthorizationDenied);
    }
    if !grant.reidentification_authorized
        || grant.purpose != AnalyticalPurpose::ScientificValidation
    {
        return Err(ApiError::AuthorizationDenied);
    }
    Ok(DisclosedIdentityMapping {
        opaque_analytical_id: mapping.opaque_analytical_id.clone(),
        direct_identity: mapping.direct_identity.clone(),
    })
}
"""
    new_function = """pub fn disclose_identity_mapping<S: ReidentificationAuditSink>(
    grant: &PurposeGrant,
    mapping: &IdentityMappingRecord,
    decision_time: &str,
    decision_digest: &str,
    audit_sink: &mut S,
) -> Result<(DisclosedIdentityMapping, ReidentificationAuditRecord), ApiError> {
    validate_grant(grant)?;
    require_nonempty(&mapping.tenant_workspace_id)?;
    require_nonempty(&mapping.opaque_analytical_id)?;
    require_nonempty(&mapping.direct_identity)?;
    require_rfc3339_utc(decision_time)?;
    require_sha256_digest(decision_digest)?;

    let allowed = grant_covers(grant, decision_time)
        && mapping.tenant_workspace_id == grant.tenant_workspace_id
        && grant.reidentification_authorized
        && grant.purpose == AnalyticalPurpose::ScientificValidation;
    let audit_record = ReidentificationAuditRecord {
        tenant_workspace_id: grant.tenant_workspace_id.clone(),
        principal_id: grant.principal_id.clone(),
        purpose_wire_name: grant.purpose.wire_name().into(),
        action_code: "reidentify_identity_mapping",
        opaque_analytical_id: mapping.opaque_analytical_id.clone(),
        decision_time: decision_time.into(),
        outcome: if allowed {
            ReidentificationAuditOutcome::Allowed
        } else {
            ReidentificationAuditOutcome::Denied
        },
        decision_digest: decision_digest.into(),
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
"""
    text = replace_once(text, old_function, new_function, "audited disclosure function")

    text = replace_once(
        text,
        "fn validate_grant(grant: &PurposeGrant) -> Result<(), ApiError> {\n",
        """fn require_sha256_digest(value: &str) -> Result<(), ApiError> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(ApiError::InvalidWirePayload);
    };
    if digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

fn validate_grant(grant: &PurposeGrant) -> Result<(), ApiError> {
""",
        "digest helper insertion",
    )

    marker = "#[cfg(test)]\nmod tests {"
    if text.count(marker) != 1:
        raise SystemExit("provider test module marker mismatch")
    production, tests_tail = text.split(marker, 1)
    tests = marker + tests_tail
    tests = replace_once(
        tests,
        """        DisclosedIdentityMapping, IdentityMappingRecord, MinimizedProviderPayload,
        ProviderDisclosureLog, ProviderEvidenceOffer, PurposeGrant, disclose_identity_mapping,
        is_rfc3339_utc, minimize_provider_payload,
""",
        """        DisclosedIdentityMapping, IdentityMappingRecord, MinimizedProviderPayload,
        ProviderDisclosureLog, ProviderEvidenceOffer, PurposeGrant, ReidentificationAuditRecord,
        ReidentificationAuditSink, disclose_identity_mapping as disclose_identity_mapping_with_audit,
        is_rfc3339_utc, minimize_provider_payload,
""",
        "internal test imports",
    )
    mapping_helper = """    fn mapping() -> IdentityMappingRecord {
        IdentityMappingRecord {
            tenant_workspace_id: "tenant-ws-1".into(),
            opaque_analytical_id: "entity-1".into(),
            direct_identity: "Pat Lee".into(),
        }
    }
"""
    audit_helper = mapping_helper + f"""
    #[derive(Default)]
    struct RecordingAuditSink {{
        records: Vec<ReidentificationAuditRecord>,
    }}

    impl ReidentificationAuditSink for RecordingAuditSink {{
        fn append_reidentification_audit(
            &mut self,
            record: &ReidentificationAuditRecord,
        ) -> Result<(), ApiError> {{
            self.records.push(record.clone());
            Ok(())
        }}
    }}

    fn disclose(
        grant: &PurposeGrant,
        mapping: &IdentityMappingRecord,
        decision_time: &str,
    ) -> Result<(DisclosedIdentityMapping, ReidentificationAuditRecord), ApiError> {{
        let mut audit_sink = RecordingAuditSink::default();
        disclose_identity_mapping_with_audit(
            grant,
            mapping,
            decision_time,
            "{DECISION_DIGEST}",
            &mut audit_sink,
        )
    }}
"""
    tests = replace_once(tests, mapping_helper, audit_helper, "internal audit helper")
    tests = tests.replace("disclose_identity_mapping(", "disclose(")
    tests = tests.replace("disclosed.direct_identity()", "disclosed.0.direct_identity()")
    tests = tests.replace(
        "disclosed.opaque_analytical_id()", "disclosed.0.opaque_analytical_id()"
    )
    path.write_text(production + tests, encoding="utf-8")


def update_public_exports() -> None:
    """Export the new append-only audit contract."""
    path = Path("crates/tepp_api/src/lib.rs")
    text = path.read_text(encoding="utf-8")
    anchor = """/// Elevated re-identification result.
pub use provider_payload::DisclosedIdentityMapping;
"""
    replacement = anchor + """/// Redacted re-identification decision outcome.
pub use provider_payload::ReidentificationAuditOutcome;
/// Redacted append-only re-identification audit record.
pub use provider_payload::ReidentificationAuditRecord;
/// Append-only persistence port for re-identification audit evidence.
pub use provider_payload::ReidentificationAuditSink;
"""
    path.write_text(replace_once(text, anchor, replacement, "provider exports"), encoding="utf-8")


def update_contract_tests() -> None:
    """Adapt existing public contracts to the audited function signature."""
    path = Path("crates/tepp_api/tests/provider_payload_contract.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        """    AnalyticalPurpose, ApiError, IdentityMappingRecord, ProviderEvidenceOffer, PurposeGrant,
    disclose_identity_mapping, minimize_provider_payload,
""",
        """    AnalyticalPurpose, ApiError, DisclosedIdentityMapping, IdentityMappingRecord,
    ProviderEvidenceOffer, PurposeGrant, ReidentificationAuditRecord, ReidentificationAuditSink,
    disclose_identity_mapping as disclose_identity_mapping_with_audit, minimize_provider_payload,
""",
        "provider contract imports",
    )
    offer_helper = """fn scientific_offer() -> ProviderEvidenceOffer {
    ProviderEvidenceOffer {
        tenant_workspace_id: "tenant-ws-1".into(),
        artifact_id: "artifact-quarterly-review-1".into(),
        opaque_analytical_id: "entity-opaque-42".into(),
        source_text: Some("Q3 pipeline slipped after the Acme renewal stalled.".into()),
        identity_mapping: None,
        membership_role: Some("author".into()),
    }
}
"""
    audit_helper = offer_helper + f"""
#[derive(Default)]
struct RecordingAuditSink {{
    records: Vec<ReidentificationAuditRecord>,
}}

impl ReidentificationAuditSink for RecordingAuditSink {{
    fn append_reidentification_audit(
        &mut self,
        record: &ReidentificationAuditRecord,
    ) -> Result<(), ApiError> {{
        self.records.push(record.clone());
        Ok(())
    }}
}}

fn disclose(
    grant: &PurposeGrant,
    mapping: &IdentityMappingRecord,
    decision_time: &str,
) -> Result<(DisclosedIdentityMapping, ReidentificationAuditRecord), ApiError> {{
    let mut sink = RecordingAuditSink::default();
    disclose_identity_mapping_with_audit(
        grant,
        mapping,
        decision_time,
        "{DECISION_DIGEST}",
        &mut sink,
    )
}}
"""
    text = replace_once(text, offer_helper, audit_helper, "provider contract audit helper")
    text = text.replace("disclose_identity_mapping(", "disclose(")
    text = text.replace("disclosed.direct_identity()", "disclosed.0.direct_identity()")
    text = text.replace(
        "disclosed.opaque_analytical_id()", "disclosed.0.opaque_analytical_id()"
    )
    path.write_text(text, encoding="utf-8")


def update_documents() -> None:
    """Keep the five-condition grant matrix and audit evidence wording aligned."""
    replacements = [
        (
            "docs/API_CONTRACT.md",
            "refuses expired, impossible-calendar, or cross-tenant grants",
            "refuses expired, not-yet-valid, inverted, cross-tenant, or impossible-calendar grants",
        ),
        (
            "docs/research/task-12-versioned-api-contracts.md",
            "expired-purpose denial, provider mapping refusal, and elevated re-identification;",
            "expired, not-yet-valid, inverted, cross-tenant, and impossible-calendar grant denial; provider mapping refusal; and audited elevated re-identification replay;",
        ),
        (
            "docs/validation/temporal-event-foundation.md",
            "expired/impossible-calendar grant, mapping refusal, elevated re-id",
            "expired/not-yet-valid/inverted/cross-tenant/impossible-calendar grant, mapping refusal, audited elevated re-id replay",
        ),
    ]
    for path_string, old, new in replacements:
        path = Path(path_string)
        text = path.read_text(encoding="utf-8")
        path.write_text(replace_once(text, old, new, path_string), encoding="utf-8")


update_provider_module()
update_public_exports()
update_contract_tests()
update_documents()
