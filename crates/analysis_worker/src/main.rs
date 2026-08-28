#![forbid(unsafe_code)]
//! One-shot durable TEPP analysis worker executable.

use analysis_worker::{
    AnalysisWorkerError, AnalysisWorkerInput, MAX_WORKER_INPUT_BYTES, TopicLineageWorkerInput,
    WorkerRuntimeIdentity,
};
use persistence_postgres::{LiveSqlxConfig, PersistenceError};
use std::{io::Read, process::ExitCode};
use uuid::Uuid;

mod sqlx_live;
use sqlx_live::execute_live;

const PERMANENT_EXIT_CODE: u8 = 64;
const RETRYABLE_EXIT_CODE: u8 = 75;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SchedulerDisposition {
    Permanent,
    Retryable,
}

struct CliArguments {
    tenant_record_id: Uuid,
    analysis_run_id: Uuid,
    input_path: String,
    completed_at: String,
}

enum CliInput {
    Evidence(AnalysisWorkerInput),
    TopicLineage(Box<TopicLineageWorkerInput>),
}

fn main() -> ExitCode {
    exit_code(run_from_env())
}

fn exit_code(result: Result<String, Box<dyn std::error::Error>>) -> ExitCode {
    match result {
        Ok(output) => {
            println!("{output}");
            ExitCode::from(0)
        }
        Err(error) => {
            let disposition = scheduler_disposition(error.as_ref());
            eprintln!("analysis_worker: {disposition:?}");
            ExitCode::from(match disposition {
                SchedulerDisposition::Permanent => PERMANENT_EXIT_CODE,
                SchedulerDisposition::Retryable => RETRYABLE_EXIT_CODE,
            })
        }
    }
}

