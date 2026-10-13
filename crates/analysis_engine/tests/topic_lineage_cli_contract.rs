//! Offline topic-lineage CLI contract: cutoff exclusion, refusal, and output.

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
#[cfg(unix)]
use std::time::{Duration, Instant};

use analysis_engine::{
    AnalysisEngineError, TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION, TOPIC_LINEAGE_CLI_INPUT_BYTE_LIMIT,
    TOPIC_LINEAGE_CLI_OUTPUT_SCHEMA_VERSION, TopicLineageCliError, run_topic_lineage,
    run_topic_lineage_cli,
};
use serde_json::{Value, json};
use tepp_api::{AnalysisRunTerminalState, ApiError};
use topic_measurement::TopicMeasurementError;

fn example_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("examples");
    path.push("topic_lineage_cli_input_v1.json");
    path
}

fn example() -> Value {
    serde_json::from_slice(&std::fs::read(example_path()).expect("example")).expect("json")
}

fn run(value: &Value) -> Result<analysis_engine::TopicLineageCliOutput, TopicLineageCliError> {
    run_topic_lineage(&serde_json::to_vec(value).expect("bytes"))
}

fn rejected(value: &Value) -> TopicLineageCliError {
    run(value).expect_err("input must be refused")
}

fn mutated(mutate: impl FnOnce(&mut Value)) -> Value {
    let mut value = example();
    mutate(&mut value);
    value
}

fn document_id(index: usize) -> String {
    format!("00000000-0000-0000-0000-00000000000{}", index + 1)
}

#[test]
fn inferred_only_transitions_cannot_supply_observed_lineage() {
    let value = mutated(|value| {
        for edge in value["relations"].as_array_mut().expect("relations") {
            edge["evidence_status"] = json!("inferred");
        }
    });
    assert_eq!(
        rejected(&value),
        TopicLineageCliError::Engine(AnalysisEngineError::TopicMeasurement(
            TopicMeasurementError::InvalidModelInput
        ))
    );
}

#[test]
fn mixed_proposals_are_equivalent_to_removing_inferred_transitions() {
    let mut mixed = example();
    mixed["relations"][1]["evidence_status"] = json!("inferred");
    let mut observed_only = mixed.clone();
    observed_only["relations"].as_array_mut().unwrap().remove(1);
    let expected = run(&observed_only).expect("observed positive");
    assert!(!expected.artifact.sequence_edges.is_empty());
    let actual = run(&mixed).expect("mixed positive");
    assert_eq!(actual.artifact, expected.artifact);
    assert_eq!(actual.terminal_result, expected.terminal_result);

    // A parallel proposal must neither replace nor double-weight observed data.
    let mut duplicate = observed_only.clone();
    let mut proposal = duplicate["relations"][0].clone();
    proposal["evidence_status"] = json!("inferred");
    duplicate["relations"]
        .as_array_mut()
        .unwrap()
        .push(proposal);
    assert_eq!(
        run(&duplicate).expect("parallel status").artifact,
        expected.artifact
    );
}

