//! PostgreSQL lexical normalization for migration contract parsing.

use std::collections::BTreeMap;

const INVALID_QUOTED_IDENTIFIER: &[u8] = b"INVALID_QUOTED_IDENTIFIER";
const INVALID_QUALIFIED_IDENTIFIER: &str = "INVALID_QUALIFIED_IDENTIFIER";

/// Final security-relevant attributes tracked for one PostgreSQL role.
///
/// The migration contract does not model the full PostgreSQL role catalog; it
/// keeps only attributes that can bypass TEPP tenant row-level security.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RoleSecurityState {
    is_superuser: bool,
    bypasses_rls: bool,
}

/// Normalize migration SQL for bounded structural contract parsing.
///
/// The lexical pass removes declaration-shaped trivia while preserving the few
/// atomic literals required by downstream contracts. The structural pass then
/// canonicalizes PostgreSQL aliases without pretending to be a full SQL parser.
/// Returns `None` when any lexical region is malformed or unterminated.
pub(super) fn normalize_migration_sql(sql: &str) -> Option<String> {
    let normalized = lexically_normalize_migration_sql(sql)?;
    Some(canonicalize_structural_keywords(&normalized))
}

/// Return whether the expected runtime role exists in the final migration state
/// and remains subject to PostgreSQL row-level security. Role creation aliases,
/// drops, renames, and later ALTER ROLE/USER/GROUP attribute changes share this
/// lifecycle scan so SUPERUSER/BYPASSRLS cannot survive under the expected name.
pub(super) fn declares_created_role(sql: &str, expected_role: &str) -> Option<bool> {
    let normalized = lexically_normalize_migration_sql(sql)?;
    // The lexical pass has already masked literals/comments and converted
    // representable quoted identifiers. PostgreSQL statement/list delimiters
    // remain structural, so make them explicit tokens before role lifecycle
    // scanning; whitespace is not required around either delimiter.
    let role_tokens = normalized.replace(';', " ; ").replace(',', " , ");
    let tokens = role_tokens.split_whitespace().collect::<Vec<_>>();
    let mut declared_roles = BTreeMap::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if is_role_creation_alias(&tokens, index) {
            let name = normalized_role_identifier(tokens.get(index + 2).copied().unwrap_or_default());
            if name.is_empty() {
                return Some(false);
            }
            let mut state = RoleSecurityState::default();
            apply_role_security_attributes(
                &tokens[index + 3..statement_end(&tokens, index + 3)],
                &mut state,
            );
            declared_roles.insert(name, state);
        } else if is_role_drop_alias(&tokens, index) {
            let Some(names) = drop_statement_role_names(&tokens, index) else {
                return Some(false);
            };
            for name in names {
                declared_roles.remove(&name);
            }
        } else if is_role_rename_alias(&tokens, index) {
            let source = normalized_role_identifier(tokens.get(index + 2).copied().unwrap_or_default());
            let target = normalized_role_identifier(tokens.get(index + 5).copied().unwrap_or_default());
            if source.is_empty() || target.is_empty() {
                return Some(false);
            }
            if let Some(state) = declared_roles.remove(&source) {
                declared_roles.insert(target, state);
            }
        } else if is_role_alter_alias(&tokens, index) {
            let name = normalized_role_identifier(tokens.get(index + 2).copied().unwrap_or_default());
            if name.is_empty() {
                return Some(false);
            }
            let end = statement_end(&tokens, index + 3);
            let attributes = &tokens[index + 3..end];
            if attributes.is_empty()
                || attributes
                    .first()
                    .is_some_and(|token| token.eq_ignore_ascii_case("RENAME"))
            {
                return Some(false);
            }
            if let Some(state) = declared_roles.get_mut(&name) {
                apply_role_security_attributes(attributes, state);
            }
        }
        index += 1;
    }
    Some(
        declared_roles
            .get(&expected_role.to_ascii_lowercase())
            .is_some_and(|state| !state.is_superuser && !state.bypasses_rls),
    )
}

