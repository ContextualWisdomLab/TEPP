//! Compiled `SQLx`/`PostgreSQL` transport for the `live-sqlx` feature.
//!
//! Excluded from the authored LLVM coverage gate via
//! `--ignore-filename-regex sqlx_live\\.rs` because a live server is required
//! for the success path. Unreachable-host failure is still unit-tested.

use crate::classify_lifecycle_sql_failure;
use crate::classify_write_conflict;
use crate::live_pool::{LiveSqlCommand, LiveSqlResult, LiveSqlxPool, LiveSqlxPoolOptions};
use crate::sqlx_gate::LiveSqlxConfig;
use crate::{
    AnalysisRunRequestRecord, AnalysisRunWorkerSnapshot, PersistenceError,
    ReproducibilityManifestRecord,
};
use sha2::{Digest, Sha256};
use sqlx::{Acquire, Row};
use std::future::Future;
use temporal_core::{AvailableTime, SystemTime};
use tepp_api::{
    AnalysisRunRequest, AnalysisRunStatus, AnalysisRunStatusState, require_status_binding,
};
use uuid::Uuid;

/// Open a live `SQLx` pool and wrap it as [`LiveSqlxPool`].
///
/// # Errors
///
/// Returns [`PersistenceError::SqlExecutionFailed`] when the runtime or
/// connection cannot be established.
pub fn open_sqlx_pool(
    config: &LiveSqlxConfig,
    options: LiveSqlxPoolOptions,
) -> Result<LiveSqlxPool, PersistenceError> {
    let mut transport = SqlxTransport::connect(config, options)?;
    Ok(LiveSqlxPool::from_live_executor(
        Box::new(move |command| transport.run(command)),
        options,
    ))
}

/// Owned Tokio runtime and one pool-backed connection.
///
/// The connection is deliberately acquired once and retained for the lifetime
/// of the transport. Tenant GUC binding and the following statement must use
/// the same `PostgreSQL` session; acquiring independently for every statement
/// could send them to different pool connections and violate row-level
/// isolation.
#[derive(Debug)]
struct SqlxTransport {
    connection: Option<sqlx::pool::PoolConnection<sqlx::Postgres>>,
    runtime: Option<tokio::runtime::Runtime>,
}

/// Drive a future on the transport-owned runtime from any caller context.
///
/// Tokio's `block_on` panics when invoked from within an asynchronous
/// execution context, so the future is driven on a scoped OS thread that has
/// no ambient Tokio context. The call stays synchronous for the caller and the
/// joined result keeps teardown deterministic.
///
/// # Panics
///
/// Propagates panics from the driven future itself.
fn drive_on_owned_runtime<'env, F, T>(runtime: &tokio::runtime::Runtime, future: F) -> T
where
    F: Future<Output = T> + Send + 'env,
    T: Send + 'env,
{
    let handle = runtime.handle().clone();
    std::thread::scope(|scope| {
        scope
            .spawn(move || handle.block_on(future))
            .join()
            .expect("owned-runtime bridge thread")
    })
}

impl SqlxTransport {
    fn run(&mut self, command: LiveSqlCommand) -> Result<LiveSqlResult, PersistenceError> {
        match command {
            LiveSqlCommand::Execute(sql) => {
                self.execute(&sql)?;
                Ok(LiveSqlResult::Executed)
            }
            LiveSqlCommand::ExecuteTransaction(statements) => {
                self.execute_transaction(&statements)?;
                Ok(LiveSqlResult::Executed)
            }
            LiveSqlCommand::TryAnalysisRunLock {
                tenant_record_id,
                analysis_run_id,
            } => self
                .analysis_run_lock("pg_try_advisory_lock", tenant_record_id, analysis_run_id)
                .map(LiveSqlResult::LockState),
            LiveSqlCommand::UnlockAnalysisRun {
                tenant_record_id,
                analysis_run_id,
            } => self
                .analysis_run_lock("pg_advisory_unlock", tenant_record_id, analysis_run_id)
                .map(LiveSqlResult::LockState),
            LiveSqlCommand::LoadAnalysisRun {
                tenant_record_id,
                analysis_run_id,
            } => self
                .load_analysis_run(tenant_record_id, analysis_run_id)
                .map(Box::new)
                .map(LiveSqlResult::AnalysisRun),
            LiveSqlCommand::LoadReproducibilityManifest {
                tenant_record_id,
                reproducibility_manifest_id,
            } => self
                .load_reproducibility_manifest(tenant_record_id, reproducibility_manifest_id)
                .map(Box::new)
                .map(LiveSqlResult::ReproducibilityManifest),
        }
    }

