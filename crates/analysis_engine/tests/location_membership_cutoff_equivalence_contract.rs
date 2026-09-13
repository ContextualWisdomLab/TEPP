//! Regression for semantically equivalent RFC 3339 cutoff spellings.

use analysis_engine::{
    LOCATION_MEMBERSHIP_MODEL_CONTRACT_VERSION, LOCATION_MEMBERSHIP_OUTPUT_PROFILE,
    LocationMembershipDocument, execute_location_membership_run,
};
use location_membership::LocationKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn available(value: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(value).expect("availability")
}

#[test]
fn equivalent_offset_cutoff_is_the_same_analysis_instant() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "location-membership-cutoff-equivalence".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-location-membership".into(),
        knowledge_cutoff: "2026-08-01T09:00:00+09:00".into(),
        model_contract_version: LOCATION_MEMBERSHIP_MODEL_CONTRACT_VERSION.into(),
        output_profile: LOCATION_MEMBERSHIP_OUTPUT_PROFILE.into(),
    };
    let accepted = AnalysisRunAccepted::new(
        "run-location-membership-cutoff-equivalence",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff");
    let documents = vec![
        LocationMembershipDocument::new(
            "loc-a",
            LocationKind::Location,
            available("2026-07-31T23:59:59Z"),
        )
        .expect("location"),
        LocationMembershipDocument::new(
            "ent-b",
            LocationKind::EntityIdentity,
            available("2026-07-31T23:59:59Z"),
        )
        .expect("entity"),
    ];

    execute_location_membership_run(
        &request,
        &accepted,
        "snapshot-location-membership",
        cutoff,
        &documents,
        "2026-08-02T00:00:00Z",
    )
    .expect("equivalent cutoff instants must be admitted");
}
