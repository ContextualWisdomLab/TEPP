"""Contract tests for the database object naming checker (AGENTS.md rule 12).

Rule 12 requires every database object name to contain at least two words and
to use ``snake_case``. The rule was documented without an executable gate, so
these tests pin both the detection power of the checker and the current
conformance of ``migrations/``.
"""

from __future__ import annotations

import contextlib
import io
import runpy
import sys
import tempfile
import unittest
import unittest.mock
from pathlib import Path

from scripts import check_database_naming

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def names(sql: str) -> set[tuple[str, str]]:
    """Return the ``(kind, name)`` pairs the checker extracts from ``sql``."""

    return {(kind, name) for _, kind, name in check_database_naming.declared_names(sql)}


class NameExtractionTests(unittest.TestCase):
    """Declarations, constraints, and columns all enter the contract."""

    def test_table_columns_and_constraints_are_extracted(self) -> None:
        sql = (
            "CREATE TABLE tenant_record (\n"
            "    tenant_record_id uuid PRIMARY KEY,\n"
            "    tenant_status_code text NOT NULL,\n"
            "    PRIMARY KEY (tenant_record_id),\n"
            "    CONSTRAINT tenant_record_unique UNIQUE (tenant_record_id)\n"
            ");\n"
        )
        self.assertEqual(
            names(sql),
            {
                ("table", "tenant_record"),
                ("column", "tenant_record_id"),
                ("column", "tenant_status_code"),
                ("constraint", "tenant_record_unique"),
            },
        )

    def test_every_declaration_kind_is_extracted(self) -> None:
        sql = (
            "CREATE UNIQUE INDEX IF NOT EXISTS event_instance_time_index ON event_instance (a);\n"
            "CREATE OR REPLACE FUNCTION reject_append_only_mutation() RETURNS trigger;\n"
            "CREATE TRIGGER audit_event_reject_mutation BEFORE UPDATE ON audit_event;\n"
            "CREATE TYPE membership_kind_code AS ENUM ('a');\n"
            "CREATE VIEW event_mention_view AS SELECT 1;\n"
            "CREATE SEQUENCE revision_number_sequence;\n"
            "CREATE POLICY tenant_isolation_policy ON audit_event;\n"
        )
        self.assertEqual(
            names(sql),
            {
                ("index", "event_instance_time_index"),
                ("function", "reject_append_only_mutation"),
                ("trigger", "audit_event_reject_mutation"),
                ("type", "membership_kind_code"),
                ("view", "event_mention_view"),
                ("sequence", "revision_number_sequence"),
                ("policy", "tenant_isolation_policy"),
            },
        )

    def test_conditional_constraint_drops_name_the_constraint(self) -> None:
        sql = (
            "ALTER TABLE event_instance\n"
            "    DROP CONSTRAINT IF EXISTS event_instance_valid_order;\n"
        )
        self.assertEqual(names(sql), {("constraint", "event_instance_valid_order")})

    def test_nested_parentheses_and_empty_segments_do_not_shift_columns(self) -> None:
        sql = "CREATE TABLE model_run (\n    run_cost numeric(12, 4) NOT NULL,\n    ,\n);\n"
        self.assertEqual(names(sql), {("table", "model_run"), ("column", "run_cost")})

    def test_unbalanced_table_body_is_read_to_end_of_file(self) -> None:
        sql = "CREATE TABLE model_run (\n    run_started_at timestamptz\n"
        self.assertEqual(names(sql), {("table", "model_run"), ("column", "run_started_at")})

    def test_reported_line_numbers_point_at_the_offending_name(self) -> None:
        sql = "CREATE TABLE audit_event (\n    id uuid,\n    CONSTRAINT pk PRIMARY KEY (id)\n);\n"
        self.assertEqual(
            check_database_naming.violations(sql, Path("migrations/x.sql")),
            [
                "migrations/x.sql:2: column name 'id' is not multi-word snake_case",
                "migrations/x.sql:3: constraint name 'pk' is not multi-word snake_case",
            ],
        )


class ContractNameTests(unittest.TestCase):
    """Only lowercase names of two or more underscore-joined words conform."""

    def test_accepted_names(self) -> None:
        for name in ("tenant_record", "content_sha256", "event_instance_id"):
            with self.subTest(name=name):
                self.assertTrue(check_database_naming.is_contract_name(name))

    def test_rejected_names(self) -> None:
        for name in ("tenant", "TenantRecord", "tenant_Record", "_tenant_record", "tenant__record"):
            with self.subTest(name=name):
                self.assertFalse(check_database_naming.is_contract_name(name))


class RepositoryConformanceTests(unittest.TestCase):
    """The shipped migrations satisfy the contract and the gate exits zero."""

    def test_migrations_declare_names_and_have_no_violations(self) -> None:
        self.assertGreaterEqual(len(check_database_naming.migration_files(REPOSITORY_ROOT)), 14)
        self.assertEqual(check_database_naming.check_migrations(REPOSITORY_ROOT), [])

    def test_main_reports_success_for_the_repository(self) -> None:
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            exit_code = check_database_naming.main([str(REPOSITORY_ROOT)])
        self.assertEqual(exit_code, 0)
        self.assertIn("satisfied", stdout.getvalue())

    def test_main_reports_each_violation_and_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "migrations").mkdir()
            (root / "migrations" / "0001_bad.up.sql").write_text(
                "CREATE TABLE Tenant (\n    id uuid PRIMARY KEY\n);\n", encoding="utf-8"
            )
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = check_database_naming.main([str(root)])
        self.assertEqual(exit_code, 1)
        self.assertEqual(
            stdout.getvalue().splitlines(),
            [
                "migrations/0001_bad.up.sql:1: table name 'Tenant' is not multi-word snake_case",
                "migrations/0001_bad.up.sql:2: column name 'id' is not multi-word snake_case",
            ],
        )

    def test_repository_root_defaults_to_the_working_directory(self) -> None:
        stdout = io.StringIO()
        with contextlib.chdir(REPOSITORY_ROOT), contextlib.redirect_stdout(stdout):
            self.assertEqual(check_database_naming.main([]), 0)

    def test_module_entrypoint_exits_with_the_gate_status(self) -> None:
        loaded = sys.modules.pop("scripts.check_database_naming")
        stdout = io.StringIO()
        try:
            with unittest.mock.patch(
                "sys.argv", ["check_database_naming.py", str(REPOSITORY_ROOT)]
            ):
                with contextlib.redirect_stdout(stdout):
                    with self.assertRaises(SystemExit) as raised:
                        runpy.run_module("scripts.check_database_naming", run_name="__main__")
        finally:
            sys.modules["scripts.check_database_naming"] = loaded
        self.assertEqual(raised.exception.code, 0)

if __name__ == "__main__":
    unittest.main()