/// Detect whether normalized migration SQL declares an RLS surface.
///
/// Either table enablement or policy creation activates downstream tenant RLS
/// validation so partially declared isolation cannot remain outside the gate.
pub(super) fn declares_row_level_security(normalized_sql: &str) -> bool {
    let lower = normalized_sql.to_ascii_lowercase();
    lower.contains("enable row level security") || lower.contains("create policy")
}

/// Mask quoted/comment bodies and preserve contract-relevant lexical atoms.
///
/// This pass maintains byte positions only insofar as needed for scanning; it
/// intentionally inserts spaces around removed trivia so adjacent SQL tokens
/// cannot become a synthetic declaration. Any unterminated lexical construct
/// fails closed by returning `None`.
fn lexically_normalize_migration_sql(sql: &str) -> Option<String> {
    let bytes = sql.as_bytes();
    let mut normalized = Vec::with_capacity(bytes.len());
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'E' | b'e' if bytes.get(index + 1) == Some(&b'\'') => {
                let (next, literal) = scan_escape_quoted_literal(bytes, index + 1)?;
                normalized.push(b' ');
                if literal_is_atomic(literal) {
                    normalized.push(b'\'');
                    normalized.extend_from_slice(literal);
                    normalized.push(b'\'');
                }
                normalized.push(b' ');
                index = next;
            }
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
                if quoted_identifier_is_structurally_safe(&identifier)
                    && !quoted_identifier_collides_with_table_syntax(&identifier)
                {
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

    String::from_utf8(normalized).ok()
}

/// Return whether `tokens[create_index..]` starts PostgreSQL CREATE ROLE/USER/GROUP.
///
/// `CREATE USER MAPPING` is deliberately excluded because it is an SQL/MED
/// object rather than a role alias and must not enter role lifecycle evidence.
fn is_role_creation_alias(tokens: &[&str], create_index: usize) -> bool {
    if !tokens
        .get(create_index)
        .is_some_and(|token| token.eq_ignore_ascii_case("CREATE"))
    {
        return false;
    }
    let Some(kind) = tokens.get(create_index + 1) else {
        return false;
    };
    if kind.eq_ignore_ascii_case("USER")
        && tokens
            .get(create_index + 2)
            .is_some_and(|token| token.eq_ignore_ascii_case("MAPPING"))
    {
        return false;
    }
    kind.eq_ignore_ascii_case("ROLE")
        || kind.eq_ignore_ascii_case("USER")
        || kind.eq_ignore_ascii_case("GROUP")
}

/// Return whether `tokens[drop_index..]` starts PostgreSQL DROP ROLE/USER/GROUP.
///
/// `DROP USER MAPPING` remains outside the role lifecycle for the same SQL/MED
/// ownership reason as its CREATE counterpart.
fn is_role_drop_alias(tokens: &[&str], drop_index: usize) -> bool {
    if !tokens
        .get(drop_index)
        .is_some_and(|token| token.eq_ignore_ascii_case("DROP"))
    {
        return false;
    }
    let Some(kind) = tokens.get(drop_index + 1) else {
        return false;
    };
    if kind.eq_ignore_ascii_case("USER")
        && tokens
            .get(drop_index + 2)
            .is_some_and(|token| token.eq_ignore_ascii_case("MAPPING"))
    {
        return false;
    }
    kind.eq_ignore_ascii_case("ROLE")
        || kind.eq_ignore_ascii_case("USER")
        || kind.eq_ignore_ascii_case("GROUP")
}

/// Return whether an ALTER ROLE/USER/GROUP statement has a complete RENAME TO shape.
///
/// A complete target is required because rename changes the durable role name;
/// malformed rename syntax must not be reinterpreted as a generic attribute
/// change or leave stale role evidence alive.
fn is_role_rename_alias(tokens: &[&str], alter_index: usize) -> bool {
    if !tokens
        .get(alter_index)
        .is_some_and(|token| token.eq_ignore_ascii_case("ALTER"))
    {
        return false;
    }
    let Some(kind) = tokens.get(alter_index + 1) else {
        return false;
    };
    if kind.eq_ignore_ascii_case("USER")
        && tokens
            .get(alter_index + 2)
            .is_some_and(|token| token.eq_ignore_ascii_case("MAPPING"))
    {
        return false;
    }
    (kind.eq_ignore_ascii_case("ROLE")
        || kind.eq_ignore_ascii_case("USER")
        || kind.eq_ignore_ascii_case("GROUP"))
        && tokens
            .get(alter_index + 3)
            .is_some_and(|token| token.eq_ignore_ascii_case("RENAME"))
        && tokens
            .get(alter_index + 4)
            .is_some_and(|token| token.eq_ignore_ascii_case("TO"))
        && tokens.get(alter_index + 5).is_some()
}

/// Return whether `tokens[alter_index..]` starts a role-alias ALTER statement.
///
/// `ALTER USER MAPPING` is excluded so SQL/MED options cannot be interpreted as
/// security attributes on `tepp_app_runtime`.
fn is_role_alter_alias(tokens: &[&str], alter_index: usize) -> bool {
    if !tokens
        .get(alter_index)
        .is_some_and(|token| token.eq_ignore_ascii_case("ALTER"))
    {
        return false;
    }
    let Some(kind) = tokens.get(alter_index + 1) else {
        return false;
    };
    if kind.eq_ignore_ascii_case("USER")
        && tokens
            .get(alter_index + 2)
            .is_some_and(|token| token.eq_ignore_ascii_case("MAPPING"))
    {
        return false;
    }
    kind.eq_ignore_ascii_case("ROLE")
        || kind.eq_ignore_ascii_case("USER")
        || kind.eq_ignore_ascii_case("GROUP")
}

/// Return the exclusive token index of the current semicolon-delimited statement.
fn statement_end(tokens: &[&str], start: usize) -> usize {
    tokens[start..]
        .iter()
        .position(|token| *token == ";")
        .map_or(tokens.len(), |relative| start + relative)
}

/// Apply the security attributes that determine whether PostgreSQL RLS can be bypassed.
///
/// Later contradictory attributes intentionally win because PostgreSQL ALTER
/// statements mutate role state in order; this helper models that final state.
fn apply_role_security_attributes(tokens: &[&str], state: &mut RoleSecurityState) {
    for token in tokens {
        if token.eq_ignore_ascii_case("SUPERUSER") {
            state.is_superuser = true;
        } else if token.eq_ignore_ascii_case("NOSUPERUSER") {
            state.is_superuser = false;
        } else if token.eq_ignore_ascii_case("BYPASSRLS") {
            state.bypasses_rls = true;
        } else if token.eq_ignore_ascii_case("NOBYPASSRLS") {
            state.bypasses_rls = false;
        }
    }
}

/// Return whether `ch` may start a PostgreSQL unquoted role identifier.
///
/// PostgreSQL's scanner accepts ASCII letters, underscore, and high-bit bytes
/// at identifier start. Rust presents valid UTF-8 high-bit sequences as
/// non-ASCII `char`s, which is the representation this lexical boundary sees.
fn is_postgresql_role_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || !ch.is_ascii()
}

