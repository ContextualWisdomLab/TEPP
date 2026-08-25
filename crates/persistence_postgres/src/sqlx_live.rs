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
use std::future::Future;

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
