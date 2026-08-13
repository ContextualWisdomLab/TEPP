//! Estimators must keep every active membership row (no atomistic collapse).

use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipError, MembershipNetwork, MembershipRole,
    MembershipWeight, refuse_atomistic_collapse,
};
use temporal_core::EventTime;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("event time")
}

#[test]
fn one_document_emits_three_estimation_rows_and_refuses_collapse() {
    let document = MemberId::new();
    let author = GroupId::new();
    let department = GroupId::new();
    let project = GroupId::new();
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T23:59:59Z");
    let as_of = event_time("2026-06-15T12:00:00Z");

    let mut network = MembershipNetwork::new();
    for (group, role, weight) in [
        (author, MembershipRole::Author, 1.0),
        (department, MembershipRole::Department, 0.5),
        (project, MembershipRole::Project, 0.5),
    ] {
        network
            .insert(
                MembershipAssignment::new(
                    document,
                    group,
                    role,
                    MembershipWeight::new(weight).expect("weight"),
                    start,
                    end,
                )
                .expect("assignment"),
            )
            .expect("insert");
    }

    let rows = network.estimation_rows_at(document, as_of).expect("rows");
    assert_eq!(rows.len(), 3);
    let recovered_weight: f64 = rows.iter().map(|row| row.weight()).sum();
    let truth_weight = 2.0;
    let rmse = {
        let residual = recovered_weight - truth_weight;
        (residual * residual).sqrt()
    };
    assert!(rmse < 1e-15, "weight recovery RMSE {rmse}");
    let multiplicity = network.active_group_multiplicity(document, as_of);
    refuse_atomistic_collapse(&rows, multiplicity).expect("keep all rows");

    let collapsed = vec![rows[0]];
    assert_eq!(
        refuse_atomistic_collapse(&collapsed, multiplicity),
        Err(MembershipError::AtomisticCollapseRefused)
    );
}

#[test]
fn empty_or_inactive_membership_fails_closed() {
    let network = MembershipNetwork::new();
    let member = MemberId::new();
    let as_of = event_time("2026-06-15T12:00:00Z");
    assert_eq!(
        network.estimation_rows_at(member, as_of),
        Err(MembershipError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_atomistic_collapse(&[], 1),
        Err(MembershipError::InvalidWirePayload)
    );
}