/// Return whether `ch` may continue a PostgreSQL unquoted role identifier.
///
/// Dollar signs and non-ASCII continuations are part of the same PostgreSQL
/// token. Preserving them here prevents a distinct role such as
/// `tepp_app_runtime$shadow` from aliasing the protected runtime in lifecycle
/// state. TEPP's stricter durable naming policy remains a separate authority.
fn is_postgresql_role_identifier_continuation(ch: char) -> bool {
    is_postgresql_role_identifier_start(ch) || ch.is_ascii_digit() || ch == '$'
}

/// Extract one complete PostgreSQL unquoted role identifier from a lifecycle token.
///
/// The lifecycle tokenizer has already separated commas and semicolons. This
/// helper preserves PostgreSQL identifier continuations instead of truncating
/// them to an ASCII prefix, so CREATE/ALTER/RENAME/DROP share the same exact
/// role identity before TEPP applies any stricter naming policy elsewhere.
fn role_identifier(fragment: &str) -> String {
    let mut chars = fragment.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    if !is_postgresql_role_identifier_start(first) {
        return String::new();
    }

    let mut identifier = String::from(first);
    identifier.extend(chars.take_while(|ch| is_postgresql_role_identifier_continuation(*ch)));
    identifier
}

/// Extract and case-fold a role identifier for PostgreSQL lifecycle comparison.
fn normalized_role_identifier(fragment: &str) -> String {
    role_identifier(fragment).to_ascii_lowercase()
}

