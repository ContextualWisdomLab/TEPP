//! Fitted candidate-`K` scoring uses real TRSL-TM reference fits.

use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{
    FittedCandidateKConfig, ModelSelectionError, select_fitted_candidate_k,
    selected_k_root_mean_square_error, statistical_candidate_from_fit,
};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use topic_measurement::{ReferenceTopicInput, ReferenceTopicModel, SparseMatrix};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("event time")
}

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

fn separated_topic_input() -> ReferenceTopicInput {
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

    let counts = SparseMatrix::from_csr(
        6,
        4,
        vec![0, 2, 4, 6, 8, 10, 12],
        vec![0, 1, 0, 1, 0, 1, 2, 3, 2, 3, 2, 3],
        vec![
            90.0, 10.0, 85.0, 15.0, 80.0, 20.0, 10.0, 90.0, 15.0, 85.0, 20.0, 80.0,
        ],
    )
    .expect("counts");
    ReferenceTopicInput::new(
        &snapshot,
        document_ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("validated input")
}

fn two_document_input(term_values: [f64; 4]) -> ReferenceTopicInput {
    let document_ids = vec![Uuid::from_u128(1), Uuid::from_u128(2)];
    let times = vec![event_time(1), event_time(2)];
    let available = AvailableTime::parse_rfc3339("2026-01-10T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff");
    let mut snapshot = CorpusSnapshot::new();
    for id in &document_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*id, available), &cutoff)
            .expect("eligible");
    }
    let mut memberships = MembershipNetwork::new();
    memberships
        .insert(
            MembershipAssignment::new(
                MemberId::from_uuid(document_ids[0]),
                GroupId::from_uuid(Uuid::from_u128(100)),
                MembershipRole::Organization,
                MembershipWeight::full().expect("full"),
                event_time(1),
                event_time(9),
            )
            .expect("membership"),
        )
        .expect("insert");
    memberships
        .insert(
            MembershipAssignment::new(
                MemberId::from_uuid(document_ids[1]),
                GroupId::from_uuid(Uuid::from_u128(100)),
                MembershipRole::Organization,
                MembershipWeight::full().expect("full"),
                event_time(1),
                event_time(9),
            )
            .expect("membership"),
        )
        .expect("insert");
    let mut relations = RelationGraph::new();
    relations
        .insert(relation(document_ids[0], document_ids[1], 1, 2))
        .expect("relation");
    let counts =
        SparseMatrix::from_csr(2, 2, vec![0, 2, 4], vec![0, 1, 0, 1], term_values.to_vec())
            .expect("counts");
    ReferenceTopicInput::new(
        &snapshot,
        document_ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("two-document input")
}

fn recovery_config(seeds: Vec<u64>) -> FittedCandidateKConfig {
    FittedCandidateKConfig::new(vec![2, 3], seeds, 2_000, 1e-5)
        .expect("candidate configuration")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters")
}

#[test]
fn fitted_diagnostics_select_true_k_over_overspecified_and_llm_only() {
    let input = separated_topic_input();
    let config = recovery_config(vec![7, 11, 19]);
    let selected = select_fitted_candidate_k(&input, &config, "trsl_tm_reference", &[3])
        .expect("fitted statistical selection");
    assert_eq!(selected, 2);
}

#[test]
fn selected_k_rmse_on_fitted_replications_recovers_true_k() {
    let input = separated_topic_input();
    let selected: Vec<u32> = [vec![7, 11, 19], vec![3, 5, 13], vec![17, 23, 29]]
        .into_iter()
        .map(|seeds| {
            select_fitted_candidate_k(
                &input,
                &recovery_config(seeds),
                "tepp_topic_measurement",
                &[3],
            )
            .expect("replication")
        })
        .collect();
    assert_eq!(selected, vec![2, 2, 2]);
    let rmse = selected_k_root_mean_square_error(&selected, 2).expect("selected-K RMSE");
    assert!(rmse.abs() < f64::EPSILON);
}

#[test]
fn failed_and_non_finite_fits_fail_closed() {
    let input = separated_topic_input();
    let exhausted = FittedCandidateKConfig::new(vec![2], vec![1], 2, 1e-12).expect("exhausted");
    assert_eq!(
        select_fitted_candidate_k(&input, &exhausted, "trsl_tm_reference", &[]),
        Err(ModelSelectionError::NoSuccessfulFit)
    );

    let oversized = FittedCandidateKConfig::new(vec![5], vec![1], 10, 1e-6).expect("K > V");
    assert_eq!(
        select_fitted_candidate_k(&input, &oversized, "trsl_tm_reference", &[]),
        Err(ModelSelectionError::NoSuccessfulFit)
    );

    let unstable = FittedCandidateKConfig::new(vec![2], vec![1], 10, 1e-6)
        .expect("unstable")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, f64::MAX)
        .expect("finite-declared step");
    assert_eq!(
        select_fitted_candidate_k(&input, &unstable, "trsl_tm_reference", &[]),
        Err(ModelSelectionError::NoSuccessfulFit)
    );

    let only_failed_plus_llm =
        FittedCandidateKConfig::new(vec![5], vec![1], 10, 1e-6).expect("failed statistical");
    assert_eq!(
        select_fitted_candidate_k(&input, &only_failed_plus_llm, "trsl_tm_reference", &[3]),
        Err(ModelSelectionError::LlmVoteIsNotStatisticalAuthority)
    );
}

