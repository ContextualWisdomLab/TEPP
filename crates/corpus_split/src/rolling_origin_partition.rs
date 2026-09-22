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
/// selected window's exact knowledge cutoffs. Evaluation identities must exist
/// at the test cutoff but not in the training snapshot, so this scientific path
/// measures genuinely later-available evidence instead of relabeling an
/// already-available row as temporal holdout.
///
/// Governed revisions, translations, copied variants, shared episodes, and
/// canonically equivalent records are checked with the existing connected-group
/// owner and may not straddle the two partitions.
///
/// # Errors
///
/// Returns [`CorpusSplitError::InvalidSplitConfiguration`] for a missing window,
/// empty partition, overlap, or evaluation identity already available in the
/// training snapshot; [`CorpusSplitError::DuplicateDocumentIdentity`] for a
/// duplicate identity inside either partition;
/// [`CorpusSplitError::KnowledgeCutoffMismatch`] when either snapshot is bound to
/// a different horizon; [`CorpusSplitError::UnavailableAtCutoff`] when a listed
/// identity is absent from its required snapshot; or
/// [`CorpusSplitError::RelationLeakage`] when a governed connected group crosses
/// train/evaluation.
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
    if training
        .iter()
        .any(|document_id| !training_snapshot.contains(*document_id))
        || evaluation
            .iter()
            .any(|document_id| !evaluation_snapshot.contains(*document_id))
    {
        return Err(CorpusSplitError::UnavailableAtCutoff);
    }
    if evaluation
        .iter()
        .any(|document_id| training_snapshot.contains(*document_id))
    {
        return Err(CorpusSplitError::InvalidSplitConfiguration);
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