/// Parse the `DROP ROLE|USER|GROUP [IF EXISTS] name [, ...]` target list.
///
/// The lifecycle validator must reject malformed separators instead of silently
/// compacting them: PostgreSQL requires an alternating `name (, name)*` list,
/// and accepting leading, trailing, adjacent, or missing commas would let an
/// invalid migration retain a previously safe runtime-role state in evidence.
fn drop_statement_role_names(tokens: &[&str], drop_index: usize) -> Option<Vec<String>> {
    let mut name_index = drop_index + 2;
    if tokens
        .get(name_index)
        .is_some_and(|token| token.eq_ignore_ascii_case("IF"))
        && tokens
            .get(name_index + 1)
            .is_some_and(|token| token.eq_ignore_ascii_case("EXISTS"))
    {
        name_index += 2;
    }

    let mut names = Vec::new();
    let mut expects_name = true;
    while let Some(token) = tokens.get(name_index) {
        if *token == ";" {
            break;
        }
        if expects_name {
            if *token == "," {
                return None;
            }
            let name = role_identifier(token);
            if name.is_empty() || name.len() != token.len() {
                return None;
            }
            names.push(name.to_ascii_lowercase());
            expects_name = false;
        } else {
            if *token != "," {
                return None;
            }
            expects_name = true;
        }
        name_index += 1;
    }
    if names.is_empty() || expects_name {
        None
    } else {
        Some(names)
    }
}

