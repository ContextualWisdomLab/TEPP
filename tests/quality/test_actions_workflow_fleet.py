"""Fail-closed contracts for orphaned GitHub Actions workflow fleet audit."""

from __future__ import annotations

import io
import json
import os
import unittest
import urllib.error
from typing import Any
from unittest import mock

from scripts import actions_workflow_fleet as fleet


class FakeTransport:
    """Deterministic GitHub HTTP stub keyed by method and path."""

    def __init__(self, responses: dict[tuple[str, str], fleet.HttpResponse]) -> None:
        """Store exact method/path responses for later lookup."""

        self.responses = responses
        self.calls: list[tuple[str, str]] = []

    def request(self, method: str, path: str) -> fleet.HttpResponse:
        """Return the stubbed response or raise KeyError for unexpected calls."""

        self.calls.append((method, path))
        return self.responses[(method, path)]


def _json_response(
    payload: Any,
    *,
    status: int = 200,
    headers: dict[str, str] | None = None,
) -> fleet.HttpResponse:
    """Build a JSON HTTP response used by the fake transport."""

    return fleet.HttpResponse(
        status=status,
        headers=headers or {},
        body=json.dumps(payload).encode("utf-8"),
    )


def _workflow(
    workflow_id: int,
    path: str,
    *,
    name: str | None = None,
    state: str = "active",
) -> dict[str, Any]:
    """Return one GitHub workflow identity mapping."""

    return {
        "id": workflow_id,
        "name": name or path,
        "path": path,
        "state": state,
    }


