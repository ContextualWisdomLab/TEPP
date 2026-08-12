//! Live pool options and fail-closed open gate for `SQLx`/`PostgreSQL`.

use crate::sqlx_gate::LiveSqlxConfig;
use crate::{PersistenceError, RecordingSqlSession, SqlSession};

/// Default maximum connections for a live analytical pool.
pub const DEFAULT_MAX_CONNECTIONS: u32 = 8;

/// Default acquire timeout in milliseconds.
pub const DEFAULT_ACQUIRE_TIMEOUT_MS: u64 = 5_000;

/// Operator-facing pool sizing and acquire limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LiveSqlxPoolOptions {
    max_connections: u32,
    acquire_timeout_ms: u64,
}

impl LiveSqlxPoolOptions {
    /// Construct validated pool options.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::DatabaseUrlInvalid`] when either limit is zero.
    pub fn new(max_connections: u32, acquire_timeout_ms: u64) -> Result<Self, PersistenceError> {
        if max_connections == 0 || acquire_timeout_ms == 0 {
            return Err(PersistenceError::DatabaseUrlInvalid);
        }
        Ok(Self {
            max_connections,
            acquire_timeout_ms,
        })
    }

    /// Production defaults: eight connections and a five-second acquire timeout.
    #[must_use]
    pub fn production_defaults() -> Self {
        Self {
            max_connections: DEFAULT_MAX_CONNECTIONS,
            acquire_timeout_ms: DEFAULT_ACQUIRE_TIMEOUT_MS,
        }
    }

    /// Maximum concurrent connections.
    #[must_use]
    pub const fn max_connections(self) -> u32 {
        self.max_connections
    }

    /// Acquire timeout in milliseconds.
    #[must_use]
    pub const fn acquire_timeout_ms(self) -> u64 {
        self.acquire_timeout_ms
    }
}

impl Default for LiveSqlxPoolOptions {
    fn default() -> Self {
        Self::production_defaults()
    }
}

/// Open a live pool after validating configuration and options.
///
/// Production `SQLx` driver attachment is intentionally separate from this gate:
/// CI and offline verification keep deterministic transports. When a process
/// lacks a live driver registration (the default for TEPP foundation builds),
/// this function fails closed with
/// [`PersistenceError::LiveAdapterNotConfigured`] even if `DATABASE_URL` is
/// present, so operators cannot accidentally believe a pool is open without a
/// compiled live driver.
///
/// # Errors
///
/// Returns [`PersistenceError::LiveAdapterNotConfigured`] until a live driver
/// is linked for this process.
pub fn open_live_sqlx_pool(
    config: &LiveSqlxConfig,
    options: LiveSqlxPoolOptions,
) -> Result<LiveSqlxPool, PersistenceError> {
    LiveSqlxPool::connect(config, options)
}

/// Live pool handle implementing [`SqlSession`].
///
/// Foundation builds carry an offline recording backend so repository wiring
/// can be exercised without `PostgreSQL`. The public open path refuses to
/// return a handle until a live driver is registered for the process.
#[derive(Debug)]
pub struct LiveSqlxPool {
    backend: LiveSqlxBackend,
    options: LiveSqlxPoolOptions,
}

#[derive(Debug)]
enum LiveSqlxBackend {
    Offline(RecordingSqlSession),
}

impl LiveSqlxPool {
    /// Connect using a validated URL and pool options.
    ///
    /// # Errors
    ///
    /// See [`open_live_sqlx_pool`].
    pub fn connect(
        config: &LiveSqlxConfig,
        options: LiveSqlxPoolOptions,
    ) -> Result<Self, PersistenceError> {
        // Touch validated URL so configuration is not dead in the open path.
        let _url = config.database_url();
        let _ = options.max_connections();
        let _ = options.acquire_timeout_ms();
        // Live driver registration is intentionally absent in foundation CI.
        Err(PersistenceError::LiveAdapterNotConfigured)
    }

    /// Construct an offline pool for deterministic repository tests.
    #[must_use]
    pub fn offline_for_tests(options: LiveSqlxPoolOptions) -> Self {
        Self {
            backend: LiveSqlxBackend::Offline(RecordingSqlSession::new()),
            options,
        }
    }

    /// Whether this handle is a live (non-offline) transport.
    #[must_use]
    pub const fn is_live(&self) -> bool {
        match self.backend {
            LiveSqlxBackend::Offline(_) => false,
        }
    }

    /// Pool options used when the handle was constructed.
    #[must_use]
    pub const fn options(&self) -> LiveSqlxPoolOptions {
        self.options
    }

    /// Borrow executed offline statements when using the test backend.
    #[must_use]
    pub fn offline_executed(&self) -> Option<&[String]> {
        match &self.backend {
            LiveSqlxBackend::Offline(session) => Some(session.executed()),
        }
    }
}

impl SqlSession for LiveSqlxPool {
    fn execute(&mut self, sql: &str) -> Result<(), PersistenceError> {
        match &mut self.backend {
            LiveSqlxBackend::Offline(session) => session.execute(sql),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_ACQUIRE_TIMEOUT_MS, DEFAULT_MAX_CONNECTIONS, LiveSqlxPool, LiveSqlxPoolOptions,
        open_live_sqlx_pool,
    };
    use crate::sqlx_gate::LiveSqlxConfig;
    use crate::{PersistenceError, SqlSession};

    #[test]
    fn pool_options_and_open_gate_fail_closed_without_live_driver() {
        assert_eq!(
            LiveSqlxPoolOptions::new(0, 1_000),
            Err(PersistenceError::DatabaseUrlInvalid)
        );
        assert_eq!(
            LiveSqlxPoolOptions::new(4, 0),
            Err(PersistenceError::DatabaseUrlInvalid)
        );
        let opts = LiveSqlxPoolOptions::new(4, 2_500).expect("opts");
        assert_eq!(opts.max_connections(), 4);
        assert_eq!(opts.acquire_timeout_ms(), 2_500);
        let defaults = LiveSqlxPoolOptions::production_defaults();
        assert_eq!(defaults.max_connections(), DEFAULT_MAX_CONNECTIONS);
        assert_eq!(defaults.acquire_timeout_ms(), DEFAULT_ACQUIRE_TIMEOUT_MS);
        assert_eq!(LiveSqlxPoolOptions::default(), defaults);

        let cfg = LiveSqlxConfig::parse("postgres://127.0.0.1:5432/tepp").expect("cfg");
        assert!(matches!(
            open_live_sqlx_pool(&cfg, defaults),
            Err(PersistenceError::LiveAdapterNotConfigured)
        ));
        assert!(matches!(
            LiveSqlxPool::connect(&cfg, opts),
            Err(PersistenceError::LiveAdapterNotConfigured)
        ));

        let mut offline = LiveSqlxPool::offline_for_tests(defaults);
        assert!(!offline.is_live());
        assert_eq!(offline.options(), defaults);
        offline.execute("SELECT 1").expect("offline");
        let executed = offline.offline_executed().expect("offline log");
        assert_eq!(executed, ["SELECT 1"]);
        let mut failing = LiveSqlxPool {
            backend: super::LiveSqlxBackend::Offline(crate::RecordingSqlSession::failing_on(
                "boom",
            )),
            options: defaults,
        };
        assert_eq!(
            failing.execute("do boom now"),
            Err(PersistenceError::SqlExecutionFailed)
        );
    }
}
