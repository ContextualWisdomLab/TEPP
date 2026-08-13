//! Live document repository over a SQL transport.

use crate::document_sql::{
    append_audit_sql, as_known_at_sql, as_valid_at_sql, insert_document_sql, revise_document_sqls,
};
use crate::document_store::{AuditEvent, DocumentRecord};
use crate::manifest_sql::{
    ReproducibilityManifestRecord, insert_reproducibility_manifest_sql,
    select_reproducibility_manifest_by_digests_sql, select_reproducibility_manifest_by_id_sql,
};
use crate::migration::{MigrationCatalog, validate_migration_catalog};
use crate::model_run_sql::{
    CorpusSplitManifestRecord, ModelArtifactRecord, ModelRunRecord,
    insert_corpus_split_manifest_sql, insert_model_artifact_sql, insert_model_run_sql,
    select_model_artifacts_by_run_sql, select_model_run_by_id_sql,
};
use crate::sql_session::{SqlSession, apply_sql_batch};
use crate::{MigrationContractError, PersistenceError};
use temporal_core::{EventTime, SystemTime};
use uuid::Uuid;

/// Fail-closed live document/audit repository backed by [`SqlSession`].
///
/// This is the production-facing adapter surface for `SQLx`/`PostgreSQL`. The
/// in-memory [`crate::DocumentStore`] remains the CPU-local contract reference.
#[derive(Debug)]
pub struct LiveDocumentRepository<S> {
    session: S,
}

impl<S: SqlSession> LiveDocumentRepository<S> {
    /// Wrap an existing SQL session.
    #[must_use]
    pub const fn new(session: S) -> Self {
        Self { session }
    }

    /// Borrow the underlying session.
    #[must_use]
    pub const fn session(&self) -> &S {
        &self.session
    }

    /// Mutably borrow the underlying session.
    #[must_use]
    pub const fn session_mut(&mut self) -> &mut S {
        &mut self.session
    }

    /// Consume the repository and return the session.
    #[must_use]
    pub fn into_session(self) -> S {
        self.session
    }

    /// Validate and apply a migration catalog through the live session.
    ///
    /// # Errors
    ///
    /// Returns migration contract or SQL transport failures.
    pub fn apply_migrations(
        &mut self,
        catalog: &MigrationCatalog,
    ) -> Result<usize, LiveMigrationError> {
        validate_migration_catalog(catalog).map_err(LiveMigrationError::Contract)?;
        apply_sql_batch(&mut self.session, catalog.up_sql()).map_err(LiveMigrationError::Transport)
    }

    /// Insert the first system-time version of a document identity.
    ///
    /// # Errors
    ///
    /// Returns digest or transport failures.
    pub fn insert(&mut self, record: &DocumentRecord) -> Result<(), PersistenceError> {
        let sql = insert_document_sql(record)?;
        self.session.execute(&sql)
    }

    /// Close the open system-time row and insert a revision.
    ///
    /// # Errors
    ///
    /// Returns digest or transport failures.
    pub fn revise(&mut self, record: &DocumentRecord) -> Result<(), PersistenceError> {
        let [close, insert] = revise_document_sqls(record)?;
        self.session.execute(&close)?;
        self.session.execute(&insert)
    }

