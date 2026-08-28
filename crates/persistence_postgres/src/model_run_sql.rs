//! SQL contracts for append-only model runs and model artifacts (ADR 0013).

use crate::PersistenceError;
use event_core::{INTERVAL_CONSISTENCY_ARTIFACT_TYPE, IntervalConsistencyArtifact};
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

/// Append-only corpus split identity bound to a tenant and knowledge cutoff.
///
/// Maps to `corpus_split_manifest` in migration `0003`. Digests are lowercase
/// hex `SHA-256` (exactly 64 `0-9a-f` characters).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusSplitManifestRecord {
    /// Primary key for this split identity.
    pub corpus_split_manifest_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Canonical digest of the relation-aware split payload.
    pub split_manifest_digest: String,
    /// Knowledge cutoff applied when the split was materialised.
    pub knowledge_cutoff: AvailableTime,
    /// System/record time when the split identity was asserted.
    pub system_time: SystemTime,
    /// Availability time of the split evidence.
    pub available_time: AvailableTime,
}

impl CorpusSplitManifestRecord {
    /// Fail-closed digest validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidContentDigest`] for non-canonical digests.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        validate_sha256_hex(&self.split_manifest_digest)
    }
}

/// Append-only model run bound to a reproducibility manifest (and optional split).
///
/// Maps to `model_run` in migration `0003`. Configuration and seed digests are
/// lowercase hex `SHA-256`; `engine_version_label` and `compute_backend_code`
/// are non-empty multi-character labels without control characters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRunRecord {
    /// Primary key for this model run identity.
    pub model_run_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Linked reproducibility manifest identity.
    pub reproducibility_manifest_id: Uuid,
    /// Optional linked corpus split identity.
    pub corpus_split_manifest_id: Option<Uuid>,
    /// Digest of the deterministic configuration envelope.
    pub configuration_digest: String,
    /// Digest of the random-seed manifest bound into the run.
    pub random_seed_manifest_digest: String,
    /// Engine/version label for the estimator implementation.
    pub engine_version_label: String,
    /// Compute backend code (for example `cpu_f64` or `gpu_cuda`).
    pub compute_backend_code: String,
    /// Knowledge cutoff applied for this run.
    pub knowledge_cutoff: AvailableTime,
    /// System/record time when the run was asserted.
    pub system_time: SystemTime,
    /// Availability time of the run evidence.
    pub available_time: AvailableTime,
}

impl ModelRunRecord {
    /// Fail-closed field validation for digests and labels.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidContentDigest`] for invalid digests or
    /// empty labels.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        validate_sha256_hex(&self.configuration_digest)?;
        validate_sha256_hex(&self.random_seed_manifest_digest)?;
        validate_nonempty_label(&self.engine_version_label)?;
        validate_nonempty_label(&self.compute_backend_code)?;
        Ok(())
    }
}

/// Append-only model artifact produced by a model run.
///
/// Maps to `model_artifact` in migration `0003`. Content digests are lowercase
/// hex `SHA-256`; `artifact_type_code` is a non-empty label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelArtifactRecord {
    /// Primary key for this artifact identity.
    pub model_artifact_id: Uuid,
    /// Owning tenant boundary (denormalised for RLS).
    pub tenant_record_id: Uuid,
    /// Owning model run identity.
    pub model_run_id: Uuid,
    /// Artifact kind code (for example `checkpoint` or `posterior_summary`).
    pub artifact_type_code: String,
    /// Content digest of the immutable artifact payload.
    pub artifact_content_digest: String,
    /// Optional protected object reference for external blob storage.
    pub protected_object_ref: Option<String>,
    /// System/record time when the artifact was published.
    pub system_time: SystemTime,
    /// Availability time of the artifact evidence.
    pub available_time: AvailableTime,
}

impl ModelArtifactRecord {
    /// Bind a canonical TDT/CHRONOS artifact to the append-only model-run chain.
    ///
    /// The protected object reference must resolve to the exact canonical JSON
    /// bytes whose digest is stored in this record.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed digest error for an invalid artifact or object reference.
    #[allow(clippy::too_many_arguments)]
    pub fn for_interval_consistency(
        model_artifact_id: Uuid,
        tenant_record_id: Uuid,
        model_run_id: Uuid,
        artifact: &IntervalConsistencyArtifact,
        protected_object_ref: impl Into<String>,
        system_time: SystemTime,
        available_time: AvailableTime,
    ) -> Result<Self, PersistenceError> {
        let artifact_content_digest = artifact
            .sha256()
            .map_err(|_| PersistenceError::InvalidContentDigest)?;
        let record = Self {
            model_artifact_id,
            tenant_record_id,
            model_run_id,
            artifact_type_code: INTERVAL_CONSISTENCY_ARTIFACT_TYPE.to_owned(),
            artifact_content_digest,
            protected_object_ref: Some(protected_object_ref.into()),
            system_time,
            available_time,
        };
        record.validate()?;
        Ok(record)
    }

