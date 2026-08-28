//! Durable analysis-run requests and lifecycle events remain tenant-bound and append-only.

use persistence_postgres::{
    AnalysisRunRequestRecord, AnalysisRunState, AnalysisRunStateEventRecord, MigrationCatalog,
    PersistenceError, insert_analysis_run_request_sql, insert_analysis_run_state_event_sql,
    model_artifact_from_analysis_result, select_analysis_run_status_sql,
    validate_migration_catalog,
};
use temporal_core::{AvailableTime, SystemTime};
use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisResultSummary, AnalysisRunRequest, AnalysisRunStatus,
    AnalysisRunTerminalResult,
};
use uuid::Uuid;

fn clocks() -> (SystemTime, AvailableTime) {
    (
        SystemTime::parse_rfc3339("2026-01-02T00:00:00Z").expect("system"),
        AvailableTime::parse_rfc3339("2026-01-02T00:00:00Z").expect("available"),
    )
}

fn request_record() -> AnalysisRunRequestRecord {
    let request = AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "buyer-import-42".into(),
        tenant_workspace_id: "workspace-7".into(),
        snapshot_id: "snapshot-9".into(),
        knowledge_cutoff: "2026-01-01T00:00:00Z".into(),
        model_contract_version: "trsl-v1".into(),
        output_profile: "measurement-summary-v1".into(),
    };
    let (system_time, available_time) = clocks();
    AnalysisRunRequestRecord::from_request(Uuid::nil(), &request, system_time, available_time)
        .expect("validated request record")
}

#[test]
fn request_insert_is_exactly_idempotent_and_status_read_is_tenant_scoped() {
    let request = AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "buyer-import-42".into(),
        tenant_workspace_id: "workspace-7".into(),
        snapshot_id: "snapshot-9".into(),
        knowledge_cutoff: "2026-01-01T00:00:00Z".into(),
        model_contract_version: "trsl-v1".into(),
        output_profile: "measurement-summary-v1".into(),
    };
    let record = request_record();

    let insert = insert_analysis_run_request_sql(&record).expect("request insert SQL");
    assert!(insert.contains("ON CONFLICT (tenant_record_id, idempotency_key) DO NOTHING"));
    assert!(insert.contains("request_payload_sha256"));
    assert!(insert.contains("request_payload"));
    assert!(insert.contains("analysis-run idempotency conflict"));
    assert!(insert.contains("GET DIAGNOSTICS inserted_count = ROW_COUNT"));
    assert!(insert.contains("1, 'accepted'"));
    let replay =
        AnalysisRunRequestRecord::from_request(Uuid::nil(), &request, clocks().0, clocks().1)
            .expect("replay");
    assert_eq!(record.analysis_run_id, replay.analysis_run_id);
    assert_eq!(
        record.accepted().expect("receipt").run_id,
        record.analysis_run_id.to_string()
    );
    let mut wrong_id = record.clone();
    wrong_id.analysis_run_id = Uuid::from_u128(1);
    assert_eq!(
        insert_analysis_run_request_sql(&wrong_id),
        Err(PersistenceError::InvalidAnalysisRun)
    );
    let mut delimiter_request = request;
    delimiter_request.output_profile = "$tepp$ RAISE EXCEPTION 'escaped';".into();
    let delimiter_record = AnalysisRunRequestRecord::from_request(
        Uuid::nil(),
        &delimiter_request,
        clocks().0,
        clocks().1,
    )
    .expect("delimiter request is valid before SQL rendering");
    assert_eq!(
        insert_analysis_run_request_sql(&delimiter_record),
        Err(PersistenceError::InvalidAnalysisRun)
    );
    let mut escaped_request = delimiter_request;
    escaped_request.output_profile = "folder\\o'clock".into();
    let escaped_record = AnalysisRunRequestRecord::from_request(
        Uuid::nil(),
        &escaped_request,
        clocks().0,
        clocks().1,
    )
    .expect("escaped request");
    assert!(
        insert_analysis_run_request_sql(&escaped_record)
            .expect("escaped SQL")
            .contains("E'folder\\\\o''clock'")
    );

    let select = select_analysis_run_status_sql(Uuid::nil(), record.analysis_run_id);
    assert!(!select.contains("r.*"));
    assert!(!select.contains("e.*"));
    assert!(select.contains("r.system_time AS request_system_time"));
    assert!(select.contains("e.system_time AS event_system_time"));
    assert!(select.contains("tenant_record_id = '00000000-0000-0000-0000-000000000000'::uuid"));
    assert!(select.contains("ORDER BY state_sequence DESC LIMIT 1"));
}

