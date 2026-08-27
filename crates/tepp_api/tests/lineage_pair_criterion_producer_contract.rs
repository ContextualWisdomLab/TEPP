//! Synthetic contracts for independent pair evidence and branching journeys.

use tepp_api::{
    ApiError, LINEAGE_PAIR_CRITERION_POSTERIOR_SCHEMA, LineageAnchorBasis, LineageComputeReceipt,
    LineageComputeReceipts, LineageDrawProvenance, LineagePairCriterionPosterior,
    LineagePairCriterionPosteriorArtifact, LineageTemporalProvenance,
    PROJECT_JOURNEY_POSTERIOR_SCHEMA, ProjectJourneyEventPosterior,
    ProjectJourneyPosteriorArtifact, ProjectJourneyRelationPosterior,
};

fn digest(character: char) -> String {
    character.to_string().repeat(64)
}

fn pair_artifact() -> LineagePairCriterionPosteriorArtifact {
    let receipt = |backend: &str| LineageComputeReceipt {
        backend_code: backend.into(),
        execution_environment_code: if backend == "mlx_metal_macos_native" {
            "macos_native".into()
        } else {
            "linux_container".into()
        },
        objective_sha256: digest('a'),
        parameter_sha256: digest('b'),
        draw_sha256: digest('c'),
        observed_maximum_difference: if backend == "rust_cpu" { 0.0 } else { 5.0e-9 },
    };
    LineagePairCriterionPosteriorArtifact {
        schema_version: LINEAGE_PAIR_CRITERION_POSTERIOR_SCHEMA.into(),
        estimation_run_id: "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b1".into(),
        tepp_run_id: "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b2".into(),
        source_snapshot_sha256: digest('d'),
        knowledge_cutoff: "2026-08-25T00:00:00Z".into(),
        channel_codes: vec!["temporal".into(), "text".into()],
        admitted_pair_ids: vec!["018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b3".into()],
        anchor_basis: LineageAnchorBasis {
            basis_id: "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b4".into(),
            basis_sha256: digest('e'),
            alignment_status: "unique".into(),
            tie_count: 0,
        },
        temporal_provenance: LineageTemporalProvenance {
            method_code: "TDT_CHRONOS_JOINT".into(),
            configuration_sha256: digest('f'),
            event_clock_code: "event_valid_time".into(),
            temporal_dependency_sha256: digest('1'),
            branch_transition_sha256: digest('2'),
        },
        draw_provenance: LineageDrawProvenance {
            seed_domain: "independent-lineage-criterion".into(),
            draw_count: 2,
        },
        pair_posteriors: vec![LineagePairCriterionPosterior {
            pair_id: "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b3".into(),
            predecessor_record_id: "record-a".into(),
            successor_record_id: "record-b".into(),
            predecessor_record_created_at: "2026-01-03T00:00:00Z".into(),
            predecessor_available_at: "2026-01-04T00:00:00Z".into(),
            successor_record_created_at: "2026-01-01T00:00:00Z".into(),
            successor_available_at: "2026-01-05T00:00:00Z".into(),
            predecessor_event_time_draws: vec![
                "2025-12-01T00:00:00Z".into(),
                "2025-12-02T00:00:00Z".into(),
            ],
            successor_event_time_draws: vec![
                "2025-12-10T00:00:00Z".into(),
                "2025-12-11T00:00:00Z".into(),
            ],
            criterion_draws: vec![0.35, 0.65],
        }],
        compute_receipts: LineageComputeReceipts {
            cpu: receipt("rust_cpu"),
            gpu: receipt("mlx_metal_macos_native"),
            parity_method_code: "producer_method_derived_v1".into(),
            parity_bound: 1.0e-8,
        },
    }
}

#[test]
fn pair_posterior_preserves_uncertainty_and_separate_clocks() {
    let artifact = pair_artifact();
    let json = artifact.to_json().expect("valid producer artifact");
    let parsed = LineagePairCriterionPosteriorArtifact::from_json(&json).expect("round trip");
    assert_eq!(parsed, artifact);
    assert_ne!(
        parsed.pair_posteriors[0].predecessor_record_created_at,
        parsed.pair_posteriors[0].predecessor_event_time_draws[0]
    );
    assert_eq!(parsed.pair_posteriors[0].criterion_draws, vec![0.35, 0.65]);
}

