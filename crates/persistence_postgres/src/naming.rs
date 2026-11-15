//! Database object naming contracts for TEPP persistence.

/// TEPP's portable PostgreSQL identifier ceiling in bytes.
///
/// PostgreSQL's default `NAMEDATALEN = 64` truncates identifiers after 63 bytes.
/// TEPP pins that default ceiling even if a custom server build raises it, so
/// migration identity remains stable across supported environments.
const POSTGRESQL_IDENTIFIER_BYTE_LIMIT: usize = 63;

/// Return whether `name` is descriptive multi-word `snake_case`.
///
/// TEPP requires at least two underscore-separated lowercase segments so
/// physical schema objects remain self-describing in reviews and audits. Names
/// must also fit PostgreSQL's default 63-byte identifier ceiling; accepting a
/// longer spelling would let the server silently truncate the durable object
/// identity.
#[must_use]
pub fn is_multi_word_snake_case(name: &str) -> bool {
    if name.is_empty()
        || name.len() > POSTGRESQL_IDENTIFIER_BYTE_LIMIT
        || name.starts_with('_')
        || name.ends_with('_')
        || name.contains("__")
    {
        return false;
    }
    let mut parts = 0usize;
    for part in name.split('_') {
        // Empty segments are already rejected by the `__` / boundary checks above.
        if !part
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
        {
            return false;
        }
        if part.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
            return false;
        }
        parts += 1;
    }
    parts >= 2
}

#[cfg(test)]
mod tests {
    use super::is_multi_word_snake_case;

    #[test]
    fn multi_word_names_pass_and_invalid_names_fail() {
        assert!(is_multi_word_snake_case("document_record"));
        assert!(is_multi_word_snake_case("audit_event"));
        assert!(is_multi_word_snake_case(
            "document_record_projection_snapshot_archive_registry_history_v1"
        ));
        assert!(!is_multi_word_snake_case(
            "document_record_projection_snapshot_archive_registry_history_v1x"
        ));
        assert!(!is_multi_word_snake_case("documents"));
        assert!(!is_multi_word_snake_case("Document_Record"));
        assert!(!is_multi_word_snake_case("_leading_underscore"));
        assert!(!is_multi_word_snake_case("trailing_underscore_"));
        assert!(!is_multi_word_snake_case("double__underscore"));
        assert!(!is_multi_word_snake_case("1bad_name"));
        assert!(!is_multi_word_snake_case(""));
    }
}
