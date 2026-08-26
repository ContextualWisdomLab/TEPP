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
        observed_maximum_difference: if backend == "cpu_f64" { 0.0 } else { 5.0e-9 },
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
            successor_record_created_at: "2026-01-01T00:00:00Z".into(),
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
            cpu: receipt("cpu_f64"),
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
}

fn journey() -> ProjectJourneyPosteriorArtifact {
    let event = |id: &str, kind: &str, created: &str, first: &str, second: &str| {
        ProjectJourneyEventPosterior {
            event_id: id.into(),
            event_type_code: kind.into(),
            record_created_at: created.into(),
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
}