    /// Issue as-known-at SQL for a document identity.
    ///
    /// Live row materialization remains transport-specific; this method verifies
    /// the statement can be submitted fail-closed.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn submit_as_known_at(
        &mut self,
        document_record_id: Uuid,
        known_at: &SystemTime,
    ) -> Result<(), PersistenceError> {
        let sql = as_known_at_sql(document_record_id, &known_at.to_rfc3339());
        self.session.execute(&sql)
    }

    /// Issue as-valid-at SQL under a system-time as-of.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn submit_as_valid_at(
        &mut self,
        document_record_id: Uuid,
        valid_at: &EventTime,
        known_at: &SystemTime,
    ) -> Result<(), PersistenceError> {
        let sql = as_valid_at_sql(
            document_record_id,
            &valid_at.to_rfc3339(),
            &known_at.to_rfc3339(),
        );
        self.session.execute(&sql)
    }

    /// Append an immutable audit event through SQL.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn append_audit(&mut self, event: &AuditEvent) -> Result<(), PersistenceError> {
        let sql = append_audit_sql(event);
        self.session.execute(&sql)
    }

    /// Insert an append-only reproducibility manifest under the active tenant.
    ///
    /// # Errors
    ///
    /// Returns digest validation or transport failures.
    pub fn insert_reproducibility_manifest(
        &mut self,
        record: &ReproducibilityManifestRecord,
    ) -> Result<(), PersistenceError> {
        let sql = insert_reproducibility_manifest_sql(record)?;
        self.session.execute(&sql)
    }

    /// Look up a reproducibility manifest by the unique digest triple.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn submit_reproducibility_manifest_by_digests(
        &mut self,
        evidence_digest: &str,
        code_commit_sha: &str,
        dependency_lock_digest: &str,
    ) -> Result<(), PersistenceError> {
        let sql = select_reproducibility_manifest_by_digests_sql(
            evidence_digest,
            code_commit_sha,
            dependency_lock_digest,
        );
        self.session.execute(&sql)
    }

    /// Look up a reproducibility manifest by primary key.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn submit_reproducibility_manifest_by_id(
        &mut self,
        reproducibility_manifest_id: Uuid,
    ) -> Result<(), PersistenceError> {
        let sql = select_reproducibility_manifest_by_id_sql(reproducibility_manifest_id);
        self.session.execute(&sql)
    }

    /// Insert an append-only corpus split manifest under the active tenant.
    ///
    /// # Errors
    ///
    /// Returns digest validation or transport failures.
    pub fn insert_corpus_split_manifest(
        &mut self,
        record: &CorpusSplitManifestRecord,
    ) -> Result<(), PersistenceError> {
        let sql = insert_corpus_split_manifest_sql(record)?;
        self.session.execute(&sql)
    }

    /// Insert an append-only model run under the active tenant.
    ///
    /// # Errors
    ///
    /// Returns digest/label validation or transport failures.
    pub fn insert_model_run(&mut self, record: &ModelRunRecord) -> Result<(), PersistenceError> {
        let sql = insert_model_run_sql(record)?;
        self.session.execute(&sql)
    }

    /// Insert an append-only model artifact under the active tenant.
    ///
    /// # Errors
    ///
    /// Returns digest/label validation or transport failures.
    pub fn insert_model_artifact(
        &mut self,
        record: &ModelArtifactRecord,
    ) -> Result<(), PersistenceError> {
        let sql = insert_model_artifact_sql(record)?;
        self.session.execute(&sql)
    }

    /// Look up a model run by primary key.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn submit_model_run_by_id(&mut self, model_run_id: Uuid) -> Result<(), PersistenceError> {
        let sql = select_model_run_by_id_sql(model_run_id);
        self.session.execute(&sql)
    }

    /// Look up model artifacts for a run identity.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn submit_model_artifacts_by_run(
        &mut self,
        model_run_id: Uuid,
    ) -> Result<(), PersistenceError> {
        let sql = select_model_artifacts_by_run_sql(model_run_id);
        self.session.execute(&sql)
    }
}

/// Migration application failures distinguishing contract vs transport errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiveMigrationError {
    /// Embedded/ad-hoc SQL failed TEPP naming/temporal contracts.
    Contract(MigrationContractError),
    /// The live transport rejected a validated statement.
    Transport(PersistenceError),
}

impl std::fmt::Display for LiveMigrationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "migration contract: {error}"),
            Self::Transport(error) => write!(formatter, "migration transport: {error}"),
        }
    }
}

impl std::error::Error for LiveMigrationError {}

#[cfg(test)]
mod tests {
    use super::{LiveDocumentRepository, LiveMigrationError};
    use crate::document_store::{AuditEvent, DocumentRecord};
    use crate::manifest_sql::ReproducibilityManifestRecord;
    use crate::migration::MigrationCatalog;
    use crate::model_run_sql::{CorpusSplitManifestRecord, ModelArtifactRecord, ModelRunRecord};
    use crate::sql_session::RecordingSqlSession;
    use crate::{MigrationContractError, PersistenceError};
    use temporal_core::{AvailableTime, EventTime, SystemTime};

    fn sample_record() -> DocumentRecord {
        DocumentRecord {
            document_record_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            content_digest: "cd".repeat(32),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
            valid_from: EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("v"),
            valid_to: None,
            system_from: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            system_to: None,
            revision_number: 1,
        }
    }