/// Canonicalize syntax variants that the structural migration parser owns as one concept.
///
/// Role aliases are projected through the existing one-name scanner; materialized
/// and replaceable views are collapsed to `CREATE VIEW`; and schema-qualified
/// created names are replaced with an invalid sentinel so downstream parsing
/// fails closed instead of truncating to a locally valid prefix.
fn canonicalize_structural_keywords(sql: &str) -> String {
    let tokens = sql.split_whitespace().collect::<Vec<_>>();
    let mut canonical = Vec::with_capacity(tokens.len());
    let mut index = 0usize;
    while index < tokens.len() {
        if is_role_creation_alias(&tokens, index) {
            // ROLE is a cluster-level object used by TEPP's shipped RLS migration;
            // USER and GROUP are PostgreSQL aliases for CREATE ROLE. CREATE USER
            // MAPPING is a distinct SQL/MED statement and must not enter this path.
            // The downstream structural parser only needs a one-name CREATE shape,
            // so route role aliases through its existing CREATE TYPE name scanner.
            canonical.push(tokens[index]);
            canonical.push("TYPE");
            index += 2;
        } else if is_role_rename_alias(&tokens, index) {
            // RENAME changes the durable database-object name. Project the target
            // through the same one-name scanner so ALTER ROLE/USER/GROUP cannot
            // bypass the canonical snake_case naming authority.
            canonical.push("CREATE");
            canonical.push("TYPE");
            canonical.push(tokens[index + 5]);
            index += 6;
        } else if tokens[index].eq_ignore_ascii_case("CREATE")
            && tokens
                .get(index + 1)
                .is_some_and(|token| token.eq_ignore_ascii_case("MATERIALIZED"))
            && tokens
                .get(index + 2)
                .is_some_and(|token| token.eq_ignore_ascii_case("VIEW"))
        {
            canonical.push(tokens[index]);
            canonical.push(tokens[index + 2]);
            index += 3;
        } else if tokens[index].eq_ignore_ascii_case("CREATE")
            && tokens
                .get(index + 1)
                .is_some_and(|token| token.eq_ignore_ascii_case("OR"))
            && tokens
                .get(index + 2)
                .is_some_and(|token| token.eq_ignore_ascii_case("REPLACE"))
            && tokens
                .get(index + 3)
                .is_some_and(|token| token.eq_ignore_ascii_case("VIEW"))
        {
            canonical.push(tokens[index]);
            canonical.push(tokens[index + 3]);
            index += 4;
        } else {
            canonical.push(tokens[index]);
            index += 1;
        }
    }

    let mut guarded = canonical
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<String>>();
    let mut index = 0usize;
    while index < guarded.len() {
        if guarded[index].eq_ignore_ascii_case("CREATE") {
            let kind_index = if guarded
                .get(index + 1)
                .is_some_and(|token| token.eq_ignore_ascii_case("OR"))
                && guarded
                    .get(index + 2)
                    .is_some_and(|token| token.eq_ignore_ascii_case("REPLACE"))
            {
                index + 3
            } else if guarded
                .get(index + 1)
                .is_some_and(|token| token.eq_ignore_ascii_case("UNIQUE"))
            {
                index + 2
            } else {
                index + 1
            };
            let mut name_index = kind_index + 1;
            if guarded
                .get(name_index)
                .is_some_and(|token| token.eq_ignore_ascii_case("IF"))
                && guarded
                    .get(name_index + 1)
                    .is_some_and(|token| token.eq_ignore_ascii_case("NOT"))
                && guarded
                    .get(name_index + 2)
                    .is_some_and(|token| token.eq_ignore_ascii_case("EXISTS"))
            {
                name_index += 3;
            }
            let qualified = guarded
                .get(name_index)
                .is_some_and(|token| token.contains('.'))
                || guarded
                    .get(name_index + 1)
                    .is_some_and(|token| token.starts_with('.'));
            if qualified && name_index < guarded.len() {
                guarded[name_index] = INVALID_QUALIFIED_IDENTIFIER.to_owned();
            }
        }
        index += 1;
    }
    guarded.join(" ")
}

/// Scan a standard PostgreSQL single-quoted literal, honoring doubled quotes.
///
/// Returns the index immediately after the closing quote plus the raw literal
/// payload. Unterminated literals return `None` and fail the outer lexical pass.
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

/// Scan a PostgreSQL `E'...'` literal with backslash and doubled-quote escapes.
///
/// `start` points at the opening quote rather than the preceding `E`. A trailing
/// escape or missing close quote is treated as malformed SQL and returns `None`.
fn scan_escape_quoted_literal(bytes: &[u8], start: usize) -> Option<(usize, &[u8])> {
    let mut index = start + 1;
    let content_start = index;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                index += 2;
            }
            b'\'' => {
                if bytes.get(index + 1) == Some(&b'\'') {
                    index += 2;
                    continue;
                }
                return Some((index + 1, &bytes[content_start..index]));
            }
            _ => {
                index += 1;
            }
        }
    }
    None
}

/// Scan a PostgreSQL double-quoted identifier and unescape doubled quotes.
///
/// The returned bytes retain declared spelling for later naming checks. Missing
/// closing quotes return `None` rather than donating partial identifier evidence.
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

/// Scan a possibly nested PostgreSQL block comment.
///
/// PostgreSQL permits nested `/* ... */` comments, so depth is tracked until the
/// matching outer terminator. An unterminated comment returns `None`.
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

