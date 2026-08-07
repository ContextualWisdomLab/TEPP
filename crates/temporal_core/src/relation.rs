//! Allen-style qualitative relations over proper bounded temporal intervals.

use crate::{TemporalBoundary, TemporalCertainty, TemporalClock, TemporalError, TemporalInterval};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::OnceLock;

const ELEMENTARY_RELATION_COUNT: usize = 13;
const ALL_RELATION_BITS: u16 = (1_u16 << ELEMENTARY_RELATION_COUNT) - 1;

/// One of Allen's thirteen elementary relations between proper intervals.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum AllenRelation {
    /// The left interval ends before the right interval starts.
    Before = 0,
    /// The left interval starts after the right interval ends.
    After = 1,
    /// The left interval ends exactly when the right interval starts.
    Meets = 2,
    /// The left interval starts exactly when the right interval ends.
    MetBy = 3,
    /// The left interval starts first, intersects the right interval, and ends first.
    Overlaps = 4,
    /// The right interval starts first, intersects the left interval, and ends first.
    OverlappedBy = 5,
    /// Both intervals start together and the left interval ends first.
    Starts = 6,
    /// Both intervals start together and the right interval ends first.
    StartedBy = 7,
    /// The left interval is strictly inside the right interval.
    During = 8,
    /// The right interval is strictly inside the left interval.
    Contains = 9,
    /// Both intervals end together and the left interval starts later.
    Finishes = 10,
    /// Both intervals end together and the right interval starts later.
    FinishedBy = 11,
    /// Both interval endpoints are equal.
    Equals = 12,
}

impl AllenRelation {
    /// Every elementary relation in stable bit-index order.
    pub const ALL: [Self; ELEMENTARY_RELATION_COUNT] = [
        Self::Before,
        Self::After,
        Self::Meets,
        Self::MetBy,
        Self::Overlaps,
        Self::OverlappedBy,
        Self::Starts,
        Self::StartedBy,
        Self::During,
        Self::Contains,
        Self::Finishes,
        Self::FinishedBy,
        Self::Equals,
    ];

    /// Return the relation obtained by swapping the left and right intervals.
    #[must_use]
    pub const fn inverse(self) -> Self {
        match self {
            Self::Before => Self::After,
            Self::After => Self::Before,
            Self::Meets => Self::MetBy,
            Self::MetBy => Self::Meets,
            Self::Overlaps => Self::OverlappedBy,
            Self::OverlappedBy => Self::Overlaps,
            Self::Starts => Self::StartedBy,
            Self::StartedBy => Self::Starts,
            Self::During => Self::Contains,
            Self::Contains => Self::During,
            Self::Finishes => Self::FinishedBy,
            Self::FinishedBy => Self::Finishes,
            Self::Equals => Self::Equals,
        }
    }