#[test]
fn state_events_enforce_forward_shapes_before_sql() {
    let record = request_record();
    let (system_time, available_time) = clocks();
    let accepted = AnalysisRunStateEventRecord {
        analysis_run_state_event_id: Uuid::from_u128(2),
        tenant_record_id: Uuid::nil(),
        analysis_run_id: record.analysis_run_id,
        state_sequence: 2,
        run_state: AnalysisRunState::Running,
        terminal_status: None,
        system_time,
        available_time,
    };
    let sql = insert_analysis_run_state_event_sql(&record, &accepted).expect("running event SQL");
    assert!(sql.contains("analysis_run_state_event"));
    assert!(sql.contains("'running'"));

    let invalid = AnalysisRunStateEventRecord {
        state_sequence: 0,
        ..accepted.clone()
    };
    assert!(insert_analysis_run_state_event_sql(&record, &invalid).is_err());
    let overflowing = AnalysisRunStateEventRecord {
        state_sequence: i64::MAX as u64 + 1,
        ..accepted.clone()
    };
    assert!(insert_analysis_run_state_event_sql(&record, &overflowing).is_err());
    let accepted_shape = AnalysisRunStateEventRecord {
        run_state: AnalysisRunState::Accepted,
        state_sequence: 1,
        ..accepted.clone()
    };
    assert!(
        insert_analysis_run_state_event_sql(&record, &accepted_shape)
            .expect("accepted shape")
            .contains("'accepted'")
    );
    let missing_terminal = AnalysisRunStateEventRecord {
        run_state: AnalysisRunState::Succeeded,
        ..accepted
    };
    assert!(insert_analysis_run_state_event_sql(&record, &missing_terminal).is_err());
}

#[test]
fn terminal_event_is_bound_to_the_exact_durable_request() {
    let record = request_record();
    let accepted = record.accepted().expect("accepted");
    let result = AnalysisRunTerminalResult::failed(
        &record.request,
        &accepted,
        "2026-01-03T00:00:00Z",
        "no_eligible_evidence",
    )
    .expect("failed result");
    let status = AnalysisRunStatus::terminal(&record.request, &accepted, result).expect("status");
    let (system_time, available_time) = clocks();
    let event = AnalysisRunStateEventRecord {
        analysis_run_state_event_id: Uuid::from_u128(3),
        tenant_record_id: record.tenant_record_id,
        analysis_run_id: record.analysis_run_id,
        state_sequence: 2,
        run_state: AnalysisRunState::Failed,
        terminal_status: Some(status),
        system_time,
        available_time,
    };
    let sql = insert_analysis_run_state_event_sql(&record, &event).expect("bound terminal SQL");
    assert!(sql.contains("'failed'"));
    assert!(sql.contains("'no_eligible_evidence'"));

    let mut other = request_record();
    other.analysis_run_id = Uuid::from_u128(99);
    assert!(insert_analysis_run_state_event_sql(&other, &event).is_err());
    let cross_tenant = AnalysisRunStateEventRecord {
        tenant_record_id: Uuid::from_u128(98),
        ..event.clone()
    };
    assert!(insert_analysis_run_state_event_sql(&record, &cross_tenant).is_err());
    let mismatched = AnalysisRunStateEventRecord {
        run_state: AnalysisRunState::Succeeded,
        ..event.clone()
    };
    assert!(insert_analysis_run_state_event_sql(&record, &mismatched).is_err());

    let summary = AnalysisResultSummary::new("trsl", 3, 2, "validated").expect("summary");
    let succeeded = AnalysisRunTerminalResult::succeeded(
        &record.request,
        &accepted,
        Uuid::from_u128(44).to_string(),
        "a".repeat(64),
        "result-v1",
        "2026-01-03T00:00:00Z",
        summary,
    )
    .expect("success result");
    let success_status =
        AnalysisRunStatus::terminal(&record.request, &accepted, succeeded).expect("success status");
    let success_event = AnalysisRunStateEventRecord {
        run_state: AnalysisRunState::Succeeded,
        terminal_status: Some(success_status),
        ..event
    };
    assert!(
        insert_analysis_run_state_event_sql(&record, &success_event)
            .expect("success SQL")
            .contains("'00000000-0000-0000-0000-00000000002c'::uuid")
    );
}