#[test]
fn pair_posterior_rejects_temporal_reversal_anchor_ties_and_gpu_divergence() {
    let mut reversed = pair_artifact();
    reversed.pair_posteriors[0].predecessor_event_time_draws[0] = "2026-01-01T00:00:00Z".into();
    assert_eq!(reversed.to_json(), Err(ApiError::InvalidWirePayload));

    let mut tied = pair_artifact();
    tied.anchor_basis.tie_count = 1;
    assert_eq!(tied.to_json(), Err(ApiError::InvalidWirePayload));

    let mut divergent = pair_artifact();
    divergent.compute_receipts.gpu.observed_maximum_difference = 2.0e-8;
    assert_eq!(divergent.to_json(), Err(ApiError::InvalidWirePayload));

    let mut forged_metal = pair_artifact();
    forged_metal.compute_receipts.gpu.execution_environment_code = "linux_container".into();
    assert_eq!(forged_metal.to_json(), Err(ApiError::InvalidWirePayload));

    let mut duplicate_admission = pair_artifact();
    duplicate_admission
        .admitted_pair_ids
        .push(duplicate_admission.admitted_pair_ids[0].clone());
    assert_eq!(
        duplicate_admission.to_json(),
        Err(ApiError::InvalidWirePayload)
    );

    let mut future_evidence = pair_artifact();
    future_evidence.pair_posteriors[0].predecessor_available_at = "2026-08-26T00:00:00Z".into();
    assert_eq!(future_evidence.to_json(), Err(ApiError::InvalidWirePayload));
}

fn journey() -> ProjectJourneyPosteriorArtifact {
    let event = |id: &str, kind: &str, created: &str, first: &str, second: &str| {
        ProjectJourneyEventPosterior {
            event_id: id.into(),
            event_type_code: kind.into(),
            record_created_at: created.into(),
            available_at: created.into(),
            event_time_draws: vec![first.into(), second.into()],
            evidence_record_ids: vec![format!("evidence-{id}")],
        }
    };
    ProjectJourneyPosteriorArtifact {
        schema_version: PROJECT_JOURNEY_POSTERIOR_SCHEMA.into(),
        tepp_run_id: "journey-run-1".into(),
        source_snapshot_sha256: digest('a'),
        knowledge_cutoff: "2026-08-25T00:00:00Z".into(),
        draw_count: 2,
        inference_status: "posterior_temporal_relation_not_causal".into(),
        events: vec![
            event(
                "request",
                "customer_request",
                "2026-03-03T00:00:00Z",
                "2026-01-01T00:00:00Z",
                "2026-01-02T00:00:00Z",
            ),
            event(
                "notice",
                "procurement_notice",
                "2026-03-01T00:00:00Z",
                "2026-01-03T00:00:00Z",
                "2026-01-04T00:00:00Z",
            ),
            event(
                "sensing",
                "external_sensing",
                "2026-02-01T00:00:00Z",
                "2026-01-01T00:00:00Z",
                "2026-01-01T00:00:00Z",
            ),
            event(
                "bid",
                "negotiated_bid",
                "2026-02-02T00:00:00Z",
                "2026-01-10T00:00:00Z",
                "2026-01-11T00:00:00Z",
            ),
        ],
        relations: vec![
            ProjectJourneyRelationPosterior {
                relation_id: "r1".into(),
                predecessor_event_id: "request".into(),
                successor_event_id: "bid".into(),
                relation_type_code: "precedes".into(),
                relation_draws: vec![true, true],
                evidence_record_ids: vec!["e1".into()],
            },
            ProjectJourneyRelationPosterior {
                relation_id: "r2".into(),
                predecessor_event_id: "notice".into(),
                successor_event_id: "bid".into(),
                relation_type_code: "enables".into(),
                relation_draws: vec![true, false],
                evidence_record_ids: vec!["e2".into()],
            },
            ProjectJourneyRelationPosterior {
                relation_id: "r3".into(),
                predecessor_event_id: "sensing".into(),
                successor_event_id: "bid".into(),
                relation_type_code: "informs".into(),
                relation_draws: vec![false, true],
                evidence_record_ids: vec!["e3".into()],
            },
        ],
    }
}

