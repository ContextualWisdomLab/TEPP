//! Live pool options and fail-closed open gate for `SQLx`/`PostgreSQL`.

use crate::sqlx_gate::LiveSqlxConfig;
use crate::{
    AnalysisRunWorkerSnapshot, PersistenceError, RecordingSqlSession,
    ReproducibilityManifestRecord, SqlSession,
};
use std::fmt;
use uuid::Uuid;

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

pub(crate) enum LiveSqlCommand {
    Execute(String),
    ExecuteTransaction(Vec<String>),
    TryAnalysisRunLock {
        tenant_record_id: Uuid,
        analysis_run_id: Uuid,
    },
    UnlockAnalysisRun {
        tenant_record_id: Uuid,
        analysis_run_id: Uuid,
    },
    LoadAnalysisRun {
        tenant_record_id: Uuid,
        analysis_run_id: Uuid,
    },
    LoadReproducibilityManifest {
        tenant_record_id: Uuid,
        reproducibility_manifest_id: Uuid,
        expected_evidence_digest: String,
    },
}

pub(crate) enum LiveSqlResult {
    Executed,
    LockState(bool),
    AnalysisRun(Box<AnalysisRunWorkerSnapshot>),
    ReproducibilityManifest(Box<ReproducibilityManifestRecord>),
}

