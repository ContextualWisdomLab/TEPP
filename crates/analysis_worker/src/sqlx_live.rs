//! Live PostgreSQL execution boundary excluded from hermetic coverage.

use super::{CliArguments, CliInput};
use analysis_worker::{WorkerRuntimeIdentity, execute_one, execute_topic_lineage_one};
use persistence_postgres::{LiveSqlxConfig, LiveSqlxPoolOptions, open_live_sqlx_pool};

/// Execute one validated CLI input against the configured live PostgreSQL store.
pub(super) fn execute_live(
    config: &LiveSqlxConfig,
    arguments: &CliArguments,
    input: &CliInput,
    identity: &WorkerRuntimeIdentity,
) -> Result<analysis_worker::AnalysisWorkerOutcome, Box<dyn std::error::Error>> {
    let pool = &mut open_live_sqlx_pool(config, LiveSqlxPoolOptions::production_defaults())?;
    Ok(match input {
        CliInput::Evidence(input) => execute_one(
            pool,
            arguments.tenant_record_id,
            arguments.analysis_run_id,
            input,
            identity,
            &arguments.completed_at,
        )?,
        CliInput::TopicLineage(input) => execute_topic_lineage_one(
            pool,
            arguments.tenant_record_id,
            arguments.analysis_run_id,
            input,
            identity,
            &arguments.completed_at,
        )?,
    })
}
