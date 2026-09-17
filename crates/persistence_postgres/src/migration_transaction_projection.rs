//! Committed-statement projection for PostgreSQL migration safety evidence.
//!
//! The shared lexical authority has already masked comments, strings, quoted
//! bodies, and quoted semicolons before this boundary runs. This module therefore
//! owns only top-level transaction outcome: statements in a committed explicit
//! transaction survive, statements in a rolled-back transaction disappear, and
//! ambiguous savepoint/two-phase shapes fail closed instead of donating safety
//! evidence to downstream runtime-role validators.

/// Project normalized SQL onto statements whose effects survive transaction outcome.
///
/// Statements outside an explicit transaction model PostgreSQL autocommit and
/// are retained immediately. `BEGIN` / `START TRANSACTION` opens a buffer;
/// `COMMIT` / `END` flush it and `ROLLBACK` / `ABORT` discards it. `AND CHAIN`
/// begins a fresh transaction after the boundary. Savepoints and prepared
/// transactions require a state stack or external prepared-state proof that this
/// bounded validator does not own, so those shapes return `None`. An unterminated
/// explicit transaction also returns `None` because durability is unresolved.
#[must_use]
pub(super) fn project_committed_statements(sql: &str) -> Option<String> {
    let tokenized = sql.replace(';', " ; ");
    let tokens = tokenized.split_whitespace().collect::<Vec<_>>();
    let mut committed = Vec::<String>::new();
    let mut pending = Vec::<String>::new();
    let mut in_transaction = false;
    let mut index = 0usize;

    while index < tokens.len() {
        let end = statement_end(&tokens, index);
        let statement = &tokens[index..end];
        index = end.saturating_add(1);

        if statement.is_empty() {
            continue;
        }
        if is_unsupported_transaction_control(statement) {
            return None;
        }
        if is_transaction_start(statement) {
            if !in_transaction {
                in_transaction = true;
                pending.clear();
            }
            continue;
        }
        if let Some((outcome, and_chain)) = transaction_end(statement) {
            if in_transaction {
                if matches!(outcome, TransactionOutcome::Commit) {
                    committed.append(&mut pending);
                } else {
                    pending.clear();
                }
                in_transaction = false;
            } else if and_chain {
                return None;
            }
            if and_chain {
                in_transaction = true;
            }
            continue;
        }

        let rendered = render_statement(statement);
        if in_transaction {
            pending.push(rendered);
        } else {
            committed.push(rendered);
        }
    }

    if in_transaction {
        return None;
    }
    Some(committed.join(" "))
}

/// Find the end of one already-normalized semicolon-delimited statement.
fn statement_end(tokens: &[&str], start: usize) -> usize {
    tokens[start..]
        .iter()
        .position(|token| *token == ";")
        .map_or(tokens.len(), |relative| start + relative)
}

/// Render one retained normalized statement with an explicit delimiter.
fn render_statement(statement: &[&str]) -> String {
    format!("{} ;", statement.join(" "))
}

/// Return whether a statement starts an explicit PostgreSQL transaction block.
fn is_transaction_start(statement: &[&str]) -> bool {
    statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("BEGIN"))
        || (statement
            .first()
            .is_some_and(|token| token.eq_ignore_ascii_case("START"))
            && statement
                .get(1)
                .is_some_and(|token| token.eq_ignore_ascii_case("TRANSACTION")))
}

/// Transaction end whose mutations either survive or are discarded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TransactionOutcome {
    Commit,
    Rollback,
}

/// Parse a transaction end and whether it immediately chains a new transaction.
fn transaction_end(statement: &[&str]) -> Option<(TransactionOutcome, bool)> {
    let first = statement.first()?;
    let outcome = if first.eq_ignore_ascii_case("COMMIT") || first.eq_ignore_ascii_case("END") {
        TransactionOutcome::Commit
    } else if first.eq_ignore_ascii_case("ROLLBACK") || first.eq_ignore_ascii_case("ABORT") {
        TransactionOutcome::Rollback
    } else {
        return None;
    };

    let and_chain = statement.windows(2).any(|pair| {
        pair[0].eq_ignore_ascii_case("AND") && pair[1].eq_ignore_ascii_case("CHAIN")
    });
    Some((outcome, and_chain))
}

/// Reject transaction controls whose durable outcome cannot be proven locally.
fn is_unsupported_transaction_control(statement: &[&str]) -> bool {
    if statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("SAVEPOINT"))
        || statement
            .first()
            .is_some_and(|token| token.eq_ignore_ascii_case("RELEASE"))
        || (statement
            .first()
            .is_some_and(|token| token.eq_ignore_ascii_case("ROLLBACK"))
            && statement
                .get(1)
                .is_some_and(|token| token.eq_ignore_ascii_case("TO")))
    {
        return true;
    }

    (statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("PREPARE"))
        && statement
            .get(1)
            .is_some_and(|token| token.eq_ignore_ascii_case("TRANSACTION")))
        || ((statement.first().is_some_and(|token| {
            token.eq_ignore_ascii_case("COMMIT") || token.eq_ignore_ascii_case("ROLLBACK")
        })) && statement
            .get(1)
            .is_some_and(|token| token.eq_ignore_ascii_case("PREPARED")))
}

#[cfg(test)]
mod tests {
    use super::project_committed_statements;

    #[test]
    fn rollback_discards_transaction_statements() {
        let projected = project_committed_statements(
            "GRANT role_a TO member_a; BEGIN; REVOKE role_a FROM member_a; ROLLBACK; GRANT role_b TO member_b;",
        )
        .expect("simple transaction outcome must project");

        assert!(projected.contains("GRANT role_a TO member_a ;"));
        assert!(!projected.contains("REVOKE role_a FROM member_a"));
        assert!(projected.contains("GRANT role_b TO member_b ;"));
    }

    #[test]
    fn commit_retains_transaction_statements() {
        let projected = project_committed_statements(
            "BEGIN; REVOKE role_a FROM member_a; COMMIT;",
        )
        .expect("committed transaction must project");
        assert!(projected.contains("REVOKE role_a FROM member_a ;"));
    }

    #[test]
    fn commit_and_chain_opens_a_fresh_transaction() {
        let projected = project_committed_statements(
            "BEGIN; GRANT role_a TO member_a; COMMIT AND CHAIN; REVOKE role_a FROM member_a; ROLLBACK;",
        )
        .expect("chained transaction must project");
        assert!(projected.contains("GRANT role_a TO member_a ;"));
        assert!(!projected.contains("REVOKE role_a FROM member_a"));
    }

    #[test]
    fn and_no_chain_does_not_open_a_new_transaction() {
        let projected = project_committed_statements(
            "BEGIN; GRANT role_a TO member_a; COMMIT AND NO CHAIN; REVOKE role_a FROM member_a;",
        )
        .expect("NO CHAIN must return to autocommit");
        assert!(projected.contains("GRANT role_a TO member_a ;"));
        assert!(projected.contains("REVOKE role_a FROM member_a ;"));
    }

    #[test]
    fn savepoints_prepared_transactions_and_unfinished_blocks_fail_closed() {
        for sql in [
            "BEGIN; SAVEPOINT safety; COMMIT;",
            "PREPARE TRANSACTION 'tx';",
            "COMMIT PREPARED 'tx';",
            "BEGIN; GRANT role_a TO member_a;",
        ] {
            assert_eq!(project_committed_statements(sql), None);
        }
    }
}
