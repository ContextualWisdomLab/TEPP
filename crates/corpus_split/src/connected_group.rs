//! Relation-connected leakage groups.

use crate::CorpusSplitError;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use uuid::Uuid;

/// Governed link kinds that force co-partitioning.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LeakageLinkKind {
    /// Document revision of another document.
    Revision,
    /// Translation of another document.
    Translation,
    /// Copied or template-derived variant.
    CopiedVariant,
    /// Shared event episode membership.
    SameEpisode,
    /// NFC/NFD (or other UAX #15) canonically equivalent document bodies.
    CanonicalEquivalent,
}

/// Undirected leakage link between two document identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeakageLink {
    /// First endpoint.
    pub left: Uuid,
    /// Second endpoint.
    pub right: Uuid,
    /// Governed link kind.
    pub kind: LeakageLinkKind,
}

/// Union of documents that must remain in one split partition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectedGroup {
    members: BTreeSet<Uuid>,
}

impl ConnectedGroup {
    /// Borrow member identities.
    #[must_use]
    pub fn members(&self) -> &BTreeSet<Uuid> {
        &self.members
    }

    /// Return the number of members.
    #[must_use]
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Return whether the group is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

/// Build connected components over leakage links restricted to `universe`.
#[must_use]
pub fn build_connected_groups(universe: &[Uuid], links: &[LeakageLink]) -> Vec<ConnectedGroup> {
    let allowed: BTreeSet<Uuid> = universe.iter().copied().collect();
    let mut adjacency: BTreeMap<Uuid, BTreeSet<Uuid>> = BTreeMap::new();
    for id in &allowed {
        adjacency.entry(*id).or_default();
    }
    for link in links {
        if !allowed.contains(&link.left) || !allowed.contains(&link.right) {
            continue;
        }
        if link.left == link.right {
            continue;
        }
        adjacency.entry(link.left).or_default().insert(link.right);
        adjacency.entry(link.right).or_default().insert(link.left);
    }

    let mut seen = BTreeSet::new();
    let mut groups = Vec::new();
    for start in universe {
        if !seen.insert(*start) {
            continue;
        }
        let mut members = BTreeSet::new();
        let mut queue = VecDeque::from([*start]);
        while let Some(node) = queue.pop_front() {
            // Each node is enqueued at most once via `seen`, so membership insert always succeeds.
            members.insert(node);
            // Every universe identity is pre-inserted into `adjacency`.
            for neighbor in &adjacency[&node] {
                if seen.insert(*neighbor) {
                    queue.push_back(*neighbor);
                }
            }
        }
        groups.push(ConnectedGroup { members });
    }
    groups
}

/// Reject a proposed partition map that separates any connected group.
///
/// # Errors
///
/// Returns [`CorpusSplitError::RelationLeakage`] when linked members diverge.
pub fn assert_no_group_leakage(
    groups: &[ConnectedGroup],
    partition_by_document: &BTreeMap<Uuid, u8>,
) -> Result<(), CorpusSplitError> {
    for group in groups {
        let mut partition = None;
        for member in &group.members {
            let Some(assigned) = partition_by_document.get(member) else {
                continue;
            };
            match partition {
                None => partition = Some(*assigned),
                Some(existing) if existing == *assigned => {}
                Some(_) => return Err(CorpusSplitError::RelationLeakage),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ConnectedGroup, LeakageLink, LeakageLinkKind, assert_no_group_leakage,
        build_connected_groups,
    };
    use crate::CorpusSplitError;
    use std::collections::{BTreeMap, BTreeSet};
    use uuid::Uuid;

    #[test]
    fn revisions_and_translations_form_one_group() {
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        let c = Uuid::now_v7();
        let groups = build_connected_groups(
            &[a, b, c],
            &[
                LeakageLink {
                    left: a,
                    right: b,
                    kind: LeakageLinkKind::Revision,
                },
                LeakageLink {
                    left: b,
                    right: c,
                    kind: LeakageLinkKind::Translation,
                },
            ],
        );
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
        assert!(!groups[0].is_empty());
        assert_eq!(groups[0].members(), &BTreeSet::from([a, b, c]));
    }

    #[test]
    fn cross_partition_assignment_is_rejected() {
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        let groups = build_connected_groups(
            &[a, b],
            &[LeakageLink {
                left: a,
                right: b,
                kind: LeakageLinkKind::CopiedVariant,
            }],
        );
        let mut map = BTreeMap::new();
        map.insert(a, 0);
        map.insert(b, 1);
        assert_eq!(
            assert_no_group_leakage(&groups, &map),
            Err(CorpusSplitError::RelationLeakage)
        );
        map.insert(b, 0);
        assert_no_group_leakage(&groups, &map).expect("same partition");
        let empty = ConnectedGroup {
            members: BTreeSet::new(),
        };
        assert!(empty.is_empty());
        // Self-links and out-of-universe endpoints are ignored.
        let lonely = Uuid::now_v7();
        let outside = Uuid::now_v7();
        let isolated = build_connected_groups(
            &[lonely],
            &[
                LeakageLink {
                    left: lonely,
                    right: lonely,
                    kind: LeakageLinkKind::Revision,
                },
                LeakageLink {
                    left: lonely,
                    right: outside,
                    kind: LeakageLinkKind::Translation,
                },
                LeakageLink {
                    left: outside,
                    right: lonely,
                    kind: LeakageLinkKind::CopiedVariant,
                },
            ],
        );
        assert_eq!(isolated.len(), 1);
        assert_eq!(isolated[0].len(), 1);
        // Unassigned members do not induce leakage.
        let partial = BTreeMap::from([(a, 0_u8)]);
        assert_no_group_leakage(&groups, &partial).expect("partial assignment");
    }
}
