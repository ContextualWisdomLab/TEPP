//! Live SQL transport contracts for `PostgreSQL` adapters.

use crate::PersistenceError;

/// Synchronous SQL transport used by live migration and document adapters.
///
/// Production deployments may back this trait with `SQLx`/`PostgreSQL`. CI and
/// unit tests use deterministic in-process implementations so scientific
/// contracts remain exercisable without a live database.
pub trait SqlSession {
    /// Execute one SQL statement that does not return document rows.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::SqlExecutionFailed`] when the transport
    /// rejects the statement, or domain-mapped variants when the adapter maps
    /// constraint failures.
    fn execute(&mut self, sql: &str) -> Result<(), PersistenceError>;
}

/// Split migration SQL into executable statements without executing them.
///
/// Strips `--` line comments, ignores empty fragments, and fails closed when no
/// statements remain. Statement boundaries are `;` outside single quotes and
/// PostgreSQL dollar-quoted strings (`$tag$ ... $tag$`), so `DO` blocks remain
/// intact.
///
/// # Errors
///
/// Returns [`PersistenceError::EmptySqlBatch`] when the input yields no
/// executable statements.
pub fn split_sql_statements(sql: &str) -> Result<Vec<String>, PersistenceError> {
    let without_line_comments = strip_line_comments(sql);
    let statements = split_on_semicolons_respecting_quotes(&without_line_comments);
    if statements.is_empty() {
        return Err(PersistenceError::EmptySqlBatch);
    }
    Ok(statements)
}

fn split_on_semicolons_respecting_quotes(sql: &str) -> Vec<String> {
    let chars: Vec<char> = sql.chars().collect();
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut index = 0usize;
    let mut in_single_quote = false;
    let mut dollar_tag: Option<String> = None;

    while index < chars.len() {
        if let Some(tag) = dollar_tag.as_ref() {
            let tag_len = tag.chars().count();
            if index + tag_len <= chars.len() {
                let candidate: String = chars[index..index + tag_len].iter().collect();
                if candidate == *tag {
                    current.push_str(&candidate);
                    index += tag_len;
                    dollar_tag = None;
                    continue;
                }
            }
            current.push(chars[index]);
            index += 1;
            continue;
        }

        if in_single_quote {
            current.push(chars[index]);
            if chars[index] == '\'' {
                if index + 1 < chars.len() && chars[index + 1] == '\'' {
                    current.push(chars[index + 1]);
                    index += 2;
                    continue;
                }
                in_single_quote = false;
            }
            index += 1;
            continue;
        }

        match chars[index] {
            '\'' => {
                in_single_quote = true;
                current.push('\'');
                index += 1;
            }
            '$' => match read_dollar_tag(&chars[index..]) {
                Some(tag) => {
                    let tag_len = tag.chars().count();
                    current.push_str(&tag);
                    index += tag_len;
                    dollar_tag = Some(tag);
                }
                None => {
                    current.push('$');
                    index += 1;
                }
            },
            ';' => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    statements.push(trimmed.to_owned());
                }
                current.clear();
                index += 1;
            }
            other => {
                current.push(other);
                index += 1;
            }
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed.to_owned());
    }
    statements
}

fn read_dollar_tag(chars: &[char]) -> Option<String> {
    if chars.first() != Some(&'$') {
        return None;
    }
    let mut end = 1usize;
    while end < chars.len() {
        let ch = chars[end];
        if ch == '$' {
            return Some(chars[..=end].iter().collect());
        }
        if !(ch.is_ascii_alphanumeric() || ch == '_') {
            return None;
        }
        end += 1;
    }
    None
}

/// Apply every statement from `sql` through `session` in order.
///
/// # Errors
///
/// Returns empty-batch or transport failures without continuing after the first
/// error.
pub fn apply_sql_batch<S: SqlSession>(
    session: &mut S,
    sql: &str,
) -> Result<usize, PersistenceError> {
    let statements = split_sql_statements(sql)?;
    for statement in &statements {
        session.execute(statement)?;
    }
    Ok(statements.len())
}

fn strip_line_comments(sql: &str) -> String {
    sql.lines()
        .map(|line| match line.find("--") {
            Some(index) => &line[..index],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Recording transport used by contract tests and offline verification.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordingSqlSession {
    executed: Vec<String>,
    fail_on_substring: Option<String>,
}

impl RecordingSqlSession {
    /// Create an empty recording session.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Fail any statement containing `needle` with
    /// [`PersistenceError::SqlExecutionFailed`].
    #[must_use]
    pub fn failing_on(needle: impl Into<String>) -> Self {
        Self {
            executed: Vec::new(),
            fail_on_substring: Some(needle.into()),
        }
    }

    /// Borrow executed statements in submission order.
    #[must_use]
    pub fn executed(&self) -> &[String] {
        &self.executed
    }
}

impl SqlSession for RecordingSqlSession {
    fn execute(&mut self, sql: &str) -> Result<(), PersistenceError> {
        if let Some(needle) = &self.fail_on_substring
            && sql.contains(needle.as_str())
        {
            return Err(PersistenceError::SqlExecutionFailed);
        }
        self.executed.push(sql.to_owned());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RecordingSqlSession, SqlSession, apply_sql_batch, split_sql_statements, strip_line_comments,
    };
    use crate::PersistenceError;

    #[test]
    fn split_strips_comments_and_rejects_empty_batches() {
        let statements = split_sql_statements(
            "-- header\nCREATE TABLE tenant_record (tenant_record_id uuid);\n-- tail\n",
        )
        .expect("one statement");
        assert_eq!(statements.len(), 1);
        assert!(statements[0].contains("tenant_record"));
        assert!(!statements[0].contains("--"));
        assert_eq!(
            split_sql_statements("   -- only comments\n"),
            Err(PersistenceError::EmptySqlBatch)
        );
        assert_eq!(
            split_sql_statements(";;;"),
            Err(PersistenceError::EmptySqlBatch)
        );
        assert!(strip_line_comments("a -- b\nc").contains('c'));
    }

    #[test]
    fn split_preserves_dollar_quoted_and_single_quoted_semicolons() {
        let statements =
            split_sql_statements("DO $tepp$ BEGIN PERFORM 1; END $tepp$;\nSELECT 'a;b';")
                .expect("two statements");
        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("DO $tepp$"));
        assert!(statements[0].contains("PERFORM 1;"));
        assert!(statements[0].contains("END $tepp$"));
        assert_eq!(statements[1], "SELECT 'a;b'");

        let escaped = split_sql_statements("SELECT 'it''s;ok'; SELECT 2;").expect("escaped quotes");
        assert_eq!(escaped.len(), 2);
        assert!(escaped[0].contains("it''s;ok"));
    }

    #[test]
    fn apply_batch_records_and_stops_on_failure() {
        let mut session = RecordingSqlSession::new();
        let count = apply_sql_batch(&mut session, "SELECT 1; SELECT 2;").expect("batch");
        assert_eq!(count, 2);
        assert_eq!(session.executed().len(), 2);

        let mut failing = RecordingSqlSession::failing_on("boom");
        assert_eq!(
            apply_sql_batch(&mut failing, "SELECT ok; SELECT boom; SELECT later;"),
            Err(PersistenceError::SqlExecutionFailed)
        );
        assert_eq!(failing.executed().len(), 1);

        let mut direct = RecordingSqlSession::new();
        direct.execute("SELECT 1").expect("direct");
        assert_eq!(direct.executed(), &["SELECT 1".to_owned()]);
    }
}
