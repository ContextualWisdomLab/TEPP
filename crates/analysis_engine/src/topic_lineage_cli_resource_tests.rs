//! Low-memory regression for dense CLI admission.
use super::*;

fn expanded_input(documents: usize, groups_per_document: usize) -> serde_json::Value {
    let mut value: serde_json::Value = serde_json::from_str(include_str!(
        "../../../examples/topic_lineage_cli_input_v1.json"
    ))
    .unwrap();
    let document = value["documents"][0].clone();
    let membership = value["memberships"][0].clone();
    value["documents"] = serde_json::json!([]);
    value["memberships"] = serde_json::json!([]);
    value["relations"] = serde_json::json!([]);
    for row in 0..documents {
        let id = Uuid::from_u128(row as u128 + 1).to_string();
        let mut item = document.clone();
        item["document_id"] = serde_json::json!(id);
        value["documents"].as_array_mut().unwrap().push(item);
        for group in 0..groups_per_document {
            let mut item = membership.clone();
            item["document_id"] = serde_json::json!(id);
            item["group_id"] = serde_json::json!(
                Uuid::from_u128((row * groups_per_document + group + 1) as u128).to_string()
            );
            value["memberships"].as_array_mut().unwrap().push(item);
        }
    }
    // Identical event times stop the old constructor before design allocation;
    // this witness establishes guard ordering, not an observed OOM. The
    // tolerance is also invalid, but that validation would happen later.
    value["model"]["tolerance"] = serde_json::json!(0.0);
    value
}

#[test]
fn dense_coefficients_are_refused_before_reference_construction() {
    let mut value = expanded_input(2, 10_000);
    value["vocabulary_size"] = serde_json::json!(500);
    value["model"]["topic_count"] = serde_json::json!(500);
    value["model"]["seeds"] = serde_json::json!([1]);
    value["model"]["maximum_iterations"] = serde_json::json!(2);
    assert_eq!(preflight(value), Err(TopicLineageCliError::LimitExceeded));
}

#[test]
fn dense_expansion_is_refused_before_reference_allocation() {
    let bytes = serde_json::to_vec(&expanded_input(512, 8)).unwrap();
    assert!(bytes.len() < TOPIC_LINEAGE_CLI_INPUT_BYTE_LIMIT);
    assert_eq!(
        run_topic_lineage(&bytes),
        Err(TopicLineageCliError::LimitExceeded)
    );
}

fn preflight(value: serde_json::Value) -> Result<(), TopicLineageCliError> {
    let input: CliInput = serde_json::from_value(value).unwrap();
    check_limits(&input)?;
    let cutoff = KnowledgeCutoff::parse_rfc3339(&input.request.knowledge_cutoff).unwrap();
    let corpus = admit_documents(&input, cutoff)?;
    let (network, _) = admit_memberships(&input.memberships, &corpus, cutoff)?;
    check_dense_design_limits(
        corpus.ids.len(),
        &corpus.ids,
        &corpus.event_times,
        &network,
        input.vocabulary_size,
        &input.model,
    )
}

#[test]
fn exact_payload_boundary_and_one_context_over() {
    let mut value = expanded_input(512, 8);
    value["model"]["maximum_iterations"] = serde_json::json!(4);
    value["model"]["seeds"] = serde_json::json!([1]);
    value["memberships"].as_array_mut().unwrap().truncate(4094);
    assert_eq!(
        512_u128 * (4094 + 2) * 8,
        TOPIC_LINEAGE_CLI_DESIGN_BYTE_LIMIT
    );
    assert_eq!(preflight(value.clone()), Ok(()));
    let mut assignment = value["memberships"][0].clone();
    assignment["group_id"] = serde_json::json!(Uuid::from_u128(99999).to_string());
    value["memberships"]
        .as_array_mut()
        .unwrap()
        .push(assignment);
    assert_eq!(preflight(value), Err(TopicLineageCliError::LimitExceeded));
}

#[test]
fn repeated_inactive_late_and_zero_weight_contexts_match_reference_semantics() {
    let mut value = expanded_input(512, 8);
    value["model"]["maximum_iterations"] = serde_json::json!(4);
    value["model"]["seeds"] = serde_json::json!([1]);
    // Distinct member keys, eight shared contexts rather than 4096 columns.
    for (index, assignment) in value["memberships"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        assignment["group_id"] =
            serde_json::json!(Uuid::from_u128((index % 8 + 1) as u128).to_string());
    }
    assert_eq!(preflight(value), Ok(()));
    for field in ["available_time", "valid_from", "valid_to"] {
        let mut value = expanded_input(512, 8);
        for assignment in value["memberships"].as_array_mut().unwrap() {
            if field == "valid_from" {
                assignment["valid_from"] = serde_json::json!("2026-07-02T00:00:00Z");
            } else if field == "valid_to" {
                assignment["valid_from"] = serde_json::json!("2026-06-01T00:00:00Z");
                assignment["valid_to"] = serde_json::json!("2026-07-01T05:59:59Z");
            } else {
                assignment[field] = serde_json::json!("2026-08-02T00:00:00Z");
            }
        }
        assert_eq!(preflight(value), Ok(()), "{field}");
    }
    let mut value = expanded_input(512, 8);
    for assignment in value["memberships"].as_array_mut().unwrap() {
        assignment["weight"] = serde_json::json!(0.0);
    }
    assert_eq!(preflight(value), Err(TopicLineageCliError::LimitExceeded));
}

#[test]
fn dense_fit_work_is_bounded_even_for_a_small_design() {
    let mut value = expanded_input(32, 4);
    value["model"]["seeds"] = serde_json::json!([1]);
    value["model"]["maximum_iterations"] = serde_json::json!(100_000);
    assert_eq!(preflight(value.clone()), Ok(()));
    value["model"]["seeds"] = serde_json::json!([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]);
    assert_eq!(preflight(value), Err(TopicLineageCliError::LimitExceeded));
}

#[test]
fn offered_late_documents_do_not_charge_admitted_design_rows() {
    let mut value = expanded_input(512, 8);
    value["model"]["maximum_iterations"] = serde_json::json!(4);
    value["model"]["seeds"] = serde_json::json!([1]);
    for document in value["documents"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .skip(256)
    {
        document["available_time"] = serde_json::json!("2026-08-02T00:00:00Z");
    }
    assert_eq!(preflight(value), Ok(()));
}

#[test]
fn same_group_with_distinct_roles_remains_distinct_contexts() {
    let mut value = expanded_input(512, 8);
    value["model"]["maximum_iterations"] = serde_json::json!(4);
    value["model"]["seeds"] = serde_json::json!([1]);
    value["memberships"].as_array_mut().unwrap().truncate(4094);
    let mut assignment = value["memberships"][0].clone();
    assignment["role"] = serde_json::json!("author");
    value["memberships"]
        .as_array_mut()
        .unwrap()
        .push(assignment);
    assert_eq!(preflight(value), Err(TopicLineageCliError::LimitExceeded));
}