    const fn bit(self) -> u16 {
        1_u16 << (self as u8)
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// A compact set of possible elementary interval relations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RelationSet(u16);

impl RelationSet {
    /// Return the empty relation set.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Return the universal set containing all thirteen elementary relations.
    #[must_use]
    pub const fn all() -> Self {
        Self(ALL_RELATION_BITS)
    }

    /// Return a set containing exactly one elementary relation.
    #[must_use]
    pub const fn singleton(relation: AllenRelation) -> Self {
        Self(relation.bit())
    }

    /// Build a set from a slice of elementary relations.
    #[must_use]
    pub fn from_relations(relations: &[AllenRelation]) -> Self {
        let mut result = Self::empty();
        for relation in relations {
            result = result.with(*relation);
        }
        result
    }

    /// Return whether this set contains `relation`.
    #[must_use]
    pub const fn contains(self, relation: AllenRelation) -> bool {
        self.0 & relation.bit() != 0
    }

    /// Return the number of elementary relations in this set.
    #[must_use]
    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    /// Return whether this set contains no relation.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Iterate over contained relations once in stable elementary order.
    pub fn iter(self) -> impl Iterator<Item = AllenRelation> {
        AllenRelation::ALL
            .into_iter()
            .filter(move |relation| self.contains(*relation))
    }

    /// Return the intersection of two relation sets.
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Return the union of two relation sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Return the relation set obtained by swapping the left and right intervals.
    #[must_use]
    pub fn inverse(self) -> Self {
        let mut result = Self::empty();
        for relation in AllenRelation::ALL {
            if self.contains(relation) {
                result = result.with(relation.inverse());
            }
        }
        result
    }

    /// Compose this left-to-middle set with a middle-to-right set.
    ///
    /// The result contains every elementary left-to-right relation compatible
    /// with at least one relation from each input set.
    #[must_use]
    pub fn compose(self, other: Self) -> Self {
        let table = composition_table();
        let mut result = Self::empty();
        for left in AllenRelation::ALL {
            if !self.contains(left) {
                continue;
            }
            for right in AllenRelation::ALL {
                if other.contains(right) {
                    result = result.union(table[left.index()][right.index()]);
                }
            }
        }
        result
    }

    const fn with(self, relation: AllenRelation) -> Self {
        Self(self.0 | relation.bit())
    }
}

/// Classify two proper, two-sided, nonzero intervals with Allen's algebra.
///
/// Boundary inclusion does not change the qualitative endpoint relation. Exact,
/// open-ended, and explicitly unknown intervals are rejected because Allen's
/// elementary interval algebra assumes proper intervals with distinct starts
/// and ends.
///
/// # Errors
///
/// Returns [`TemporalError::RelationRequiresProperBoundedInterval`] when either
/// interval is not a proper two-sided bounded interval.
pub fn classify_interval_relation<T: TemporalClock>(
    left: &TemporalInterval<T>,
    right: &TemporalInterval<T>,
) -> Result<AllenRelation, TemporalError> {
    let (left_start, left_end) = proper_endpoints(left)?;
    let (right_start, right_end) = proper_endpoints(right)?;
    Ok(classify_endpoints(
        left_start,
        left_end,
        right_start,
        right_end,
    ))
}

fn proper_endpoints<T: TemporalClock>(
    interval: &TemporalInterval<T>,
) -> Result<(i128, i128), TemporalError> {
    if interval.certainty() != TemporalCertainty::Bounded {
        return Err(TemporalError::RelationRequiresProperBoundedInterval);
    }
    let Some(start) = boundary_nanosecond(interval.lower()) else {
        return Err(TemporalError::RelationRequiresProperBoundedInterval);
    };
    let Some(end) = boundary_nanosecond(interval.upper()) else {
        return Err(TemporalError::RelationRequiresProperBoundedInterval);
    };
    Ok((start, end))
}

fn boundary_nanosecond<T: TemporalClock>(boundary: TemporalBoundary<T>) -> Option<i128> {
    match boundary {
        TemporalBoundary::Unbounded => None,
        TemporalBoundary::Included(value) | TemporalBoundary::Excluded(value) => {
            Some(value.instant().as_nanosecond())
        }
    }
}

fn classify_endpoints(
    left_start: i128,
    left_end: i128,
    right_start: i128,
    right_end: i128,
) -> AllenRelation {
    if left_end < right_start {
        AllenRelation::Before
    } else if left_start > right_end {
        AllenRelation::After
    } else if left_end == right_start {
        AllenRelation::Meets
    } else if left_start == right_end {
        AllenRelation::MetBy
    } else if left_start == right_start {
        match left_end.cmp(&right_end) {
            Ordering::Less => AllenRelation::Starts,
            Ordering::Greater => AllenRelation::StartedBy,
            Ordering::Equal => AllenRelation::Equals,
        }
    } else if left_end == right_end {
        if left_start > right_start {
            AllenRelation::Finishes
        } else {
            AllenRelation::FinishedBy
        }
    } else if left_start < right_start {
        if left_end < right_end {
            AllenRelation::Overlaps
        } else {
            AllenRelation::Contains
        }
    } else if left_end < right_end {
        AllenRelation::During
    } else {
        AllenRelation::OverlappedBy
    }
}

fn composition_table()
-> &'static [[RelationSet; ELEMENTARY_RELATION_COUNT]; ELEMENTARY_RELATION_COUNT] {
    static TABLE: OnceLock<[[RelationSet; ELEMENTARY_RELATION_COUNT]; ELEMENTARY_RELATION_COUNT]> =
        OnceLock::new();
    TABLE.get_or_init(build_composition_table)
}

fn build_composition_table() -> [[RelationSet; ELEMENTARY_RELATION_COUNT]; ELEMENTARY_RELATION_COUNT]
{
    let mut table = [[RelationSet::empty(); ELEMENTARY_RELATION_COUNT]; ELEMENTARY_RELATION_COUNT];

    for left_start in 0_i128..6 {
        for left_end in (left_start + 1)..6 {
            for middle_start in 0_i128..6 {
                for middle_end in (middle_start + 1)..6 {
                    let left_relation =
                        classify_endpoints(left_start, left_end, middle_start, middle_end);
                    for right_start in 0_i128..6 {
                        for right_end in (right_start + 1)..6 {
                            let right_relation = classify_endpoints(
                                middle_start,
                                middle_end,
                                right_start,
                                right_end,
                            );
                            let composed_relation =
                                classify_endpoints(left_start, left_end, right_start, right_end);
                            let current = table[left_relation.index()][right_relation.index()];
                            table[left_relation.index()][right_relation.index()] =
                                current.with(composed_relation);
                        }
                    }
                }
            }
        }
    }

    table
}
