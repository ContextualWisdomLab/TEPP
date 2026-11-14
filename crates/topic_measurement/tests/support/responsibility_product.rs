//! Regressions for count conservation on public finite count inputs.

use super::{
    ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix, expectation,
    fit_reference_topic_model,
};
use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use uuid::Uuid;

/// Parse a deterministic UTC event time for the cutoff-safe corpus.
fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("event time")
}

/// Build an observed forward transition between the supplied document identities.
fn relation(source: Uuid, target: Uuid, source_day: u8, target_day: u8) -> RelationEdge {
    let interval = |day| {
        TemporalInterval::bounded(
            TemporalBoundary::Included(event_time(day)),
            TemporalBoundary::Included(
                EventTime::parse_rfc3339(&format!("2026-01-{day:02}T12:00:00Z"))
                    .expect("interval end"),
            ),
            TemporalPrecision::Second,
        )
        .expect("bounded interval")
    };
    RelationEdge::new(
        RelationKind::TransitionsTo,
        RelationEndpointId::from_uuid(source),
        RelationEndpointId::from_uuid(target),
        RelationEvidenceStatus::Observed,
        interval(source_day),
        interval(target_day),
    )
    .expect("forward relation")
}

/// Build six eligible documents with crossed organization/project memberships.
fn fixture() -> (
    CorpusSnapshot,
    Vec<Uuid>,
    Vec<EventTime>,
    MembershipNetwork,
    RelationGraph,
) {
    let document_ids: Vec<_> = (1_u128..=6).map(Uuid::from_u128).collect();
    let times: Vec<_> = (1_u8..=6).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339("2026-01-10T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff");
    let mut snapshot = CorpusSnapshot::new();
    for id in &document_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*id, available), &cutoff)
            .expect("eligible");
    }

    let organization = GroupId::from_uuid(Uuid::from_u128(100));
    let projects = [
        GroupId::from_uuid(Uuid::from_u128(101)),
        GroupId::from_uuid(Uuid::from_u128(102)),
    ];
    let validity_start = event_time(1);
    let validity_end = event_time(9);
    let mut memberships = MembershipNetwork::new();
    for (index, id) in document_ids.iter().enumerate() {
        let member = MemberId::from_uuid(*id);
        memberships
            .insert(
                MembershipAssignment::new(
                    member,
                    organization,
                    MembershipRole::Organization,
                    MembershipWeight::full().expect("full"),
                    validity_start,
                    validity_end,
                )
                .expect("organization membership"),
            )
            .expect("insert organization");
        memberships
            .insert(
                MembershipAssignment::new(
                    member,
                    projects[usize::from(index >= 3)],
                    MembershipRole::Project,
                    MembershipWeight::new(0.75).expect("partial"),
                    validity_start,
                    validity_end,
                )
                .expect("project membership"),
            )
            .expect("insert project");
    }

    let mut relations = RelationGraph::new();
    for (source, target, source_day, target_day) in [
        (0, 1, 1, 2),
        (1, 2, 2, 3),
        (2, 3, 3, 4),
        (3, 4, 4, 5),
        (4, 5, 5, 6),
    ] {
        relations
            .insert(relation(
                document_ids[source],
                document_ids[target],
                source_day,
                target_day,
            ))
            .expect("insert relation");
    }
    (snapshot, document_ids, times, memberships, relations)
}

/// Fit the public CPU reference with identical finite CSR or CSC rare-term counts.
fn fit_tiny_term(count: f64, csc: bool) -> (ReferenceTopicInput, super::ReferenceTopicModel) {
    let (snapshot, ids, times, memberships, relations) = fixture();
    let matrix = if csc {
        SparseMatrix::from_csc(
            6,
            2,
            vec![0, 6, 12],
            vec![0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5],
            [vec![100.0; 6], vec![count; 6]].concat(),
        )
    } else {
        SparseMatrix::from_csr(
            6,
            2,
            vec![0, 2, 4, 6, 8, 10, 12],
            vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            (0..6).flat_map(|_| [100.0, count]).collect(),
        )
    }
    .expect("finite positive canonical matrix");
    let input = ReferenceTopicInput::new(
        &snapshot,
        ids,
        &matrix,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("actual public constructor admits counts");
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2000, 1e-5)
        .unwrap()
        .with_hyperparameters(1.0, 0.25, 0.01, 1e-300, 0.2)
        .unwrap();
    let fit =
        fit_reference_topic_model(&input, &config).expect("public fit must return finite model");
    assert!(fit.objective.is_finite());
    assert!(input.in_sample_log_likelihood(&fit).unwrap().is_finite());
    assert!(
        fit.topic_term_probabilities
            .iter()
            .all(|row| row.len() == 2 && row.iter().all(|x| x.is_finite() && *x > 0.0))
    );
    (input, fit)
}
/// Each observed term distributes all its representable count across topics.
fn assert_rare_count_conservation(count: f64, csc: bool) {
    let (input, fit) = fit_tiny_term(count, csc);
    let (_, counts, ll) = expectation(
        &input,
        &fit.document_topic_proportions,
        &fit.topic_term_probabilities,
        2,
    )
    .expect("actual unchanged expectation on public fitted state");
    let actual = counts.iter().map(|row| row[1]).sum::<f64>();
    let expected = 6.0 * count;
    println!(
        "csc={csc} count={count:e} expected_rare_total={expected:e} actual_rare_total={actual:e} beta={:?} likelihood={ll:e}",
        fit.topic_term_probabilities
    );
    assert!(expected.is_finite() && expected > 0.0);
    assert!(ll.is_finite());
    assert!(
        (actual / expected - 1.0).abs() < 1e-12,
        "real expectation must conserve representable rare counts: actual={actual:e} expected={expected:e}"
    );
}
/// Preserve aggregate rare-term counts when all intermediate products are normal.
#[test]
fn normal_product_control_preserves_rare_term_counts() {
    for csc in [false, true] {
        assert_rare_count_conservation(1e-100, csc);
    }
}
/// Preserve representable rare counts even when a count-first product underflows.
#[test]
fn representable_rare_term_counts_survive_intermediate_products() {
    for csc in [false, true] {
        assert_rare_count_conservation(1e-200, csc);
    }
}