    fn exercise_model_run_chain(
        repo: &mut LiveDocumentRepository<RecordingSqlSession>,
        manifest: &ReproducibilityManifestRecord,
    ) {
        let available = AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a");
        let system = SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s");
        let split = CorpusSplitManifestRecord {
            corpus_split_manifest_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            split_manifest_digest: "11".repeat(32),
            knowledge_cutoff: available,
            system_time: system,
            available_time: available,
        };
        repo.insert_corpus_split_manifest(&split)
            .expect("split insert");
        let run = ModelRunRecord {
            model_run_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            reproducibility_manifest_id: manifest.reproducibility_manifest_id,
            corpus_split_manifest_id: Some(split.corpus_split_manifest_id),
            configuration_digest: "22".repeat(32),
            random_seed_manifest_digest: "33".repeat(32),
            engine_version_label: "tepp-estimator/0.1.0".into(),
            compute_backend_code: "cpu_f64".into(),
            knowledge_cutoff: available,
            system_time: system,
            available_time: available,
        };
        repo.insert_model_run(&run).expect("run insert");
        let artifact = ModelArtifactRecord {
            model_artifact_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            model_run_id: run.model_run_id,
            artifact_type_code: "checkpoint".into(),
            artifact_content_digest: "44".repeat(32),
            protected_object_ref: None,
            system_time: system,
            available_time: available,
        };
        repo.insert_model_artifact(&artifact)
            .expect("artifact insert");
        repo.submit_model_run_by_id(run.model_run_id)
            .expect("run by id");
        repo.submit_model_artifacts_by_run(run.model_run_id)
            .expect("artifacts by run");
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO model_run"))
        );
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO model_artifact"))
        );
    }

    #[test]
    fn live_repository_applies_migrations_and_document_sql() {
        let mut repo = LiveDocumentRepository::new(RecordingSqlSession::new());
        let catalog = MigrationCatalog::from_embedded().expect("embedded");
        let applied = repo.apply_migrations(&catalog).expect("migrate");
        assert!(applied >= 1);
        assert!(!repo.session().executed().is_empty());

        repo.insert(&sample_record()).expect("insert");
        let mut revised = sample_record();
        revised.revision_number = 2;
        revised.system_from =
            SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("later system");
        repo.revise(&revised).expect("revise");
        repo.submit_as_known_at(
            uuid::Uuid::nil(),
            &SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("k"),
        )
        .expect("known");
        repo.submit_as_valid_at(
            uuid::Uuid::nil(),
            &EventTime::parse_rfc3339("2026-01-15T00:00:00Z").expect("v"),
            &SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("k"),
        )
        .expect("valid");
        let manifest = ReproducibilityManifestRecord {
            reproducibility_manifest_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            knowledge_cutoff: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("k"),
            evidence_digest: "ab".repeat(32),
            code_commit_sha: "c".repeat(40),
            dependency_lock_digest: "de".repeat(32),
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        };
        repo.insert_reproducibility_manifest(&manifest)
            .expect("manifest insert");
        repo.submit_reproducibility_manifest_by_digests(
            &manifest.evidence_digest,
            &manifest.code_commit_sha,
            &manifest.dependency_lock_digest,
        )
        .expect("manifest by digests");
        repo.submit_reproducibility_manifest_by_id(manifest.reproducibility_manifest_id)
            .expect("manifest by id");
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO reproducibility_manifest"))
        );
        exercise_model_run_chain(&mut repo, &manifest);

        let audit = AuditEvent {
            audit_event_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            action_code: "insert".into(),
            subject_record_id: uuid::Uuid::nil(),
            recorded_system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
        };
        repo.append_audit(&audit).expect("audit");

        let session = repo.into_session();
        assert!(
            session
                .executed()
                .iter()
                .any(|sql| sql.contains("document_record"))
        );
        assert!(
            session
                .executed()
                .iter()
                .any(|sql| sql.contains("audit_event"))
        );
    }

    #[test]
    fn migration_contract_and_transport_failures_are_distinguished() {
        let mut repo = LiveDocumentRepository::new(RecordingSqlSession::new());
        let bad = MigrationCatalog::from_sql("CREATE TABLE x (id int);", "DROP TABLE x;");
        assert_eq!(
            repo.apply_migrations(&bad),
            Err(LiveMigrationError::Contract(
                MigrationContractError::SingleWordObjectName
            ))
        );

        let mut failing = LiveDocumentRepository::new(RecordingSqlSession::failing_on("document"));
        assert_eq!(
            failing.insert(&sample_record()),
            Err(PersistenceError::SqlExecutionFailed)
        );
        assert_eq!(
            failing.revise(&sample_record()),
            Err(PersistenceError::SqlExecutionFailed)
        );

        let mut bad_digest = sample_record();
        bad_digest.content_digest = "zz".into();
        assert_eq!(
            LiveDocumentRepository::new(RecordingSqlSession::new()).insert(&bad_digest),
            Err(PersistenceError::InvalidContentDigest)
        );

        assert!(
            LiveMigrationError::Contract(MigrationContractError::EmptyMigrationSql)
                .to_string()
                .contains("contract")
        );
        assert!(
            LiveMigrationError::Transport(PersistenceError::SqlExecutionFailed)
                .to_string()
                .contains("transport")
        );

        let mut repo = LiveDocumentRepository::new(RecordingSqlSession::new());
        let _ = repo.session_mut();
        assert!(repo.session().executed().is_empty());
    }
}
