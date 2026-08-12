#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! `PostgreSQL`-oriented bitemporal persistence contracts for TEPP.
//!
//! This crate owns migration SQL contracts, knowledge-cutoff eligibility,
//! in-memory bitemporal adapters, live SQL session/migration ports, document
//! SQL contracts, and a fail-closed `DATABASE_URL` gate for `SQLx` pool wiring
//! (ADR 0013). In-process transports keep CI deterministic; a validated live
//! URL is required before any production pool is opened.

mod cutoff;
mod document_sql;
mod document_store;
mod error;
mod live_repository;
mod migration;
mod naming;
mod sql_session;
mod sqlx_gate;

/// Knowledge-cutoff eligibility for historical analytical reads.
pub use cutoff::is_cutoff_eligible;
/// Render append-only audit insert SQL.
pub use document_sql::append_audit_sql;
/// Render as-known-at selection SQL.
pub use document_sql::as_known_at_sql;
/// Render as-valid-at selection SQL.
pub use document_sql::as_valid_at_sql;
/// Render open-document insert SQL.
pub use document_sql::insert_document_sql;
/// Render revise close+insert SQL pair.
pub use document_sql::revise_document_sqls;
/// Append-only audit event.
pub use document_store::AuditEvent;
/// Bitemporal document version.
pub use document_store::DocumentRecord;
/// In-memory bitemporal document store.
pub use document_store::DocumentStore;
/// Migration SQL contract violations.
pub use error::MigrationContractError;
/// Fail-closed persistence domain errors.
pub use error::PersistenceError;
/// Live document repository over a SQL transport.
pub use live_repository::LiveDocumentRepository;
/// Migration application failures on the live path.
pub use live_repository::LiveMigrationError;
/// Embedded and ad-hoc migration catalogs.
pub use migration::MigrationCatalog;
/// Validate migration SQL against TEPP contracts.
pub use migration::validate_migration_catalog;
/// Multi-word `snake_case` database object naming.
pub use naming::is_multi_word_snake_case;
/// Recording SQL transport for offline contract tests.
pub use sql_session::RecordingSqlSession;
/// Live SQL transport contract.
pub use sql_session::SqlSession;
/// Apply a SQL batch through a live session.
pub use sql_session::apply_sql_batch;
/// Split migration SQL into executable statements.
pub use sql_session::split_sql_statements;
/// Environment variable name for live `SQLx` configuration.
pub use sqlx_gate::DATABASE_URL_ENV;
/// Validated live `SQLx` connection configuration.
pub use sqlx_gate::LiveSqlxConfig;
/// Require a validated live `SQLx` configuration from the environment.
pub use sqlx_gate::require_live_sqlx_config;
/// Require live `SQLx` configuration from an explicit optional value.
pub use sqlx_gate::require_live_sqlx_config_from;