#[test]
fn late_observed_transition_cannot_rescue_inferred_only_admitted_graph() {
    let value = mutated(|value| {
        for edge in value["relations"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .take(3)
        {
            edge["evidence_status"] = json!("inferred");
        }
    });
    assert_eq!(
        rejected(&value),
        TopicLineageCliError::Engine(AnalysisEngineError::TopicMeasurement(
            TopicMeasurementError::InvalidModelInput
        ))
    );
}

#[test]
fn inferred_status_does_not_bypass_relation_temporal_validation() {
    let value = mutated(|value| {
        value["relations"][0]["evidence_status"] = json!("inferred");
        value["relations"][0]["source_document_id"] = json!(document_id(1));
        value["relations"][0]["target_document_id"] = json!(document_id(0));
    });
    assert_eq!(rejected(&value), TopicLineageCliError::InvalidRelation);
}

#[test]
fn committed_example_excludes_late_evidence_and_emits_a_digest_bound_artifact() {
    let output = run(&example()).expect("example run");
    assert_eq!(
        output.schema_version,
        TOPIC_LINEAGE_CLI_OUTPUT_SCHEMA_VERSION
    );
    assert_eq!(output.admitted_document_count, 4);
    assert_eq!(output.excluded_document_count, 1);
    assert_eq!(output.excluded_membership_count, 2);
    assert_eq!(output.excluded_relation_count, 1);
    assert_eq!(output.input_sha256.len(), 64);
    assert_eq!(output.artifact.evidence_count, 4);
    assert_eq!(output.artifact.topic_count, 2);
    assert_eq!(
        output.artifact.inference_status,
        "fitted_topic_association_not_causation"
    );
    assert_eq!(
        output.terminal_result.run_state,
        AnalysisRunTerminalState::Succeeded
    );
    assert_eq!(
        output.terminal_result.result_sha256.as_deref(),
        Some(output.artifact.sha256().expect("digest").as_str())
    );
    assert_eq!(
        output.terminal_result.result_schema_version.as_deref(),
        Some(TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION)
    );
    let late = document_id(4);
    let serialized = serde_json::to_string(&output).expect("output json");
    assert!(!serialized.contains(&late));
    assert_eq!(
        serialized.matches("caus").count(),
        1,
        "only the fixed non-causation claim boundary may mention causation"
    );
    assert_eq!(run(&example()).expect("rerun"), output);
}

#[test]
fn exclusion_is_equivalent_to_removing_late_evidence_by_hand() {
    let baseline = run(&example()).expect("baseline");
    let manual = mutated(|value| {
        value["documents"].as_array_mut().expect("documents").pop();
        let memberships = value["memberships"].as_array_mut().expect("memberships");
        memberships.retain(|item| {
            item["document_id"] != document_id(4)
                && item["available_time"] != "2026-09-01T00:00:00Z"
        });
        value["relations"].as_array_mut().expect("relations").pop();
    });
    let manual = run(&manual).expect("manual");
    assert_eq!(manual.artifact, baseline.artifact);
    assert_eq!(manual.terminal_result, baseline.terminal_result);
    assert_eq!(manual.excluded_document_count, 0);
    assert_eq!(manual.excluded_membership_count, 0);
    assert_eq!(manual.excluded_relation_count, 0);
    assert_ne!(manual.input_sha256, baseline.input_sha256);
}

#[test]
fn availability_exactly_at_the_cutoff_is_admitted() {
    let output = run(&mutated(|value| {
        value["documents"][4]["available_time"] = json!("2026-08-01T00:00:00Z");
    }))
    .expect("boundary run");
    assert_eq!(output.admitted_document_count, 5);
    assert_eq!(output.excluded_document_count, 0);
    assert_eq!(output.excluded_membership_count, 1);
    assert_eq!(output.excluded_relation_count, 0);
}

#[test]
fn late_relation_and_membership_are_excluded_without_reaching_the_estimator() {
    let output = run(&mutated(|value| {
        value["relations"][0]["available_time"] = json!("2026-08-01T00:00:01Z");
    }))
    .expect("late relation run");
    assert_eq!(output.excluded_relation_count, 2);

    let only_late_memberships = mutated(|value| {
        for item in value["memberships"].as_array_mut().expect("memberships") {
            item["available_time"] = json!("2026-08-02T00:00:00Z");
        }
    });
    assert_eq!(
        rejected(&only_late_memberships),
        TopicLineageCliError::Engine(AnalysisEngineError::TopicMeasurement(
            TopicMeasurementError::InvalidModelInput
        ))
    );
    let only_late_relations = mutated(|value| {
        for item in value["relations"].as_array_mut().expect("relations") {
            item["available_time"] = json!("2026-08-02T00:00:00Z");
        }
    });
    assert_eq!(
        rejected(&only_late_relations),
        TopicLineageCliError::Engine(AnalysisEngineError::TopicMeasurement(
            TopicMeasurementError::InvalidModelInput
        ))
    );
}

#[test]
fn identified_causal_relation_kinds_are_refused() {
    for kind in ["causes", "intervenes_on"] {
        assert_eq!(
            rejected(&mutated(|value| value["relations"][0]["kind"] = json!(kind))),
            TopicLineageCliError::CausalClaimRefused
        );
    }
    let provenance = run(&mutated(|value| {
        value["relations"][0]["kind"] = json!("references");
    }))
    .expect("provenance relation is not a transition");
    assert_eq!(provenance.excluded_relation_count, 1);
}

#[test]
fn malformed_unknown_and_nonfinite_input_fails_closed() {
    let malformed_inputs: Vec<Vec<u8>> = vec![
        b"not json".to_vec(),
        br#"{"schema_version":1}"#.to_vec(),
        serde_json::to_vec(&mutated(|value| value["unexpected"] = json!(true))).expect("bytes"),
        serde_json::to_vec(&mutated(|value| {
            value["documents"][0]["source_text"] = json!("private body");
        }))
        .expect("bytes"),
        serde_json::to_vec(&mutated(|value| {
            value["memberships"][0]["role"] = json!("unknown_role");
        }))
        .expect("bytes"),
        serde_json::to_vec(&mutated(|value| {
            value["relations"][0]["kind"] = json!("unknown_kind");
        }))
        .expect("bytes"),
        serde_json::to_vec(&mutated(|value| {
            value["model"]["inference_status"] = json!("causal");
        }))
        .expect("bytes"),
        serde_json::to_vec(&mutated(|value| {
            value["documents"][0]["document_id"] = json!("not-a-uuid");
        }))
        .expect("bytes"),
        serde_json::to_vec(&mutated(|value| {
            value
                .as_object_mut()
                .expect("object")
                .remove("completed_at");
        }))
        .expect("bytes"),
    ];
    for input in malformed_inputs {
        assert_eq!(
            run_topic_lineage(&input),
            Err(TopicLineageCliError::MalformedInput)
        );
    }
    let text = serde_json::to_string(&example()).expect("text");
    for literal in ["NaN", "1e999", "-1e999", "Infinity"] {
        let replaced = text.replacen("90.0", literal, 1);
        assert_ne!(replaced, text);
        assert_eq!(
            run_topic_lineage(replaced.as_bytes()),
            Err(TopicLineageCliError::MalformedInput)
        );
    }
    assert_eq!(
        run_topic_lineage(&vec![b' '; TOPIC_LINEAGE_CLI_INPUT_BYTE_LIMIT + 1]),
        Err(TopicLineageCliError::InputTooLarge)
    );
    assert_eq!(
        rejected(&mutated(|value| value["schema_version"] = json!("v0"))),
        TopicLineageCliError::UnsupportedSchemaVersion
    );
    assert_eq!(
        rejected(&mutated(|value| value["model"]["topic_count"] = json!(5))),
        TopicLineageCliError::LimitExceeded
    );
}

#[test]
fn timestamps_on_every_clock_are_strict() {
    let paths: [&[&str]; 13] = [
        &["request", "knowledge_cutoff"],
        &["documents", "0", "event_time"],
        &["documents", "0", "available_time"],
        &["memberships", "0", "available_time"],
        &["memberships", "0", "valid_from"],
        &["memberships", "0", "valid_to"],
        &["relations", "0", "available_time"],
        &["relations", "0", "source_event_start"],
        &["relations", "0", "source_event_end"],
        &["relations", "0", "target_event_start"],
        &["relations", "0", "target_event_end"],
        &["documents", "4", "event_time"],
        &["documents", "4", "available_time"],
    ];
    for path in paths {
        let input = mutated(|value| {
            let mut cursor = value;
            for key in path {
                cursor = match key.parse::<usize>() {
                    Ok(index) => &mut cursor[index],
                    Err(_) => &mut cursor[*key],
                };
            }
            *cursor = json!("2026-07-01 00:00:00");
        });
        assert_eq!(
            rejected(&input),
            TopicLineageCliError::InvalidTimestamp,
            "{path:?}"
        );
    }
}

#[test]
fn document_shape_and_identity_errors_fail_closed() {
    assert_eq!(
        rejected(&mutated(|value| {
            value["documents"][1]["document_id"] = json!(document_id(0));
        })),
        TopicLineageCliError::DuplicateDocument
    );
    assert_eq!(
        rejected(&mutated(|value| {
            value["documents"][4]["document_id"] = json!(document_id(0));
        })),
        TopicLineageCliError::DuplicateDocument
    );
    assert_eq!(
        rejected(&mutated(|value| {
            value["documents"][0]["term_counts"] = json!([1.0]);
        })),
        TopicLineageCliError::InvalidTermCounts
    );
    for (indices, counts) in [
        (json!([1, 0]), json!([1.0, 1.0])),
        (json!([0, 0]), json!([1.0, 1.0])),
        (json!([0, 4]), json!([1.0, 1.0])),
    ] {
        assert_eq!(
            rejected(&mutated(|value| {
                value["documents"][0]["term_indices"] = indices;
                value["documents"][0]["term_counts"] = counts;
            })),
            TopicLineageCliError::Engine(AnalysisEngineError::TopicMeasurement(
                TopicMeasurementError::InvalidSparseMatrix
            ))
        );
    }
    for (indices, counts) in [
        (json!([]), json!([])),
        (json!([0, 1]), json!([0.0, 0.0])),
        (json!([0, 1]), json!([-1.0, 2.0])),
    ] {
        assert_eq!(
            rejected(&mutated(|value| {
                value["documents"][0]["term_indices"] = indices;
                value["documents"][0]["term_counts"] = counts;
            })),
            TopicLineageCliError::Engine(AnalysisEngineError::TopicMeasurement(
                TopicMeasurementError::InvalidModelInput
            ))
        );
    }
    let all_documents_late = mutated(|value| {
        for item in value["documents"].as_array_mut().expect("documents") {
            item["available_time"] = json!("2026-08-02T00:00:00Z");
        }
    });
    assert_eq!(
        rejected(&all_documents_late),
        TopicLineageCliError::Engine(AnalysisEngineError::TopicMeasurement(
            TopicMeasurementError::InvalidSparseMatrix
        ))
    );
}

#[test]
fn membership_errors_fail_closed() {
    assert_eq!(
        rejected(&mutated(|value| {
            value["memberships"][0]["document_id"] = json!("00000000-0000-0000-0000-0000000000ff");
        })),
        TopicLineageCliError::UnknownDocumentReference
    );
    assert_eq!(
        rejected(&mutated(
            |value| value["memberships"][0]["weight"] = json!(-0.5)
        )),
        TopicLineageCliError::InvalidMembership
    );
    assert_eq!(
        rejected(&mutated(|value| {
            value["memberships"][0]["valid_from"] = json!("2026-08-01T00:00:00Z");
        })),
        TopicLineageCliError::InvalidMembership
    );
    assert_eq!(
        rejected(&mutated(|value| {
            let duplicate = value["memberships"][0].clone();
            value["memberships"]
                .as_array_mut()
                .expect("memberships")
                .push(duplicate);
        })),
        TopicLineageCliError::InvalidMembership
    );
}

#[test]
fn relation_errors_fail_closed() {
    let unknown = "00000000-0000-0000-0000-0000000000ff";
    for field in ["source_document_id", "target_document_id"] {
        assert_eq!(
            rejected(&mutated(
                |value| value["relations"][0][field] = json!(unknown)
            )),
            TopicLineageCliError::UnknownDocumentReference
        );
    }
    let invalid_relations = [
        mutated(|value| {
            value["relations"][0]["source_event_end"] = json!("2026-07-01T00:00:00Z");
        }),
        mutated(|value| {
            value["relations"][0]["target_event_end"] = json!("2026-07-02T00:00:00Z");
        }),
        mutated(|value| {
            value["relations"][0]["source_event_start"] = json!("2026-07-01T07:00:00Z");
        }),
        mutated(|value| {
            value["relations"][0]["target_event_start"] = json!("2026-07-02T07:00:00Z");
        }),
        mutated(|value| {
            let duplicate = value["relations"][0].clone();
            value["relations"]
                .as_array_mut()
                .expect("relations")
                .push(duplicate);
        }),
        mutated(|value| {
            let relation = &mut value["relations"][0];
            relation["source_document_id"] = json!(document_id(1));
            relation["target_document_id"] = json!(document_id(0));
            relation["source_event_start"] = json!("2026-07-02T00:00:00Z");
            relation["source_event_end"] = json!("2026-07-02T12:00:00Z");
            relation["target_event_start"] = json!("2026-07-01T00:00:00Z");
            relation["target_event_end"] = json!("2026-07-01T12:00:00Z");
        }),
        mutated(|value| {
            let mut reverse = value["relations"][0].clone();
            for relation in [&mut value["relations"][0], &mut reverse] {
                for field in ["source_event_start", "target_event_start"] {
                    relation[field] = json!("2026-07-01T00:00:00Z");
                }
                for field in ["source_event_end", "target_event_end"] {
                    relation[field] = json!("2026-07-02T12:00:00Z");
                }
            }
            reverse["source_document_id"] = json!(document_id(1));
            reverse["target_document_id"] = json!(document_id(0));
            value["relations"]
                .as_array_mut()
                .expect("relations")
                .push(reverse);
        }),
    ];
    for input in invalid_relations {
        assert_eq!(rejected(&input), TopicLineageCliError::InvalidRelation);
    }
    let excluded_source = run(&mutated(|value| {
        let relation = &mut value["relations"][3];
        relation["source_document_id"] = json!(document_id(4));
        relation["target_document_id"] = json!(document_id(3));
        relation["kind"] = json!("references");
        relation["source_event_start"] = json!("2026-07-05T00:00:00Z");
        relation["source_event_end"] = json!("2026-07-05T12:00:00Z");
        relation["target_event_start"] = json!("2026-07-04T00:00:00Z");
        relation["target_event_end"] = json!("2026-07-04T12:00:00Z");
    }))
    .expect("excluded provenance source");
    assert_eq!(excluded_source.excluded_relation_count, 1);
}

#[test]
fn overlapping_intervals_do_not_license_backward_exact_event_transitions() {
    let reverse = mutated(|value| {
        let relation = &mut value["relations"][0];
        relation["source_document_id"] = json!(document_id(1));
        relation["target_document_id"] = json!(document_id(0));
        for field in ["source_event_start", "target_event_start"] {
            relation[field] = json!("2026-07-01T00:00:00Z");
        }
        for field in ["source_event_end", "target_event_end"] {
            relation[field] = json!("2026-07-02T12:00:00Z");
        }
    });
    assert_eq!(rejected(&reverse), TopicLineageCliError::InvalidRelation);
    let mut provenance = reverse;
    provenance["relations"][0]["kind"] = json!("references");
    assert!(run(&provenance).is_ok());
    assert!(run(&example()).is_ok());
}

#[test]
fn model_request_and_convergence_errors_fail_closed() {
    let invalid_model_inputs = [
        mutated(|value| {
            for item in value["documents"].as_array_mut().expect("documents") {
                item["event_time"] = json!("2026-07-01T06:00:00Z");
            }
            for item in value["relations"].as_array_mut().expect("relations") {
                for field in ["source_event_start", "target_event_start"] {
                    item[field] = json!("2026-07-01T00:00:00Z");
                }
                for field in ["source_event_end", "target_event_end"] {
                    item[field] = json!("2026-07-01T12:00:00Z");
                }
            }
            value["relations"]
                .as_array_mut()
                .expect("relations")
                .truncate(1);
        }),
        mutated(|value| value["model"]["topic_count"] = json!(1)),
        mutated(|value| value["model"]["prior_variance"] = json!(0.0)),
    ];
    for input in invalid_model_inputs {
        assert_eq!(
            rejected(&input),
            TopicLineageCliError::Engine(AnalysisEngineError::TopicMeasurement(
                TopicMeasurementError::InvalidModelInput
            ))
        );
    }
    assert_eq!(
        rejected(&mutated(|value| value["run_id"] = json!(""))),
        TopicLineageCliError::Engine(AnalysisEngineError::Api(ApiError::InvalidWirePayload))
    );
    assert_eq!(
        rejected(&mutated(|value| {
            value["request"]["model_contract_version"] = json!("other-model");
        })),
        TopicLineageCliError::Engine(AnalysisEngineError::InvalidEvidence)
    );
    let nonconvergent = rejected(&mutated(|value| {
        value["model"]["maximum_iterations"] = json!(2);
        value["model"]["tolerance"] = json!(1e-12);
        value["model"]["seeds"] = json!([1]);
    }));
    assert_eq!(
        nonconvergent,
        TopicLineageCliError::Engine(AnalysisEngineError::TopicMeasurement(
            TopicMeasurementError::DidNotConverge
        ))
    );
    assert_eq!(nonconvergent.exit_code(), 4);
}

#[test]
fn error_codes_messages_and_exit_statuses_are_stable() {
    let cases = [
        (TopicLineageCliError::Usage, "usage", 2),
        (TopicLineageCliError::Io, "io", 5),
        (TopicLineageCliError::InputTooLarge, "input_too_large", 3),
        (TopicLineageCliError::MalformedInput, "malformed_input", 3),
        (
            TopicLineageCliError::UnsupportedSchemaVersion,
            "unsupported_schema_version",
            3,
        ),
        (TopicLineageCliError::LimitExceeded, "limit_exceeded", 3),
        (
            TopicLineageCliError::InvalidTimestamp,
            "invalid_timestamp",
            3,
        ),
        (
            TopicLineageCliError::DuplicateDocument,
            "duplicate_document",
            3,
        ),
        (
            TopicLineageCliError::InvalidTermCounts,
            "invalid_term_counts",
            3,
        ),
        (
            TopicLineageCliError::UnknownDocumentReference,
            "unknown_document_reference",
            3,
        ),
        (
            TopicLineageCliError::InvalidMembership,
            "invalid_membership",
            3,
        ),
        (TopicLineageCliError::InvalidRelation, "invalid_relation", 3),
        (
            TopicLineageCliError::CausalClaimRefused,
            "causal_claim_refused",
            3,
        ),
        (
            TopicLineageCliError::Engine(AnalysisEngineError::InvalidEvidence),
            "analysis_engine_rejected",
            3,
        ),
        (
            TopicLineageCliError::Engine(AnalysisEngineError::TopicMeasurement(
                TopicMeasurementError::NonFiniteEstimate,
            )),
            "analysis_engine_rejected",
            4,
        ),
    ];
    for (error, code, exit) in cases {
        assert_eq!(error.code(), code);
        assert_eq!(error.exit_code(), exit);
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn error_messages_are_stable() {
    let messages = [
        (
            TopicLineageCliError::Usage,
            "usage: tepp-topic-lineage <input.json>",
        ),
        (
            TopicLineageCliError::Io,
            "topic-lineage input or output I/O failed",
        ),
        (
            TopicLineageCliError::InputTooLarge,
            "topic-lineage input exceeded its byte limit",
        ),
        (
            TopicLineageCliError::MalformedInput,
            "malformed topic-lineage input",
        ),
        (
            TopicLineageCliError::UnsupportedSchemaVersion,
            "unsupported topic-lineage input schema version",
        ),
        (
            TopicLineageCliError::LimitExceeded,
            "topic-lineage input exceeded an execution bound",
        ),
        (
            TopicLineageCliError::InvalidTimestamp,
            "invalid topic-lineage timestamp",
        ),
        (
            TopicLineageCliError::DuplicateDocument,
            "duplicate topic-lineage document identity",
        ),
        (
            TopicLineageCliError::InvalidTermCounts,
            "topic-lineage term indices and counts differ in length",
        ),
        (
            TopicLineageCliError::UnknownDocumentReference,
            "topic-lineage reference to an unknown document",
        ),
        (
            TopicLineageCliError::InvalidMembership,
            "invalid topic-lineage membership",
        ),
        (
            TopicLineageCliError::InvalidRelation,
            "invalid topic-lineage relation",
        ),
        (
            TopicLineageCliError::CausalClaimRefused,
            "identified causal relation kinds are refused by the association estimator",
        ),
        (
            TopicLineageCliError::Engine(AnalysisEngineError::InvalidEvidence),
            "invalid analysis evidence",
        ),
    ];
    for (error, message) in messages {
        assert_eq!(error.to_string(), message);
    }
    let error: &dyn std::error::Error = &TopicLineageCliError::Usage;
    assert!(error.source().is_none());
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("closed"))
    }
}

fn cli(arguments: &[OsString]) -> (u8, String, String) {
    let mut output = Vec::new();
    let mut diagnostics = Vec::new();
    let status = run_topic_lineage_cli(arguments.to_vec(), &mut output, &mut diagnostics);
    (
        status,
        String::from_utf8(output).expect("utf8"),
        String::from_utf8(diagnostics).expect("utf8"),
    )
}

#[test]
fn command_contract_reads_one_file_and_reports_typed_failures() {
    let example = example_path().into_os_string();
    let (status, output, diagnostics) = cli(std::slice::from_ref(&example));
    assert_eq!(status, 0);
    assert!(diagnostics.is_empty());
    let parsed: Value = serde_json::from_str(&output).expect("output json");
    assert_eq!(
        parsed["schema_version"],
        TOPIC_LINEAGE_CLI_OUTPUT_SCHEMA_VERSION
    );
    assert!(output.ends_with('\n'));

    let (status, output, diagnostics) = cli(&[]);
    assert_eq!((status, output.as_str()), (2, ""));
    assert!(diagnostics.contains("\"error_code\":\"usage\""));
    assert_eq!(cli(&[example.clone(), example.clone()]).0, 2);
    assert_eq!(cli(&["/nonexistent/tepp/input.json".into()]).0, 5);
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).into_os_string();
    assert_eq!(cli(&[directory]).0, 5);

    let scratch = std::env::temp_dir().join(format!(
        "tepp-topic-lineage-cli-{}.json",
        std::process::id()
    ));
    std::fs::write(&scratch, b"{\"private\":\"never echoed\"}").expect("scratch");
    let (status, output, diagnostics) = cli(&[scratch.clone().into_os_string()]);
    std::fs::remove_file(&scratch).expect("cleanup");
    assert_eq!((status, output.as_str()), (3, ""));
    assert!(diagnostics.contains("\"error_code\":\"malformed_input\""));
    assert!(!diagnostics.contains("never echoed"));

    let mut diagnostics = Vec::new();
    assert_eq!(
        run_topic_lineage_cli([example.clone()], &mut FailingWriter, &mut diagnostics),
        5
    );
    assert!(
        String::from_utf8(diagnostics)
            .expect("utf8")
            .contains("\"io\"")
    );
    assert_eq!(
        run_topic_lineage_cli(Vec::<OsString>::new(), &mut Vec::new(), &mut FailingWriter),
        2
    );
    let mut failing = FailingWriter;
    assert!(failing.flush().is_err());
}

