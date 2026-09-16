//! PostgreSQL lexical normalization for migration contract parsing.

const INVALID_QUOTED_IDENTIFIER: &[u8] = b"INVALID_QUOTED_IDENTIFIER";

pub(super) fn normalize_migration_sql(sql: &str) -> Option<String> {
    let bytes = sql.as_bytes();
    let mut normalized = Vec::with_capacity(bytes.len());
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'\'' => {
                let (next, literal) = scan_single_quoted_literal(bytes, index)?;
                normalized.push(b' ');
                if literal_is_atomic(literal) {
                    normalized.push(b'\'');
                    normalized.extend_from_slice(literal);
                    normalized.push(b'\'');
                }
                normalized.push(b' ');
                index = next;
            }
            b'"' => {
                let (next, identifier) = scan_quoted_identifier(bytes, index)?;
                normalized.push(b' ');
                if quoted_identifier_is_structurally_safe(&identifier) {
                    normalized.extend_from_slice(&identifier);
                } else {
                    normalized.extend_from_slice(INVALID_QUOTED_IDENTIFIER);
                }
                normalized.push(b' ');
                index = next;
            }
            b'-' if bytes.get(index + 1) == Some(&b'-') => {
                normalized.push(b' ');
                index += 2;
                while index < bytes.len() && !matches!(bytes[index], b'\n' | b'\r') {
                    index += 1;
                }
                if index < bytes.len() {
                    normalized.push(bytes[index]);
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                normalized.push(b' ');
                index = scan_block_comment(bytes, index)?;
                normalized.push(b' ');
            }
            b'$' => {
                if let Some(delimiter) = dollar_quote_delimiter(bytes, index) {
                    normalized.push(b' ');
                    index = scan_dollar_quoted_body(bytes, index, delimiter)?;
                    normalized.push(b' ');
                } else {
                    normalized.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                normalized.push(byte);
                index += 1;
            }
        }
    }

    let normalized = String::from_utf8(normalized).ok()?;
    Some(normalized.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn scan_single_quoted_literal(bytes: &[u8], start: usize) -> Option<(usize, &[u8])> {
    let mut index = start + 1;
    let content_start = index;
    while index < bytes.len() {
        if bytes[index] == b'\'' {
            if bytes.get(index + 1) == Some(&b'\'') {
                index += 2;
                continue;
            }
            return Some((index + 1, &bytes[content_start..index]));
        }
        index += 1;
    }
    None
}

fn scan_quoted_identifier(bytes: &[u8], start: usize) -> Option<(usize, Vec<u8>)> {
    let mut index = start + 1;
    let mut identifier = Vec::new();
    while index < bytes.len() {
        if bytes[index] == b'"' {
            if bytes.get(index + 1) == Some(&b'"') {
                identifier.push(b'"');
                index += 2;
                continue;
            }
            return Some((index + 1, identifier));
        }
        identifier.push(bytes[index]);
        index += 1;
    }
    None
}

fn scan_block_comment(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut index = start + 2;
    while index < bytes.len() {
        if bytes.get(index) == Some(&b'/') && bytes.get(index + 1) == Some(&b'*') {
            depth += 1;
            index += 2;
        } else if bytes.get(index) == Some(&b'*') && bytes.get(index + 1) == Some(&b'/') {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return Some(index);
            }
        } else {
            index += 1;
        }
    }
    None
}

fn dollar_quote_delimiter(bytes: &[u8], start: usize) -> Option<&[u8]> {
    if bytes.get(start) != Some(&b'$') {
        return None;
    }
    let mut index = start + 1;
    if bytes.get(index) == Some(&b'$') {
        return Some(&bytes[start..=index]);
    }
    let first = *bytes.get(index)?;
    if first != b'_' && !first.is_ascii_alphabetic() {
        return None;
    }
    index += 1;
    while let Some(byte) = bytes.get(index) {
        if *byte == b'$' {
            return Some(&bytes[start..=index]);
        }
        if *byte != b'_' && !byte.is_ascii_alphanumeric() {
            return None;
        }
        index += 1;
    }
    None
}

fn scan_dollar_quoted_body(bytes: &[u8], start: usize, delimiter: &[u8]) -> Option<usize> {
    let mut index = start + delimiter.len();
    while index + delimiter.len() <= bytes.len() {
        if &bytes[index..index + delimiter.len()] == delimiter {
            return Some(index + delimiter.len());
        }
        index += 1;
    }
    None
}

fn literal_is_atomic(literal: &[u8]) -> bool {
    !literal.is_empty()
        && literal
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'.'))
}

