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
use std::thread;

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
    connection: Option<sqlx::pool::PoolConnection<sqlx::Postgres>>,
    runtime: Option<tokio::runtime::Runtime>,
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
            connection: Some(connection),
            runtime: Some(runtime),
        })
    }

    fn execute(&mut self, sql: &str) -> Result<(), PersistenceError> {
        let sql = sql.to_owned();
        if tokio::runtime::Handle::try_current().is_ok() {
            let mut connection = self
                .connection
                .take()
                .ok_or(PersistenceError::SqlExecutionFailed)?;
            let Some(runtime) = self.runtime.take() else {
                self.connection = Some(connection);
                return Err(PersistenceError::SqlExecutionFailed);
            };
            let joined = thread::spawn(move || {
                let result =
                    runtime.block_on(async { sqlx::query(&sql).execute(&mut *connection).await });
                (connection, runtime, result)
            })
            .join()
            .map_err(|_| PersistenceError::SqlExecutionFailed)?;
            self.connection = Some(joined.0);
            self.runtime = Some(joined.1);
            joined.2.map(|_| ()).map_err(|error| map_sqlx_error(&error))
        } else {
            let connection = self
                .connection
                .as_mut()
                .ok_or(PersistenceError::SqlExecutionFailed)?;
            let runtime = self
                .runtime
                .as_ref()
                .ok_or(PersistenceError::SqlExecutionFailed)?;
            runtime
                .block_on(async { sqlx::query(&sql).execute(&mut **connection).await })
                .map(|_| ())
                .map_err(|error| map_sqlx_error(&error))
        }
    }
}

impl Drop for SqlxTransport {
    fn drop(&mut self) {
        let connection = self.connection.take();
        let runtime = self.runtime.take();
        let Some(runtime) = runtime else {
            return;
        };
        if connection.is_none() {
            drop_runtime_safely(runtime);
            return;
        }
        let close = move || {
            if let Some(connection) = connection {
                let _ = runtime.block_on(connection.close());
            }
        };

        // Both closing a PoolConnection and dropping a Tokio runtime may panic
        // when performed inside another runtime. Move both operations to a
        // dedicated thread in that case so synchronous teardown remains safe.
        if tokio::runtime::Handle::try_current().is_ok() {
            let _ = thread::spawn(close).join();
        } else {
            close();
        }
    }
}

fn drop_runtime_safely(runtime: tokio::runtime::Runtime) {
    if tokio::runtime::Handle::try_current().is_ok() {
        let _ = thread::spawn(move || drop(runtime)).join();
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
    #[test]
    fn owned_runtime_drop_is_safe_inside_another_runtime() {
        let outer = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("outer runtime");
        outer.block_on(async {
            let owned = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("owned runtime");
            super::drop_runtime_safely(owned);
        });
    }
}
