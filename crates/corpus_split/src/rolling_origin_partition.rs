//! Owner-issued admission for newly available rolling-origin evaluation partitions.

use std::collections::{BTreeMap, BTreeSet};

use temporal_core::KnowledgeCutoff;
use uuid::Uuid;

use crate::{
    CorpusSnapshot, CorpusSplitError, LeakageLink, RollingOriginWindow, assert_no_group_leakage,
    build_connected_groups, rolling_origin_windows,
};

/// One leakage-checked train/evaluation partition for a canonical rolling-origin window.
///
/// This value proves only local split integrity over cutoff-bound
/// [`CorpusSnapshot`] values and governed [`LeakageLink`] components. It does
/// not authenticate source evidence, event-valid time, Membership provenance,
/// or relation activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollingOriginPartition {
    window: RollingOriginWindow,
    training_document_ids: BTreeSet<Uuid>,
    evaluation_document_ids: BTreeSet<Uuid>,
}

impl RollingOriginPartition {
    /// Return the canonical train/test cutoff window used for admission.
    #[must_use]
    pub const fn window(&self) -> &RollingOriginWindow {
        &self.window
    }

    /// Return sorted identities admitted to the training partition.
    #[must_use]
    pub const fn training_document_ids(&self) -> &BTreeSet<Uuid> {
        &self.training_document_ids
    }

    /// Return sorted identities admitted to the newly available evaluation partition.
    #[must_use]
    pub const fn evaluation_document_ids(&self) -> &BTreeSet<Uuid> {
        &self.evaluation_document_ids
    }
}

/// Leakage-safe expanding-history admission plus its owner-derived exclusions.
///
/// The wrapped partition starts from every document in the training snapshot,
/// then removes only historical identities whose governed leakage component
/// intersects the current evaluation set. The exclusion set therefore records
/// why an otherwise cutoff-eligible historical document was withheld from this
/// expanding-history fit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpandingRollingOriginPartition {
    partition: RollingOriginPartition,
    leakage_excluded_training_document_ids: BTreeSet<Uuid>,
}

impl ExpandingRollingOriginPartition {
    /// Return the canonical leakage-checked partition consumed by estimators.
    #[must_use]
    pub const fn partition(&self) -> &RollingOriginPartition {
        &self.partition
    }

    /// Return historical identities excluded because their leakage component touches evaluation.
    #[must_use]
    pub const fn leakage_excluded_training_document_ids(&self) -> &BTreeSet<Uuid> {
        &self.leakage_excluded_training_document_ids
    }
}

fn canonical_nonempty_document_set(
    document_ids: &[Uuid],
) -> Result<BTreeSet<Uuid>, CorpusSplitError> {
    if document_ids.is_empty() {
        return Err(CorpusSplitError::InvalidSplitConfiguration);
    }
    let ids: BTreeSet<_> = document_ids.iter().copied().collect();
    if ids.len() != document_ids.len() {
        return Err(CorpusSplitError::DuplicateDocumentIdentity);
    }
    Ok(ids)
}

