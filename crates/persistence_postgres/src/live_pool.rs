//! Live pool options and fail-closed open gate for `SQLx`/`PostgreSQL`.

use crate::sqlx_gate::LiveSqlxConfig;
use crate::{PersistenceError, RecordingSqlSession, SqlSession};
use std::fmt;

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
    /// Returns [`PersistenceError::PoolOptionsInvalid`] when either limit is zero.
    pub fn new(max_connections: u32, acquire_timeout_ms: u64) -> Result<Self, PersistenceError> {
        if max_connections == 0 || acquire_timeout_ms == 0 {
            return Err(PersistenceError::PoolOptionsInvalid);
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

type LiveSqlExecutor = dyn FnMut(&str) -> Result<(), PersistenceError> + Send;

/// Open a live pool after validating configuration and options.
///
/// When the `live-sqlx` feature is enabled, opens a real `SQLx`/`PostgreSQL`
/// pool. Otherwise fails closed with
/// [`PersistenceError::LiveAdapterNotConfigured`].
///
/// # Errors
///
/// Returns configuration or transport failures without opening a half-initialized
/// handle.
pub fn open_live_sqlx_pool(
    config: &LiveSqlxConfig,
    options: LiveSqlxPoolOptions,
) -> Result<LiveSqlxPool, PersistenceError> {
    LiveSqlxPool::connect(config, options)
}

/// Live pool handle implementing [`SqlSession`].
pub struct LiveSqlxPool {
    backend: LiveSqlxBackend,
    options: LiveSqlxPoolOptions,
}

impl fmt::Debug for LiveSqlxPool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LiveSqlxPool")
            .field("is_live", &self.is_live())
            .field("options", &self.options)
            .finish_non_exhaustive()
    }
}

#[allow(dead_code)] // Offline is test-only; Live is feature-gated construction.
enum LiveSqlxBackend {
    Offline(RecordingSqlSession),
    Live(Box<LiveSqlExecutor>),
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
        let touched = (
            config.database_url(),
            options.max_connections(),
            options.acquire_timeout_ms(),
        );
        open_with_optional_sqlx(config, options, touched)
    }

    /// Construct an offline pool for deterministic repository tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn offline_for_tests(options: LiveSqlxPoolOptions) -> Self {
        Self {
            backend: LiveSqlxBackend::Offline(RecordingSqlSession::new()),
            options,
        }
    }

    /// Wrap a live executor produced by the compiled `SQLx` driver or tests.
    #[must_use]
    #[cfg_attr(not(any(test, feature = "live-sqlx")), allow(dead_code))]
    pub(crate) fn from_live_executor(
        executor: Box<LiveSqlExecutor>,
        options: LiveSqlxPoolOptions,
    ) -> Self {
        Self {
            backend: LiveSqlxBackend::Live(executor),
            options,
        }
    }

    /// Whether this handle is a live (non-offline) transport.
    #[must_use]
    pub fn is_live(&self) -> bool {
        matches!(self.backend, LiveSqlxBackend::Live(_))
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
            LiveSqlxBackend::Live(_) => None,
        }
    }
}

#[cfg(feature = "live-sqlx")]
fn open_with_optional_sqlx(
    config: &LiveSqlxConfig,
    options: LiveSqlxPoolOptions,
    touched: (&str, u32, u64),
) -> Result<LiveSqlxPool, PersistenceError> {
    let _ = touched;
    crate::sqlx_live::open_sqlx_pool(config, options)
}

#[cfg(not(feature = "live-sqlx"))]
fn open_with_optional_sqlx(
    _config: &LiveSqlxConfig,
    _options: LiveSqlxPoolOptions,
    touched: (&str, u32, u64),
) -> Result<LiveSqlxPool, PersistenceError> {
    let _ = touched;
    Err(PersistenceError::LiveAdapterNotConfigured)
}

impl SqlSession for LiveSqlxPool {
    fn execute(&mut self, sql: &str) -> Result<(), PersistenceError> {
        match &mut self.backend {
            LiveSqlxBackend::Offline(session) => session.execute(sql),
            LiveSqlxBackend::Live(executor) => executor(sql),
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
            Err(PersistenceError::PoolOptionsInvalid)
        );
        assert_eq!(
            LiveSqlxPoolOptions::new(4, 0),
            Err(PersistenceError::PoolOptionsInvalid)
        );
        let opts = LiveSqlxPoolOptions::new(4, 2_500).expect("opts");
        assert_eq!(opts.max_connections(), 4);
        assert_eq!(opts.acquire_timeout_ms(), 2_500);
        let defaults = LiveSqlxPoolOptions::production_defaults();
        assert_eq!(defaults.max_connections(), DEFAULT_MAX_CONNECTIONS);
        assert_eq!(defaults.acquire_timeout_ms(), DEFAULT_ACQUIRE_TIMEOUT_MS);
        assert_eq!(LiveSqlxPoolOptions::default(), defaults);

        let cfg = LiveSqlxConfig::parse("postgres://127.0.0.1:5432/tepp").expect("cfg");
        let result = open_live_sqlx_pool(&cfg, defaults);
        assert!(matches!(
            result,
            Err(PersistenceError::LiveAdapterNotConfigured | PersistenceError::SqlExecutionFailed)
        ));

        let mut offline = LiveSqlxPool::offline_for_tests(defaults);
        assert!(!offline.is_live());
        assert_eq!(offline.options(), defaults);
        offline.execute("SELECT 1").expect("offline");
        assert_eq!(offline.offline_executed().expect("log"), ["SELECT 1"]);
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
        let _ = format!("{offline:?}");
    }

    #[test]
    fn live_executor_backend_covers_success_and_failure() {
        let defaults = LiveSqlxPoolOptions::production_defaults();
        let mut live = LiveSqlxPool::from_live_executor(Box::new(|_sql| Ok(())), defaults);
        assert!(live.is_live());
        assert!(live.offline_executed().is_none());
        live.execute("SELECT 1").expect("ok");
        let _ = format!("{live:?}");
        let mut failing = LiveSqlxPool::from_live_executor(
            Box::new(|_sql| Err(PersistenceError::SqlExecutionFailed)),
            defaults,
        );
        assert_eq!(
            failing.execute("SELECT 1"),
            Err(PersistenceError::SqlExecutionFailed)
        );
    }

    #[cfg(feature = "live-sqlx")]
    #[test]
    fn compiled_sqlx_driver_fails_closed_on_unreachable_host() {
        let cfg = LiveSqlxConfig::parse("postgres://127.0.0.1:1/tepp_no_listener").expect("cfg");
        let opts = LiveSqlxPoolOptions::new(1, 250).expect("opts");
        assert!(matches!(
            open_live_sqlx_pool(&cfg, opts),
            Err(PersistenceError::SqlExecutionFailed)
        ));
    }
}