/// Return the PostgreSQL dollar-quote delimiter beginning at `start`, if any.
///
/// A `$...$` sequence attached to a preceding unquoted identifier is not a
/// delimiter. Tags follow PostgreSQL's scanner-level ASCII/high-bit byte rules,
/// which keeps UTF-8 tag bytes valid while positional parameters such as `$1`
/// remain ordinary SQL text.
fn dollar_quote_delimiter(bytes: &[u8], start: usize) -> Option<&[u8]> {
    if bytes.get(start) != Some(&b'$') {
        return None;
    }
    if start > 0
        && bytes.get(start - 1).is_some_and(|byte| {
            byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'$') || *byte >= 0x80
        })
    {
        return None;
    }
    let mut index = start + 1;
    if bytes.get(index) == Some(&b'$') {
        return Some(&bytes[start..=index]);
    }
    let first = *bytes.get(index)?;
    if first != b'_' && !first.is_ascii_alphabetic() && first < 0x80 {
        return None;
    }
    index += 1;
    while let Some(byte) = bytes.get(index) {
        if *byte == b'$' {
            return Some(&bytes[start..=index]);
        }
        if *byte != b'_' && !byte.is_ascii_alphanumeric() && *byte < 0x80 {
            return None;
        }
        index += 1;
    }
    None
}

/// Scan to the closing delimiter of a PostgreSQL dollar-quoted body.
///
/// The body is opaque to migration contract parsing. Missing closing delimiters
/// return `None` so declaration-shaped text cannot escape an unterminated body.
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

/// Return whether a quoted literal is safe to preserve as a contract atom.
///
/// Only non-empty alphanumeric/underscore/dot payloads survive normalization;
/// arbitrary literal SQL remains masked and cannot donate structural evidence.
fn literal_is_atomic(literal: &[u8]) -> bool {
    !literal.is_empty()
        && literal
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'.'))
}

/// Return whether a quoted identifier can be projected onto an unquoted token
/// without changing PostgreSQL identity inside the bounded structural parser.
///
/// PostgreSQL folds unquoted identifiers to lower case but preserves quoted case.
/// The lexical boundary strips quotes only for lowercase ASCII spellings whose
/// quoted and unquoted identities are therefore equivalent. Mixed/uppercase or
/// otherwise unsupported quoted identifiers fail closed through the invalid
/// sentinel instead of aliasing a distinct PostgreSQL object or role.
fn quoted_identifier_is_structurally_safe(identifier: &[u8]) -> bool {
    !identifier.is_empty()
        && identifier.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_'
        })
}

/// Return whether a quoted identifier would collide with table-clause syntax.
///
/// Quoted spellings such as `"primary"` are valid identifiers in PostgreSQL but
/// cannot safely enter the structural column parser because that parser uses the
/// same words to identify table constraints. They therefore fail closed.
fn quoted_identifier_collides_with_table_syntax(identifier: &[u8]) -> bool {
    const TABLE_CONSTRAINT_KEYWORDS: [&[u8]; 7] = [
        b"constraint",
        b"primary",
        b"foreign",
        b"unique",
        b"check",
        b"exclude",
        b"like",
    ];
    TABLE_CONSTRAINT_KEYWORDS
        .iter()
        .any(|keyword| identifier.eq_ignore_ascii_case(keyword))
}

#[cfg(test)]
mod tests {
    use super::{
        INVALID_QUALIFIED_IDENTIFIER, declares_created_role, declares_row_level_security,
        normalize_migration_sql,
    };

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
    fn postgres_escape_strings_respect_backslash_and_doubled_quote_boundaries() {
        let normalized = normalize_migration_sql(
            r"SELECT E'it\'s ''still'' one literal', e'tepp.current_tenant_record_id';",
        )
        .expect("well-formed PostgreSQL escape strings");
        assert!(!normalized.contains("still"));
        assert!(normalized.contains("'tepp.current_tenant_record_id'"));
    }

