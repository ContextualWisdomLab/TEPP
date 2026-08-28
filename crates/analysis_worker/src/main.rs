#![forbid(unsafe_code)]
//! One-shot durable TEPP analysis worker executable.

use analysis_worker::{
    AnalysisWorkerInput, MAX_WORKER_INPUT_BYTES, WorkerRuntimeIdentity, execute_one,
};
use persistence_postgres::{LiveSqlxConfig, LiveSqlxPoolOptions, open_live_sqlx_pool};
use std::io::Read;
use uuid::Uuid;

struct CliArguments {
    tenant_record_id: Uuid,
    analysis_run_id: Uuid,
    input_path: String,
    completed_at: String,
}

#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> { run_from_env().map(|output| println!("{output}")) }

fn run_from_env() -> Result<String, Box<dyn std::error::Error>> {
    let arguments = parse_arguments(&mut std::env::args().skip(1))?;
    let file = std::fs::File::open(&arguments.input_path)?;
    let mut payload = String::new();
    file.take((MAX_WORKER_INPUT_BYTES + 1) as u64)
        .read_to_string(&mut payload)?;
    let input = AnalysisWorkerInput::from_json(&payload)?;
    let config = LiveSqlxConfig::from_env()?;
    let runtime_identity = WorkerRuntimeIdentity {
        code_commit_sha: std::env::var("TEPP_CODE_COMMIT_SHA")?,
        dependency_lock_digest: std::env::var("TEPP_DEPENDENCY_LOCK_SHA256")?,
    };
    execute_live(&config, &arguments, &input, &runtime_identity).and_then(render_outcome)
}

#[rustfmt::skip]
fn execute_live(config: &LiveSqlxConfig, arguments: &CliArguments, input: &AnalysisWorkerInput, identity: &WorkerRuntimeIdentity) -> Result<analysis_worker::AnalysisWorkerOutcome, Box<dyn std::error::Error>> {
    Ok(execute_one(&mut open_live_sqlx_pool(config, LiveSqlxPoolOptions::production_defaults())?, arguments.tenant_record_id, arguments.analysis_run_id, input, identity, &arguments.completed_at)?)
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
    use super::{parse_arguments, render_outcome, render_output, required};
    use analysis_worker::AnalysisWorkerOutcome;
    use tepp_api::{AnalysisRunAccepted, AnalysisRunStatus};

    const NIL: &str = "00000000-0000-0000-0000-000000000000";

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
}
