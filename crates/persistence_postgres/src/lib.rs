#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! PostgreSQL-oriented bitemporal persistence contracts for TEPP.
//!
//! This crate owns migration SQL contracts, knowledge-cutoff eligibility, and
//! in-memory bitemporal adapters that encode `as_known_at` / `as_valid_at`
//! replay without requiring a live database in CI. Live `SQLx` execution against
//! `PostgreSQL` remains an accepted-target follow-on behind the same contracts
//! (ADR 0013).

mod cutoff;
mod document_store;
mod error;
mod migration;
mod naming;

/// Knowledge-cutoff eligibility for historical analytical reads.
pub use cutoff::is_cutoff_eligible;
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
/// Embedded and ad-hoc migration catalogs.
pub use migration::MigrationCatalog;
/// Validate migration SQL against TEPP contracts.
pub use migration::validate_migration_catalog;
/// Multi-word `snake_case` database object naming.
pub use naming::is_multi_word_snake_case;
