//! Tenant, role, and system-time lifetime cannot be replaced by event time or a mask.

use temporal_core::SystemTime;
use tenant_access::{
    AccessGrant, AccessRequest, AccessRole, InMemoryTenantAccessAdapter, PrincipalId,
    TenantAccessAdapter, TenantAccessError, TenantId, access_clock_from_wire,
    refuse_blanket_mask_as_access, tenant_role_recovery_rate,
};
use uuid::Uuid;

fn system_time(stamp: &str) -> SystemTime {
    SystemTime::parse_rfc3339(stamp).expect("rfc3339")
}

fn sample_grant() -> AccessGrant {
    AccessGrant::new(
        TenantId::from_uuid(Uuid::from_u128(11)),
        PrincipalId::from_uuid(Uuid::from_u128(21)),
        AccessRole::AnalysisOperator,
        system_time("2026-01-01T00:00:00Z"),
        Some(system_time("2026-12-31T00:00:00Z")),
    )
    .expect("valid lifetime")
}

fn sample_request(evaluation: &str) -> AccessRequest {
    AccessRequest::new(
        TenantId::from_uuid(Uuid::from_u128(11)),
        PrincipalId::from_uuid(Uuid::from_u128(21)),
        AccessRole::AnalysisOperator,
        system_time(evaluation),
    )
}

#[test]
fn a_grant_refuses_cross_tenant_expired_event_time_and_blanket_mask() {
    let grant = sample_grant();
    assert_eq!(
        grant.authorize(&AccessRequest::new(
            TenantId::from_uuid(Uuid::from_u128(99)),
            PrincipalId::from_uuid(Uuid::from_u128(21)),
            AccessRole::AnalysisOperator,
            system_time("2026-06-01T00:00:00Z"),
        )),
        Err(TenantAccessError::TenantMismatch)
    );
    assert_eq!(
        grant.authorize(&AccessRequest::new(
            TenantId::from_uuid(Uuid::from_u128(11)),
            PrincipalId::from_uuid(Uuid::from_u128(88)),
            AccessRole::AnalysisOperator,
            system_time("2026-06-01T00:00:00Z"),
        )),
        Err(TenantAccessError::PrincipalMismatch)
    );
    assert_eq!(
        grant.authorize(&AccessRequest::new(
            TenantId::from_uuid(Uuid::from_u128(11)),
            PrincipalId::from_uuid(Uuid::from_u128(21)),
            AccessRole::ExportOfficer,
            system_time("2026-06-01T00:00:00Z"),
        )),
        Err(TenantAccessError::RoleNotGranted)
    );
    assert_eq!(
        grant.authorize(&sample_request("2025-12-31T23:59:59Z")),
        Err(TenantAccessError::NotYetValid)
    );
    assert_eq!(
        grant.authorize(&sample_request("2026-12-31T00:00:00Z")),
        Err(TenantAccessError::Expired)
    );
    grant
        .authorize(&sample_request("2026-01-01T00:00:00Z"))
        .expect("inclusive start");
    grant
        .authorize(&sample_request("2026-06-15T12:00:00Z"))
        .expect("inside window");

    assert_eq!(
        access_clock_from_wire("event_time"),
        Err(TenantAccessError::EventTimeCannotAuthorize)
    );
    assert_eq!(
        access_clock_from_wire("document_time"),
        Err(TenantAccessError::EventTimeCannotAuthorize)
    );
    assert_eq!(
        access_clock_from_wire("available_time"),
        Err(TenantAccessError::EventTimeCannotAuthorize)
    );
    assert_eq!(
        access_clock_from_wire("knowledge_cutoff"),
        Err(TenantAccessError::EventTimeCannotAuthorize)
    );
    assert_eq!(
        access_clock_from_wire("marketing_clock"),
        Err(TenantAccessError::UnknownAccessClock)
    );
    access_clock_from_wire("system_time").expect("system");
    access_clock_from_wire("assertion_time").expect("assertion");
    assert_eq!(
        refuse_blanket_mask_as_access(),
        Err(TenantAccessError::BlanketMaskIsNotAuthorization)
    );
}