type LiveSqlExecutor = dyn FnMut(LiveSqlCommand) -> Result<LiveSqlResult, PersistenceError> + Send;

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

    /// Try to hold the tenant/run advisory lock on this pool's retained session.
    ///
    /// The lock is automatically released if the live connection closes.
    ///
    /// # Errors
    ///
    /// Returns a transport error when no live driver is configured or the lock
    /// query cannot be completed.
    pub fn try_lock_analysis_run(
        &mut self,
        tenant_record_id: Uuid,
        analysis_run_id: Uuid,
    ) -> Result<bool, PersistenceError> {
        match self.run_live_command(LiveSqlCommand::TryAnalysisRunLock {
            tenant_record_id,
            analysis_run_id,
        })? {
            LiveSqlResult::LockState(acquired) => Ok(acquired),
            _ => Err(PersistenceError::SqlExecutionFailed),
        }
    }

    /// Release a tenant/run advisory lock held by this pool's retained session.
    ///
    /// # Errors
    ///
    /// Returns a transport error when the live session did not hold the lock or
    /// the unlock query cannot be completed.
    pub fn unlock_analysis_run(
        &mut self,
        tenant_record_id: Uuid,
        analysis_run_id: Uuid,
    ) -> Result<(), PersistenceError> {
        match self.run_live_command(LiveSqlCommand::UnlockAnalysisRun {
            tenant_record_id,
            analysis_run_id,
        })? {
            LiveSqlResult::LockState(true) => Ok(()),
            _ => Err(PersistenceError::SqlExecutionFailed),
        }
    }

    /// Load and revalidate one tenant-bound durable run and its latest status.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed transport or analysis-run validation error when the
    /// row is absent, malformed, cross-tenant, or internally inconsistent.
    pub fn load_analysis_run(
        &mut self,
        tenant_record_id: Uuid,
        analysis_run_id: Uuid,
    ) -> Result<AnalysisRunWorkerSnapshot, PersistenceError> {
        match self.run_live_command(LiveSqlCommand::LoadAnalysisRun {
            tenant_record_id,
            analysis_run_id,
        })? {
            LiveSqlResult::AnalysisRun(snapshot) => Ok(*snapshot),
            _ => Err(PersistenceError::SqlExecutionFailed),
        }
    }

    /// Load and revalidate one tenant-bound, evidence-bound reproducibility manifest.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed transport or content-validation error when the row
    /// is absent, malformed, outside the requested tenant boundary, or does not
    /// match the digest the caller computed from its input bytes.
    pub fn load_reproducibility_manifest(
        &mut self,
        tenant_record_id: Uuid,
        reproducibility_manifest_id: Uuid,
        expected_evidence_digest: &str,
    ) -> Result<ReproducibilityManifestRecord, PersistenceError> {
        match self.run_live_command(LiveSqlCommand::LoadReproducibilityManifest {
            tenant_record_id,
            reproducibility_manifest_id,
            expected_evidence_digest: expected_evidence_digest.to_owned(),
        })? {
            LiveSqlResult::ReproducibilityManifest(record) => {
                record.validate_load_binding(
                    tenant_record_id,
                    reproducibility_manifest_id,
                    expected_evidence_digest,
                )?;
                Ok(*record)
            }
            _ => Err(PersistenceError::SqlExecutionFailed),
        }
    }

    /// Execute validated SQL statements in one database transaction.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::EmptySqlBatch`] for no statements and a
    /// transport error if any statement or the commit fails. A failed statement
    /// rolls the transaction back.
    pub fn execute_transaction(&mut self, statements: &[String]) -> Result<(), PersistenceError> {
        if statements.is_empty() {
            return Err(PersistenceError::EmptySqlBatch);
        }
        match self.run_live_command(LiveSqlCommand::ExecuteTransaction(statements.to_vec()))? {
            LiveSqlResult::Executed => Ok(()),
            _ => Err(PersistenceError::SqlExecutionFailed),
        }
    }

    fn run_live_command(
        &mut self,
        command: LiveSqlCommand,
    ) -> Result<LiveSqlResult, PersistenceError> {
        match &mut self.backend {
            LiveSqlxBackend::Offline(_) => Err(PersistenceError::LiveAdapterNotConfigured),
            LiveSqlxBackend::Live(executor) => executor(command),
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
            LiveSqlxBackend::Live(executor) => {
                match executor(LiveSqlCommand::Execute(sql.to_owned()))? {
                    LiveSqlResult::Executed => Ok(()),
                    _ => Err(PersistenceError::SqlExecutionFailed),
                }
            }
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
    use crate::{
        AnalysisRunRequestRecord, AnalysisRunWorkerSnapshot, PersistenceError,
        ReproducibilityManifestRecord, SqlSession,
    };
    use temporal_core::{AvailableTime, SystemTime};
    use tepp_api::{ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunRequest, AnalysisRunStatus};

    fn worker_snapshot() -> AnalysisRunWorkerSnapshot {
        let system_time = SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system time");
        let available_time =
            AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available time");
        let request = AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "worker-test".into(),
            tenant_workspace_id: "workspace-test".into(),
            snapshot_id: "snapshot-test".into(),
            knowledge_cutoff: "2026-01-01T00:00:00Z".into(),
            model_contract_version: "model-test-v1".into(),
            output_profile: "validation-report".into(),
        };
        let request_record = AnalysisRunRequestRecord::from_request(
            uuid::Uuid::nil(),
            &request,
            system_time,
            available_time,
        )
        .expect("request record");
        let status = AnalysisRunStatus::accepted(&request_record.accepted().expect("receipt"))
            .expect("status");
        AnalysisRunWorkerSnapshot {
            request_record,
            status,
        }
    }

    fn manifest() -> ReproducibilityManifestRecord {
        ReproducibilityManifestRecord {
            reproducibility_manifest_id: uuid::Uuid::from_u128(2),
            tenant_record_id: uuid::Uuid::nil(),
            knowledge_cutoff: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("cutoff"),
            evidence_digest: "ab".repeat(32),
            code_commit_sha: "c".repeat(40),
            dependency_lock_digest: "de".repeat(32),
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z")
                .expect("available"),
        }
    }

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
        let mut live = LiveSqlxPool::from_live_executor(
            Box::new(|command| match command {
                super::LiveSqlCommand::Execute(_)
                | super::LiveSqlCommand::ExecuteTransaction(_) => {
                    Ok(super::LiveSqlResult::Executed)
                }
                super::LiveSqlCommand::TryAnalysisRunLock { .. }
                | super::LiveSqlCommand::UnlockAnalysisRun { .. } => {
                    Ok(super::LiveSqlResult::LockState(true))
                }
                super::LiveSqlCommand::LoadAnalysisRun { .. } => {
                    Err(PersistenceError::InvalidAnalysisRun)
                }
                super::LiveSqlCommand::LoadReproducibilityManifest { .. } => {
                    Err(PersistenceError::InvalidContentDigest)
                }
            }),
            defaults,
        );
        assert!(live.is_live());
        assert!(live.offline_executed().is_none());
        live.execute("SELECT 1").expect("ok");
        let tenant = uuid::Uuid::nil();
        let run = uuid::Uuid::from_u128(1);
        assert!(live.try_lock_analysis_run(tenant, run).expect("lock"));
        live.unlock_analysis_run(tenant, run).expect("unlock");
        live.execute_transaction(&["SELECT 1".into()])
            .expect("transaction");
        assert_eq!(
            live.execute_transaction(&[]),
            Err(PersistenceError::EmptySqlBatch)
        );
        assert_eq!(
            live.load_analysis_run(tenant, run),
            Err(PersistenceError::InvalidAnalysisRun)
        );
        assert_eq!(
            live.load_reproducibility_manifest(tenant, run, &"ab".repeat(32)),
            Err(PersistenceError::InvalidContentDigest)
        );
        let mut offline = LiveSqlxPool::offline_for_tests(defaults);
        assert_eq!(
            offline.try_lock_analysis_run(tenant, run),
            Err(PersistenceError::LiveAdapterNotConfigured)
        );
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

    #[test]
    fn live_executor_loads_snapshot_and_rejects_response_type_mismatches() {
        let defaults = LiveSqlxPoolOptions::production_defaults();
        let expected = worker_snapshot();
        let returned = expected.clone();
        let mut loader = LiveSqlxPool::from_live_executor(
            Box::new(move |command| match command {
                super::LiveSqlCommand::LoadAnalysisRun { .. } => Ok(
                    super::LiveSqlResult::AnalysisRun(Box::new(returned.clone())),
                ),
                _ => Ok(super::LiveSqlResult::Executed),
            }),
            defaults,
        );
        assert_eq!(
            loader
                .load_analysis_run(uuid::Uuid::nil(), expected.request_record.analysis_run_id)
                .expect("snapshot"),
            expected
        );

        let expected_manifest = manifest();
        let returned_manifest = expected_manifest.clone();
        let mut manifest_loader = LiveSqlxPool::from_live_executor(
            Box::new(move |command| match command {
                super::LiveSqlCommand::LoadReproducibilityManifest { .. } => {
                    Ok(super::LiveSqlResult::ReproducibilityManifest(Box::new(
                        returned_manifest.clone(),
                    )))
                }
                _ => Ok(super::LiveSqlResult::Executed),
            }),
            defaults,
        );
        assert_eq!(
            manifest_loader
                .load_reproducibility_manifest(
                    expected_manifest.tenant_record_id,
                    expected_manifest.reproducibility_manifest_id,
                    &expected_manifest.evidence_digest,
                )
                .expect("manifest"),
            expected_manifest
        );
        assert_eq!(
            manifest_loader.load_reproducibility_manifest(
                expected_manifest.tenant_record_id,
                expected_manifest.reproducibility_manifest_id,
                &"ff".repeat(32),
            ),
            Err(PersistenceError::InvalidContentDigest)
        );

        let mut mismatched = LiveSqlxPool::from_live_executor(
            Box::new(|command| match command {
                super::LiveSqlCommand::Execute(_)
                | super::LiveSqlCommand::ExecuteTransaction(_) => {
                    Ok(super::LiveSqlResult::LockState(false))
                }
                _ => Ok(super::LiveSqlResult::Executed),
            }),
            defaults,
        );
        let tenant = uuid::Uuid::nil();
        let run = uuid::Uuid::from_u128(1);
        assert_eq!(
            mismatched.try_lock_analysis_run(tenant, run),
            Err(PersistenceError::SqlExecutionFailed)
        );
        assert_eq!(
            mismatched.unlock_analysis_run(tenant, run),
            Err(PersistenceError::SqlExecutionFailed)
        );
        assert_eq!(
            mismatched.load_analysis_run(tenant, run),
            Err(PersistenceError::SqlExecutionFailed)
        );
        assert_eq!(
            mismatched.load_reproducibility_manifest(tenant, run, &"ab".repeat(32)),
            Err(PersistenceError::SqlExecutionFailed)
        );
        assert_eq!(
            mismatched.execute_transaction(&["SELECT 1".into()]),
            Err(PersistenceError::SqlExecutionFailed)
        );
        assert_eq!(
            mismatched.execute("SELECT 1"),
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
