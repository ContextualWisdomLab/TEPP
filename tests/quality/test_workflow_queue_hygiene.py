"""Queue-isolation contracts for repository-local GitHub Actions."""

from pathlib import Path
import unittest


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

        workflow = (WORKFLOW_DIRECTORY / "ci.yml").read_text(encoding="utf-8")
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