#[test]
fn journey_preserves_multiple_predecessors_branches_ties_and_record_time_disagreement() {
    let artifact = journey();
    let parsed = ProjectJourneyPosteriorArtifact::from_json(&artifact.to_json().expect("json"))
        .expect("valid journey");
    assert_eq!(parsed.relations.len(), 3);
    assert_eq!(parsed.relations[1].relation_draws, vec![true, false]);
    assert_eq!(
        parsed.events[0].event_time_draws[0],
        parsed.events[2].event_time_draws[0]
    );
    assert!(parsed.events[0].record_created_at > parsed.events[1].record_created_at);
}

#[test]
fn journey_refuses_backward_transition_and_fixed_start_status() {
    let mut backward = journey();
    backward.events[3].event_time_draws[0] = "2025-01-01T00:00:00Z".into();
    assert_eq!(backward.to_json(), Err(ApiError::InvalidWirePayload));

    let mut fixed_start = journey();
    fixed_start.inference_status = "earliest_record_is_start".into();
    assert_eq!(fixed_start.to_json(), Err(ApiError::InvalidWirePayload));

    let mut future_evidence = journey();
    future_evidence.events[0].available_at = "2026-08-26T00:00:00Z".into();
    assert_eq!(future_evidence.to_json(), Err(ApiError::InvalidWirePayload));

    let mut invalid_relation_evidence = journey();
    invalid_relation_evidence.relations[0].evidence_record_ids = vec![" ".into()];
    assert_eq!(
        invalid_relation_evidence.to_json(),
        Err(ApiError::InvalidWirePayload)
    );

    let mut cyclic = journey();
    cyclic.events[0].event_time_draws = cyclic.events[3].event_time_draws.clone();
    cyclic.relations.push(ProjectJourneyRelationPosterior {
        relation_id: "cycle".into(),
        predecessor_event_id: "bid".into(),
        successor_event_id: "request".into(),
        relation_type_code: "precedes".into(),
        relation_draws: vec![true, false],
        evidence_record_ids: vec!["cycle-evidence".into()],
    });
    assert_eq!(cyclic.to_json(), Err(ApiError::InvalidWirePayload));
}