    /// Fail-closed field validation for digests and labels.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidContentDigest`] for invalid digests or
    /// empty type codes.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        validate_sha256_hex(&self.artifact_content_digest)?;
        validate_nonempty_label(&self.artifact_type_code)?;
        if let Some(object_ref) = &self.protected_object_ref
            && object_ref.is_empty()
        {
            return Err(PersistenceError::InvalidContentDigest);
        }
        Ok(())
    }
}

/// Render insert SQL for a validated corpus split manifest.
///
/// # Errors
///
/// Returns digest validation failures before any SQL is produced.
pub fn insert_corpus_split_manifest_sql(
    record: &CorpusSplitManifestRecord,
) -> Result<String, PersistenceError> {
    record.validate()?;
    Ok(format!(
        "INSERT INTO corpus_split_manifest (\
            corpus_split_manifest_id, tenant_record_id, split_manifest_digest, \
            knowledge_cutoff, system_time, available_time\
        ) VALUES (\
            '{split_id}'::uuid, '{tenant_id}'::uuid, '{digest}', \
            '{cutoff}'::timestamptz, '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        split_id = record.corpus_split_manifest_id,
        tenant_id = record.tenant_record_id,
        digest = record.split_manifest_digest,
        cutoff = record.knowledge_cutoff.to_rfc3339(),
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render insert SQL for a validated model run.
///
/// # Errors
///
/// Returns digest/label validation failures before any SQL is produced.
pub fn insert_model_run_sql(record: &ModelRunRecord) -> Result<String, PersistenceError> {
    record.validate()?;
    let split_sql = match record.corpus_split_manifest_id {
        Some(split_id) => format!("'{split_id}'::uuid"),
        None => "NULL".to_owned(),
    };
    Ok(format!(
        "INSERT INTO model_run (\
            model_run_id, tenant_record_id, reproducibility_manifest_id, \
            corpus_split_manifest_id, configuration_digest, random_seed_manifest_digest, \
            engine_version_label, compute_backend_code, knowledge_cutoff, \
            system_time, available_time\
        ) VALUES (\
            '{run_id}'::uuid, '{tenant_id}'::uuid, '{manifest_id}'::uuid, \
            {split_sql}, '{config}', '{seed}', \
            '{engine}', '{backend}', '{cutoff}'::timestamptz, \
            '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        run_id = record.model_run_id,
        tenant_id = record.tenant_record_id,
        manifest_id = record.reproducibility_manifest_id,
        config = record.configuration_digest,
        seed = record.random_seed_manifest_digest,
        engine = escape_literal(&record.engine_version_label),
        backend = escape_literal(&record.compute_backend_code),
        cutoff = record.knowledge_cutoff.to_rfc3339(),
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render insert SQL for a validated model artifact.
///
/// # Errors
///
/// Returns digest/label validation failures before any SQL is produced.
pub fn insert_model_artifact_sql(record: &ModelArtifactRecord) -> Result<String, PersistenceError> {
    record.validate()?;
    let object_ref_sql = match &record.protected_object_ref {
        Some(value) => format!("'{}'", escape_literal(value)),
        None => "NULL".to_owned(),
    };
    Ok(format!(
        "INSERT INTO model_artifact (\
            model_artifact_id, tenant_record_id, model_run_id, artifact_type_code, \
            artifact_content_digest, protected_object_ref, system_time, available_time\
        ) VALUES (\
            '{artifact_id}'::uuid, '{tenant_id}'::uuid, '{run_id}'::uuid, '{type_code}', \
            '{digest}', {object_ref_sql}, '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        artifact_id = record.model_artifact_id,
        tenant_id = record.tenant_record_id,
        run_id = record.model_run_id,
        type_code = escape_literal(&record.artifact_type_code),
        digest = record.artifact_content_digest,
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render selection of a model run by primary key.
#[must_use]
pub fn select_model_run_by_id_sql(model_run_id: Uuid) -> String {
    format!(
        "SELECT model_run_id, tenant_record_id, reproducibility_manifest_id, \
                corpus_split_manifest_id, configuration_digest, random_seed_manifest_digest, \
                engine_version_label, compute_backend_code, knowledge_cutoff, \
                system_time, available_time \
         FROM model_run \
         WHERE model_run_id = '{model_run_id}'::uuid \
         LIMIT 1"
    )
}

/// Render selection of artifacts for a model run ordered by system time.
#[must_use]
pub fn select_model_artifacts_by_run_sql(model_run_id: Uuid) -> String {
    format!(
        "SELECT model_artifact_id, tenant_record_id, model_run_id, artifact_type_code, \
                artifact_content_digest, protected_object_ref, system_time, available_time \
         FROM model_artifact \
         WHERE model_run_id = '{model_run_id}'::uuid \
         ORDER BY system_time ASC, model_artifact_id ASC"
    )
}

fn is_lowercase_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn validate_sha256_hex(value: &str) -> Result<(), PersistenceError> {
    if value.len() != 64 || !is_lowercase_hex(value) {
        return Err(PersistenceError::InvalidContentDigest);
    }
    Ok(())
}

fn validate_nonempty_label(value: &str) -> Result<(), PersistenceError> {
    if value.is_empty()
        || value.len() > 128
        || value.chars().any(|ch| ch.is_control() || ch == '\'')
    {
        return Err(PersistenceError::InvalidContentDigest);
    }
    Ok(())
}

fn escape_literal(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::{
        CorpusSplitManifestRecord, ModelArtifactRecord, ModelRunRecord,
        insert_corpus_split_manifest_sql, insert_model_artifact_sql, insert_model_run_sql,
        select_model_artifacts_by_run_sql, select_model_run_by_id_sql, validate_nonempty_label,
        validate_sha256_hex,
    };
    use crate::PersistenceError;
    use event_core::IntervalConsistencyNetwork;
    use temporal_core::{AllenRelation, AvailableTime, RelationSet, SystemTime};
    use uuid::Uuid;

    fn sample_times() -> (AvailableTime, SystemTime) {
        (
            AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
            SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
        )
    }

    fn sample_split() -> CorpusSplitManifestRecord {
        let (available, system) = sample_times();
        CorpusSplitManifestRecord {
            corpus_split_manifest_id: Uuid::nil(),
            tenant_record_id: Uuid::nil(),
            split_manifest_digest: "ab".repeat(32),
            knowledge_cutoff: available,
            system_time: system,
            available_time: available,
        }
    }

    fn sample_run() -> ModelRunRecord {
        let (available, system) = sample_times();
        ModelRunRecord {
            model_run_id: Uuid::nil(),
            tenant_record_id: Uuid::nil(),
            reproducibility_manifest_id: Uuid::nil(),
            corpus_split_manifest_id: Some(Uuid::nil()),
            configuration_digest: "cd".repeat(32),
            random_seed_manifest_digest: "ef".repeat(32),
            engine_version_label: "tepp-estimator/0.1.0".into(),
            compute_backend_code: "cpu_f64".into(),
            knowledge_cutoff: available,
            system_time: system,
            available_time: available,
        }
    }

    fn sample_artifact() -> ModelArtifactRecord {
        let (available, system) = sample_times();
        ModelArtifactRecord {
            model_artifact_id: Uuid::nil(),
            tenant_record_id: Uuid::nil(),
            model_run_id: Uuid::nil(),
            artifact_type_code: "checkpoint".into(),
            artifact_content_digest: "12".repeat(32),
            protected_object_ref: Some("s3://bucket/obj".into()),
            system_time: system,
            available_time: available,
        }
    }

    #[test]
    fn model_run_artifact_sql_covers_insert_select_and_fail_closed() {
        let split = insert_corpus_split_manifest_sql(&sample_split()).expect("split");
        assert!(split.contains("INSERT INTO corpus_split_manifest"));
        assert!(split.contains(&"ab".repeat(32)));

        let run = insert_model_run_sql(&sample_run()).expect("run");
        assert!(run.contains("INSERT INTO model_run"));
        assert!(run.contains("cpu_f64"));
        assert!(run.contains("'::uuid"));

        let mut without_split = sample_run();
        without_split.corpus_split_manifest_id = None;
        let run_null = insert_model_run_sql(&without_split).expect("null split");
        assert!(run_null.contains("NULL"));

        let artifact = insert_model_artifact_sql(&sample_artifact()).expect("artifact");
        assert!(artifact.contains("INSERT INTO model_artifact"));
        assert!(artifact.contains("checkpoint"));
        assert!(artifact.contains("s3://bucket/obj"));

        let mut no_ref = sample_artifact();
        no_ref.protected_object_ref = None;
        let artifact_null = insert_model_artifact_sql(&no_ref).expect("null ref");
        assert!(artifact_null.contains("NULL"));

        let by_id = select_model_run_by_id_sql(Uuid::nil());
        assert!(by_id.contains("FROM model_run"));
        let by_run = select_model_artifacts_by_run_sql(Uuid::nil());
        assert!(by_run.contains("FROM model_artifact"));
        assert!(by_run.contains("ORDER BY"));

        let mut bad_split = sample_split();
        bad_split.split_manifest_digest = "short".into();
        assert_eq!(
            insert_corpus_split_manifest_sql(&bad_split),
            Err(PersistenceError::InvalidContentDigest)
        );

        let mut bad_run = sample_run();
        bad_run.configuration_digest = "FF".repeat(32);
        assert_eq!(
            insert_model_run_sql(&bad_run),
            Err(PersistenceError::InvalidContentDigest)
        );
        bad_run = sample_run();
        bad_run.engine_version_label.clear();
        assert_eq!(
            insert_model_run_sql(&bad_run),
            Err(PersistenceError::InvalidContentDigest)
        );

        let mut bad_artifact = sample_artifact();
        bad_artifact.artifact_content_digest = "g".repeat(64);
        assert_eq!(
            insert_model_artifact_sql(&bad_artifact),
            Err(PersistenceError::InvalidContentDigest)
        );
        bad_artifact = sample_artifact();
        bad_artifact.protected_object_ref = Some(String::new());
        assert_eq!(
            insert_model_artifact_sql(&bad_artifact),
            Err(PersistenceError::InvalidContentDigest)
        );

        assert!(validate_sha256_hex(&"09".repeat(32)).is_ok());
        assert!(validate_sha256_hex(&"ab".repeat(32)).is_ok());
        assert!(validate_sha256_hex("zz").is_err());
        assert!(validate_sha256_hex(&"AB".repeat(32)).is_err());
        assert!(validate_nonempty_label("cpu_f64").is_ok());
        assert!(validate_nonempty_label("").is_err());
        assert!(validate_nonempty_label(&"x".repeat(129)).is_err());
        assert!(validate_nonempty_label("bad'label").is_err());
        // Control-character branch of label validation.
        assert!(validate_nonempty_label("cpu\nf64").is_err());
        // Independent fail paths for seed digest / backend / artifact type.
        let mut bad_seed = sample_run();
        bad_seed.random_seed_manifest_digest = "zz".into();
        assert_eq!(
            insert_model_run_sql(&bad_seed),
            Err(PersistenceError::InvalidContentDigest)
        );
        let mut bad_backend = sample_run();
        bad_backend.compute_backend_code.clear();
        assert_eq!(
            insert_model_run_sql(&bad_backend),
            Err(PersistenceError::InvalidContentDigest)
        );
        let mut bad_type = sample_artifact();
        bad_type.artifact_type_code.clear();
        assert_eq!(
            insert_model_artifact_sql(&bad_type),
            Err(PersistenceError::InvalidContentDigest)
        );
        assert_eq!(super::escape_literal("a'b"), "a''b");
    }

    #[test]
    fn interval_consistency_artifact_binds_canonical_bytes_to_model_run() {
        let mut network = IntervalConsistencyNetwork::with_limits(2, 1, 8).expect("limits");
        let left = network.add_variable().expect("left");
        let right = network.add_variable().expect("right");
        network
            .assert_qualitative_relations(
                left,
                right,
                RelationSet::singleton(AllenRelation::Before),
            )
            .expect("assertion");
        let artifact = event_core::IntervalConsistencyArtifact::from_network(
            "run-1",
            "snapshot-1",
            "ab".repeat(32),
            "2026-08-28T00:00:00Z",
            &network,
            &[("event-1".into(), left), ("event-2".into(), right)],
        )
        .expect("artifact");
        let (available_time, system_time) = sample_times();
        let record = ModelArtifactRecord::for_interval_consistency(
            Uuid::nil(),
            Uuid::nil(),
            Uuid::nil(),
            &artifact,
            "s3://protected/artifact.json",
            system_time,
            available_time,
        )
        .expect("record");
        assert_eq!(
            record.artifact_type_code,
            event_core::INTERVAL_CONSISTENCY_ARTIFACT_TYPE
        );
        assert_eq!(
            record.artifact_content_digest,
            artifact.sha256().expect("digest")
        );
        assert!(
            insert_model_artifact_sql(&record)
                .expect("sql")
                .contains("tdt_chronos")
        );
    }
}