class ActionsWorkflowFleetTests(unittest.TestCase):
    """Exercise classification, pagination, fail-closed errors, and disable CAS."""

    def test_normalize_decodes_percent_encoding_without_changing_case(self) -> None:
        """Encoded workflow paths compare to the decoded tree path; case stays exact."""

        encoded = ".github/workflows/%62ootstrap-materialize.yml"
        self.assertEqual(
            fleet.normalize_workflow_path(encoded),
            ".github/workflows/bootstrap-materialize.yml",
        )
        self.assertEqual(
            fleet.normalize_workflow_path(".github/workflows/CI.YML"),
            ".github/workflows/CI.YML",
        )

    def test_classify_present_disabled_orphan_and_dynamic(self) -> None:
        """Classification binds to the protected tree, live state, and GitHub dynamic paths."""

        tree = {".github/workflows/ci.yml", ".github/workflows/docs-quality.yml"}
        present = fleet.classify_workflow(
            _workflow(1, ".github/workflows/ci.yml"), tree_paths=tree
        )
        orphan = fleet.classify_workflow(
            _workflow(2, ".github/workflows/bootstrap-materialize.yml"),
            tree_paths=tree,
        )
        disabled = fleet.classify_workflow(
            _workflow(3, ".github/workflows/old.yml", state="disabled"),
            tree_paths=tree,
        )
        dynamic = fleet.classify_workflow(
            _workflow(4, "dynamic/github-code-scanning/codeql", name="CodeQL"),
            tree_paths=tree,
        )
        self.assertEqual(present, "present")
        self.assertEqual(orphan, "orphan")
        self.assertEqual(disabled, "disabled")
        self.assertEqual(dynamic, "github_dynamic")

    def test_classify_github_disabled_state_family(self) -> None:
        """Official GitHub disabled_* and deleted states are not actionable orphans."""

        tree: set[str] = set()
        for state in (
            "disabled_manually",
            "disabled_fork",
            "disabled_inactivity",
            "deleted",
        ):
            classification = fleet.classify_workflow(
                _workflow(8, ".github/workflows/old.yml", state=state),
                tree_paths=tree,
            )
            self.assertEqual(classification, "disabled", state)

    def test_name_containing_bootstrap_is_present_when_path_is_on_main(self) -> None:
        """Name-only bootstrap/repair heuristics must not reclassify a live main workflow."""

        tree = {".github/workflows/hourly-pr-maintenance.yml"}
        classification = fleet.classify_workflow(
            _workflow(
                5,
                ".github/workflows/hourly-pr-maintenance.yml",
                name="Materialize TEPP Bootstrap",
            ),
            tree_paths=tree,
        )
        self.assertEqual(classification, "present")

    def test_case_mismatch_is_not_the_same_tree_path(self) -> None:
        """GitHub tree paths are case-sensitive; CI.yml is not ci.yml."""

        tree = {".github/workflows/ci.yml"}
        classification = fleet.classify_workflow(
            _workflow(6, ".github/workflows/CI.yml"),
            tree_paths=tree,
        )
        self.assertEqual(classification, "orphan")

    def test_encoded_path_matching_tree_is_present(self) -> None:
        """Percent-encoded paths that decode to a protected-main file are present."""

        tree = {".github/workflows/ci.yml"}
        classification = fleet.classify_workflow(
            _workflow(7, ".github/workflows/%63i.yml"),
            tree_paths=tree,
        )
        self.assertEqual(classification, "present")

    def test_paginate_workflows_follows_link_and_checks_total(self) -> None:
        """Pagination receipts bind collected count to GitHub's reported total_count."""

        transport = FakeTransport(
            {
                (
                    "GET",
                    "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100",
                ): _json_response(
                    {
                        "total_count": 2,
                        "workflows": [_workflow(1, ".github/workflows/ci.yml")],
                    },
                    headers={
                        "Link": (
                            "<https://api.github.com/repos/ContextualWisdomLab/"
                            "TEPP/actions/workflows?per_page=100&page=2>; rel=\"next\""
                        )
                    },
                ),
                (
                    "GET",
                    "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100&page=2",
                ): _json_response(
                    {
                        "total_count": 2,
                        "workflows": [
                            _workflow(2, "dynamic/github-code-scanning/codeql")
                        ],
                    }
                ),
            }
        )
        workflows, receipt = fleet.paginate_workflows(
            transport, "ContextualWisdomLab", "TEPP"
        )
        self.assertEqual(len(workflows), 2)
        self.assertEqual(receipt.page_count, 2)
        self.assertEqual(receipt.collected_count, 2)
        self.assertEqual(receipt.reported_total_count, 2)

    def test_pagination_truncation_without_next_link_fails_closed(self) -> None:
        """A short page with no next link is truncation, not an empty remainder."""

        transport = FakeTransport(
            {
                (
                    "GET",
                    "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100",
                ): _json_response(
                    {
                        "total_count": 14,
                        "workflows": [_workflow(1, ".github/workflows/ci.yml")],
                    }
                )
            }
        )
        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.paginate_workflows(transport, "ContextualWisdomLab", "TEPP")
        self.assertEqual(raised.exception.reason, "pagination_truncated")

    def test_permission_not_found_and_upstream_errors_are_distinct(self) -> None:
        """403, 404, and 5xx stay distinct so operators cannot treat them as empty."""

        def expect(status: int, reason: str) -> None:
            transport = FakeTransport(
                {
                    (
                        "GET",
                        "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100",
                    ): fleet.HttpResponse(
                        status=status, headers={}, body=b'{"message":"no"}'
                    )
                }
            )
            with self.assertRaises(fleet.FleetAuditError) as raised:
                fleet.paginate_workflows(transport, "ContextualWisdomLab", "TEPP")
            self.assertEqual(raised.exception.reason, reason)

        expect(403, "permission_loss")
        expect(404, "not_found")
        expect(500, "upstream_unavailable")
        expect(502, "upstream_unavailable")

    def test_audit_binds_default_branch_sha_and_classifies_live_inventory(self) -> None:
        """A complete audit records SHA, pagination, timestamp, and every identity."""

        transport = FakeTransport(self._happy_responses())
        audit = fleet.audit_repository(
            transport,
            "ContextualWisdomLab",
            "TEPP",
            now="2026-08-13T03:02:43Z",
        )
        self.assertEqual(audit.branch_binding.default_branch, "main")
        self.assertEqual(audit.branch_binding.sha, "abc123def456")
        self.assertEqual(audit.pagination.collected_count, 7)
        self.assertEqual(audit.timestamp, "2026-08-13T03:02:43Z")
        by_id = {item.workflow_id: item for item in audit.identities}
        self.assertEqual(by_id[1].classification, "present")
        self.assertEqual(by_id[10].classification, "orphan")
        self.assertEqual(by_id[11].classification, "orphan")
        self.assertEqual(by_id[20].classification, "disabled")
        self.assertEqual(by_id[30].classification, "github_dynamic")
        self.assertEqual(
            [item.workflow_id for item in audit.orphan_identities()],
            [10, 11],
        )

    def test_truncated_git_tree_fails_closed(self) -> None:
        """A truncated recursive tree is not a complete protected-main binding."""

        responses = self._happy_responses()
        responses[
            ("GET", "/repos/ContextualWisdomLab/TEPP/git/trees/abc123def456?recursive=1")
        ] = _json_response({"sha": "abc123def456", "tree": [], "truncated": True})
        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.audit_repository(
                FakeTransport(responses), "ContextualWisdomLab", "TEPP"
            )
        self.assertEqual(raised.exception.reason, "tree_truncated")

    def test_disable_refuses_when_default_branch_moved(self) -> None:
        """Branch movement between audit and mutation is stale-head refusal."""

        audit_transport = FakeTransport(self._happy_responses())
        audit = fleet.audit_repository(
            audit_transport, "ContextualWisdomLab", "TEPP"
        )
        moved = self._happy_responses()
        moved[("GET", "/repos/ContextualWisdomLab/TEPP")] = _json_response(
            {"default_branch": "main"}
        )
        moved[("GET", "/repos/ContextualWisdomLab/TEPP/git/ref/heads/main")] = (
            _json_response({"object": {"sha": "fff999moved", "type": "commit"}})
        )
        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.disable_orphans(
                FakeTransport(moved), audit, apply_changes=True
            )
        self.assertEqual(raised.exception.reason, "branch_moved")

    def test_disable_refuses_workflow_id_reuse_and_path_change(self) -> None:
        """Re-fetch must prove the same path still belongs to the same workflow id."""

        audit = fleet.audit_repository(
            FakeTransport(self._happy_responses()), "ContextualWisdomLab", "TEPP"
        )
        reused = self._happy_responses()
        reused[("GET", "/repos/ContextualWisdomLab/TEPP/actions/workflows/10")] = (
            _json_response(
                _workflow(10, ".github/workflows/ci.yml", name="stolen-id")
            )
        )
        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.disable_orphans(
                FakeTransport(reused), audit, apply_changes=True
            )
        self.assertEqual(raised.exception.reason, "workflow_identity_changed")

    def test_disable_refuses_when_path_reappears_on_protected_main(self) -> None:
        """A workflow that returned to the tree is no longer an orphan."""

        audit = fleet.audit_repository(
            FakeTransport(self._happy_responses()), "ContextualWisdomLab", "TEPP"
        )
        restored = self._happy_responses()
        restored[
            ("GET", "/repos/ContextualWisdomLab/TEPP/git/trees/abc123def456?recursive=1")
        ] = _json_response(
            {
                "truncated": False,
                "tree": [
                    {"path": ".github/workflows/ci.yml", "type": "blob"},
                    {
                        "path": ".github/workflows/bootstrap-materialize.yml",
                        "type": "blob",
                    },
                    {
                        "path": ".github/workflows/diagnose-pr5-coverage-file.yml",
                        "type": "blob",
                    },
                ],
            }
        )
        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.disable_orphans(
                FakeTransport(restored), audit, apply_changes=True
            )
        self.assertEqual(raised.exception.reason, "no_longer_orphan")

    def test_disable_refuses_protected_paths_even_if_tree_is_wrong(self) -> None:
        """Exact protected production paths are never disabled."""

        identity = fleet.WorkflowIdentity(
            workflow_id=99,
            name="Rust Foundation CI",
            path=".github/workflows/ci.yml",
            state="active",
            classification="orphan",
        )
        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.refuse_protected_disable(identity)
        self.assertEqual(raised.exception.reason, "protected_workflow")

    def test_disable_applies_only_after_refetch_and_records_evidence(self) -> None:
        """Successful disable re-fetches each orphan, PUTs disable, and confirms state."""

        audit = fleet.audit_repository(
            FakeTransport(self._happy_responses()), "ContextualWisdomLab", "TEPP"
        )
        apply_responses = self._happy_responses()
        apply_responses[
            ("PUT", "/repos/ContextualWisdomLab/TEPP/actions/workflows/10/disable")
        ] = fleet.HttpResponse(status=204, headers={}, body=b"")
        apply_responses[
            ("PUT", "/repos/ContextualWisdomLab/TEPP/actions/workflows/11/disable")
        ] = fleet.HttpResponse(status=204, headers={}, body=b"")
        apply_responses[
            ("GET", "/repos/ContextualWisdomLab/TEPP/actions/workflows/10")
        ] = _json_response(
            _workflow(
                10,
                ".github/workflows/bootstrap-materialize.yml",
                state="disabled",
            )
        )
        apply_responses[
            ("GET", "/repos/ContextualWisdomLab/TEPP/actions/workflows/11")
        ] = _json_response(
            _workflow(
                11,
                ".github/workflows/diagnose-pr5-coverage-file.yml",
                state="disabled",
            )
        )
        # Pre-disable refetch still shows active; post-disable GET is the second GET.
        # The implementation GETs once before PUT (active) then once after (disabled).
        pre_after = _AlternatingWorkflowTransport(apply_responses)
        outcomes = fleet.disable_orphans(pre_after, audit, apply_changes=True)
        self.assertEqual([item.workflow_id for item in outcomes], [10, 11])
        self.assertTrue(all(item.disabled for item in outcomes))
        self.assertIn(
            ("PUT", "/repos/ContextualWisdomLab/TEPP/actions/workflows/10/disable"),
            pre_after.calls,
        )

    def test_dry_run_does_not_put_disable(self) -> None:
        """Audit-only disable planning never mutates the Actions registry."""

        audit = fleet.audit_repository(
            FakeTransport(self._happy_responses()), "ContextualWisdomLab", "TEPP"
        )
        transport = FakeTransport(self._happy_responses())
        outcomes = fleet.disable_orphans(transport, audit, apply_changes=False)
        self.assertEqual(len(outcomes), 2)
        self.assertTrue(all(not item.disabled for item in outcomes))
        self.assertFalse(any(method == "PUT" for method, _path in transport.calls))

    def test_missing_token_fails_closed(self) -> None:
        """No TEPP-specific PAT is invented when the standard token is absent."""

        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.require_github_token({})
        self.assertEqual(raised.exception.reason, "missing_token")

    def test_accepts_standard_github_or_gh_token(self) -> None:
        """GITHUB_TOKEN or GH_TOKEN is the only accepted credential source."""

        self.assertEqual(fleet.require_github_token({"GITHUB_TOKEN": "a"}), "a")
        self.assertEqual(fleet.require_github_token({"GH_TOKEN": "b"}), "b")

    def test_link_next_ignores_unrelated_relations(self) -> None:
        """Only rel=next is followed; last/prev links are not treated as continuation."""

        header = (
            "<https://api.github.com/x?page=1>; rel=\"prev\", "
            "<https://api.github.com/x?page=3>; rel=\"last\""
        )
        self.assertIsNone(fleet.next_link_path(header))
        next_header = (
            "<https://api.github.com/repos/o/r/actions/workflows?per_page=100&page=2>; "
            'rel="next"'
        )
        self.assertEqual(
            fleet.next_link_path(next_header),
            "/repos/o/r/actions/workflows?per_page=100&page=2",
        )

    def test_invalid_json_and_missing_workflow_fields_fail_closed(self) -> None:
        """Malformed registry payloads are not coerced into an empty inventory."""

        transport = FakeTransport(
            {
                (
                    "GET",
                    "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100",
                ): fleet.HttpResponse(status=200, headers={}, body=b"not-json")
            }
        )
        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.paginate_workflows(transport, "ContextualWisdomLab", "TEPP")
        self.assertEqual(raised.exception.reason, "invalid_payload")

        bad_fields = FakeTransport(
            {
                (
                    "GET",
                    "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100",
                ): _json_response({"total_count": 1, "workflows": [{"id": 1}]})
            }
        )
        with self.assertRaises(fleet.FleetAuditError) as raised_fields:
            fleet.paginate_workflows(bad_fields, "ContextualWisdomLab", "TEPP")
        self.assertEqual(raised_fields.exception.reason, "invalid_payload")

    def test_cli_audit_and_disable_and_unknown_command(self) -> None:
        """The CLI drives the shipped audit/disable functions and rejects unknown verbs."""

        audit = fleet.audit_repository(
            FakeTransport(self._happy_responses()), "ContextualWisdomLab", "TEPP"
        )
        stdout = io.StringIO()
        with mock.patch.object(fleet, "build_transport") as builder:
            builder.return_value = FakeTransport(self._happy_responses())
            with mock.patch.object(fleet, "audit_repository", return_value=audit):
                code = fleet.main(
                    [
                        "audit",
                        "--owner",
                        "ContextualWisdomLab",
                        "--repo",
                        "TEPP",
                    ],
                    environ={"GITHUB_TOKEN": "token"},
                    stdout=stdout,
                )
        self.assertEqual(code, 0)
        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["orphan_count"], 2)
        self.assertEqual(payload["default_branch_sha"], "abc123def456")

        disable_out = io.StringIO()
        with mock.patch.object(fleet, "build_transport") as builder:
            builder.return_value = FakeTransport(self._happy_responses())
            with mock.patch.object(fleet, "audit_repository", return_value=audit):
                with mock.patch.object(
                    fleet,
                    "disable_orphans",
                    return_value=[
                        fleet.DisableOutcome(
                            workflow_id=10,
                            path=".github/workflows/bootstrap-materialize.yml",
                            disabled=False,
                        )
                    ],
                ) as disabler:
                    code = fleet.main(
                        [
                            "disable-orphans",
                            "--owner",
                            "ContextualWisdomLab",
                            "--repo",
                            "TEPP",
                        ],
                        environ={"GITHUB_TOKEN": "token"},
                        stdout=disable_out,
                    )
        self.assertEqual(code, 0)
        disabler.assert_called_once()
        self.assertFalse(disabler.call_args.kwargs["apply_changes"])

        apply_out = io.StringIO()
        with mock.patch.object(fleet, "build_transport") as builder:
            builder.return_value = FakeTransport(self._happy_responses())
            with mock.patch.object(fleet, "audit_repository", return_value=audit):
                with mock.patch.object(
                    fleet,
                    "disable_orphans",
                    return_value=[],
                ) as applier:
                    fleet.main(
                        [
                            "disable-orphans",
                            "--owner",
                            "ContextualWisdomLab",
                            "--repo",
                            "TEPP",
                            "--apply",
                        ],
                        environ={"GITHUB_TOKEN": "token"},
                        stdout=apply_out,
                    )
        self.assertTrue(applier.call_args.kwargs["apply_changes"])

        with self.assertRaises(SystemExit):
            fleet.main(["wat"], environ={"GITHUB_TOKEN": "token"}, stdout=io.StringIO())

    def test_cli_reads_process_environment_when_environ_omitted(self) -> None:
        """Invoking the CLI without an environ argument uses the real process environment."""

        audit = fleet.audit_repository(
            FakeTransport(self._happy_responses()), "ContextualWisdomLab", "TEPP"
        )
        stdout = io.StringIO()
        with mock.patch.dict(os.environ, {"GITHUB_TOKEN": "from-process"}, clear=False):
            with mock.patch.object(fleet, "build_transport") as builder:
                builder.return_value = FakeTransport(self._happy_responses())
                with mock.patch.object(fleet, "audit_repository", return_value=audit):
                    code = fleet.main(
                        ["audit", "--owner", "ContextualWisdomLab", "--repo", "TEPP"],
                        stdout=stdout,
                    )
        self.assertEqual(code, 0)
        passed_env = builder.call_args.args[0]
        self.assertEqual(passed_env.get("GITHUB_TOKEN"), "from-process")

    def test_cli_missing_token_returns_fail_closed_exit(self) -> None:
        """The process exits non-zero when no standard GitHub token is present."""

        stderr = io.StringIO()
        code = fleet.main(
            ["audit", "--owner", "o", "--repo", "r"],
            environ={},
            stdout=io.StringIO(),
            stderr=stderr,
        )
        self.assertEqual(code, 2)
        self.assertIn("missing_token", stderr.getvalue())

    def test_normalize_strips_leading_dot_slash(self) -> None:
        """A leading ``./`` is stripped so tree and registry paths compare."""

        self.assertEqual(
            fleet.normalize_workflow_path("./.github/workflows/ci.yml"),
            ".github/workflows/ci.yml",
        )

    def test_header_miss_and_empty_link_are_none(self) -> None:
        """Missing Link headers do not invent a next page."""

        self.assertIsNone(fleet.next_link_path(None))
        self.assertIsNone(fleet.next_link_path(""))
        self.assertIsNone(fleet._header({"X-Other": "1"}, "Link"))

    def test_other_client_errors_are_invalid_payload(self) -> None:
        """A 409 is not treated as permission loss, not-found, or empty."""

        transport = FakeTransport(
            {
                (
                    "GET",
                    "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100",
                ): fleet.HttpResponse(status=409, headers={}, body=b"{}")
            }
        )
        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.paginate_workflows(transport, "ContextualWisdomLab", "TEPP")
        self.assertEqual(raised.exception.reason, "invalid_payload")

    def test_non_object_json_and_non_integer_total_fail_closed(self) -> None:
        """Arrays, booleans-as-ints, and missing workflow lists fail closed."""

        array_body = FakeTransport(
            {
                (
                    "GET",
                    "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100",
                ): fleet.HttpResponse(status=200, headers={}, body=b"[]")
            }
        )
        with self.assertRaises(fleet.FleetAuditError) as array_error:
            fleet.paginate_workflows(array_body, "ContextualWisdomLab", "TEPP")
        self.assertEqual(array_error.exception.reason, "invalid_payload")

        bool_total = FakeTransport(
            {
                (
                    "GET",
                    "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100",
                ): _json_response({"total_count": True, "workflows": []})
            }
        )
        with self.assertRaises(fleet.FleetAuditError) as bool_error:
            fleet.paginate_workflows(bool_total, "ContextualWisdomLab", "TEPP")
        self.assertEqual(bool_error.exception.reason, "invalid_payload")

        missing_list = FakeTransport(
            {
                (
                    "GET",
                    "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100",
                ): _json_response({"total_count": 0})
            }
        )
        with self.assertRaises(fleet.FleetAuditError) as list_error:
            fleet.paginate_workflows(missing_list, "ContextualWisdomLab", "TEPP")
        self.assertEqual(list_error.exception.reason, "invalid_payload")

        not_object = FakeTransport(
            {
                (
                    "GET",
                    "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100",
                ): _json_response({"total_count": 1, "workflows": ["nope"]})
            }
        )
        with self.assertRaises(fleet.FleetAuditError) as item_error:
            fleet.paginate_workflows(not_object, "ContextualWisdomLab", "TEPP")
        self.assertEqual(item_error.exception.reason, "invalid_payload")

    def test_workflow_name_defaults_to_path(self) -> None:
        """A missing name does not drop a valid identity from the inventory."""

        parsed = fleet._parse_workflow(
            {"id": 8, "path": ".github/workflows/ci.yml", "state": "active"}
        )
        self.assertEqual(parsed["name"], ".github/workflows/ci.yml")

    def test_git_ref_and_tree_payload_errors_fail_closed(self) -> None:
        """Malformed ref/tree documents cannot produce a false-empty inventory."""

        responses = self._happy_responses()
        responses[("GET", "/repos/ContextualWisdomLab/TEPP/git/ref/heads/main")] = (
            _json_response({"object": "nope"})
        )
        with self.assertRaises(fleet.FleetAuditError) as ref_error:
            fleet.audit_repository(
                FakeTransport(responses), "ContextualWisdomLab", "TEPP"
            )
        self.assertEqual(ref_error.exception.reason, "invalid_payload")

        not_list = self._happy_responses()
        not_list[
            ("GET", "/repos/ContextualWisdomLab/TEPP/git/trees/abc123def456?recursive=1")
        ] = _json_response({"truncated": False, "tree": {}})
        with self.assertRaises(fleet.FleetAuditError) as tree_error:
            fleet.audit_repository(
                FakeTransport(not_list), "ContextualWisdomLab", "TEPP"
            )
        self.assertEqual(tree_error.exception.reason, "invalid_payload")

        bad_entry = self._happy_responses()
        bad_entry[
            ("GET", "/repos/ContextualWisdomLab/TEPP/git/trees/abc123def456?recursive=1")
        ] = _json_response({"truncated": False, "tree": ["nope"]})
        with self.assertRaises(fleet.FleetAuditError) as entry_error:
            fleet.audit_repository(
                FakeTransport(bad_entry), "ContextualWisdomLab", "TEPP"
            )
        self.assertEqual(entry_error.exception.reason, "invalid_payload")

        missing_path = self._happy_responses()
        missing_path[
            ("GET", "/repos/ContextualWisdomLab/TEPP/git/trees/abc123def456?recursive=1")
        ] = _json_response(
            {"truncated": False, "tree": [{"type": "blob", "path": ""}]}
        )
        with self.assertRaises(fleet.FleetAuditError) as path_error:
            fleet.audit_repository(
                FakeTransport(missing_path), "ContextualWisdomLab", "TEPP"
            )
        self.assertEqual(path_error.exception.reason, "invalid_payload")

    def test_tree_skips_non_blob_entries_and_default_clock_is_used(self) -> None:
        """Directories are ignored and a missing timestamp uses the UTC clock."""

        responses = self._happy_responses()
        responses[
            ("GET", "/repos/ContextualWisdomLab/TEPP/git/trees/abc123def456?recursive=1")
        ] = _json_response(
            {
                "truncated": False,
                "tree": [
                    {"path": ".github/workflows", "type": "tree"},
                    {"path": ".github/workflows/ci.yml", "type": "blob"},
                    {"path": ".github/workflows/docs-quality.yml", "type": "blob"},
                    {
                        "path": ".github/workflows/hourly-pr-maintenance.yml",
                        "type": "blob",
                    },
                    {
                        "path": ".github/workflows/hourly-nim-product-development.yml",
                        "type": "blob",
                    },
                ],
            }
        )
        with mock.patch.object(fleet, "_utc_now", return_value="clock-stamp"):
            audit = fleet.audit_repository(
                FakeTransport(responses), "ContextualWisdomLab", "TEPP"
            )
        self.assertEqual(audit.timestamp, "clock-stamp")
        present = {
            item.path
            for item in audit.identities
            if item.classification == "present"
        }
        self.assertIn(".github/workflows/ci.yml", present)

    def test_urllib_transport_maps_httperror_without_raising(self) -> None:
        """4xx/5xx from urllib stay as HttpResponse so callers can classify them."""

        class _Error(urllib.error.HTTPError):
            def __init__(self) -> None:
                super().__init__(
                    url="https://api.github.com/x",
                    code=403,
                    msg="no",
                    hdrs={"X-GitHub": "yes"},
                    fp=io.BytesIO(b'{"message":"no"}'),
                )

        transport = fleet.UrllibTransport("secret-token")
        with mock.patch("urllib.request.urlopen", side_effect=_Error()):
            response = transport.request("GET", "/repos/o/r")
        self.assertEqual(response.status, 403)
        self.assertEqual(response.headers["X-GitHub"], "yes")

    def test_urllib_transport_maps_status_and_headers(self) -> None:
        """The real urllib transport preserves status, headers, and body bytes."""

        class _FakeHttp:
            def __init__(self) -> None:
                self.status = 200
                self.headers = {"Content-Type": "application/json"}

            def read(self) -> bytes:
                return b'{"ok":true}'

            def __enter__(self) -> _FakeHttp:
                return self

            def __exit__(self, *args: object) -> None:
                return None

        transport = fleet.UrllibTransport("secret-token")
        with mock.patch("urllib.request.urlopen", return_value=_FakeHttp()):
            response = transport.request("GET", "/repos/o/r")
        self.assertEqual(response.status, 200)
        self.assertEqual(json.loads(response.body), {"ok": True})

    def test_build_transport_uses_standard_token(self) -> None:
        """build_transport reads GITHUB_TOKEN and does not invent a TEPP PAT name."""

        transport = fleet.build_transport({"GITHUB_TOKEN": "abc"})
        self.assertIsInstance(transport, fleet.UrllibTransport)
        self.assertEqual(transport.token, "abc")

    def test_audit_rejects_missing_default_branch_or_sha(self) -> None:
        """Incomplete repository or ref payloads fail closed before classification."""

        responses = self._happy_responses()
        responses[("GET", "/repos/ContextualWisdomLab/TEPP")] = _json_response({})
        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.audit_repository(
                FakeTransport(responses), "ContextualWisdomLab", "TEPP"
            )
        self.assertEqual(raised.exception.reason, "invalid_payload")

    def test_disable_refuses_when_live_state_is_no_longer_active(self) -> None:
        """A workflow that became disabled before PUT is not mutated again."""

        audit = fleet.audit_repository(
            FakeTransport(self._happy_responses()), "ContextualWisdomLab", "TEPP"
        )
        already = self._happy_responses()
        already[("GET", "/repos/ContextualWisdomLab/TEPP/actions/workflows/10")] = (
            _json_response(
                _workflow(
                    10,
                    ".github/workflows/bootstrap-materialize.yml",
                    state="disabled",
                )
            )
        )
        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.disable_orphans(
                FakeTransport(already), audit, apply_changes=True
            )
        self.assertEqual(raised.exception.reason, "workflow_identity_changed")

    def test_disable_fails_when_post_put_state_is_not_disabled(self) -> None:
        """A 204 that leaves the identity active is not treated as success."""

        audit = fleet.audit_repository(
            FakeTransport(self._happy_responses()),
            "ContextualWisdomLab",
            "TEPP",
        )
        stuck = self._happy_responses()
        stuck[
            ("PUT", "/repos/ContextualWisdomLab/TEPP/actions/workflows/10/disable")
        ] = fleet.HttpResponse(status=204, headers={}, body=b"")
        transport = _ConfirmFailsTransport(stuck)
        with self.assertRaises(fleet.FleetAuditError) as raised:
            fleet.disable_orphans(transport, audit, apply_changes=True)
        self.assertEqual(raised.exception.reason, "disable_unconfirmed")

    def test_disable_confirms_official_disabled_manually_state(self) -> None:
        """GitHub's disable PUT sets disabled_manually, not a bare disabled token."""

        audit = fleet.audit_repository(
            FakeTransport(self._happy_responses()), "ContextualWisdomLab", "TEPP"
        )
        apply_responses = self._happy_responses()
        apply_responses[
            ("PUT", "/repos/ContextualWisdomLab/TEPP/actions/workflows/10/disable")
        ] = fleet.HttpResponse(status=204, headers={}, body=b"")
        apply_responses[
            ("PUT", "/repos/ContextualWisdomLab/TEPP/actions/workflows/11/disable")
        ] = fleet.HttpResponse(status=204, headers={}, body=b"")
        apply_responses[
            ("GET", "/repos/ContextualWisdomLab/TEPP/actions/workflows/10")
        ] = _json_response(
            _workflow(
                10,
                ".github/workflows/bootstrap-materialize.yml",
                state="disabled_manually",
            )
        )
        apply_responses[
            ("GET", "/repos/ContextualWisdomLab/TEPP/actions/workflows/11")
        ] = _json_response(
            _workflow(
                11,
                ".github/workflows/diagnose-pr5-coverage-file.yml",
                state="disabled_manually",
            )
        )
        outcomes = fleet.disable_orphans(
            _AlternatingWorkflowTransport(apply_responses),
            audit,
            apply_changes=True,
        )
        self.assertTrue(all(item.disabled for item in outcomes))

    def test_parse_link_rejects_absolute_url_outside_api_host(self) -> None:
        """Next links must stay on api.github.com so the client cannot be redirected."""

        self.assertIsNone(
            fleet.next_link_path(
                "<https://evil.example/workflows?page=2>; rel=\"next\""
            )
        )

    def _happy_responses(self) -> dict[tuple[str, str], fleet.HttpResponse]:
        """Shared live inventory: present, two orphans, disabled, and CodeQL."""

        workflows = [
            _workflow(1, ".github/workflows/ci.yml", name="Rust Foundation CI"),
            _workflow(
                2,
                ".github/workflows/docs-quality.yml",
                name="Documentation Quality",
            ),
            _workflow(
                3,
                ".github/workflows/hourly-pr-maintenance.yml",
                name="Hourly PR Maintenance",
            ),
            _workflow(
                10,
                ".github/workflows/bootstrap-materialize.yml",
                name="Materialize TEPP Bootstrap",
            ),
            _workflow(
                11,
                ".github/workflows/diagnose-pr5-coverage-file.yml",
                name="Diagnose PR 5 coverage file",
            ),
            _workflow(
                20,
                ".github/workflows/repair-pr5-temporal-coverage.yml",
                state="disabled",
            ),
            _workflow(30, "dynamic/github-code-scanning/codeql", name="CodeQL"),
        ]
        return {
            ("GET", "/repos/ContextualWisdomLab/TEPP"): _json_response(
                {"default_branch": "main"}
            ),
            ("GET", "/repos/ContextualWisdomLab/TEPP/git/ref/heads/main"): (
                _json_response({"object": {"sha": "abc123def456", "type": "commit"}})
            ),
            (
                "GET",
                "/repos/ContextualWisdomLab/TEPP/git/trees/abc123def456?recursive=1",
            ): _json_response(
                {
                    "truncated": False,
                    "tree": [
                        {"path": ".github/workflows/ci.yml", "type": "blob"},
                        {"path": ".github/workflows/docs-quality.yml", "type": "blob"},
                        {
                            "path": ".github/workflows/hourly-pr-maintenance.yml",
                            "type": "blob",
                        },
                        {
                            "path": ".github/workflows/hourly-nim-product-development.yml",
                            "type": "blob",
                        },
                        {"path": "README.md", "type": "blob"},
                    ],
                }
            ),
            (
                "GET",
                "/repos/ContextualWisdomLab/TEPP/actions/workflows?per_page=100",
            ): _json_response({"total_count": 7, "workflows": workflows}),
            ("GET", "/repos/ContextualWisdomLab/TEPP/actions/workflows/10"): (
                _json_response(
                    _workflow(10, ".github/workflows/bootstrap-materialize.yml")
                )
            ),
            ("GET", "/repos/ContextualWisdomLab/TEPP/actions/workflows/11"): (
                _json_response(
                    _workflow(11, ".github/workflows/diagnose-pr5-coverage-file.yml")
                )
            ),
        }


