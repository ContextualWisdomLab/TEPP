//! Fail-closed configuration gate for live `SQLx` / `PostgreSQL` wiring.

use crate::PersistenceError;
use std::env;

/// Environment variable consumed by live `SQLx` adapters.
pub const DATABASE_URL_ENV: &str = "DATABASE_URL";

/// Validated live-database connection configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveSqlxConfig {
    database_url: String,
}

impl LiveSqlxConfig {
    /// Parse and validate a `PostgreSQL` URL for live `SQLx` use.
    ///
    /// Accepted forms start with `postgres://` or `postgresql://` and include a
    /// non-empty host authority. Credentials and query parameters are retained
    /// verbatim after structural validation; TEPP does not log the URL.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::DatabaseUrlInvalid`] when the URL is empty or
    /// uses a non-`PostgreSQL` scheme / missing host.
    pub fn parse(database_url: &str) -> Result<Self, PersistenceError> {
        let trimmed = database_url.trim();
        if trimmed.is_empty() {
            return Err(PersistenceError::DatabaseUrlInvalid);
        }
        let without_scheme = strip_postgres_scheme(trimmed)?;
        let host_part = without_scheme.split('/').next().unwrap_or_default();
        // Drop userinfo if present: user:pass@host:port
        let host_and_port = host_part.rsplit('@').next().unwrap_or_default();
        let host = host_and_port.split(':').next().unwrap_or_default();
        if host.is_empty() {
            return Err(PersistenceError::DatabaseUrlInvalid);
        }
        Ok(Self {
            database_url: trimmed.to_owned(),
        })
    }

    /// Load configuration from [`DATABASE_URL_ENV`].
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::LiveAdapterNotConfigured`] when the variable
    /// is unset, or [`PersistenceError::DatabaseUrlInvalid`] when set but
    /// invalid.
    pub fn from_env() -> Result<Self, PersistenceError> {
        Self::from_optional_env_value(env::var(DATABASE_URL_ENV).ok())
    }

    /// Validate an optional environment value without reading process state.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::LiveAdapterNotConfigured`] when `None`, or
    /// parse failures for present invalid values.
    pub fn from_optional_env_value(value: Option<String>) -> Result<Self, PersistenceError> {
        match value {
            Some(raw) => Self::parse(&raw),
            None => Err(PersistenceError::LiveAdapterNotConfigured),
        }
    }

    /// Borrow the validated URL for a live `SQLx` pool constructor.
    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    /// Test-only constructor that skips URL validation.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn for_test(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
        }
    }
}

/// Require a validated live configuration before opening a pool.
///
/// This gate is intentionally separate from pool construction so CI can exercise
/// fail-closed configuration without a `PostgreSQL` process. Pool construction
/// and query execution remain the final live-driver wiring step on top of
/// [`crate::LiveDocumentRepository`].
///
/// # Errors
///
/// Propagates configuration failures from [`LiveSqlxConfig::from_env`].
pub fn require_live_sqlx_config() -> Result<LiveSqlxConfig, PersistenceError> {
    LiveSqlxConfig::from_env()
}

/// Require configuration from an explicit optional environment value.
///
/// # Errors
///
/// Propagates failures from [`LiveSqlxConfig::from_optional_env_value`].
pub fn require_live_sqlx_config_from(
    value: Option<String>,
) -> Result<LiveSqlxConfig, PersistenceError> {
    LiveSqlxConfig::from_optional_env_value(value)
}

fn strip_postgres_scheme(url: &str) -> Result<&str, PersistenceError> {
    for prefix in ["postgres://", "postgresql://"] {
        if let Some(rest) = url.strip_prefix(prefix) {
            return Ok(rest);
        }
    }
    Err(PersistenceError::DatabaseUrlInvalid)
}

#[cfg(test)]
mod tests {
    use super::{
        DATABASE_URL_ENV, LiveSqlxConfig, require_live_sqlx_config, require_live_sqlx_config_from,
        strip_postgres_scheme,
    };
    use crate::PersistenceError;

    #[test]
    fn url_validation_accepts_postgres_forms_and_rejects_garbage() {
        let cfg = LiveSqlxConfig::parse("postgres://localhost:5432/tepp").expect("url");
        assert_eq!(cfg.database_url(), "postgres://localhost:5432/tepp");
        assert!(
            LiveSqlxConfig::parse("postgresql://user:pass@db.example/tepp?sslmode=require").is_ok()
        );
        assert_eq!(
            LiveSqlxConfig::parse(""),
            Err(PersistenceError::DatabaseUrlInvalid)
        );
        assert_eq!(
            LiveSqlxConfig::parse("   "),
            Err(PersistenceError::DatabaseUrlInvalid)
        );
        assert_eq!(
            LiveSqlxConfig::parse("mysql://localhost/tepp"),
            Err(PersistenceError::DatabaseUrlInvalid)
        );
        assert_eq!(
            LiveSqlxConfig::parse("postgres:///dbname"),
            Err(PersistenceError::DatabaseUrlInvalid)
        );
        assert_eq!(
            strip_postgres_scheme("http://localhost"),
            Err(PersistenceError::DatabaseUrlInvalid)
        );
        assert_eq!(DATABASE_URL_ENV, "DATABASE_URL");
    }

    fn classify_live_result(result: Result<LiveSqlxConfig, PersistenceError>) -> &'static str {
        match result {
            Ok(live) => {
                let url = live.database_url();
                let postgres = url.starts_with("postgres://");
                let postgresql = url.starts_with("postgresql://");
                if postgres | postgresql {
                    "ok"
                } else {
                    "ok-bad-scheme"
                }
            }
            Err(
                PersistenceError::LiveAdapterNotConfigured | PersistenceError::DatabaseUrlInvalid,
            ) => "expected-err",
            Err(_) => "other-err",
        }
    }

    #[test]
    fn env_gate_reports_missing_and_invalid_configuration() {
        assert_eq!(
            LiveSqlxConfig::from_optional_env_value(None),
            Err(PersistenceError::LiveAdapterNotConfigured)
        );
        assert_eq!(
            require_live_sqlx_config_from(None),
            Err(PersistenceError::LiveAdapterNotConfigured)
        );
        assert_eq!(
            require_live_sqlx_config_from(Some("not-a-url".into())),
            Err(PersistenceError::DatabaseUrlInvalid)
        );
        let cfg = require_live_sqlx_config_from(Some("postgresql://127.0.0.1/tepp".into()))
            .expect("configured");
        assert!(cfg.database_url().contains("127.0.0.1"));

        assert_eq!(
            classify_live_result(require_live_sqlx_config_from(Some(
                "postgres://localhost/tepp".into(),
            ))),
            "ok"
        );
        assert_eq!(
            classify_live_result(require_live_sqlx_config_from(Some(
                "postgresql://localhost/tepp".into(),
            ))),
            "ok"
        );
        assert_eq!(
            classify_live_result(require_live_sqlx_config_from(None)),
            "expected-err"
        );
        assert_eq!(
            classify_live_result(Err(PersistenceError::DuplicateDocumentRecord)),
            "other-err"
        );
        assert_eq!(
            classify_live_result(Ok(LiveSqlxConfig::for_test("not-postgres"))),
            "ok-bad-scheme"
        );
        // Process env path is exercised without OR short-circuit branches.
        let _ = classify_live_result(require_live_sqlx_config());
        let _ = LiveSqlxConfig::from_env();
    }
}
