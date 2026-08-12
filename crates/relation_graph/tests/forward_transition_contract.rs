//! Forward-transition and provenance contracts for the typed relation graph.

use relation_graph::{
    RelationEdge, RelationEdgeId, RelationEndpointId, RelationError, RelationEvidenceStatus,
    RelationGraph, RelationKind, validate_forward_event_order,
};
use temporal_core::{EventTime, TemporalBoundary, TemporalInterval, TemporalPrecision};

fn closed(start: &str, end: &str) -> TemporalInterval<EventTime> {
    TemporalInterval::bounded(
        TemporalBoundary::Included(EventTime::parse_rfc3339(start).expect("start")),
        TemporalBoundary::Included(EventTime::parse_rfc3339(end).expect("end")),
        TemporalPrecision::Second,
    )
    .expect("interval")
}

#[test]
fn input_process_outcome_chain_is_forward_only() {
    let input = RelationEndpointId::new();
    let process = RelationEndpointId::new();
    let outcome = RelationEndpointId::new();
    let t0 = closed("2026-03-01T00:00:00Z", "2026-03-01T01:00:00Z");
    let t1 = closed("2026-03-01T01:00:00Z", "2026-03-01T02:00:00Z");
    let t2 = closed("2026-03-01T02:00:00Z", "2026-03-01T03:00:00Z");

    let mut graph = RelationGraph::new();
    let e1 = RelationEdge::new(
        RelationKind::InputTo,
        input,
        process,
        RelationEvidenceStatus::Observed,
        t0,
        t1,
    )
    .expect("input_to");
    let e2 = RelationEdge::new(
        RelationKind::ProcessTo,
        process,
        outcome,
        RelationEvidenceStatus::Observed,
        t1,
        t2,
    )
    .expect("process_to");
    graph.insert(e1).expect("insert input");
    graph.insert(e2).expect("insert process");

    assert_eq!(graph.edge_count(), 2);
    assert_eq!(graph.transition_edge_count(), 2);
    assert!(graph.edges().all(|edge| edge.is_transition_edge()));
}

#[test]
fn reverse_transition_is_rejected_while_citation_may_point_backward() {
    let earlier_event = RelationEndpointId::new();
    let later_report = RelationEndpointId::new();
    let early = closed("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z");
    let late = closed("2026-06-01T00:00:00Z", "2026-06-02T00:00:00Z");

    assert_eq!(
        RelationEdge::new(
            RelationKind::TransitionsTo,
            later_report,
            earlier_event,
            RelationEvidenceStatus::Observed,
            late,
            early,
        ),
        Err(RelationError::ReverseTemporalOrder)
    );

    let citation = RelationEdge::new(
        RelationKind::References,
        later_report,
        earlier_event,
        RelationEvidenceStatus::Observed,
        late,
        early,
    )
    .expect("citation may point backward");
    assert!(!citation.is_transition_edge());
    assert_eq!(citation.kind(), RelationKind::References);
    assert_eq!(citation.evidence_status(), RelationEvidenceStatus::Observed);
    assert_eq!(citation.source(), later_report);
    assert_eq!(citation.target(), earlier_event);
    assert_eq!(citation.source_event_time(), late);
    assert_eq!(citation.target_event_time(), early);
    assert_ne!(citation.edge_id(), RelationEdgeId::new());

    let mut graph = RelationGraph::new();
    let edge_id = citation.edge_id();
    graph.insert(citation).expect("insert citation");
    assert_eq!(graph.transition_edge_count(), 0);
    assert!(graph.edge(edge_id).is_some());
    assert!(graph.edge(RelationEdgeId::new()).is_none());
}

#[test]
fn outcome_of_is_provenance_not_a_reverse_transition() {
    let producer = RelationEndpointId::new();
    let result = RelationEndpointId::new();
    let early = closed("2026-02-01T00:00:00Z", "2026-02-02T00:00:00Z");
    let late = closed("2026-02-03T00:00:00Z", "2026-02-04T00:00:00Z");

    let production = RelationEdge::new(
        RelationKind::Produces,
        producer,
        result,
        RelationEvidenceStatus::Observed,
        early,
        late,
    )
    .expect("produces");
    assert!(production.is_transition_edge());

    let outcome_of = RelationEdge::new(
        RelationKind::OutcomeOf,
        result,
        producer,
        RelationEvidenceStatus::Observed,
        late,
        early,
    )
    .expect("outcome_of points backward as provenance");
    assert!(!outcome_of.is_transition_edge());
}

#[test]
fn self_transition_and_uncertain_order_fail_closed() {
    let node = RelationEndpointId::new();
    let span = closed("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z");
    assert_eq!(
        RelationEdge::new(
            RelationKind::Causes,
            node,
            node,
            RelationEvidenceStatus::Inferred,
            span,
            span,
        ),
        Err(RelationError::SelfTransition)
    );

    let unknown = TemporalInterval::<EventTime>::unknown();
    assert_eq!(
        validate_forward_event_order(&span, &unknown),
        Err(RelationError::UncertainTemporalOrder)
    );
    assert_eq!(
        RelationEdge::new(
            RelationKind::Enables,
            RelationEndpointId::new(),
            RelationEndpointId::new(),
            RelationEvidenceStatus::Inferred,
            span,
            unknown,
        ),
        Err(RelationError::UncertainTemporalOrder)
    );
}