class _AlternatingWorkflowTransport(FakeTransport):
    """Return active then disabled for the same workflow GET during apply."""

    def __init__(self, responses: dict[tuple[str, str], fleet.HttpResponse]) -> None:
        super().__init__(responses)
        self._get_counts: dict[str, int] = {}

    def request(self, method: str, path: str) -> fleet.HttpResponse:
        if method == "GET" and path.endswith("/actions/workflows/10"):
            count = self._get_counts.get(path, 0)
            self._get_counts[path] = count + 1
            self.calls.append((method, path))
            if count == 0:
                return _json_response(
                    _workflow(10, ".github/workflows/bootstrap-materialize.yml")
                )
            return self.responses[(method, path)]
        if method == "GET" and path.endswith("/actions/workflows/11"):
            count = self._get_counts.get(path, 0)
            self._get_counts[path] = count + 1
            self.calls.append((method, path))
            if count == 0:
                return _json_response(
                    _workflow(11, ".github/workflows/diagnose-pr5-coverage-file.yml")
                )
            return self.responses[(method, path)]
        return super().request(method, path)


class _ConfirmFailsTransport(FakeTransport):
    """Leave workflow 10 active after a successful disable PUT."""

    def __init__(self, responses: dict[tuple[str, str], fleet.HttpResponse]) -> None:
        super().__init__(responses)
        self._gets = 0

    def request(self, method: str, path: str) -> fleet.HttpResponse:
        if method == "GET" and path.endswith("/actions/workflows/10"):
            self.calls.append((method, path))
            self._gets += 1
            return _json_response(
                _workflow(10, ".github/workflows/bootstrap-materialize.yml")
            )
        return super().request(method, path)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