#[test]
fn succeeded_result_builds_its_referenced_artifact_row() {
    let record = request_record();
    let accepted = record.accepted().expect("accepted");
    let result = AnalysisRunTerminalResult::succeeded(
        &record.request,
        &accepted,
        Uuid::from_u128(44).to_string(),
        "a".repeat(64),
        "result-v1",
        "2026-01-03T00:00:00Z",
        AnalysisResultSummary::new("trsl", 3, 2, "validated").expect("summary"),
    )
    .expect("success result");
    let (system_time, available_time) = clocks();
    let artifact = model_artifact_from_analysis_result(
        record.tenant_record_id,
        Uuid::from_u128(45),
        &result,
        Some("protected/result/44".into()),
        system_time,
        available_time,
    )
    .expect("artifact row");
    assert_eq!(artifact.model_artifact_id, Uuid::from_u128(44));
    assert_eq!(artifact.artifact_content_digest, "a".repeat(64));
    assert_eq!(artifact.artifact_type_code, "result-v1");

    let failed = AnalysisRunTerminalResult::failed(
        &record.request,
        &accepted,
        "2026-01-03T00:00:00Z",
        "no_eligible_evidence",
    )
    .expect("failed result");
    assert!(
        model_artifact_from_analysis_result(
            record.tenant_record_id,
            Uuid::from_u128(45),
            &failed,
            None,
            system_time,
            available_time,
        )
        .is_err()
    );
    let mutators: [fn(&mut AnalysisRunTerminalResult); 3] = [
        |value: &mut AnalysisRunTerminalResult| value.result_artifact_id = None,
        |value: &mut AnalysisRunTerminalResult| value.result_sha256 = None,
        |value: &mut AnalysisRunTerminalResult| value.result_schema_version = None,
    ];
    for mutate in mutators {
        let mut invalid = result.clone();
        mutate(&mut invalid);
        assert!(
            model_artifact_from_analysis_result(
                record.tenant_record_id,
                Uuid::from_u128(45),
                &invalid,
                None,
                system_time,
                available_time,
            )
            .is_err()
        );
    }
    let mut invalid_id = result;
    invalid_id.result_artifact_id = Some("opaque-artifact".into());
    assert!(
        model_artifact_from_analysis_result(
            record.tenant_record_id,
            Uuid::from_u128(45),
            &invalid_id,
            None,
            system_time,
            available_time,
        )
        .is_err()
    );
}

#[test]
fn tampered_request_records_fail_before_sql() {
    let mut invalid = request_record();
    invalid.request.output_profile.clear();
    assert!(insert_analysis_run_request_sql(&invalid).is_err());

    let mut tampered = request_record();
    tampered.request_payload.push(' ');
    assert!(insert_analysis_run_request_sql(&tampered).is_err());
    let mut tampered_digest = request_record();
    tampered_digest.request_payload_sha256 = "0".repeat(64);
    assert!(insert_analysis_run_request_sql(&tampered_digest).is_err());
    assert_eq!(
        PersistenceError::InvalidAnalysisRun.to_string(),
        "invalid analysis run"
    );
}

#[test]
fn embedded_migration_normalizes_requests_and_append_only_events() {
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    validate_migration_catalog(&catalog).expect("valid embedded migration");
    let sql = catalog.up_sql().to_ascii_lowercase();
    assert!(sql.contains("create table analysis_run_request"));
    assert!(sql.contains("create table analysis_run_state_event"));
    assert!(sql.contains("analysis_run_request_tenant_isolation"));
    assert!(sql.contains("analysis_run_state_event_tenant_isolation"));
    assert!(sql.contains("analysis_run_request_reject_mutation"));
    assert!(sql.contains("analysis_run_state_event_reject_mutation"));
    assert!(sql.contains("where run_state_code in ('succeeded', 'failed')"));
    assert!(sql.contains("create trigger analysis_run_state_event_validate_transition"));
    assert!(sql.contains("for update"));
    assert!(sql.contains("analysis-run artifact digest mismatch"));
    assert!(sql.contains("analysis_run_state_event_artifact_fk"));
    assert!(
        sql.contains(
            "create unique index concurrently model_artifact_tenant_identity_unique_index"
        )
    );
    assert!(sql.contains("unique using index model_artifact_tenant_identity_unique_index"));
    assert!(
        catalog
            .down_sql()
            .to_ascii_lowercase()
            .contains("drop index if exists model_artifact_tenant_identity_unique_index")
    );
}
