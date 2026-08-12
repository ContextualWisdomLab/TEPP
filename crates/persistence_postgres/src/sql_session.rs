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
/// statements remain. Statement boundaries are plain `;` separators; TEPP
/// foundation migrations do not embed quoted semicolons.
///
/// # Errors
///
/// Returns [`PersistenceError::EmptySqlBatch`] when the input yields no
/// executable statements.
pub fn split_sql_statements(sql: &str) -> Result<Vec<String>, PersistenceError> {
    let without_line_comments = strip_line_comments(sql);
    let mut statements = Vec::new();
    for fragment in without_line_comments.split(';') {
        let trimmed = fragment.trim();
        if !trimmed.is_empty() {
            statements.push(trimmed.to_owned());
        }
    }
    if statements.is_empty() {
        return Err(PersistenceError::EmptySqlBatch);
    }
    Ok(statements)
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
