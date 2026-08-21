//! Live document repository over a SQL transport.

use crate::artifact_sql::{
    SourceArtifactRecord, assert_source_artifact_matches_sql, insert_source_artifact_sql,
    select_source_artifact_by_id_sql,
};
use crate::document_sql::{
    append_audit_sql, as_known_at_sql, as_valid_at_sql, insert_document_sql,
    revise_document_atomic_sql,
};
use crate::document_store::{AuditEvent, DocumentRecord};
use crate::entity_sql::{EntityRecord, insert_entity_record_sql, select_entity_record_by_id_sql};
use crate::instance_sql::{
    EventInstanceRecord, insert_event_instance_sql, select_event_instance_as_known_at_sql,
};
use crate::manifest_sql::{
    ReproducibilityManifestRecord, insert_reproducibility_manifest_sql,
    select_reproducibility_manifest_by_digests_sql, select_reproducibility_manifest_by_id_sql,
};
use crate::membership_sql::{
    MembershipAssignmentRecord, insert_membership_assignment_sql,
    select_membership_assignments_for_document_sql,
};
use crate::mention_sql::{EventMentionRecord, insert_event_mention_sql};
use crate::migration::{MigrationCatalog, validate_migration_catalog};
use crate::model_run_sql::{
    CorpusSplitManifestRecord, ModelArtifactRecord, ModelRunRecord,
    insert_corpus_split_manifest_sql, insert_model_artifact_sql, insert_model_run_sql,
    select_model_artifacts_by_run_sql, select_model_run_by_id_sql,
};
use crate::project_sql::{
    ProjectRecord, insert_project_record_sql, select_project_record_by_id_sql,
};
use crate::relation_sql::{EventRelationRecord, insert_event_relation_sql};
use crate::restore_integrity::restore_integrity_probe_sqls;
use crate::retention_sql::{
    DeletionRequestRecord, EvidenceTombstoneRecord, LegalHoldRecord, RetentionPolicyRecord,
    insert_completed_deletion_request_sql, insert_deletion_request_sql,
    insert_evidence_tombstone_sql, insert_legal_hold_sql, insert_retention_policy_sql,
    select_active_analysis_document_sql,
};
use crate::sql_session::{SqlSession, apply_sql_batch};
use crate::tenant_session::set_session_tenant_sql;
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

    /// Revalidate restored physical rows before analytical state is usable.
    ///
    /// # Errors
    ///
    /// Returns transport failures, including a mapped restore-integrity raise.
    pub fn assert_restore_integrity(&mut self) -> Result<(), PersistenceError> {
        for sql in restore_integrity_probe_sqls() {
            self.session.execute(&sql)?;
        }
        Ok(())
    }

    /// Bind the session tenant GUC before tenant-scoped persistence operations.
    fn bind_session_tenant(&mut self, tenant_record_id: Uuid) -> Result<(), PersistenceError> {
        self.session
            .execute(&set_session_tenant_sql(tenant_record_id))
    }

    /// Insert the first system-time version of a document identity.
    ///
    /// The session tenant GUC is bound first so
    /// `reject_tombstoned_evidence_restore` can fail closed instead of aborting
    /// a legitimate first insert.
    ///
    /// # Errors
    ///
    /// Returns digest or transport failures.
    pub fn insert(&mut self, record: &DocumentRecord) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_document_sql(record)?;
        self.session.execute(&sql)
    }

    /// Close the open system-time row and insert a revision atomically.
    ///
    /// # Errors
    ///
    /// Returns digest, concurrent-write, or transport failures.
    pub fn revise(&mut self, record: &DocumentRecord) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = revise_document_atomic_sql(record)?;
        self.session.execute(&sql)
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
    /// Returns action-code validation or transport failures.
    pub fn append_audit(&mut self, event: &AuditEvent) -> Result<(), PersistenceError> {
        self.bind_session_tenant(event.tenant_record_id)?;
        let sql = append_audit_sql(event)?;
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
        self.bind_session_tenant(record.tenant_record_id)?;
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
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_corpus_split_manifest_sql(record)?;
        self.session.execute(&sql)
    }

    /// Insert an append-only model run under the active tenant.
    ///
    /// # Errors
    ///
    /// Returns digest/label validation or transport failures.
    pub fn insert_model_run(&mut self, record: &ModelRunRecord) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
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
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_model_artifact_sql(record)?;
        self.session.execute(&sql)
    }

    /// Insert a typed entity membership target under the active tenant.
    ///
    /// # Errors
    ///
    /// Returns label validation or transport failures.
    pub fn insert_entity_record(&mut self, record: &EntityRecord) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_entity_record_sql(record)?;
        self.session.execute(&sql)
    }

    /// Look up an entity membership target by primary key.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn submit_entity_record_by_id(
        &mut self,
        entity_record_id: Uuid,
    ) -> Result<(), PersistenceError> {
        let sql = select_entity_record_by_id_sql(entity_record_id);
        self.session.execute(&sql)
    }

    /// Insert a typed project membership target under the active tenant.
    ///
    /// # Errors
    ///
    /// Returns label validation or transport failures.
    pub fn insert_project_record(
        &mut self,
        record: &ProjectRecord,
    ) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_project_record_sql(record)?;
        self.session.execute(&sql)
    }

    /// Look up a project membership target by primary key.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn submit_project_record_by_id(
        &mut self,
        project_record_id: Uuid,
    ) -> Result<(), PersistenceError> {
        let sql = select_project_record_by_id_sql(project_record_id);
        self.session.execute(&sql)
    }

    /// Insert a typed membership assignment under the active tenant.
    ///
    /// # Errors
    ///
    /// Returns exactly-one/weight validation or transport failures.
    pub fn insert_membership_assignment(
        &mut self,
        record: &MembershipAssignmentRecord,
    ) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_membership_assignment_sql(record)?;
        self.session.execute(&sql)
    }

    /// Look up document-level membership assignments by document identity.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn submit_membership_assignments_for_document(
        &mut self,
        document_record_id: Uuid,
    ) -> Result<(), PersistenceError> {
        let sql = select_membership_assignments_for_document_sql(document_record_id);
        self.session.execute(&sql)
    }

    /// Insert a typed event relation under the active tenant.
    ///
    /// # Errors
    ///
    /// Returns vocabulary/flag validation or transport failures.
    pub fn insert_event_relation(
        &mut self,
        record: &EventRelationRecord,
    ) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_event_relation_sql(record)?;
        self.session.execute(&sql)
    }

    /// Insert an event mention that is not an instance identity.
    ///
    /// # Errors
    ///
    /// Returns mention/instance or confidence validation failures, or transport
    /// failures.
    pub fn insert_event_mention(
        &mut self,
        record: &EventMentionRecord,
    ) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_event_mention_sql(record)?;
        self.session.execute(&sql)
    }

    /// Insert a bitemporal event-instance version.
    ///
    /// # Errors
    ///
    /// Returns inverted-window/label validation or transport failures.
    pub fn insert_event_instance(
        &mut self,
        record: &EventInstanceRecord,
    ) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_event_instance_sql(record)?;
        self.session.execute(&sql)
    }

    /// Look up the event-instance version visible as of a system-time instant.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn submit_event_instance_as_known_at(
        &mut self,
        event_instance_id: Uuid,
        known_at: &SystemTime,
    ) -> Result<(), PersistenceError> {
        let sql = select_event_instance_as_known_at_sql(event_instance_id, &known_at.to_rfc3339());
        self.session.execute(&sql)
    }

    /// Insert an append-only source artifact under the active tenant.
    ///
    /// A retry of the same immutable identity is a no-op. A same-id payload
    /// change fails closed after `ON CONFLICT DO NOTHING`.
    ///
    /// # Errors
    ///
    /// Returns digest/size/label validation, identity-conflict, or transport
    /// failures.
    pub fn insert_source_artifact(
        &mut self,
        record: &SourceArtifactRecord,
    ) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_source_artifact_sql(record)?;
        self.session.execute(&sql)?;
        let assertion = assert_source_artifact_matches_sql(record)?;
        self.session.execute(&assertion)
    }

    /// Look up a source artifact by primary key.
    ///
    /// # Errors
    ///
    /// Returns transport failures from the underlying session.
    pub fn submit_source_artifact_by_id(
        &mut self,
        source_artifact_id: Uuid,
    ) -> Result<(), PersistenceError> {
        let sql = select_source_artifact_by_id_sql(source_artifact_id);
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

    /// Insert a tenant-scoped retention policy.
    ///
    /// # Errors
    ///
    /// Returns lifecycle validation or transport failures.
    pub fn insert_retention_policy(
        &mut self,
        record: &RetentionPolicyRecord,
    ) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_retention_policy_sql(record)?;
        self.session.execute(&sql)
    }

    /// Insert a legal or contractual hold.
    ///
    /// # Errors
    ///
    /// Returns lifecycle validation or transport failures.
    pub fn insert_legal_hold(&mut self, record: &LegalHoldRecord) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_legal_hold_sql(record)?;
        self.session.execute(&sql)
    }

    /// Insert a deletion request that is not evaluated against holds.
    ///
    /// # Errors
    ///
    /// Returns lifecycle validation, a cited-policy mismatch, or transport
    /// failures.
    pub fn insert_deletion_request(
        &mut self,
        record: &DeletionRequestRecord,
        policy: &RetentionPolicyRecord,
    ) -> Result<(), PersistenceError> {
        record.bind_cited_policy(policy)?;
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_deletion_request_sql(record)?;
        self.session.execute(&sql)
    }

    /// Complete a deletion only when no supplied hold covers the target.
    ///
    /// # Errors
    ///
    /// Returns lifecycle validation, a cited-policy mismatch,
    /// [`PersistenceError::LegalHoldBlocksDeletion`], or transport failures.
    pub fn insert_completed_deletion_request(
        &mut self,
        record: &DeletionRequestRecord,
        policy: &RetentionPolicyRecord,
        holds: &[LegalHoldRecord],
    ) -> Result<(), PersistenceError> {
        record.bind_cited_policy(policy)?;
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_completed_deletion_request_sql(record, holds)?;
        self.session.execute(&sql)
    }

    /// Insert an evidence tombstone without raw source text.
    ///
    /// # Errors
    ///
    /// Returns lifecycle validation, ungoverned-restore, or transport failures.
    pub fn insert_evidence_tombstone(
        &mut self,
        record: &EvidenceTombstoneRecord,
    ) -> Result<(), PersistenceError> {
        self.bind_session_tenant(record.tenant_record_id)?;
        let sql = insert_evidence_tombstone_sql(record)?;
        self.session.execute(&sql)
    }

    /// Look up a document only when it remains eligible for active analysis.
    ///
    /// # Errors
    ///
    /// Returns transport failures.
    pub fn submit_active_analysis_document(
        &mut self,
        document_record_id: Uuid,
    ) -> Result<(), PersistenceError> {
        let sql = select_active_analysis_document_sql(document_record_id);
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
    use crate::artifact_sql::SourceArtifactRecord;
    use crate::document_store::{AuditEvent, DocumentRecord};
    use crate::instance_sql::EventInstanceRecord;
    use crate::manifest_sql::ReproducibilityManifestRecord;
    use crate::mention_sql::EventMentionRecord;
    use crate::migration::MigrationCatalog;
    use crate::model_run_sql::{CorpusSplitManifestRecord, ModelArtifactRecord, ModelRunRecord};
    use crate::relation_sql::EventRelationRecord;
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

    fn sample_membership() -> crate::MembershipAssignmentRecord {
        crate::MembershipAssignmentRecord {
            membership_assignment_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            document_record_id: Some(uuid::Uuid::nil()),
            text_segment_id: None,
            target_entity_id: Some(uuid::Uuid::nil()),
            target_project_id: None,
            membership_type_code: "author".into(),
            membership_weight: 1.0,
            valid_from: EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("v"),
            valid_to: None,
            valid_time_precision_code: "second".into(),
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        }
    }

    fn exercise_entity_project_targets(repo: &mut LiveDocumentRepository<RecordingSqlSession>) {
        let (available, system) = (
            AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
            SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
        );
        let entity = crate::EntityRecord {
            entity_record_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            entity_type_code: "author".into(),
            system_time: system,
            available_time: available,
        };
        repo.insert_entity_record(&entity).expect("entity insert");
        repo.submit_entity_record_by_id(entity.entity_record_id)
            .expect("entity lookup");
        let mut bad_entity = entity;
        bad_entity.entity_type_code.clear();
        assert_eq!(
            repo.insert_entity_record(&bad_entity),
            Err(PersistenceError::InvalidEntityRecord)
        );

        let project = crate::ProjectRecord {
            project_record_id: uuid::Uuid::from_u128(2),
            tenant_record_id: uuid::Uuid::nil(),
            project_status_code: "active".into(),
            system_time: system,
            available_time: available,
        };
        repo.insert_project_record(&project)
            .expect("project insert");
        repo.submit_project_record_by_id(project.project_record_id)
            .expect("project lookup");
        let mut bad_project = project;
        bad_project.project_status_code = "active';x".into();
        assert_eq!(
            repo.insert_project_record(&bad_project),
            Err(PersistenceError::InvalidProjectRecord)
        );
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO entity_record"))
        );
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO project_record"))
        );
    }

    fn exercise_membership_assignment(repo: &mut LiveDocumentRepository<RecordingSqlSession>) {
        let membership = sample_membership();
        repo.insert_membership_assignment(&membership)
            .expect("membership insert");
        repo.submit_membership_assignments_for_document(uuid::Uuid::nil())
            .expect("membership lookup");
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO membership_assignment"))
        );
        let mut invalid = membership;
        invalid.target_entity_id = None;
        assert_eq!(
            repo.insert_membership_assignment(&invalid),
            Err(PersistenceError::InvalidMembershipAssignment)
        );
    }

    fn exercise_event_relation(repo: &mut LiveDocumentRepository<RecordingSqlSession>) {
        let (available, system) = (
            AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
            SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
        );
        let relation = EventRelationRecord {
            event_relation_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            source_event_id: uuid::Uuid::from_u128(1),
            target_event_id: uuid::Uuid::from_u128(2),
            relation_type_code: "causes".into(),
            transition_edge: true,
            system_time: system,
            available_time: available,
        };
        repo.insert_event_relation(&relation)
            .expect("relation insert");
        let mut bad = relation;
        bad.transition_edge = false;
        assert_eq!(
            repo.insert_event_relation(&bad),
            Err(PersistenceError::InvalidEventRelation)
        );
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO event_relation"))
        );
    }

    fn exercise_event_mention(repo: &mut LiveDocumentRepository<RecordingSqlSession>) {
        let mention = EventMentionRecord {
            event_mention_id: uuid::Uuid::from_u128(2),
            event_instance_id: uuid::Uuid::from_u128(1),
            tenant_record_id: uuid::Uuid::nil(),
            confidence_score: 0.75,
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        };
        repo.insert_event_mention(&mention).expect("mention insert");
        let mut collapsed = mention.clone();
        collapsed.event_mention_id = collapsed.event_instance_id;
        assert_eq!(
            repo.insert_event_mention(&collapsed),
            Err(PersistenceError::InvalidEventMention)
        );
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO event_mention"))
        );
    }

    fn exercise_event_instance(repo: &mut LiveDocumentRepository<RecordingSqlSession>) {
        let instance = EventInstanceRecord {
            event_instance_id: uuid::Uuid::from_u128(1),
            tenant_record_id: uuid::Uuid::nil(),
            event_type_code: "occurrence".into(),
            valid_from: EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("v"),
            valid_to: None,
            system_from: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            system_to: None,
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
            lifecycle_status_code: "asserted".into(),
        };
        repo.insert_event_instance(&instance)
            .expect("instance insert");
        repo.submit_event_instance_as_known_at(instance.event_instance_id, &instance.system_from)
            .expect("instance known-at");
        let mut inverted = instance.clone();
        inverted.valid_to =
            Some(EventTime::parse_rfc3339("2025-12-31T00:00:00Z").expect("earlier"));
        assert_eq!(
            repo.insert_event_instance(&inverted),
            Err(PersistenceError::InvalidEventInstance)
        );
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO event_instance"))
        );
    }

    fn exercise_source_artifact(repo: &mut LiveDocumentRepository<RecordingSqlSession>) {
        let artifact = SourceArtifactRecord {
            source_artifact_id: uuid::Uuid::from_u128(1),
            tenant_record_id: uuid::Uuid::nil(),
            content_sha256: "ab".repeat(32),
            source_size_bytes: 4,
            media_type_code: "text/plain".into(),
            protected_object_ref: None,
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        };
        repo.insert_source_artifact(&artifact)
            .expect("artifact insert");
        repo.insert_source_artifact(&artifact)
            .expect("identical retry");
        repo.submit_source_artifact_by_id(artifact.source_artifact_id)
            .expect("artifact by id");
        let mut invalid = artifact.clone();
        invalid.source_size_bytes = -1;
        assert_eq!(
            repo.insert_source_artifact(&invalid),
            Err(PersistenceError::InvalidSourceArtifact)
        );
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("ON CONFLICT (source_artifact_id) DO NOTHING"))
        );
    }

    fn sample_policy() -> crate::RetentionPolicyRecord {
        crate::RetentionPolicyRecord {
            retention_policy_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            data_class_code: "raw_source".into(),
            processing_purpose_code: "psychometric_analysis".into(),
            retention_period_days: 365,
            policy_status_code: "active".into(),
            authority_citation: "adr-0009".into(),
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            system_to: None,
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        }
    }

    fn sample_hold() -> crate::LegalHoldRecord {
        crate::LegalHoldRecord {
            legal_hold_id: uuid::Uuid::from_u128(2),
            tenant_record_id: uuid::Uuid::nil(),
            hold_scope_code: "document".into(),
            held_document_id: Some(uuid::Uuid::nil()),
            hold_authority_code: "contract".into(),
            hold_status_code: "active".into(),
            authority_citation: "hold-authority".into(),
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            system_to: None,
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        }
    }

    fn sample_deletion() -> crate::DeletionRequestRecord {
        crate::DeletionRequestRecord {
            deletion_request_id: uuid::Uuid::from_u128(4),
            tenant_record_id: uuid::Uuid::nil(),
            retention_policy_id: uuid::Uuid::nil(),
            target_document_id: uuid::Uuid::nil(),
            target_data_class_code: "raw_source".into(),
            processing_purpose_code: "psychometric_analysis".into(),
            deletion_kind_code: "identity_tombstone".into(),
            request_status_code: "blocked_by_hold".into(),
            legal_hold_id: Some(uuid::Uuid::from_u128(2)),
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        }
    }

    fn sample_tombstone() -> crate::EvidenceTombstoneRecord {
        crate::EvidenceTombstoneRecord {
            evidence_tombstone_id: uuid::Uuid::from_u128(5),
            tenant_record_id: uuid::Uuid::nil(),
            tombstoned_document_id: uuid::Uuid::from_u128(9),
            deletion_request_id: uuid::Uuid::from_u128(4),
            evidence_digest: "ab".repeat(32),
            target_data_class_code: "raw_source".into(),
            deletion_kind_code: "identity_tombstone".into(),
            reproduction_status_code: "unavailable".into(),
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        }
    }

    fn exercise_retention_legal_hold(repo: &mut LiveDocumentRepository<RecordingSqlSession>) {
        repo.insert_retention_policy(&sample_policy())
            .expect("policy insert");
        repo.insert_legal_hold(&sample_hold()).expect("hold insert");
        repo.insert_deletion_request(&sample_deletion(), &sample_policy())
            .expect("blocked request");
        let mut completed = sample_deletion();
        completed.request_status_code = "completed".into();
        completed.legal_hold_id = None;
        assert_eq!(
            repo.insert_completed_deletion_request(&completed, &sample_policy(), &[sample_hold()]),
            Err(PersistenceError::LegalHoldBlocksDeletion)
        );
        repo.insert_completed_deletion_request(&completed, &sample_policy(), &[])
            .expect("unheld completion");
        repo.insert_evidence_tombstone(&sample_tombstone())
            .expect("tombstone");
        repo.submit_active_analysis_document(uuid::Uuid::nil())
            .expect("active analysis");
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO retention_policy"))
        );
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO legal_hold"))
        );
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("INSERT INTO evidence_tombstone"))
        );
        let mut invalid = sample_policy();
        invalid.retention_period_days = 0;
        assert_eq!(
            repo.insert_retention_policy(&invalid),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        let mut bad_hold = sample_hold();
        bad_hold.held_document_id = None;
        assert_eq!(
            repo.insert_legal_hold(&bad_hold),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        let mut bad_request = sample_deletion();
        bad_request.deletion_kind_code = "hard_delete".into();
        assert_eq!(
            repo.insert_deletion_request(&bad_request, &sample_policy()),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        let mut mismatched_purpose = sample_deletion();
        mismatched_purpose.processing_purpose_code = "export_fulfillment".into();
        assert_eq!(
            repo.insert_deletion_request(&mismatched_purpose, &sample_policy()),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        let mut restore = sample_tombstone();
        restore.reproduction_status_code = "unaffected".into();
        assert_eq!(
            repo.insert_evidence_tombstone(&restore),
            Err(PersistenceError::UngovernedEvidenceRestore)
        );
    }

    #[test]
    fn insert_and_revise_bind_tenant_session_before_document_sql() {
        let mut repo = LiveDocumentRepository::new(RecordingSqlSession::new());
        let record = sample_record();
        repo.insert(&record).expect("insert");
        let executed = repo.session().executed();
        assert!(
            executed.len() >= 2,
            "tenant GUC must precede document insert so 0007 restore trigger can run"
        );
        let tenant_bind = &executed[0];
        assert!(
            tenant_bind.contains("tepp.current_tenant_record_id"),
            "insert must set the tenant GUC before SQL reaches document_record"
        );
        assert!(
            tenant_bind.contains(&record.tenant_record_id.to_string()),
            "tenant GUC bind must include the document tenant identity"
        );
        assert!(
            executed[1].contains("INSERT INTO document_record"),
            "document insert follows tenant bind"
        );

        let mut revised = record.clone();
        revised.revision_number = 2;
        revised.system_from =
            SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("later system");
        repo.revise(&revised).expect("revise");
        let executed = repo.session().executed();
        let revise_bind = executed
            .iter()
            .rposition(|sql| sql.contains("tepp.current_tenant_record_id"))
            .expect("revise must bind tenant session");
        assert!(
            executed[revise_bind + 1].contains("DO $tepp$"),
            "atomic revise must run after tenant bind"
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
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("DO $tepp$") && sql.contains("GET DIAGNOSTICS"))
        );
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
        exercise_entity_project_targets(&mut repo);
        exercise_membership_assignment(&mut repo);
        exercise_event_relation(&mut repo);
        exercise_event_mention(&mut repo);
        exercise_event_instance(&mut repo);
        exercise_source_artifact(&mut repo);
        exercise_retention_legal_hold(&mut repo);
        repo.assert_restore_integrity()
            .expect("restore integrity probes render through the session");
        assert!(
            repo.session()
                .executed()
                .iter()
                .any(|sql| sql.contains("restore integrity failed") || sql.contains("tepp_restore"))
        );

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