#[test]
fn lexical_and_llm_label_methods_are_refused() {
    let input = separated_topic_input();
    let config = FittedCandidateKConfig::new(vec![2], vec![1], 10, 1e-6).expect("config");
    for method in [
        "tf-idf",
        "tfidf",
        "BM25",
        "bm25",
        "keyword",
        "stopword-deletion",
        "stopword",
        "stopwords",
        "llm-label",
        "llm-labels",
        "llm",
        "llm_vote",
        "llm_vote_only",
        "",
    ] {
        assert_eq!(
            select_fitted_candidate_k(&input, &config, method, &[]),
            Err(ModelSelectionError::LexicalWeightForbidden)
        );
    }
}

#[test]
fn invalid_fitted_configuration_fails_closed() {
    assert_eq!(
        FittedCandidateKConfig::new(vec![], vec![1], 10, 1e-6),
        Err(ModelSelectionError::EmptyCandidateSet)
    );
    assert_eq!(
        FittedCandidateKConfig::new(vec![1], vec![1], 10, 1e-6),
        Err(ModelSelectionError::NonPositiveCandidateK)
    );
    assert_eq!(
        FittedCandidateKConfig::new(vec![2, 2], vec![1], 10, 1e-6),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
    assert_eq!(
        FittedCandidateKConfig::new(vec![2], vec![], 10, 1e-6),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
    assert_eq!(
        FittedCandidateKConfig::new(vec![2], vec![1], 1, 1e-6),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
    assert_eq!(
        FittedCandidateKConfig::new(vec![2], vec![1], 10, f64::NAN),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
    assert_eq!(
        FittedCandidateKConfig::new(vec![2], vec![1], 10, 0.0),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
    assert_eq!(
        FittedCandidateKConfig::new(vec![2], vec![1], 10, f64::INFINITY),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
    let base = FittedCandidateKConfig::new(vec![2], vec![1], 10, 1e-6).expect("base");
    for values in [
        (f64::NAN, 0.5, 0.01, 0.05, 0.2),
        (f64::INFINITY, 0.5, 0.01, 0.05, 0.2),
        (0.0, 0.5, 0.01, 0.05, 0.2),
        (1.0, f64::NAN, 0.01, 0.05, 0.2),
        (1.0, f64::INFINITY, 0.01, 0.05, 0.2),
        (1.0, -1.0, 0.01, 0.05, 0.2),
        (1.0, 0.5, f64::NAN, 0.05, 0.2),
        (1.0, 0.5, f64::INFINITY, 0.05, 0.2),
        (1.0, 0.5, -1.0, 0.05, 0.2),
        (1.0, 0.5, 0.01, f64::NAN, 0.2),
        (1.0, 0.5, 0.01, f64::INFINITY, 0.2),
        (1.0, 0.5, 0.01, 0.0, 0.2),
        (1.0, 0.5, 0.01, 0.05, f64::NAN),
        (1.0, 0.5, 0.01, 0.05, f64::INFINITY),
        (1.0, 0.5, 0.01, 0.05, 0.0),
    ] {
        assert_eq!(
            base.clone()
                .with_hyperparameters(values.0, values.1, values.2, values.3, values.4),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
    }
    assert_eq!(
        select_fitted_candidate_k(&separated_topic_input(), &base, "trsl_tm_reference", &[1]),
        Err(ModelSelectionError::NonPositiveCandidateK)
    );
}

#[test]
fn statistical_candidate_from_fit_refuses_unusable_diagnostics() {
    let input = separated_topic_input();
    let matching = ReferenceTopicModel {
        seed: 1,
        iterations: 4,
        objective: -1.0,
        topic_term_probabilities: vec![vec![0.25; 4]; 2],
        document_topic_proportions: vec![vec![0.5, 0.5]; 6],
        document_coordinate_variances: vec![vec![0.1]; 6],
        prevalence_coefficients: vec![vec![0.0]; 5],
        prevalence_features: Vec::new(),
        sequence_edges: Vec::new(),
        connected_post_count: 0,
        lineage_count: 0,
    };
    assert!(statistical_candidate_from_fit(&input, 2, &matching).is_ok());
    assert_eq!(
        statistical_candidate_from_fit(&input, 1, &matching),
        Err(ModelSelectionError::NonPositiveCandidateK)
    );
    let mut short = matching.clone();
    short.document_topic_proportions.pop();
    assert_eq!(
        statistical_candidate_from_fit(&input, 2, &short),
        Err(ModelSelectionError::InvalidDiagnostic)
    );

    let tiny = two_document_input([0.2, 0.2, 0.2, 0.2]);
    let tiny_model = ReferenceTopicModel {
        seed: 1,
        iterations: 4,
        objective: -1.0,
        topic_term_probabilities: vec![vec![0.5, 0.5], vec![0.5, 0.5]],
        document_topic_proportions: vec![vec![0.5, 0.5], vec![0.5, 0.5]],
        document_coordinate_variances: vec![vec![0.1], vec![0.1]],
        prevalence_coefficients: vec![vec![0.0]; 3],
        prevalence_features: Vec::new(),
        sequence_edges: Vec::new(),
        connected_post_count: 0,
        lineage_count: 0,
    };
    assert_eq!(
        statistical_candidate_from_fit(&tiny, 2, &tiny_model),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
}