#[test]
fn map_point_estimates_are_not_labeled_posterior_means() {
    let reference = include_str!("../../topic_measurement/src/reference.rs");
    let artifact = include_str!("../src/topic_lineage_artifact.rs");
    for source in [reference, artifact] {
        assert!(
            source.contains("MAP-fitted"),
            "MAP point-estimate label missing"
        );
        assert!(!source.contains("Minimum dominant-topic posterior mean"));
        assert!(!source.contains("Document-by-topic posterior mean proportions"));
    }
    let contract = include_str!("../../../docs/API_CONTRACT.md");
    assert!(contract.contains("not a posterior mean or an uncertainty probability"));
}

#[cfg(unix)]
#[test]
fn packaged_binary_refuses_fifo_without_waiting_for_a_writer() {
    let root = std::env::temp_dir().join(format!("tepp-cli-fifo-{}", std::process::id()));
    std::fs::create_dir(&root).expect("exclusive fixture directory");
    let fifo = root.join("input.json");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .expect("mkfifo")
            .success()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_tepp-topic-lineage"))
        .arg(&fifo)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    let deadline = Instant::now() + Duration::from_secs(2);
    let completed = loop {
        if child.try_wait().expect("observe child").is_some() {
            break true;
        }
        if Instant::now() >= deadline {
            child.kill().expect("terminate blocked reader");
            break false;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let result = child.wait_with_output().expect("reap child");
    std::fs::remove_file(&fifo).expect("remove fifo");
    std::fs::remove_dir(&root).expect("remove fixture directory");
    assert!(completed, "CLI blocked opening a FIFO without a writer");
    assert_eq!(result.status.code(), Some(5));
    assert!(result.stdout.is_empty());
    let diagnostic: Value = serde_json::from_slice(&result.stderr).expect("error json");
    assert_eq!(diagnostic["error_code"], "io");
    assert_eq!(
        diagnostic["message"],
        "topic-lineage input or output I/O failed"
    );
}

#[cfg(unix)]
#[test]
fn nonregular_paths_fail_closed_but_regular_file_symlinks_are_admitted() {
    let root = std::env::temp_dir().join(format!("tepp-cli-types-{}", std::process::id()));
    std::fs::create_dir(&root).expect("exclusive fixture directory");
    let socket_path = root.join("input.sock");
    let socket = std::os::unix::net::UnixListener::bind(&socket_path).expect("socket");
    let device = PathBuf::from("/dev/null");
    for path in [&root, &socket_path, &device] {
        let (status, output, diagnostics) = cli(&[path.clone().into_os_string()]);
        assert_eq!((status, output.as_str()), (5, ""));
        let diagnostic: Value = serde_json::from_str(&diagnostics).expect("error json");
        assert_eq!(diagnostic["error_code"], "io");
        assert!(!diagnostics.contains(path.to_str().expect("fixture path")));
    }
    let link = root.join("regular.json");
    std::os::unix::fs::symlink(example_path(), &link).expect("regular-file symlink");
    assert_eq!(cli(&[link.clone().into_os_string()]).0, 0);
    std::fs::remove_file(&link).expect("remove symlink");
    drop(socket);
    std::fs::remove_file(&socket_path).expect("remove socket");
    std::fs::remove_dir(&root).expect("remove fixture directory");
}

#[test]
fn packaged_binary_runs_the_committed_example() {
    let binary = env!("CARGO_BIN_EXE_tepp-topic-lineage");
    let success = Command::new(binary)
        .arg(example_path())
        .output()
        .expect("spawn");
    assert_eq!(success.status.code(), Some(0));
    let parsed: Value = serde_json::from_slice(&success.stdout).expect("stdout json");
    assert_eq!(parsed["admitted_document_count"], 4);
    assert_eq!(parsed["terminal_result"]["run_state"], "succeeded");

    let usage = Command::new(binary).output().expect("spawn");
    assert_eq!(usage.status.code(), Some(2));
    assert!(usage.stdout.is_empty());
}
