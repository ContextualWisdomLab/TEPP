//! Purpose-bound provider payloads refuse identity mappings and expired grants.

use tepp_api::{
    AnalyticalPurpose, ApiError, DisclosedIdentityMapping, IdentityMappingRecord,
    ProviderEvidenceOffer, PurposeGrant, ReidentificationAuditRecord, ReidentificationAuditSink,
    disclose_identity_mapping as disclose_identity_mapping_with_audit, minimize_provider_payload,
};

fn active_grant(purpose: AnalyticalPurpose, reidentification: bool) -> PurposeGrant {
    PurposeGrant {
        tenant_workspace_id: "tenant-ws-1".into(),
        principal_id: "principal-analyst-1".into(),
        purpose,
        valid_from: "2026-01-01T00:00:00Z".into(),
        valid_to: Some("2026-12-31T23:59:59Z".into()),
        reidentification_authorized: reidentification,
    }
}

fn scientific_offer() -> ProviderEvidenceOffer {
    ProviderEvidenceOffer {
        tenant_workspace_id: "tenant-ws-1".into(),
        artifact_id: "artifact-quarterly-review-1".into(),
        opaque_analytical_id: "entity-opaque-42".into(),
        source_text: Some("Q3 pipeline slipped after the Acme renewal stalled.".into()),
        identity_mapping: None,
        membership_role: Some("author".into()),
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

fn disclose(
    grant: &PurposeGrant,
    mapping: &IdentityMappingRecord,
    decision_time: &str,
) -> Result<(DisclosedIdentityMapping, ReidentificationAuditRecord), ApiError> {
    let mut sink = RecordingAuditSink::default();
    disclose_identity_mapping_with_audit(grant, mapping, decision_time, &mut sink)
}

#[test]
fn scientific_provider_payload_keeps_opaque_ids_and_roles_without_mapping() {
    let (payload, log) = minimize_provider_payload(
        &active_grant(AnalyticalPurpose::ScientificValidation, false),
        &scientific_offer(),
        "2026-06-15T12:00:00Z",
    )
    .expect("scientific minimize");

    assert_eq!(payload.artifact_id(), "artifact-quarterly-review-1");
    assert_eq!(payload.opaque_analytical_id(), "entity-opaque-42");
    assert_eq!(payload.membership_role(), Some("author"));
    assert_eq!(
        payload.source_text(),
        Some("Q3 pipeline slipped after the Acme renewal stalled.")
    );
    assert!(payload.identity_mapping().is_none());
    assert!(log.included_source_text());
    assert!(!log.included_identity_mapping());
    assert_eq!(log.purpose_wire_name(), "scientific_validation");
    assert!(!log.to_string().contains("Acme"));
}

#[test]
fn operational_monitoring_cannot_receive_source_text() {
    let error = minimize_provider_payload(
        &active_grant(AnalyticalPurpose::OperationalMonitoring, false),
        &scientific_offer(),
        "2026-06-15T12:00:00Z",
    )
    .expect_err("ops source");
    assert_eq!(error, ApiError::AuthorizationDenied);
}

#[test]
fn expired_and_not_yet_valid_grants_fail_closed() {
    let expired = PurposeGrant {
        valid_to: Some("2026-03-01T00:00:00Z".into()),
        ..active_grant(AnalyticalPurpose::ScientificValidation, false)
    };
    assert_eq!(
        minimize_provider_payload(&expired, &scientific_offer(), "2026-06-15T12:00:00Z"),
        Err(ApiError::AuthorizationDenied)
    );

    let future = PurposeGrant {
        valid_from: "2026-07-01T00:00:00Z".into(),
        ..active_grant(AnalyticalPurpose::ScientificValidation, false)
    };
    assert_eq!(
        minimize_provider_payload(&future, &scientific_offer(), "2026-06-15T12:00:00Z"),
        Err(ApiError::AuthorizationDenied)
    );
}

#[test]
fn open_ended_grant_stays_valid_and_cross_tenant_is_denied() {
    let open = PurposeGrant {
        valid_to: None,
        ..active_grant(AnalyticalPurpose::ModularServiceConsumer, false)
    };
    let (payload, _) =
        minimize_provider_payload(&open, &scientific_offer(), "2027-01-02T00:00:00Z")
            .expect("open grant");
    assert_eq!(payload.opaque_analytical_id(), "entity-opaque-42");

    let foreign = ProviderEvidenceOffer {
        tenant_workspace_id: "tenant-ws-other".into(),
        ..scientific_offer()
    };
    assert_eq!(
        minimize_provider_payload(
            &active_grant(AnalyticalPurpose::ScientificValidation, false),
            &foreign,
            "2026-06-15T12:00:00Z",
        ),
        Err(ApiError::AuthorizationDenied)
    );
}

#[test]
fn identity_mapping_never_enters_a_provider_payload() {
    let mut offer = scientific_offer();
    offer.identity_mapping = Some("Jane Roe <jane.roe@acme.example>".into());
    assert_eq!(
        minimize_provider_payload(
            &active_grant(AnalyticalPurpose::ScientificValidation, true),
            &offer,
            "2026-06-15T12:00:00Z",
        ),
        Err(ApiError::AuthorizationDenied)
    );
}

#[test]
fn reidentification_is_a_separate_elevated_path() {
    let mapping = IdentityMappingRecord {
        tenant_workspace_id: "tenant-ws-1".into(),
        opaque_analytical_id: "entity-opaque-42".into(),
        direct_identity: "Jane Roe <jane.roe@acme.example>".into(),
    };

    let disclosed = disclose(
        &active_grant(AnalyticalPurpose::ScientificValidation, true),
        &mapping,
        "2026-06-15T12:00:00Z",
    )
    .expect("elevated");
    assert_eq!(
        disclosed.0.direct_identity(),
        "Jane Roe <jane.roe@acme.example>"
    );
    assert_eq!(disclosed.0.opaque_analytical_id(), "entity-opaque-42");

    assert_eq!(
        disclose(
            &active_grant(AnalyticalPurpose::ScientificValidation, false),
            &mapping,
            "2026-06-15T12:00:00Z",
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        disclose(
            &active_grant(AnalyticalPurpose::OperationalMonitoring, true),
            &mapping,
            "2026-06-15T12:00:00Z",
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        disclose(
            &active_grant(AnalyticalPurpose::ModularServiceConsumer, true),
            &mapping,
            "2026-06-15T12:00:00Z",
        ),
        Err(ApiError::AuthorizationDenied)
    );
}

#[test]
fn empty_identities_and_inverted_windows_are_invalid() {
    let mut grant = active_grant(AnalyticalPurpose::ScientificValidation, false);
    grant.tenant_workspace_id.clear();
    assert_eq!(
        minimize_provider_payload(&grant, &scientific_offer(), "2026-06-15T12:00:00Z"),
        Err(ApiError::InvalidWirePayload)
    );

    let inverted = PurposeGrant {
        valid_from: "2026-12-31T00:00:00Z".into(),
        valid_to: Some("2026-01-01T00:00:00Z".into()),
        ..active_grant(AnalyticalPurpose::ScientificValidation, false)
    };
    assert_eq!(
        minimize_provider_payload(&inverted, &scientific_offer(), "2026-06-15T12:00:00Z"),
        Err(ApiError::InvalidWirePayload)
    );

    assert_eq!(
        minimize_provider_payload(
            &active_grant(AnalyticalPurpose::ScientificValidation, false),
            &scientific_offer(),
            "not-a-timestamp",
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
