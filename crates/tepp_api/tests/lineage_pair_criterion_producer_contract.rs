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
fn pair_posterior_accepts_linux_accelerator_backends_and_rejects_unknown() {
    for backend in ["mlx_cpu", "mlx_cuda", "rust_opencl"] {
        let mut artifact = pair_artifact();
        artifact.compute_receipts.gpu.backend_code = backend.into();
        artifact.compute_receipts.gpu.execution_environment_code = "linux_container".into();
        artifact
            .to_json()
            .unwrap_or_else(|_| panic!("{backend} on linux_container must be admitted"));
    }

    let mut unknown = pair_artifact();
    unknown.compute_receipts.gpu.backend_code = "unknown_backend".into();
    assert_eq!(unknown.to_json(), Err(ApiError::InvalidWirePayload));

    let mut forged_cpu = pair_artifact();
    forged_cpu.compute_receipts.gpu.backend_code = "mlx_cpu".into();
    forged_cpu.compute_receipts.gpu.execution_environment_code = "macos_native".into();
    assert_eq!(forged_cpu.to_json(), Err(ApiError::InvalidWirePayload));
}

#[test]
fn journey_accepts_every_evidence_grounded_event_type() {
    const EVENT_TYPES: &[&str] = &[
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
    ];
    for event_type in EVENT_TYPES {
        let mut artifact = journey();
        artifact.events[0].event_type_code = (*event_type).into();
        artifact
            .to_json()
            .unwrap_or_else(|_| panic!("{event_type} must remain an admitted journey event"));
    }
}

