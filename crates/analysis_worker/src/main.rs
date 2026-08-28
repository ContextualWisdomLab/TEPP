#![forbid(unsafe_code)]
//! One-shot durable TEPP analysis worker executable.

use analysis_worker::{
    AnalysisWorkerInput, MAX_WORKER_INPUT_BYTES, WorkerRuntimeIdentity, execute_one,
};
use persistence_postgres::{LiveSqlxConfig, LiveSqlxPoolOptions, open_live_sqlx_pool};
use std::io::Read;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let tenant_record_id = Uuid::parse_str(&required(&mut arguments, "tenant UUID")?)?;
    let analysis_run_id = Uuid::parse_str(&required(&mut arguments, "analysis-run UUID")?)?;
    let input_path = required(&mut arguments, "input JSON path")?;
    let completed_at = required(&mut arguments, "completion RFC3339 instant")?;
    if arguments.next().is_some() {
        return Err("unexpected extra arguments".into());
    }
    let file = std::fs::File::open(input_path)?;
    let mut payload = String::new();
    file.take((MAX_WORKER_INPUT_BYTES + 1) as u64)
        .read_to_string(&mut payload)?;
    let input = AnalysisWorkerInput::from_json(&payload)?;
    let config = LiveSqlxConfig::from_env()?;
    let runtime_identity = WorkerRuntimeIdentity {
        code_commit_sha: std::env::var("TEPP_CODE_COMMIT_SHA")?,
        dependency_lock_digest: std::env::var("TEPP_DEPENDENCY_LOCK_SHA256")?,
    };
    let mut pool = open_live_sqlx_pool(&config, LiveSqlxPoolOptions::production_defaults())?;
    let outcome = execute_one(
        &mut pool,
        tenant_record_id,
        analysis_run_id,
        &input,
        &runtime_identity,
        &completed_at,
    )?;
    if let Some(artifact) = outcome.artifact_json {
        println!("{artifact}");
    } else {
        println!("{}", outcome.status.to_json()?);
    }
    Ok(())
}

fn required(
    arguments: &mut impl Iterator<Item = String>,
    name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    arguments
        .next()
        .ok_or_else(|| format!("missing {name}").into())
}
