//! Knowledge-cutoff eligibility for historical analytical reads.

use temporal_core::{AvailableTime, KnowledgeCutoff};

/// Return whether evidence with `available_time` may enter a historical fit.
///
/// Enforces `available_time <= knowledge_cutoff` on absolute instants so no
/// analysis consumes evidence that was unavailable at the declared cutoff.
#[must_use]
pub fn is_cutoff_eligible(
    available_time: &AvailableTime,
    knowledge_cutoff: &KnowledgeCutoff,
) -> bool {
    available_time.instant() <= knowledge_cutoff.instant()
}

#[cfg(test)]
mod tests {
    use super::is_cutoff_eligible;
    use temporal_core::{AvailableTime, KnowledgeCutoff};

    #[test]
    fn equal_instants_are_eligible() {
        let stamp = "2026-05-01T12:00:00Z";
        assert!(is_cutoff_eligible(
            &AvailableTime::parse_rfc3339(stamp).expect("available"),
            &KnowledgeCutoff::parse_rfc3339(stamp).expect("cutoff")
        ));
        assert!(!is_cutoff_eligible(
            &AvailableTime::parse_rfc3339("2026-06-01T00:00:00Z").expect("later"),
            &KnowledgeCutoff::parse_rfc3339("2026-05-01T00:00:00Z").expect("cutoff")
        ));
        assert!(is_cutoff_eligible(
            &AvailableTime::parse_rfc3339("2026-04-01T00:00:00Z").expect("earlier"),
            &KnowledgeCutoff::parse_rfc3339("2026-05-01T00:00:00Z").expect("cutoff")
        ));
    }
}
