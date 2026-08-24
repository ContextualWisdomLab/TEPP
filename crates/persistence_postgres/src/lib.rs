#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! `PostgreSQL`-oriented bitemporal persistence contracts for TEPP.
//!
//! This crate owns migration SQL contracts, knowledge-cutoff eligibility,
//! in-memory bitemporal adapters, live SQL session/migration ports, document
//! SQL contracts, tenant session GUC / application-role helpers for row-level
//! security, a fail-closed `DATABASE_URL` gate, and a fail-closed live pool open
//! path with validated sizing options (ADR 0013). In-process transports keep
//! default CI deterministic; optional `live-sqlx` feature compiles a real
//! `PgPool` driver behind validated URL/options, with a gated live `PostgreSQL`
//! CI job (`TEPP_LIVE_POSTGRES=1`). Append-only reproducibility-manifest SQL
//! contracts bind evidence digests, code commit, dependency lock, and knowledge
//! cutoff for run provenance (ADR 0013). Model-run and model-artifact SQL
//! contracts chain immutable run identities to those manifests. Typed
//! membership-assignment SQL (migration `0006`) replaces the polymorphic 0001 stub so documents
//! can belong to multiple entities and projects without atomistic collapse.
//! Typed `text_segment` SQL persists exact UTF-8 byte spans and cutoff-eligible
//! document lookups so segment-level membership is not raw SQL.
//! Concurrent document revises use one transactional `DO` block that requires
//! exactly one open row to close, and live `SQLx` maps racing SQLSTATEs onto
//! typed conflict errors. Restore integrity probes refuse to mark analytical
//! state usable until tenant, digest, cutoff, temporal windows, and append-only
//! triggers revalidate. Retention, deletion, and legal-hold SQL (migration
//! `0007`) records policy-driven lifecycle without restoring tombstoned
//! evidence or completing a deletion under an active hold. Analysis exclusion
//! is kind-aligned, and deletion requests bind to the cited policy.

mod artifact_sql;
mod concurrent_write;
mod cutoff;
mod document_sql;
mod document_store;
mod error;
mod instance_sql;
mod live_pool;
mod live_repository;
mod manifest_sql;
mod membership_sql;
mod mention_sql;
mod migration;
mod model_run_sql;
mod naming;
mod relation_sql;
mod restore_integrity;
mod retention_sql;
mod segment_sql;
mod sql_session;
mod sqlx_gate;
#[cfg(feature = "live-sqlx")]
mod sqlx_live;
mod tenant_session;