    fn connect(
        config: &LiveSqlxConfig,
        options: LiveSqlxPoolOptions,
    ) -> Result<Self, PersistenceError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|_| PersistenceError::SqlExecutionFailed)?;
        let pool = drive_on_owned_runtime(&runtime, async {
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(options.max_connections())
                .acquire_timeout(std::time::Duration::from_millis(
                    options.acquire_timeout_ms(),
                ))
                .connect(config.database_url())
                .await
        });
        let pool = pool.map_err(|_| PersistenceError::SqlExecutionFailed)?;
        let connection = drive_on_owned_runtime(&runtime, async { pool.acquire().await })
            .map_err(|_| PersistenceError::SqlExecutionFailed)?;
        Ok(Self {
            connection: Some(connection),
            runtime: Some(runtime),
        })
    }

    fn execute(&mut self, sql: &str) -> Result<(), PersistenceError> {
        let connection = self
            .connection
            .as_mut()
            .ok_or(PersistenceError::SqlExecutionFailed)?;
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(PersistenceError::SqlExecutionFailed)?;
        drive_on_owned_runtime(runtime, async {
            sqlx::query(sql).execute(&mut **connection).await
        })
        .map(|_| ())
        .map_err(|error| map_sqlx_error(&error))
    }

    fn execute_transaction(&mut self, statements: &[String]) -> Result<(), PersistenceError> {
        let connection = self
            .connection
            .as_mut()
            .ok_or(PersistenceError::SqlExecutionFailed)?;
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(PersistenceError::SqlExecutionFailed)?;
        drive_on_owned_runtime(runtime, async {
            let mut transaction = connection
                .begin()
                .await
                .map_err(|error| map_sqlx_error(&error))?;
            for statement in statements {
                sqlx::query(statement)
                    .execute(&mut *transaction)
                    .await
                    .map_err(|error| map_sqlx_error(&error))?;
            }
            transaction
                .commit()
                .await
                .map_err(|error| map_sqlx_error(&error))
        })
    }

    fn analysis_run_lock(
        &mut self,
        function_name: &str,
        tenant_record_id: Uuid,
        analysis_run_id: Uuid,
    ) -> Result<bool, PersistenceError> {
        let connection = self
            .connection
            .as_mut()
            .ok_or(PersistenceError::SqlExecutionFailed)?;
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(PersistenceError::SqlExecutionFailed)?;
        let sql = format!("SELECT {function_name}(hashtextextended($1, 0)) AS lock_state");
        let lock_identity = format!("{tenant_record_id}:{analysis_run_id}");
        drive_on_owned_runtime(runtime, async {
            sqlx::query(&sql)
                .bind(lock_identity)
                .fetch_one(&mut **connection)
                .await
                .and_then(|row| row.try_get::<bool, _>("lock_state"))
        })
        .map_err(|error| map_sqlx_error(&error))
    }

    fn load_analysis_run(
        &mut self,
        tenant_record_id: Uuid,
        analysis_run_id: Uuid,
    ) -> Result<AnalysisRunWorkerSnapshot, PersistenceError> {
        const SQL: &str = "SELECT r.analysis_run_id::text AS analysis_run_id, r.tenant_record_id::text AS tenant_record_id, r.request_payload, r.request_payload_sha256, to_char(r.system_time AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS request_system_time, to_char(r.available_time AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS request_available_time, e.run_state_code, e.terminal_payload::text AS terminal_payload FROM analysis_run_request AS r JOIN LATERAL (SELECT run_state_code, terminal_payload FROM analysis_run_state_event WHERE tenant_record_id = r.tenant_record_id AND analysis_run_id = r.analysis_run_id ORDER BY state_sequence DESC LIMIT 1) AS e ON TRUE WHERE r.tenant_record_id = $1::uuid AND r.analysis_run_id = $2::uuid";
        let connection = self
            .connection
            .as_mut()
            .ok_or(PersistenceError::SqlExecutionFailed)?;
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(PersistenceError::SqlExecutionFailed)?;
        let row = drive_on_owned_runtime(runtime, async {
            sqlx::query(SQL)
                .bind(tenant_record_id.to_string())
                .bind(analysis_run_id.to_string())
                .fetch_one(&mut **connection)
                .await
        })
        .map_err(|error| map_sqlx_error(&error))?;
        materialize_analysis_run(&row, tenant_record_id, analysis_run_id)
    }

    fn load_reproducibility_manifest(
        &mut self,
        tenant_record_id: Uuid,
        reproducibility_manifest_id: Uuid,
    ) -> Result<ReproducibilityManifestRecord, PersistenceError> {
        const SQL: &str = "SELECT reproducibility_manifest_id::text AS reproducibility_manifest_id, tenant_record_id::text AS tenant_record_id, to_char(knowledge_cutoff AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS knowledge_cutoff, evidence_digest, code_commit_sha, dependency_lock_digest, to_char(system_time AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS system_time, to_char(available_time AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS available_time FROM reproducibility_manifest WHERE tenant_record_id = $1::uuid AND reproducibility_manifest_id = $2::uuid";
        let connection = self
            .connection
            .as_mut()
            .ok_or(PersistenceError::SqlExecutionFailed)?;
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(PersistenceError::SqlExecutionFailed)?;
        let row = drive_on_owned_runtime(runtime, async {
            sqlx::query(SQL)
                .bind(tenant_record_id.to_string())
                .bind(reproducibility_manifest_id.to_string())
                .fetch_one(&mut **connection)
                .await
        })
        .map_err(|error| map_sqlx_error(&error))?;
        let record = ReproducibilityManifestRecord {
            reproducibility_manifest_id: parse_manifest_uuid_column(
                &row,
                "reproducibility_manifest_id",
            )?,
            tenant_record_id: parse_manifest_uuid_column(&row, "tenant_record_id")?,
            knowledge_cutoff: AvailableTime::parse_rfc3339(&manifest_text_column(
                &row,
                "knowledge_cutoff",
            )?)
            .map_err(|_| PersistenceError::InvalidContentDigest)?,
            evidence_digest: manifest_text_column(&row, "evidence_digest")?,
            code_commit_sha: manifest_text_column(&row, "code_commit_sha")?,
            dependency_lock_digest: manifest_text_column(&row, "dependency_lock_digest")?,
            system_time: SystemTime::parse_rfc3339(&manifest_text_column(&row, "system_time")?)
                .map_err(|_| PersistenceError::InvalidContentDigest)?,
            available_time: AvailableTime::parse_rfc3339(&manifest_text_column(
                &row,
                "available_time",
            )?)
            .map_err(|_| PersistenceError::InvalidContentDigest)?,
        };
        if record.tenant_record_id != tenant_record_id
            || record.reproducibility_manifest_id != reproducibility_manifest_id
        {
            return Err(PersistenceError::InvalidContentDigest);
        }
        record.validate()?;
        Ok(record)
    }
}

