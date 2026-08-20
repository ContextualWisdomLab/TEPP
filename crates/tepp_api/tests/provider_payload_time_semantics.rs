//! Semantic RFC 3339 validation contracts for provider-purpose grants.

use tepp_api::{
    AnalyticalPurpose, ApiError, ProviderEvidenceOffer, PurposeGrant, minimize_provider_payload,
};

fn grant() -> PurposeGrant {
    PurposeGrant {
        tenant_workspace_id: "tenant-workspace".into(),
        principal_id: "principal-analyst".into(),
        purpose: AnalyticalPurpose::ScientificValidation,
        valid_from: "2026-01-01T00:00:00Z".into(),
        valid_to: Some("2026-12-31T23:59:59Z".into()),
        reidentification_authorized: false,
    }
}

fn offer() -> ProviderEvidenceOffer {
    ProviderEvidenceOffer {
        tenant_workspace_id: "tenant-workspace".into(),
        artifact_id: "artifact-record".into(),
        opaque_analytical_id: "analytical-identity".into(),
        source_text: None,
        identity_mapping: None,
        membership_role: Some("project_member".into()),
    }
}

#[test]
fn provider_payload_rejects_semantically_invalid_utc_instants() {
    for invalid_decision_time in [
        "2026-00-01T00:00:00Z",
        "2026-13-01T00:00:00Z",
        "2026-02-30T00:00:00Z",
        "2026-04-31T00:00:00Z",
        "2026-01-01T24:00:00Z",
        "2026-01-01T00:60:00Z",
        "2026-01-01T00:00:60Z",
    ] {
        assert_eq!(
            minimize_provider_payload(&grant(), &offer(), invalid_decision_time),
            Err(ApiError::InvalidWirePayload),
            "invalid instant must fail closed: {invalid_decision_time}",
        );
    }
}

#[test]
fn provider_payload_enforces_gregorian_leap_year_semantics() {
    for valid_leap_day in ["2000-02-29T12:00:00Z", "2028-02-29T12:00:00Z"] {
        let leap_grant = PurposeGrant {
            valid_from: valid_leap_day.into(),
            valid_to: Some(valid_leap_day.into()),
            ..grant()
        };
        minimize_provider_payload(&leap_grant, &offer(), valid_leap_day)
            .expect("Gregorian leap day must be accepted");
    }

    for invalid_leap_day in ["2027-02-29T00:00:00Z", "2100-02-29T00:00:00Z"] {
        let false_leap_grant = PurposeGrant {
            valid_from: invalid_leap_day.into(),
            valid_to: None,
            ..grant()
        };
        assert_eq!(
            minimize_provider_payload(&false_leap_grant, &offer(), "2100-03-01T00:00:00Z"),
            Err(ApiError::InvalidWirePayload),
            "non-leap-century and ordinary non-leap years must fail closed: {invalid_leap_day}",
        );
    }
}
