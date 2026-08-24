"""Contracts for the repository-local hourly maintenance caller."""

from __future__ import annotations

import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
CALLER_WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "hourly-pr-maintenance.yml"
CENTRAL_SCHEDULER_REVISION = "731af58e954901c4f1cc853231c592abb1eaf617"


class HourlyMaintenanceCallerContractTests(unittest.TestCase):
    """Keep the caller bounded, immutable, and credential-separated."""

    def test_caller_runs_hourly_and_pins_the_verified_central_scheduler(self) -> None:
        """The repository delegates policy instead of copying mutable scheduler code."""

        workflow = CALLER_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn('cron: "11 * * * *"', workflow)
        self.assertIn(
            "uses: ContextualWisdomLab/.github/.github/workflows/"
            f"pr-review-merge-scheduler.yml@{CENTRAL_SCHEDULER_REVISION}",
            workflow,
        )
        self.assertNotIn("secrets: inherit", workflow)
        self.assertNotIn("COPILOT_GITHUB_TOKEN", workflow)
        self.assertNotIn("NVIDIA_NIM_API_KEY", workflow)

    def test_workflow_default_is_read_only_and_only_job_permissions_are_elevated(self) -> None:
        """Review and merge authority stays scoped to the reusable-workflow job."""

        workflow = CALLER_WORKFLOW.read_text(encoding="utf-8")
        default_permissions = workflow.split("concurrency:", maxsplit=1)[0]
        job_permissions = workflow.split("jobs:", maxsplit=1)[1]

        self.assertIn("permissions:\n  contents: read", default_permissions)
        self.assertIn("actions: write", job_permissions)
        self.assertIn("checks: read", job_permissions)
        self.assertIn("contents: write", job_permissions)
        self.assertIn("id-token: write", job_permissions)
        self.assertIn("pull-requests: write", job_permissions)
        self.assertIn("cancel-in-progress: false", workflow)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