fn materialize_analysis_run(
    row: &sqlx::postgres::PgRow,
    expected_tenant_record_id: Uuid,
    expected_analysis_run_id: Uuid,
) -> Result<AnalysisRunWorkerSnapshot, PersistenceError> {
    let stored_run_id = parse_uuid_column(row, "analysis_run_id")?;
    let stored_tenant_id = parse_uuid_column(row, "tenant_record_id")?;
    if stored_run_id != expected_analysis_run_id || stored_tenant_id != expected_tenant_record_id {
        return Err(PersistenceError::InvalidAnalysisRun);
    }
    let request_payload = text_column(row, "request_payload")?;
    let stored_digest = text_column(row, "request_payload_sha256")?;
    let request = AnalysisRunRequest::from_json(&request_payload)
        .map_err(|_| PersistenceError::InvalidAnalysisRun)?;
    let system_time = SystemTime::parse_rfc3339(&text_column(row, "request_system_time")?)
        .map_err(|_| PersistenceError::InvalidAnalysisRun)?;
    let available_time = AvailableTime::parse_rfc3339(&text_column(row, "request_available_time")?)
        .map_err(|_| PersistenceError::InvalidAnalysisRun)?;
    let request_record = AnalysisRunRequestRecord::from_request(
        stored_tenant_id,
        &request,
        system_time,
        available_time,
    )?;
    let recomputed_digest = format!("{:x}", Sha256::digest(request_payload.as_bytes()));
    if request_record.analysis_run_id != stored_run_id
        || request_record.request_payload != request_payload
        || request_record.request_payload_sha256 != stored_digest
        || recomputed_digest != stored_digest
    {
        return Err(PersistenceError::InvalidAnalysisRun);
    }
    let accepted = request_record.accepted()?;
    let run_state = text_column(row, "run_state_code")?;
    let terminal_payload = row
        .try_get::<Option<String>, _>("terminal_payload")
        .map_err(|_| PersistenceError::InvalidAnalysisRun)?;
    let status = match (run_state.as_str(), terminal_payload) {
        ("accepted", None) => AnalysisRunStatus::accepted(&accepted),
        ("running", None) => AnalysisRunStatus::running(&accepted),
        ("succeeded" | "failed", Some(payload)) => AnalysisRunStatus::from_json(&payload),
        _ => return Err(PersistenceError::InvalidAnalysisRun),
    }
    .map_err(|_| PersistenceError::InvalidAnalysisRun)?;
    require_status_binding(&request, &accepted, &status)
        .map_err(|_| PersistenceError::InvalidAnalysisRun)?;
    if matches!(
        (run_state.as_str(), status.run_state),
        ("succeeded", AnalysisRunStatusState::Succeeded)
            | ("failed", AnalysisRunStatusState::Failed)
    ) || matches!(run_state.as_str(), "accepted" | "running")
    {
        Ok(AnalysisRunWorkerSnapshot {
            request_record,
            status,
        })
    } else {
        Err(PersistenceError::InvalidAnalysisRun)
    }
}

