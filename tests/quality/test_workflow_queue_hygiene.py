"""Queue-isolation contracts for repository-local GitHub Actions."""

from pathlib import Path
import unittest

from scripts.effective_ci_source import effective_ci_source


WORKFLOW_DIRECTORY = Path(".github/workflows")


class WorkflowQueueHygieneTests(unittest.TestCase):
    """Keep local validation isolated by repository and pull request."""

    def test_pull_request_workflows_cancel_only_older_same_pr_runs(self) -> None:
        """CI groups include workflow, repository, and PR identity."""

        expected_groups = {
            "ci.yml": (
                "tepp-rust-foundation-${{ github.repository }}-"
                "${{ github.event.pull_request.number || github.run_id }}"
            ),
            "docs-quality.yml": (
                "docs-quality-${{ github.repository }}-"
                "${{ github.event.pull_request.number || github.run_id }}"
            ),
        }
        for workflow_name, expected_group in expected_groups.items():
            workflow = (WORKFLOW_DIRECTORY / workflow_name).read_text(encoding="utf-8")
            self.assertIn(f"group: {expected_group}", workflow)
            self.assertIn(
                "cancel-in-progress: ${{ github.event_name == 'pull_request' }}",
                workflow,
            )

    def test_pull_request_workflows_skip_draft_pull_requests(self) -> None:
        """A draft's iteration must not consume queue slots or runner time."""

        guard = (
            "if: ${{ github.event_name != 'pull_request' "
            "|| !github.event.pull_request.draft }}"
        )
        callers = (
            ("ci.yml", "foundation", "tepp-rust-foundation-ci.yml"),
            ("docs-quality.yml", "documentation", "tepp-documentation-quality.yml"),
        )
        for workflow_name, job_id, supplier in callers:
            workflow = (WORKFLOW_DIRECTORY / workflow_name).read_text(encoding="utf-8")
            with self.subTest(workflow=workflow_name):
                # One guard on the only caller job skips every pinned supplier gate.
                jobs = workflow.split("\njobs:\n", 1)[1]
                self.assertEqual(jobs.count("    uses: "), 1)
                self.assertEqual(workflow.count(guard), 1)
                caller = jobs.split(f"  {job_id}:\n", 1)[1]
                self.assertTrue(caller.split("    uses: ", 1)[0].endswith(f"    {guard}\n"))
                self.assertIn(f"/{supplier}@", caller)
                triggers = workflow.split("\n  pull_request:\n", 1)[1].split("\n    paths:\n", 1)[0]
                # Marking a draft ready must re-run the gates it skipped.
                self.assertIn("      - ready_for_review", triggers)
                # Returning a ready PR to draft must cancel its in-flight local run.
                self.assertIn("      - converted_to_draft", triggers)

    def test_central_scheduler_replaces_local_hourly_caller(self) -> None:
        """A local timer must not duplicate central PR scheduling."""

        self.assertFalse((WORKFLOW_DIRECTORY / "hourly-pr-maintenance.yml").exists())

    def test_documentation_workflow_does_not_duplicate_crate_ci(self) -> None:
        """Crate changes already run documentation contracts in Rust CI."""

        workflow = (WORKFLOW_DIRECTORY / "docs-quality.yml").read_text(
            encoding="utf-8"
        )
        self.assertNotIn('      - "crates/**"', workflow)

    def test_rust_ci_reuses_one_runner_for_both_coverage_reports(self) -> None:
        """Coverage keeps both gates while avoiding a second runner boot."""

        workflow = effective_ci_source(Path(__file__).resolve().parents[2])
        self.assertIn("  production-coverage:\n", workflow)
        self.assertNotIn("  line-coverage:\n", workflow)
        self.assertNotIn("  branch-coverage:\n", workflow)
        self.assertIn("id: line-report", workflow)
        self.assertIn("id: branch-report", workflow)
        self.assertIn("if: ${{ !cancelled() }}", workflow)
        self.assertNotIn(
            "cargo check --workspace --all-targets --all-features", workflow
        )
        self.assertIn(
            "cargo clippy --workspace --all-targets --all-features -- -D warnings",
            workflow,
        )


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