fn scheduler_disposition(error: &(dyn std::error::Error + 'static)) -> SchedulerDisposition {
    if let Some(error) = error.downcast_ref::<AnalysisWorkerError>() {
        return match error {
            AnalysisWorkerError::InvalidInput | AnalysisWorkerError::UnsupportedRequest => {
                SchedulerDisposition::Permanent
            }
            AnalysisWorkerError::AlreadyLocked | AnalysisWorkerError::ExecutionFailed => {
                SchedulerDisposition::Retryable
            }
        };
    }
    if let Some(error) = error.downcast_ref::<PersistenceError>() {
        return if matches!(
            error,
            PersistenceError::DatabaseUrlInvalid
                | PersistenceError::PoolOptionsInvalid
                | PersistenceError::LiveAdapterNotConfigured
        ) {
            SchedulerDisposition::Permanent
        } else {
            SchedulerDisposition::Retryable
        };
    }
    SchedulerDisposition::Permanent
}

fn run_from_env() -> Result<String, Box<dyn std::error::Error>> {
    let arguments = parse_arguments(&mut std::env::args().skip(1))?;
    let file = std::fs::File::open(&arguments.input_path)?;
    let mut payload = String::new();
    file.take((MAX_WORKER_INPUT_BYTES + 1) as u64)
        .read_to_string(&mut payload)?;
    let input = parse_input(&payload)?;
    let config = LiveSqlxConfig::from_env()?;
    let runtime_identity = WorkerRuntimeIdentity {
        code_commit_sha: std::env::var("TEPP_CODE_COMMIT_SHA")?,
        dependency_lock_digest: std::env::var("TEPP_DEPENDENCY_LOCK_SHA256")?,
    };
    execute_live(&config, &arguments, &input, &runtime_identity).and_then(render_outcome)
}

fn parse_input(payload: &str) -> Result<CliInput, AnalysisWorkerError> {
    AnalysisWorkerInput::from_json(payload)
        .map(CliInput::Evidence)
        .or_else(|_| {
            TopicLineageWorkerInput::from_json(payload)
                .map(Box::new)
                .map(CliInput::TopicLineage)
        })
}

fn parse_arguments(
    arguments: &mut dyn Iterator<Item = String>,
) -> Result<CliArguments, Box<dyn std::error::Error>> {
    let tenant_record_id = Uuid::parse_str(&required(arguments, "tenant UUID")?)?;
    let analysis_run_id = Uuid::parse_str(&required(arguments, "analysis-run UUID")?)?;
    let input_path = required(arguments, "input JSON path")?;
    let completed_at = required(arguments, "completion RFC3339 instant")?;
    if arguments.next().is_some() {
        return Err("unexpected extra arguments".into());
    }
    Ok(CliArguments {
        tenant_record_id,
        analysis_run_id,
        input_path,
        completed_at,
    })
}

fn required(
    arguments: &mut dyn Iterator<Item = String>,
    name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    arguments
        .next()
        .ok_or_else(|| format!("missing {name}").into())
}

fn render_output<E>(
    artifact_json: Option<String>,
    status_json: impl FnOnce() -> Result<String, E>,
) -> Result<String, E> {
    artifact_json.map_or_else(status_json, Ok)
}

fn render_outcome(
    outcome: analysis_worker::AnalysisWorkerOutcome,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(render_output(outcome.artifact_json, || {
        outcome.status.to_json()
    })?)
}

#[cfg(test)]
mod tests {
    use super::{
        CliInput, PERMANENT_EXIT_CODE, RETRYABLE_EXIT_CODE, SchedulerDisposition, exit_code,
        parse_arguments, parse_input, render_outcome, render_output, required,
        scheduler_disposition,
    };
    use analysis_worker::{AnalysisWorkerError, AnalysisWorkerOutcome};
    use persistence_postgres::PersistenceError;
    use tepp_api::{AnalysisRunAccepted, AnalysisRunStatus};

    const NIL: &str = "00000000-0000-0000-0000-000000000000";

    #[test]
    fn process_exit_reports_success_and_scheduler_failures() {
        assert_eq!(
            exit_code(Ok("done".into())),
            std::process::ExitCode::SUCCESS
        );
        assert_eq!(
            exit_code(Err(Box::new(AnalysisWorkerError::InvalidInput))),
            std::process::ExitCode::from(PERMANENT_EXIT_CODE)
        );
        assert_eq!(
            exit_code(Err(Box::new(AnalysisWorkerError::ExecutionFailed))),
            std::process::ExitCode::from(RETRYABLE_EXIT_CODE)
        );
    }

    #[test]
    fn arguments_are_exact_and_bounded() {
        let mut valid = [NIL, NIL, "input.json", "2026-08-28T00:00:00Z"]
            .map(str::to_owned)
            .into_iter();
        let parsed = parse_arguments(&mut valid).expect("arguments");
        assert_eq!(parsed.input_path, "input.json");
        let mut missing = [NIL, NIL, "input.json"].map(str::to_owned).into_iter();
        assert!(parse_arguments(&mut missing).is_err());
        let mut extra = [NIL, NIL, "input.json", "now", "extra"]
            .map(str::to_owned)
            .into_iter();
        assert!(parse_arguments(&mut extra).is_err());
        let mut invalid = ["invalid"].map(str::to_owned).into_iter();
        assert!(parse_arguments(&mut invalid).is_err());
        assert!(required(&mut std::iter::empty(), "value").is_err());
    }

    #[test]
    fn input_parser_selects_only_known_versioned_envelopes() {
        let topic = r#"{
            "contract_version":1,
            "reproducibility_manifest_id":"00000000-0000-0000-0000-000000000000",
            "snapshot_id":"snapshot",
            "scientific_input_sha256":"",
            "documents":[],
            "document_term":{"columns":0,"offsets":[],"indices":[],"values":[]},
            "covariates":null,
            "memberships":[],
            "relations":[],
            "model":{"topic_count":0,"seeds":[],"maximum_iterations":0,"tolerance":0.0,"prior_variance":0.0,"relation_strength":0.0,"ridge":0.0,"topic_smoothing":0.0,"step_size":0.0}
        }"#;
        assert!(matches!(parse_input(topic), Ok(CliInput::TopicLineage(_))));
        assert!(matches!(
            parse_input("{}"),
            Err(AnalysisWorkerError::InvalidInput)
        ));
    }

    #[test]
    fn output_prefers_the_artifact_and_falls_back_to_status() {
        assert_eq!(
            render_output(Some("artifact".into()), || Err::<String, _>("unused")),
            Ok("artifact".into())
        );
        assert_eq!(
            render_output(None, || Ok::<String, &str>("status".into())),
            Ok("status".into())
        );
        let accepted = AnalysisRunAccepted::new(NIL, "accepted", "key").expect("receipt");
        let status = AnalysisRunStatus::accepted(&accepted).expect("status");
        assert!(
            render_outcome(AnalysisWorkerOutcome {
                status: status.clone(),
                artifact_json: Some("artifact".into()),
            })
            .is_ok()
        );
        assert!(
            render_outcome(AnalysisWorkerOutcome {
                status,
                artifact_json: None,
            })
            .is_ok()
        );
    }

    #[test]
    fn scheduler_disposition_separates_permanent_and_retryable_failures() {
        for error in [
            AnalysisWorkerError::InvalidInput,
            AnalysisWorkerError::UnsupportedRequest,
        ] {
            assert_eq!(
                scheduler_disposition(&error),
                SchedulerDisposition::Permanent
            );
        }
        for error in [
            AnalysisWorkerError::AlreadyLocked,
            AnalysisWorkerError::ExecutionFailed,
        ] {
            assert_eq!(
                scheduler_disposition(&error),
                SchedulerDisposition::Retryable
            );
        }
        assert_eq!(
            scheduler_disposition(&PersistenceError::DatabaseUrlInvalid),
            SchedulerDisposition::Permanent
        );
        assert_eq!(
            scheduler_disposition(&PersistenceError::LiveAdapterNotConfigured),
            SchedulerDisposition::Permanent
        );
        assert_eq!(
            scheduler_disposition(&PersistenceError::PoolOptionsInvalid),
            SchedulerDisposition::Permanent
        );
        assert_eq!(
            scheduler_disposition(&PersistenceError::SqlExecutionFailed),
            SchedulerDisposition::Retryable
        );
        assert_eq!(
            scheduler_disposition(&std::io::Error::other("redacted")),
            SchedulerDisposition::Permanent
        );
    }
}