#[test]
fn pair_posterior_rejects_each_remaining_invalid_clause() {
    let mut bad_schema = pair_artifact();
    bad_schema.schema_version = "tepp.lineage_pair_criterion_posterior.v9".into();
    assert_eq!(bad_schema.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_estimation = pair_artifact();
    bad_estimation.estimation_run_id = "not-a-uuid".into();
    assert_eq!(bad_estimation.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_tepp_run = pair_artifact();
    bad_tepp_run.tepp_run_id = "not-a-uuid".into();
    assert_eq!(bad_tepp_run.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_snapshot = pair_artifact();
    bad_snapshot.source_snapshot_sha256 = "abc".into();
    assert_eq!(bad_snapshot.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_cutoff = pair_artifact();
    bad_cutoff.knowledge_cutoff = "yesterday".into();
    assert_eq!(bad_cutoff.to_json(), Err(ApiError::InvalidWirePayload));

    let mut empty_channels = pair_artifact();
    empty_channels.channel_codes = vec![];
    assert_eq!(empty_channels.to_json(), Err(ApiError::InvalidWirePayload));

    let mut dirty_channel = pair_artifact();
    dirty_channel.channel_codes.push(" bad ".into());
    assert_eq!(dirty_channel.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_seed_domain = pair_artifact();
    bad_seed_domain.draw_provenance.seed_domain = "  padded ".into();
    assert_eq!(bad_seed_domain.to_json(), Err(ApiError::InvalidWirePayload));

    let mut extra_posterior = pair_artifact();
    extra_posterior.admitted_pair_ids = vec![
        "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b3".into(),
        "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b9".into(),
    ];
    assert_eq!(extra_posterior.to_json(), Err(ApiError::InvalidWirePayload));

    let mut missing_posterior = pair_artifact();
    missing_posterior.admitted_pair_ids = vec![];
    assert_eq!(missing_posterior.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_basis_id = pair_artifact();
    bad_basis_id.anchor_basis.basis_id = "not-a-uuid".into();
    assert_eq!(bad_basis_id.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_basis_sha = pair_artifact();
    bad_basis_sha.anchor_basis.basis_sha256 = "xyz".into();
    assert_eq!(bad_basis_sha.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_provenance = pair_artifact();
    bad_provenance.temporal_provenance.method_code = "garbage".into();
    assert_eq!(bad_provenance.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_backend = pair_artifact();
    bad_backend.compute_receipts.cpu.backend_code = "python".into();
    assert_eq!(bad_backend.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_parity = pair_artifact();
    bad_parity.compute_receipts.parity_bound = 0.0;
    assert_eq!(bad_parity.to_json(), Err(ApiError::InvalidWirePayload));

    let mut cpu_difference = pair_artifact();
    cpu_difference.compute_receipts.cpu.observed_maximum_difference = 1.0e-9;
    assert_eq!(cpu_difference.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_pair_id = pair_artifact();
    bad_pair_id.pair_posteriors[0].pair_id = "not-a-uuid".into();
    assert_eq!(bad_pair_id.to_json(), Err(ApiError::InvalidWirePayload));

    let mut same_records = pair_artifact();
    same_records.pair_posteriors[0].successor_record_id = "record-a".into();
    assert_eq!(same_records.to_json(), Err(ApiError::InvalidWirePayload));

    let mut too_few_draws = pair_artifact();
    too_few_draws.draw_provenance.draw_count = 1;
    assert_eq!(too_few_draws.to_json(), Err(ApiError::InvalidWirePayload));

    let mut mismatched_draw_len = pair_artifact();
    mismatched_draw_len.pair_posteriors[0].criterion_draws = vec![0.35];
    assert_eq!(mismatched_draw_len.to_json(), Err(ApiError::InvalidWirePayload));
}

#[test]
fn journey_rejects_every_remaining_invalid_clause() {
    let mut bad_schema = journey();
    bad_schema.schema_version = "tepp.project_journey_posterior.v2".into();
    assert_eq!(bad_schema.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_run_id = journey();
    bad_run_id.tepp_run_id = " a ".into();
    assert_eq!(bad_run_id.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_snapshot = journey();
    bad_snapshot.source_snapshot_sha256 = "x".into();
    assert_eq!(bad_snapshot.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_cutoff = journey();
    bad_cutoff.knowledge_cutoff = "later".into();
    assert_eq!(bad_cutoff.to_json(), Err(ApiError::InvalidWirePayload));

    let mut one_draw = journey();
    one_draw.draw_count = 1;
    assert_eq!(one_draw.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_status = journey();
    bad_status.inference_status = "causal".into();
    assert_eq!(bad_status.to_json(), Err(ApiError::InvalidWirePayload));

    let mut orphan = journey();
    orphan.events = vec![];
    assert_eq!(orphan.to_json(), Err(ApiError::InvalidWirePayload));

    let mut duplicate_event = journey();
    duplicate_event.events.push(duplicate_event.events[0].clone());
    assert_eq!(duplicate_event.to_json(), Err(ApiError::InvalidWirePayload));

    let mut duplicate_relation = journey();
    duplicate_relation.relations.push(duplicate_relation.relations[0].clone());
    assert_eq!(duplicate_relation.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_event_identity = journey();
    bad_event_identity.events[1].event_id = "has whitespace".into();
    assert_eq!(bad_event_identity.to_json(), Err(ApiError::InvalidWirePayload));

    let mut short_relation_draws = journey();
    short_relation_draws.relations[0].relation_draws = vec![true];
    assert_eq!(short_relation_draws.to_json(), Err(ApiError::InvalidWirePayload));

    let mut empty_relation_evidence = journey();
    empty_relation_evidence.relations[0].evidence_record_ids = vec![];
    assert_eq!(
        empty_relation_evidence.to_json(),
        Err(ApiError::InvalidWirePayload)
    );

    let mut dangling_predecessor = journey();
    dangling_predecessor.relations[0].predecessor_event_id = "ghost".into();
    assert_eq!(
        dangling_predecessor.to_json(),
        Err(ApiError::InvalidWirePayload)
    );

    let mut self_relation = journey();
    self_relation.relations[0].successor_event_id = "request".into();
    self_relation.relations[0].relation_id = "self-loop".into();
    assert_eq!(self_relation.to_json(), Err(ApiError::InvalidWirePayload));

    let mut invalid_evidence = journey();
    invalid_evidence.events[1].evidence_record_ids = vec!["  padded  ".into()];
    assert_eq!(invalid_evidence.to_json(), Err(ApiError::InvalidWirePayload));
}

#[test]
fn journey_accepts_every_sanctioned_event_type_and_rejects_unknown_kinds() {
    for event_type in [
        "prior_project",
        "customer_request",
        "procurement_notice",
        "direct_bid",
        "negotiated_bid",
        "external_sensing",
        "internal_discussion",
        "lead",
        "design",
        "production",
        "delivery",
        "trial_operation",
        "operation",
        "claim",
        "rebid",
        "other_evidence_grounded_event",
    ] {
        let single = ProjectJourneyPosteriorArtifact {
            schema_version: PROJECT_JOURNEY_POSTERIOR_SCHEMA.into(),
            tepp_run_id: "journey-run-type".into(),
            source_snapshot_sha256: digest('a'),
            knowledge_cutoff: "2026-08-25T00:00:00Z".into(),
            draw_count: 2,
            inference_status: "posterior_temporal_relation_not_causal".into(),
            events: vec![ProjectJourneyEventPosterior {
                event_id: "single-event".into(),
                event_type_code: event_type.into(),
                record_created_at: "2026-03-03T00:00:00Z".into(),
                available_at: "2026-03-03T00:00:00Z".into(),
                event_time_draws: vec![
                    "2026-01-01T00:00:00Z".into(),
                    "2026-01-02T00:00:00Z".into(),
                ],
                evidence_record_ids: vec!["evidence-single".into()],
            }],
            relations: vec![],
        };
        assert!(
            single.to_json().is_ok(),
            "sanctioned event type {event_type} must be accepted"
        );
    }

    let mut unknown = journey();
    unknown.events[1].event_type_code = "telepathy".into();
    assert_eq!(unknown.to_json(), Err(ApiError::InvalidWirePayload));
}

#[test]
fn journey_rejects_each_single_clause_violation() {
    let mut dirty_event_identity = journey();
    dirty_event_identity.events[0].event_id = "  padded  ".into();
    assert_eq!(dirty_event_identity.to_json(), Err(ApiError::InvalidWirePayload));

    let mut unparsed_created = journey();
    unparsed_created.events[1].record_created_at = "not-a-time".into();
    assert_eq!(unparsed_created.to_json(), Err(ApiError::InvalidWirePayload));

    let mut unparsed_available = journey();
    unparsed_available.events[1].available_at = "not-a-time".into();
    assert_eq!(unparsed_available.to_json(), Err(ApiError::InvalidWirePayload));

    let mut short_draws = journey();
    short_draws.events[1].event_time_draws.pop();
    assert_eq!(short_draws.to_json(), Err(ApiError::InvalidWirePayload));

    let mut drawn_garbage = journey();
    drawn_garbage.events[1].event_time_draws[0] = "garbage".into();
    assert_eq!(drawn_garbage.to_json(), Err(ApiError::InvalidWirePayload));

    let mut missing_event_evidence = journey();
    missing_event_evidence.events[1].evidence_record_ids = vec![];
    assert_eq!(missing_event_evidence.to_json(), Err(ApiError::InvalidWirePayload));

    let mut bad_relation_identity = journey();
    bad_relation_identity.relations[0].relation_id = "".into();
    assert_eq!(bad_relation_identity.to_json(), Err(ApiError::InvalidWirePayload));

    let mut dangling_successor = journey();
    dangling_successor.relations[0].successor_event_id = "ghost".into();
    assert_eq!(dangling_successor.to_json(), Err(ApiError::InvalidWirePayload));

    let mut padded_relation_type = journey();
    padded_relation_type.relations[0].relation_type_code = " arrives ".into();
    assert_eq!(padded_relation_type.to_json(), Err(ApiError::InvalidWirePayload));
}
