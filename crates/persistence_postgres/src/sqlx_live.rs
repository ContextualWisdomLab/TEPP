//! Compiled `SQLx`/`PostgreSQL` transport for the `live-sqlx` feature.
//!
//! Excluded from the authored LLVM coverage gate via
//! `--ignore-filename-regex sqlx_live\\.rs` because a live server is required
//! for the success path. Unreachable-host failure is still unit-tested.

use crate::PersistenceError;
use crate::classify_lifecycle_sql_failure;
use crate::classify_write_conflict;
use crate::live_pool::{LiveSqlxPool, LiveSqlxPoolOptions};
use crate::sqlx_gate::LiveSqlxConfig;

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
        Box::new(move |sql| transport.execute(sql)),
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
    runtime: tokio::runtime::Runtime,
    connection: sqlx::pool::PoolConnection<sqlx::Postgres>,
}

impl SqlxTransport {
    fn connect(
        config: &LiveSqlxConfig,
        options: LiveSqlxPoolOptions,
    ) -> Result<Self, PersistenceError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|_| PersistenceError::SqlExecutionFailed)?;
        let pool = runtime.block_on(async {
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(options.max_connections())
                .acquire_timeout(std::time::Duration::from_millis(
                    options.acquire_timeout_ms(),
                ))
                .connect(config.database_url())
                .await
        });
        let pool = pool.map_err(|_| PersistenceError::SqlExecutionFailed)?;
        let connection = runtime
            .block_on(async { pool.acquire().await })
            .map_err(|_| PersistenceError::SqlExecutionFailed)?;
        Ok(Self {
            runtime,
            connection,
        })
    }

    fn execute(&mut self, sql: &str) -> Result<(), PersistenceError> {
        let connection = &mut self.connection;
        self.runtime
            .block_on(async { sqlx::query(sql).execute(&mut **connection).await })
            .map(|_| ())
            .map_err(|error| map_sqlx_error(&error))
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