#[test]
fn journey_rejects_duplicate_event_and_relation_identities() {
    let mut duplicate_event = journey();
    duplicate_event.events[1].event_id = duplicate_event.events[0].event_id.clone();
    assert_eq!(duplicate_event.to_json(), Err(ApiError::InvalidWirePayload));

    let mut duplicate_relation = journey();
    duplicate_relation.relations[1].relation_id =
        duplicate_relation.relations[0].relation_id.clone();
    assert_eq!(
        duplicate_relation.to_json(),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn pair_posterior_rejects_each_contract_violation() {
    let reject = |mutate: fn(&mut LineagePairCriterionPosteriorArtifact)| {
        let mut artifact = pair_artifact();
        mutate(&mut artifact);
        assert_eq!(artifact.to_json(), Err(ApiError::InvalidWirePayload));
    };

    reject(|artifact| artifact.schema_version = "tepp.invalid".into());
    reject(|artifact| artifact.estimation_run_id = "not-a-uuid".into());
    reject(|artifact| artifact.tepp_run_id = "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b1".to_uppercase());
    reject(|artifact| artifact.source_snapshot_sha256 = "D".repeat(64));
    reject(|artifact| artifact.knowledge_cutoff = "not-a-timestamp".into());
    reject(|artifact| artifact.channel_codes.clear());
    reject(|artifact| artifact.channel_codes.push("temporal".into()));
    reject(|artifact| artifact.channel_codes[0] = " temporal".into());
    reject(|artifact| artifact.draw_provenance.draw_count = 1);
    reject(|artifact| artifact.draw_provenance.seed_domain.clear());
    reject(|artifact| artifact.draw_provenance.seed_domain = "x".repeat(257));
    reject(|artifact| {
        artifact.admitted_pair_ids = vec!["018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b9".into()];
    });
    reject(|artifact| artifact.anchor_basis.alignment_status = "ambiguous".into());
    reject(|artifact| artifact.anchor_basis.basis_id = "not-a-uuid".into());
    reject(|artifact| artifact.anchor_basis.basis_sha256.push('g'));
    reject(|artifact| artifact.temporal_provenance.method_code = "LEXICAL".into());
    reject(|artifact| artifact.temporal_provenance.configuration_sha256 = "0".repeat(63));
    reject(|artifact| artifact.temporal_provenance.event_clock_code = " clock ".into());
    reject(|artifact| artifact.temporal_provenance.temporal_dependency_sha256 = "G".repeat(64));
    reject(|artifact| {
        artifact
            .temporal_provenance
            .branch_transition_sha256
            .clear();
    });
    reject(|artifact| artifact.compute_receipts.cpu.backend_code.clear());
    reject(|artifact| artifact.compute_receipts.cpu.execution_environment_code = " env".into());
    reject(|artifact| artifact.compute_receipts.cpu.objective_sha256 = "1".repeat(63));
    reject(|artifact| artifact.compute_receipts.cpu.parameter_sha256 = "h".repeat(64));
    reject(|artifact| artifact.compute_receipts.cpu.draw_sha256 = "A".repeat(64));
    reject(|artifact| artifact.compute_receipts.gpu.observed_maximum_difference = f64::NAN);
    reject(|artifact| artifact.compute_receipts.gpu.observed_maximum_difference = -1.0);
    reject(|artifact| artifact.compute_receipts.cpu.backend_code = "openblas".into());
    reject(|artifact| artifact.compute_receipts.gpu.objective_sha256 = digest('9'));
    reject(|artifact| artifact.compute_receipts.parity_method_code.clear());
    reject(|artifact| artifact.compute_receipts.parity_bound = f64::NAN);
    reject(|artifact| artifact.compute_receipts.parity_bound = 0.0);
    reject(|artifact| artifact.compute_receipts.cpu.observed_maximum_difference = 1.0e-12);
    reject(|artifact| artifact.pair_posteriors[0].pair_id = "not-a-uuid".into());
    reject(|artifact| artifact.pair_posteriors[0].predecessor_record_id.clear());
    reject(|artifact| artifact.pair_posteriors[0].successor_record_id = " successor".into());
    reject(|artifact| {
        artifact.pair_posteriors[0].successor_record_id =
            artifact.pair_posteriors[0].predecessor_record_id.clone();
    });
    reject(|artifact| {
        artifact.pair_posteriors[0].predecessor_record_created_at = "nope".into();
    });
    reject(|artifact| artifact.pair_posteriors[0].successor_record_created_at = "nope".into());
    reject(|artifact| artifact.pair_posteriors[0].predecessor_available_at = "nope".into());
    reject(|artifact| artifact.pair_posteriors[0].successor_available_at = "nope".into());
    reject(|artifact| {
        artifact.pair_posteriors[0].successor_available_at = "2026-08-26T00:00:00Z".into();
    });
    reject(|artifact| {
        artifact.pair_posteriors[0]
            .predecessor_event_time_draws
            .pop();
    });
    reject(|artifact| {
        artifact.pair_posteriors[0].successor_event_time_draws.pop();
    });
    reject(|artifact| {
        artifact.pair_posteriors[0].criterion_draws.pop();
    });
    reject(|artifact| artifact.pair_posteriors[0].criterion_draws[0] = f64::NAN);
    reject(|artifact| artifact.pair_posteriors[0].criterion_draws[1] = 1.25);
    reject(|artifact| {
        artifact.pair_posteriors[0].predecessor_event_time_draws[1] = "not-a-time".into();
    });

    let mut numeric = pair_artifact();
    numeric.source_snapshot_sha256 = digest('1');
    numeric.anchor_basis.basis_sha256 = digest('2');
    numeric.temporal_provenance.configuration_sha256 = digest('3');
    numeric.temporal_provenance.temporal_dependency_sha256 = digest('4');
    numeric.temporal_provenance.branch_transition_sha256 = digest('5');
    numeric.compute_receipts.cpu.objective_sha256 = digest('6');
    numeric.compute_receipts.gpu.objective_sha256 = digest('6');
    numeric.compute_receipts.cpu.parameter_sha256 = digest('7');
    numeric.compute_receipts.gpu.parameter_sha256 = digest('8');
    numeric.compute_receipts.cpu.draw_sha256 = digest('9');
    numeric.compute_receipts.gpu.draw_sha256 = digest('0');
    numeric
        .to_json()
        .expect("digit digests must remain valid lowercase hex");
}

#[test]
fn pair_posterior_rejects_duplicate_pairs_and_non_uuid_admitted_identity() {
    let mut duplicate_pairs = pair_artifact();
    duplicate_pairs
        .pair_posteriors
        .push(duplicate_pairs.pair_posteriors[0].clone());
    assert_eq!(duplicate_pairs.to_json(), Err(ApiError::InvalidWirePayload));

    let mut admitted_non_uuid = pair_artifact();
    admitted_non_uuid.pair_posteriors[0].pair_id = "not-a-uuid".into();
    admitted_non_uuid.admitted_pair_ids = vec!["not-a-uuid".into()];
    assert_eq!(
        admitted_non_uuid.to_json(),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn journey_rejects_each_contract_violation() {
    let reject = |mutate: fn(&mut ProjectJourneyPosteriorArtifact)| {
        let mut artifact = journey();
        mutate(&mut artifact);
        assert_eq!(artifact.to_json(), Err(ApiError::InvalidWirePayload));
    };

    reject(|artifact| artifact.schema_version = "tepp.invalid".into());
    reject(|artifact| artifact.tepp_run_id.clear());
    reject(|artifact| artifact.source_snapshot_sha256 = "A".repeat(64));
    reject(|artifact| artifact.knowledge_cutoff = "not-a-timestamp".into());
    reject(|artifact| artifact.draw_count = 1);
    reject(|artifact| artifact.events.clear());
    reject(|artifact| artifact.events[0].event_id.clear());
    reject(|artifact| artifact.events[0].event_type_code = "unlisted_event".into());
    reject(|artifact| artifact.events[0].record_created_at = "nope".into());
    reject(|artifact| artifact.events[0].available_at = "nope".into());
    reject(|artifact| artifact.source_snapshot_sha256 = "0".repeat(63));
    reject(|artifact| {
        artifact.events[0].event_time_draws.pop();
    });
    reject(|artifact| artifact.events[0].event_time_draws[0] = "nope".into());
    reject(|artifact| artifact.events[0].evidence_record_ids.clear());
    reject(|artifact| artifact.events[0].evidence_record_ids[0] = " ".into());
    reject(|artifact| artifact.relations[0].relation_id.clear());
    reject(|artifact| artifact.relations[0].predecessor_event_id = "missing".into());
    reject(|artifact| artifact.relations[0].successor_event_id = "missing".into());
    reject(|artifact| {
        artifact.relations[0].successor_event_id =
            artifact.relations[0].predecessor_event_id.clone();
    });
    reject(|artifact| artifact.relations[0].relation_type_code.clear());
    reject(|artifact| {
        artifact.relations[0].relation_draws.pop();
    });
    reject(|artifact| artifact.relations[0].evidence_record_ids.clear());

    let mut numeric = journey();
    numeric.source_snapshot_sha256 = digest('3');
    numeric
        .to_json()
        .expect("digit snapshot digest must remain valid");
}