/// Append-only source artifact row.
pub use artifact_sql::SourceArtifactRecord;
/// Render a fail-closed stored-row match assertion for a source artifact.
pub use artifact_sql::assert_source_artifact_matches_sql;
/// Render insert SQL for a validated source artifact.
pub use artifact_sql::insert_source_artifact_sql;
/// Render selection SQL for a source artifact by primary key.
pub use artifact_sql::select_source_artifact_by_id_sql;
/// Compare two source artifacts for idempotent-retry equality.
pub use artifact_sql::source_artifacts_are_idempotent_matches;
/// `PostgreSQL` `deadlock_detected` SQLSTATE.
pub use concurrent_write::DEADLOCK_DETECTED_SQLSTATE;
/// `PostgreSQL` `exclusion_violation` SQLSTATE.
pub use concurrent_write::EXCLUSION_VIOLATION_SQLSTATE;
/// `PostgreSQL` `lock_not_available` SQLSTATE (`FOR UPDATE NOWAIT`).
pub use concurrent_write::LOCK_NOT_AVAILABLE_SQLSTATE;
/// `PostgreSQL` `serialization_failure` SQLSTATE.
pub use concurrent_write::SERIALIZATION_FAILURE_SQLSTATE;
/// `PostgreSQL` `unique_violation` SQLSTATE.
pub use concurrent_write::UNIQUE_VIOLATION_SQLSTATE;
/// Map a racing-write SQLSTATE onto a domain persistence error.
pub use concurrent_write::classify_write_conflict;
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
/// Render one transactional revise that fails closed unless one open row closes.
pub use document_sql::revise_document_atomic_sql;
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
/// Bitemporal event-instance row.
pub use instance_sql::EventInstanceRecord;
/// Render insert SQL for a validated event instance.
pub use instance_sql::insert_event_instance_sql;
/// Render as-known-at selection for one event-instance identity.
pub use instance_sql::select_event_instance_as_known_at_sql;
/// Default pool acquire timeout in milliseconds.
pub use live_pool::DEFAULT_ACQUIRE_TIMEOUT_MS;
/// Default maximum pool connections.
pub use live_pool::DEFAULT_MAX_CONNECTIONS;
/// Live `SQLx` pool handle implementing [`SqlSession`].
pub use live_pool::LiveSqlxPool;
/// Operator-facing live pool sizing options.
pub use live_pool::LiveSqlxPoolOptions;
/// Open a live `SQLx` pool from validated configuration.
pub use live_pool::open_live_sqlx_pool;
/// Live document repository over a SQL transport.
pub use live_repository::LiveDocumentRepository;
/// Migration application failures on the live path.
pub use live_repository::LiveMigrationError;
/// Append-only reproducibility manifest row.
pub use manifest_sql::ReproducibilityManifestRecord;
/// Render insert SQL for a reproducibility manifest.
pub use manifest_sql::insert_reproducibility_manifest_sql;
/// Render selection SQL by digest triple.
pub use manifest_sql::select_reproducibility_manifest_by_digests_sql;
/// Render selection SQL by primary key.
pub use manifest_sql::select_reproducibility_manifest_by_id_sql;
/// Typed membership assignment row.
pub use membership_sql::MembershipAssignmentRecord;
/// Render insert SQL for a typed membership assignment.
pub use membership_sql::insert_membership_assignment_sql;
/// Render selection SQL for document-level membership assignments.
pub use membership_sql::select_membership_assignments_for_document_sql;
/// Event-mention row that cannot collapse into an instance identity.
pub use mention_sql::EventMentionRecord;
/// Render insert SQL for a validated event mention.
pub use mention_sql::insert_event_mention_sql;
/// Embedded and ad-hoc migration catalogs.
pub use migration::MigrationCatalog;
/// Validate migration SQL against TEPP contracts.
pub use migration::validate_migration_catalog;
/// Append-only corpus split manifest row.
pub use model_run_sql::CorpusSplitManifestRecord;
/// Append-only model artifact row.
pub use model_run_sql::ModelArtifactRecord;
/// Append-only model run row.
pub use model_run_sql::ModelRunRecord;
/// Render insert SQL for a corpus split manifest.
pub use model_run_sql::insert_corpus_split_manifest_sql;
/// Render insert SQL for a model artifact.
pub use model_run_sql::insert_model_artifact_sql;
/// Render insert SQL for a model run.
pub use model_run_sql::insert_model_run_sql;
/// Render selection SQL for artifacts of one model run.
pub use model_run_sql::select_model_artifacts_by_run_sql;
/// Render selection SQL for a model run by primary key.
pub use model_run_sql::select_model_run_by_id_sql;
/// Multi-word `snake_case` database object naming.
pub use naming::is_multi_word_snake_case;
/// Typed event-relation row bound to the ERD transition vocabulary.
pub use relation_sql::EventRelationRecord;
/// Render insert SQL for a validated event relation.
pub use relation_sql::insert_event_relation_sql;
/// Opaque usable-state token after restore integrity passes.
pub use restore_integrity::RestoreUsableState;
/// Restored snapshot values that must be revalidated before use.
pub use restore_integrity::RestoredAnalyticalSnapshot;
/// Physical tables a backup/restore pair must cover.
pub use restore_integrity::backup_scope_tables;
/// Mark restored analytical state usable only after integrity revalidation.
pub use restore_integrity::mark_restored_state_usable;
/// SQL probes that fail closed on unusable restored rows.
pub use restore_integrity::restore_integrity_probe_sqls;
/// Auditable deletion request row.
pub use retention_sql::DeletionRequestRecord;
/// Append-only evidence tombstone row.
pub use retention_sql::EvidenceTombstoneRecord;
/// Legal or contractual hold row.
pub use retention_sql::LegalHoldRecord;
/// Tenant-scoped retention policy row.
pub use retention_sql::RetentionPolicyRecord;
/// Map a lifecycle SQL failure message onto a typed persistence error.
pub use retention_sql::classify_lifecycle_sql_failure;
/// Render insert SQL for a completed deletion after hold evaluation.
pub use retention_sql::insert_completed_deletion_request_sql;
/// Render insert SQL for a validated deletion request.
pub use retention_sql::insert_deletion_request_sql;
/// Render insert SQL for a validated evidence tombstone.
pub use retention_sql::insert_evidence_tombstone_sql;
/// Render insert SQL for a validated legal hold.
pub use retention_sql::insert_legal_hold_sql;
/// Render insert SQL for a validated retention policy.
pub use retention_sql::insert_retention_policy_sql;
/// Render SQL that releases one active legal hold.
pub use retention_sql::release_legal_hold_sql;
/// Render active-analysis selection that excludes revoked or tombstoned documents.
pub use retention_sql::select_active_analysis_document_sql;
/// Render supersede SQL for a successive retention policy.
pub use retention_sql::supersede_retention_policy_sql;
/// Exact-span text segment row.
pub use segment_sql::TextSegmentRecord;
/// Render insert SQL for a validated text segment.
pub use segment_sql::insert_text_segment_sql;
/// Render selection SQL for a text segment by primary key.
pub use segment_sql::select_text_segment_by_id_sql;
/// Render cutoff-eligible text-segment selection for one document.
pub use segment_sql::select_text_segments_for_document_as_of_sql;
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
/// Non-superuser application role name for RLS-bound connections.
pub use tenant_session::APP_RUNTIME_ROLE;
/// Session GUC name that carries the active tenant UUID for RLS.
pub use tenant_session::TENANT_SESSION_GUC;
/// Render `SET ROLE` SQL for the application runtime role.
pub use tenant_session::assume_app_runtime_role_sql;
/// Render SQL that clears the session tenant GUC.
pub use tenant_session::clear_session_tenant_sql;
/// Render `RESET ROLE` SQL after application work.
pub use tenant_session::reset_app_runtime_role_sql;
/// Render SQL that binds the session tenant GUC.
pub use tenant_session::set_session_tenant_sql;