fn quoted_identifier_is_structurally_safe(identifier: &[u8]) -> bool {
    !identifier.is_empty()
        && identifier
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::normalize_migration_sql;

    #[test]
    fn lexical_normalization_masks_declaration_shaped_trivia() {
        let sql = r#"
            -- CREATE INDEX Bad ON tenant_record (tenant_record_id);
            SELECT 'CREATE INDEX Bad ON tenant_record (tenant_record_id)';
            /* outer /* CREATE VIEW Bad AS SELECT 1 */ still comment */
            CREATE INDEX "good_index" ON tenant_record (tenant_record_id);
        "#;
        let normalized = normalize_migration_sql(sql).expect("well-formed SQL");
        assert!(!normalized.contains("CREATE INDEX Bad"));
        assert!(!normalized.contains("CREATE VIEW Bad"));
        assert!(normalized.contains("CREATE INDEX good_index ON tenant_record"));
    }

    #[test]
    fn lexical_normalization_preserves_atomic_contract_literals() {
        let sql = "SELECT current_setting('tepp.current_tenant_record_id', true), 'x', '';";
        let normalized = normalize_migration_sql(sql).expect("well-formed SQL");
        assert!(normalized.contains("'tepp.current_tenant_record_id'"));
        assert!(normalized.contains("'x'"));
        assert!(!normalized.contains("''"));
    }

    #[test]
    fn atomic_literals_keep_boundaries_between_sql_keywords() {
        let normalized = normalize_migration_sql("SELECT 'CREATE'\n'INDEX' AS literal_text;")
            .expect("well-formed adjacent literals");
        assert!(!normalized.contains("CREATE INDEX"));
        assert!(normalized.contains("'CREATE' 'INDEX'"));
    }

    #[test]
    fn quoted_identifiers_preserve_safe_spelling_and_reject_unrepresentable_content() {
        let normalized = normalize_migration_sql(
            "CREATE INDEX \"Bad\" ON tenant_record (\"good_name\"); CREATE VIEW \"a\"\"b\" AS SELECT 1;",
        )
        .expect("well-formed quoted identifiers");
        assert!(normalized.contains("CREATE INDEX Bad ON tenant_record ( good_name )"));
        assert!(normalized.contains("CREATE VIEW INVALID_QUOTED_IDENTIFIER AS SELECT 1"));
    }

    #[test]
    fn dollar_quoted_bodies_do_not_declare_migration_objects() {
        let normalized = normalize_migration_sql(
            "CREATE FUNCTION good_function() RETURNS void AS $body$ CREATE INDEX Bad ON x(y); $body$ LANGUAGE sql;",
        )
        .expect("well-formed dollar quote");
        assert!(normalized.contains("CREATE FUNCTION good_function() RETURNS void AS"));
        assert!(!normalized.contains("CREATE INDEX Bad"));
    }

    #[test]
    fn malformed_lexical_regions_fail_closed() {
        for sql in [
            "SELECT 'unterminated",
            "CREATE TABLE \"unterminated",
            "/* unterminated",
            "DO $body$ unterminated",
        ] {
            assert!(normalize_migration_sql(sql).is_none(), "{sql}");
        }
    }

    #[test]
    fn positional_dollar_parameters_are_not_dollar_quotes() {
        let normalized = normalize_migration_sql("SELECT $1, $2;").expect("parameters");
        assert_eq!(normalized, "SELECT $1, $2;");
    }
}