#[test]
fn observed_and_inferred_edges_remain_distinct_in_the_graph() {
    let source = RelationEndpointId::new();
    let target = RelationEndpointId::new();
    let early = closed("2026-04-01T00:00:00Z", "2026-04-02T00:00:00Z");
    let late = closed("2026-04-03T00:00:00Z", "2026-04-04T00:00:00Z");

    let observed = RelationEdge::new(
        RelationKind::Supports,
        source,
        target,
        RelationEvidenceStatus::Observed,
        late,
        early,
    )
    .expect("observed support");
    let inferred = RelationEdge::new(
        RelationKind::Supports,
        source,
        target,
        RelationEvidenceStatus::Inferred,
        late,
        early,
    )
    .expect("inferred support");

    let mut graph = RelationGraph::new();
    graph.insert(observed).expect("insert observed");
    graph.insert(inferred).expect("insert inferred");
    assert_eq!(graph.edge_count(), 2);

    let statuses: Vec<_> = graph.edges().map(|edge| edge.evidence_status()).collect();
    assert!(statuses.contains(&RelationEvidenceStatus::Observed));
    assert!(statuses.contains(&RelationEvidenceStatus::Inferred));
}

#[test]
fn duplicate_edge_identity_is_rejected() {
    let source = RelationEndpointId::new();
    let target = RelationEndpointId::new();
    let early = closed("2026-05-01T00:00:00Z", "2026-05-02T00:00:00Z");
    let late = closed("2026-05-03T00:00:00Z", "2026-05-04T00:00:00Z");
    let edge = RelationEdge::new(
        RelationKind::LeadsTo,
        source,
        target,
        RelationEvidenceStatus::Observed,
        early,
        late,
    )
    .expect("edge");

    let mut graph = RelationGraph::new();
    graph.insert(edge).expect("first insert");
    assert_eq!(
        graph.insert(edge),
        Err(RelationError::DuplicateRelationEdge)
    );
}

#[test]
fn diamond_paths_revisit_nodes_and_cycles_are_rejected() {
    let root = RelationEndpointId::new();
    let left = RelationEndpointId::new();
    let right = RelationEndpointId::new();
    let sink = RelationEndpointId::new();
    let t0 = closed("2026-02-01T00:00:00Z", "2026-02-01T01:00:00Z");
    let t1 = closed("2026-02-01T01:00:00Z", "2026-02-01T02:00:00Z");
    let t2 = closed("2026-02-01T02:00:00Z", "2026-02-01T03:00:00Z");
    let t3 = closed("2026-02-01T03:00:00Z", "2026-02-01T04:00:00Z");
    let pre = closed("2026-01-31T00:00:00Z", "2026-01-31T01:00:00Z");

    let mut graph = RelationGraph::new();
    for edge in [
        RelationEdge::new(
            RelationKind::Enables,
            root,
            left,
            RelationEvidenceStatus::Observed,
            t0,
            t1,
        )
        .expect("root->left"),
        RelationEdge::new(
            RelationKind::Enables,
            root,
            right,
            RelationEvidenceStatus::Observed,
            t0,
            t1,
        )
        .expect("root->right"),
        RelationEdge::new(
            RelationKind::LeadsTo,
            left,
            sink,
            RelationEvidenceStatus::Observed,
            t1,
            t2,
        )
        .expect("left->sink"),
        RelationEdge::new(
            RelationKind::LeadsTo,
            right,
            sink,
            RelationEvidenceStatus::Observed,
            t1,
            t2,
        )
        .expect("right->sink"),
    ] {
        graph.insert(edge).expect("diamond edge");
    }

    // New predecessor into the diamond explores root's component looking for
    // seed and revisits sink through both branches before concluding there is
    // no cycle path back to seed.
    let seed = RelationEndpointId::new();
    let seed_to_root = RelationEdge::new(
        RelationKind::TransitionsTo,
        seed,
        root,
        RelationEvidenceStatus::Observed,
        pre,
        t0,
    )
    .expect("seed->root");
    graph
        .insert(seed_to_root)
        .expect("no cycle into diamond root");

    let sink_to_root = RelationEdge::new(
        RelationKind::TransitionsTo,
        sink,
        root,
        RelationEvidenceStatus::Inferred,
        t2,
        t3,
    )
    .expect("sink->root local order ok");
    assert_eq!(
        graph.insert(sink_to_root),
        Err(RelationError::TransitionCycle)
    );

    let summary = RelationEdge::new(
        RelationKind::Summarizes,
        sink,
        root,
        RelationEvidenceStatus::Observed,
        t2,
        t0,
    )
    .expect("summarize may point backward");
    graph.insert(summary).expect("provenance insert");
    assert_eq!(graph.transition_edge_count(), 5);
    assert_eq!(graph.edge_count(), 6);
}