#[test]
fn inverted_or_empty_lifetimes_fail_closed() {
    let start = system_time("2026-06-01T00:00:00Z");
    let earlier = system_time("2026-05-01T00:00:00Z");
    assert_eq!(
        AccessGrant::new(
            TenantId::from_uuid(Uuid::from_u128(1)),
            PrincipalId::from_uuid(Uuid::from_u128(2)),
            AccessRole::Auditor,
            start,
            Some(start),
        ),
        Err(TenantAccessError::InvertedLifetime)
    );
    assert_eq!(
        AccessGrant::new(
            TenantId::from_uuid(Uuid::from_u128(1)),
            PrincipalId::from_uuid(Uuid::from_u128(2)),
            AccessRole::PrivacyOfficer,
            start,
            Some(earlier),
        ),
        Err(TenantAccessError::InvertedLifetime)
    );
}

#[test]
fn in_memory_adapter_allows_one_of_several_roles_for_the_same_principal() {
    let analysis = sample_grant();
    let audit = AccessGrant::new(
        TenantId::from_uuid(Uuid::from_u128(11)),
        PrincipalId::from_uuid(Uuid::from_u128(21)),
        AccessRole::Auditor,
        system_time("2026-01-01T00:00:00Z"),
        None,
    )
    .expect("open-ended");
    let adapter = InMemoryTenantAccessAdapter::from_grants(vec![analysis, audit]);
    adapter
        .evaluate(&sample_request("2026-03-01T00:00:00Z"))
        .expect("analysis grant");
    adapter
        .evaluate(&AccessRequest::new(
            TenantId::from_uuid(Uuid::from_u128(11)),
            PrincipalId::from_uuid(Uuid::from_u128(21)),
            AccessRole::Auditor,
            system_time("2027-01-01T00:00:00Z"),
        ))
        .expect("open-ended auditor");
    assert_eq!(
        adapter.evaluate(&AccessRequest::new(
            TenantId::from_uuid(Uuid::from_u128(11)),
            PrincipalId::from_uuid(Uuid::from_u128(21)),
            AccessRole::ExportOfficer,
            system_time("2026-03-01T00:00:00Z"),
        )),
        Err(TenantAccessError::NoMatchingGrant)
    );
    assert_eq!(
        InMemoryTenantAccessAdapter::from_grants(Vec::new())
            .evaluate(&sample_request("2026-03-01T00:00:00Z")),
        Err(TenantAccessError::InvalidAccessPayload)
    );
}

#[test]
fn recovered_tenant_roles_match_known_truth_better_than_a_collapsed_role() {
    let tenant_a = TenantId::from_uuid(Uuid::from_u128(1));
    let tenant_b = TenantId::from_uuid(Uuid::from_u128(2));
    let truth = [
        (tenant_a, AccessRole::AnalysisOperator),
        (tenant_a, AccessRole::Auditor),
        (tenant_b, AccessRole::PrivacyOfficer),
    ];
    let recovered = truth;
    let collapsed = [
        (tenant_a, AccessRole::AnalysisOperator),
        (tenant_a, AccessRole::AnalysisOperator),
        (tenant_a, AccessRole::AnalysisOperator),
    ];
    let recovered_rate = tenant_role_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = tenant_role_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_row, decided_row) in truth.iter().zip(recovered.iter()) {
            if truth_row == decided_row {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
    assert_eq!(
        tenant_role_recovery_rate(&[], &[]),
        Err(TenantAccessError::InvalidAccessPayload)
    );
    assert_eq!(
        tenant_role_recovery_rate(&truth, &truth[..1]),
        Err(TenantAccessError::InvalidAccessPayload)
    );
}
