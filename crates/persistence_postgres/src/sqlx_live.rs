//! Compiled `SQLx`/`PostgreSQL` transport for the `live-sqlx` feature.
//!
//! Excluded from the authored LLVM coverage gate via
//! `--ignore-filename-regex sqlx_live\\.rs` because a live server is required
//! for the success path. Unreachable-host failure is still unit-tested.

use crate::PersistenceError;
use crate::live_pool::{LiveSqlxPool, LiveSqlxPoolOptions};
use crate::sqlx_gate::LiveSqlxConfig;
use std::sync::Arc;

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
    let transport = SqlxTransport::connect(config, options)?;
    let shared = Arc::new(transport);
    Ok(LiveSqlxPool::from_live_executor(
        Box::new(move |sql| shared.execute(sql)),
        options,
    ))
}

/// Owned Tokio runtime + `PgPool` pair.
#[derive(Debug)]
struct SqlxTransport {
    runtime: tokio::runtime::Runtime,
    pool: sqlx::PgPool,
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
        Ok(Self { runtime, pool })
    }

    fn execute(&self, sql: &str) -> Result<(), PersistenceError> {
        self.runtime
            .block_on(async { sqlx::query(sql).execute(&self.pool).await })
            .map(|_| ())
            .map_err(|_| PersistenceError::SqlExecutionFailed)
    }
}