    #[test]
    fn atomic_literals_keep_boundaries_between_sql_keywords() {
        let normalized = normalize_migration_sql("SELECT 'CREATE'\n'INDEX' AS literal_text;")
            .expect("well-formed adjacent literals");
        assert!(!normalized.contains("CREATE INDEX"));
        assert!(normalized.contains("'CREATE' 'INDEX'"));
    }

    #[test]
    fn quoted_identifiers_preserve_equivalent_spelling_and_reject_case_distinct_content() {
        let normalized = normalize_migration_sql(
            "CREATE INDEX \"good_index\" ON tenant_record (\"good_name\"); CREATE VIEW \"Bad\" AS SELECT 1; CREATE VIEW \"a\"\"b\" AS SELECT 1;",
        )
        .expect("well-formed quoted identifiers");
        assert!(normalized.contains("CREATE INDEX good_index ON tenant_record ( good_name )"));
        assert!(normalized.contains("CREATE VIEW INVALID_QUOTED_IDENTIFIER AS SELECT 1"));
    }

    #[test]
    fn role_creation_aliases_share_the_created_object_name_scanner() {
        for statement in [
            "CREATE ROLE role_name NOSUPERUSER;",
            "CREATE USER user_name NOSUPERUSER;",
            "CREATE GROUP group_name NOSUPERUSER;",
        ] {
            let normalized = normalize_migration_sql(statement).expect("well-formed role declaration");
            assert!(normalized.starts_with("CREATE TYPE "), "{statement}");
        }
        let user_mapping = normalize_migration_sql(
            "CREATE USER MAPPING FOR CURRENT_USER SERVER foreign_server;",
        )
        .expect("well-formed user mapping");
        assert!(user_mapping.starts_with("CREATE USER MAPPING "));
    }

    #[test]
    fn role_declaration_evidence_uses_the_lexical_boundary() {
        assert_eq!(
            declares_created_role("CREATE ROLE tepp_app_runtime NOSUPERUSER;", "tepp_app_runtime"),
            Some(true)
        );
        assert_eq!(
            declares_created_role("CREATE USER \"tepp_app_runtime\" NOSUPERUSER;", "tepp_app_runtime"),
            Some(true)
        );
        assert_eq!(
            declares_created_role(
                "-- CREATE ROLE tepp_app_runtime;\nSELECT 'CREATE ROLE tepp_app_runtime';",
                "tepp_app_runtime"
            ),
            Some(false)
        );
        assert_eq!(
            declares_created_role(
                "CREATE USER MAPPING FOR tepp_app_runtime SERVER foreign_server;",
                "tepp_app_runtime"
            ),
            Some(false)
        );
        assert_eq!(
            declares_created_role(
                "CREATE ROLE tepp_app_runtime; DROP ROLE tepp_app_runtime;",
                "tepp_app_runtime"
            ),
            Some(false)
        );
        assert_eq!(
            declares_created_role(
                "DROP ROLE IF EXISTS tepp_app_runtime; CREATE USER tepp_app_runtime;",
                "tepp_app_runtime"
            ),
            Some(true)
        );
        assert_eq!(
            declares_created_role(
                "CREATE GROUP tepp_app_runtime; DROP GROUP IF EXISTS other_role,tepp_app_runtime;",
                "tepp_app_runtime"
            ),
            Some(false)
        );
        assert_eq!(
            declares_created_role(
                "CREATE ROLE tepp_app_runtime; DROP USER tepp_app_runtime;",
                "tepp_app_runtime"
            ),
            Some(false)
        );
        assert_eq!(
            declares_created_role(
                "CREATE ROLE tepp_app_runtime; DROP USER MAPPING FOR tepp_app_runtime SERVER foreign_server;",
                "tepp_app_runtime"
            ),
            Some(true)
        );
        assert_eq!(
            declares_created_role(
                "CREATE ROLE tepp_app_runtime;DROP ROLE tepp_app_runtime;",
                "tepp_app_runtime"
            ),
            Some(false)
        );
        assert_eq!(
            declares_created_role(
                "CREATE ROLE tepp_app_runtime; ALTER ROLE tepp_app_runtime RENAME TO archived_runtime_role;",
                "tepp_app_runtime"
            ),
            Some(false)
        );
        assert_eq!(
            declares_created_role(
                "CREATE ROLE archived_runtime_role; ALTER USER archived_runtime_role RENAME TO tepp_app_runtime;",
                "tepp_app_runtime"
            ),
            Some(true)
        );
        assert_eq!(
            declares_created_role(
                "CREATE ROLE tepp_app_runtime; ALTER GROUP absent_role RENAME TO other_role;",
                "tepp_app_runtime"
            ),
            Some(true)
        );
    }