/// Admit one newly available rolling-origin train/evaluation partition.
///
/// The window is derived through [`rolling_origin_windows`] from the supplied
/// ordered cutoff sequence; callers cannot substitute an independently defined
/// train/test pair. The training and evaluation snapshots must be bound to the
/// selected window's exact knowledge cutoffs. Every evaluation identity must be
/// absent from the prior training snapshot, not merely omitted from the caller's
/// training-ID slice, and must carry a retained availability time strictly after
/// the training cutoff and no later than the evaluation snapshot's already-
/// validated cutoff. Prior-snapshot presence is local proof that an identity is
/// not newly available; it is not external source-provenance authentication.
/// Snapshot absence alone is never treated as proof that a row was unavailable.
///
/// Governed revisions, translations, copied variants, shared episodes, and
/// canonically equivalent records are checked with the existing connected-group
/// owner and may not straddle the two partitions.
///
/// # Errors
///
/// Returns [`CorpusSplitError::InvalidSplitConfiguration`] for a missing window,
/// empty partition, overlap, an evaluation identity already present in the prior
/// training snapshot, or an evaluation identity whose retained availability time
/// is not later than the training cutoff;
/// [`CorpusSplitError::DuplicateDocumentIdentity`] for a duplicate identity
/// inside either partition; [`CorpusSplitError::KnowledgeCutoffMismatch`] when
/// either snapshot is bound to a different horizon;
/// [`CorpusSplitError::UnavailableAtCutoff`] when a listed identity is absent
/// from its required snapshot; or [`CorpusSplitError::RelationLeakage`] when a
/// governed connected group crosses train/evaluation.
pub fn admit_rolling_origin_partition(
    ordered_cutoffs: &[KnowledgeCutoff],
    window_index: usize,
    training_snapshot: &CorpusSnapshot,
    evaluation_snapshot: &CorpusSnapshot,
    training_document_ids: &[Uuid],
    evaluation_document_ids: &[Uuid],
    leakage_links: &[LeakageLink],
) -> Result<RollingOriginPartition, CorpusSplitError> {
    let windows = rolling_origin_windows(ordered_cutoffs)?;
    let window = *windows
        .get(window_index)
        .ok_or(CorpusSplitError::InvalidSplitConfiguration)?;
    if training_snapshot.knowledge_cutoff() != Some(window.train_cutoff)
        || evaluation_snapshot.knowledge_cutoff() != Some(window.test_cutoff)
    {
        return Err(CorpusSplitError::KnowledgeCutoffMismatch);
    }

    let training = canonical_nonempty_document_set(training_document_ids)?;
    let evaluation = canonical_nonempty_document_set(evaluation_document_ids)?;
    if !training.is_disjoint(&evaluation) {
        return Err(CorpusSplitError::InvalidSplitConfiguration);
    }
    if evaluation
        .iter()
        .any(|document_id| training_snapshot.contains(*document_id))
    {
        return Err(CorpusSplitError::InvalidSplitConfiguration);
    }
    if training
        .iter()
        .any(|document_id| !training_snapshot.contains(*document_id))
    {
        return Err(CorpusSplitError::UnavailableAtCutoff);
    }
    for document_id in &evaluation {
        let available_time = evaluation_snapshot
            .available_time(*document_id)
            .ok_or(CorpusSplitError::UnavailableAtCutoff)?;
        if available_time.instant() <= window.train_cutoff.instant() {
            return Err(CorpusSplitError::InvalidSplitConfiguration);
        }
    }

    let universe: Vec<_> = training.iter().chain(&evaluation).copied().collect();
    let groups = build_connected_groups(&universe, leakage_links);
    let mut partition_by_document = BTreeMap::new();
    partition_by_document.extend(training.iter().map(|document_id| (*document_id, 0_u8)));
    partition_by_document.extend(evaluation.iter().map(|document_id| (*document_id, 1_u8)));
    assert_no_group_leakage(&groups, &partition_by_document)?;

    Ok(RollingOriginPartition {
        window,
        training_document_ids: training,
        evaluation_document_ids: evaluation,
    })
}

/// Admit one leakage-safe expanding-history rolling-origin partition.
///
/// Unlike [`admit_rolling_origin_partition`], callers do not choose training
/// identities. The owner begins with every identity in the cutoff-bound training
/// snapshot. It then removes complete governed leakage components that intersect
/// the current evaluation set, so a historical revision/translation/copied
/// variant/episode/canonical equivalent cannot be trained against its related
/// evaluation row. Every other historical identity is retained.
///
/// The returned [`ExpandingRollingOriginPartition`] preserves the exact set of
/// historical identities withheld for that leakage reason. Final cutoff,
/// newly-available evaluation, duplicate, overlap, and relation-leakage checks
/// are delegated to [`admit_rolling_origin_partition`].
///
/// # Errors
///
/// Propagates the canonical rolling-origin admission errors, including
/// [`CorpusSplitError::InvalidSplitConfiguration`] when the evaluation request is
/// invalid or leakage-safe exclusion leaves no training document.
pub fn admit_expanding_rolling_origin_partition(
    ordered_cutoffs: &[KnowledgeCutoff],
    window_index: usize,
    training_snapshot: &CorpusSnapshot,
    evaluation_snapshot: &CorpusSnapshot,
    evaluation_document_ids: &[Uuid],
    leakage_links: &[LeakageLink],
) -> Result<ExpandingRollingOriginPartition, CorpusSplitError> {
    let all_training_document_ids: Vec<_> = training_snapshot.document_ids().collect();
    let preflight = admit_rolling_origin_partition(
        ordered_cutoffs,
        window_index,
        training_snapshot,
        evaluation_snapshot,
        &all_training_document_ids,
        evaluation_document_ids,
        &[],
    )?;

    let universe: Vec<_> = preflight
        .training_document_ids()
        .iter()
        .chain(preflight.evaluation_document_ids())
        .copied()
        .collect();
    let groups = build_connected_groups(&universe, leakage_links);
    let mut leakage_excluded_training_document_ids = BTreeSet::new();
    for group in groups {
        if !group
            .members()
            .is_disjoint(preflight.evaluation_document_ids())
        {
            leakage_excluded_training_document_ids.extend(
                group
                    .members()
                    .intersection(preflight.training_document_ids())
                    .copied(),
            );
        }
    }

    let training_document_ids: Vec<_> = preflight
        .training_document_ids()
        .difference(&leakage_excluded_training_document_ids)
        .copied()
        .collect();
    let partition = admit_rolling_origin_partition(
        ordered_cutoffs,
        window_index,
        training_snapshot,
        evaluation_snapshot,
        &training_document_ids,
        evaluation_document_ids,
        leakage_links,
    )?;

    Ok(ExpandingRollingOriginPartition {
        partition,
        leakage_excluded_training_document_ids,
    })
}