fn text_column(row: &sqlx::postgres::PgRow, column: &str) -> Result<String, PersistenceError> {
    row.try_get(column)
        .map_err(|_| PersistenceError::InvalidAnalysisRun)
}

fn parse_uuid_column(row: &sqlx::postgres::PgRow, column: &str) -> Result<Uuid, PersistenceError> {
    Uuid::parse_str(&text_column(row, column)?).map_err(|_| PersistenceError::InvalidAnalysisRun)
}

fn manifest_text_column(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<String, PersistenceError> {
    row.try_get(column)
        .map_err(|_| PersistenceError::InvalidContentDigest)
}

fn parse_manifest_uuid_column(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Uuid, PersistenceError> {
    Uuid::parse_str(&manifest_text_column(row, column)?)
        .map_err(|_| PersistenceError::InvalidContentDigest)
}

impl Drop for SqlxTransport {
    fn drop(&mut self) {
        let connection = self.connection.take();
        let Some(runtime) = self.runtime.take() else {
            return;
        };

        // SQLx's PoolConnection::Drop spawns a return-to-pool task and therefore
        // needs a current Tokio context. Consume it inside the owned runtime via
        // the context-free bridge so dropping from a synchronous caller, or from
        // inside another runtime's task, closes the session instead of panicking.
        if let Some(connection) = connection {
            let _ = drive_on_owned_runtime(&runtime, connection.close());
        }
        drop_runtime_safely(runtime);
    }
}

fn drop_runtime_safely(runtime: tokio::runtime::Runtime) {
    if tokio::runtime::Handle::try_current().is_ok() {
        let _ = std::thread::spawn(move || drop(runtime)).join();
    } else {
        drop(runtime);
    }
}

fn map_sqlx_error(error: &sqlx::Error) -> PersistenceError {
    if let Some(lifecycle) = classify_lifecycle_sql_failure(&error.to_string()) {
        return lifecycle;
    }
    error
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .as_deref()
        .and_then(classify_write_conflict)
        .unwrap_or(PersistenceError::SqlExecutionFailed)
}

#[cfg(test)]
mod tests {
    use super::{drive_on_owned_runtime, drop_runtime_safely};

    #[test]
    fn bridge_drives_futures_from_inside_a_foreign_runtime() {
        // Reproduces the reviewed hazard: transport machinery entered while the
        // caller already sits inside another runtime's execution context, where
        // a direct Runtime::block_on panics.
        let outer = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("outer runtime");
        let owned = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("owned runtime");
        let driven =
            outer.block_on(async { drive_on_owned_runtime(&owned, std::future::ready(7_u8)) });
        assert_eq!(driven, 7);
    }

    #[test]
    fn teardown_bridge_closes_from_inside_a_foreign_runtime_without_panicking() {
        let outer = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("outer runtime");
        let owned = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("owned runtime");
        // Mirrors Drop for SqlxTransport: a close future is consumed on the
        // owned runtime even when drop runs inside another runtime context.
        let closed = outer.block_on(async {
            drive_on_owned_runtime(&owned, async {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                true
            })
        });
        assert!(closed);
    }

    #[test]
    fn owned_runtime_drop_is_safe_inside_another_runtime() {
        let outer = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("outer runtime");
        outer.block_on(async {
            let owned = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("owned runtime");
            drop_runtime_safely(owned);
        });
    }
}