    #[test]
    fn role_rename_targets_share_the_created_object_name_scanner() {
        for statement in [
            "ALTER ROLE role_name RENAME TO renamed_role;",
            "ALTER USER user_name RENAME TO renamed_user;",
            "ALTER GROUP group_name RENAME TO renamed_group;",
        ] {
            let normalized = normalize_migration_sql(statement).expect("well-formed role rename");
            assert!(normalized.starts_with("CREATE TYPE renamed_"), "{statement}");
        }
        let user_mapping = normalize_migration_sql(
            "ALTER USER MAPPING FOR CURRENT_USER SERVER foreign_server OPTIONS (SET user 'x');",
        )
        .expect("well-formed user mapping alteration");
        assert!(user_mapping.starts_with("ALTER USER MAPPING "));
    }

    #[test]
    fn rls_detection_runs_on_lexically_normalized_sql() {
        let normalized = normalize_migration_sql(
            "-- ENABLE ROW LEVEL SECURITY\nCREATE POLICY tenant_record_isolation ON tenant_record USING (true);",
        )
        .expect("well-formed RLS SQL");
        assert!(declares_row_level_security(&normalized));
        let trivia = normalize_migration_sql("SELECT 'CREATE POLICY hidden';")
            .expect("well-formed literal");
        assert!(!declares_row_level_security(&trivia));
    }

    #[test]
    fn view_modifiers_share_the_view_object_parser() {
        let normalized = normalize_migration_sql(
            "create materialized view materialized_view AS SELECT 1; CREATE OR REPLACE VIEW replaceable_view AS SELECT 1; CREATE VIEW ordinary_view AS SELECT 1;",
        )
        .expect("well-formed view declarations");
        assert!(normalized.contains("create view materialized_view AS SELECT 1;"));
        assert!(normalized.contains("CREATE VIEW replaceable_view AS SELECT 1;"));
        assert!(normalized.contains("CREATE VIEW ordinary_view AS SELECT 1;"));
        let upper = normalized.to_ascii_uppercase();
        assert!(!upper.contains("MATERIALIZED VIEW"));
        assert!(!upper.contains("OR REPLACE VIEW"));
    }

    #[test]
    fn qualified_created_names_fail_closed_before_prefix_truncation() {
        for sql in [
            "CREATE VIEW audit_schema.Bad AS SELECT 1;",
            "CREATE VIEW \"audit_schema\".\"Bad\" AS SELECT 1;",
            "CREATE OR REPLACE FUNCTION audit_schema.Bad() RETURNS void AS $$ SELECT 1 $$ LANGUAGE sql;",
            "CREATE UNIQUE INDEX IF NOT EXISTS audit_schema.Bad ON tenant_record (tenant_record_id);",
        ] {
            let normalized = normalize_migration_sql(sql).expect("well-formed qualified declaration");
            assert!(normalized.contains(INVALID_QUALIFIED_IDENTIFIER), "{sql}");
        }
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
            "SELECT E'unterminated",
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
