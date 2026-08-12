#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Leakage-safe corpus snapshots and relation-aware data splits.
//!
//! TEPP historical analyses may only consume documents whose availability time
//! does not exceed the declared knowledge cutoff. Train/validation/test
//! partitions keep revisions, translations, copied variants, and shared
//! episodes co-located so relation-aware leakage cannot invent independence.

mod connected_group;
mod document;
mod error;
mod rolling_origin;
mod snapshot;
mod weights;

use temporal_core::{AvailableTime, KnowledgeCutoff};

/// Knowledge-cutoff eligibility for snapshot membership.
#[must_use]
pub fn cutoff_eligible(available_time: &AvailableTime, knowledge_cutoff: &KnowledgeCutoff) -> bool {
    available_time.instant() <= knowledge_cutoff.instant()
}

/// Relation-connected leakage group.
pub use connected_group::ConnectedGroup;
/// Undirected leakage link.
pub use connected_group::LeakageLink;
/// Governed leakage link vocabulary.
pub use connected_group::LeakageLinkKind;
/// Reject partitions that separate linked groups.
pub use connected_group::assert_no_group_leakage;
/// Build connected components over leakage links.
pub use connected_group::build_connected_groups;
/// Document observation with availability provenance.
pub use document::CorpusDocument;
/// Fail-closed corpus-split errors.
pub use error::CorpusSplitError;
/// Rolling-origin train/test window.
pub use rolling_origin::RollingOriginWindow;
/// Build ordered rolling-origin windows.
pub use rolling_origin::rolling_origin_windows;
/// Cutoff-filtered corpus snapshot.
pub use snapshot::CorpusSnapshot;
/// Kish effective sample size.
pub use weights::effective_sample_size;
/// Group-normalized observation weights.
pub use weights::group_normalized_weights;

#[cfg(test)]
mod tests {
    use super::cutoff_eligible;
    use temporal_core::{AvailableTime, KnowledgeCutoff};

    #[test]
    fn cutoff_boundary_is_inclusive() {
        let stamp = "2026-05-01T00:00:00Z";
        assert!(cutoff_eligible(
            &AvailableTime::parse_rfc3339(stamp).expect("a"),
            &KnowledgeCutoff::parse_rfc3339(stamp).expect("c")
        ));
        assert!(!cutoff_eligible(
            &AvailableTime::parse_rfc3339("2026-05-02T00:00:00Z").expect("a"),
            &KnowledgeCutoff::parse_rfc3339(stamp).expect("c")
        ));
    }
}
