"""Lock the hourly contextual-orchestrator OpenCode security contract."""

from __future__ import annotations

from dataclasses import dataclass
import os
import sys
import unittest
from pathlib import Path
from types import ModuleType
from unittest.mock import patch

WORKFLOW = Path(".github/workflows/hourly-nim-product-development.yml")
BOOTSTRAP = Path("scripts/run_contextual_orchestrator.py")
PARSER = Path("scripts/prepare_agent_pr_message.py")
RUNBOOK = Path("docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md")
DOCTORING = Path("docs/doctoring/hourly-nim-opencode-development.md")


def _text(path: Path) -> str:
    """Return one required UTF-8 contract file."""

    assert path.is_file(), f"required contract file is missing: {path}"
    return path.read_text(encoding="utf-8")


def _parser_module() -> ModuleType:
    """Load the trusted pull-request metadata parser as a covered module."""

    assert PARSER.is_file()
    import scripts.prepare_agent_pr_message as module

    return module


class HourlyNimProductDevelopmentContractTests(unittest.TestCase):
    """Structural tests for the credential-separated product-development loop."""

    def test_hourly_workflow_schedule_credentials_and_queue_gate(self) -> None:
        """Keep central-admission entrypoint, provider discovery, and fail-closed inventory gates."""

        text = _text(WORKFLOW)
        bootstrap = _text(BOOTSTRAP)
        for token in (
            "# cwl-org-commercial-entrypoint: v1",
            "workflow_dispatch:",
            "dry_run:",
            "hourly-nim-product-development-${{ github.repository }}",
            "cancel-in-progress: false",
            "secrets.BYTEZ_API_KEY",
            "secrets.NVIDIA_NIM_API_KEY",
            "secrets.NVIDIA_NIM_API_KEY_SUB",
            "secrets.OPENROUTER_API_KEY",
            "secrets.OPENAI_API_KEY",
            "issues: read",
            "permission-issues: read",
            "CONTEXTUAL_ORCHESTRATOR_COMMIT",
            "CONTEXTUAL_ORCHESTRATOR_SHA256",
            "run_contextual_orchestrator.py",
            "/healthz",
            "/v1/models",
            "{env:OPENCODE_GATEWAY_TOKEN}",
            "OPENCODE_VERSION",
            "OPENCODE_SHA256",
            "sha256sum -c",
            "pull_request_inventory_unavailable",
            "open_pull_request",
            "gh issue list --repo",
            "issue_inventory_unavailable",
            "open_issue",
            "contextual_orchestrator_credentials_unavailable",
            "maintainer_app_unavailable",
            "base_branch_advanced",
            "open_pull_request_after_generation",
            "issue_inventory_unavailable_after_generation",
            "open_issue_after_generation",
            "ContextualWisdomLab/TEPP",
        ):
            self.assertIn(token, text)
        for token in ("discover_all_models", "register_credential", "PROVIDER_CREDENTIAL_NAMES"):
            self.assertIn(token, bootstrap)
        self.assertNotIn("COPILOT_GITHUB_TOKEN", text)
        self.assertNotIn("CONTEXTUAL_ORCHESTRATOR_TOKEN=", text)
        self.assertEqual(text.count("gh pr create"), 1)
        self.assertNotIn("gh pr merge", text)
        self.assertNotIn("gh release create", text)

    def test_hourly_workflow_separates_three_runner_trust_boundaries(self) -> None:
        """Separate model execution, verification, and late publication authority."""

        text = _text(WORKFLOW)
        proposer = text.split("propose_product_increment:", 1)[1].split(
            "package_product_increment:", 1
        )[0]
        verifier = text.split("package_product_increment:", 1)[1].split(
            "publish_product_increment:", 1
        )[0]
        publisher = text.split("publish_product_increment:", 1)[1]

        self.assertIn("BYTEZ_API_KEY", proposer)
        self.assertIn("OPENAI_API_KEY", proposer)
        self.assertNotIn("create-github-app-token", proposer)
        self.assertNotIn("gh pr create", proposer)
        self.assertNotIn("NVIDIA_NIM_API_KEY", verifier)
        self.assertNotIn("create-github-app-token", verifier)
        self.assertIn("Run every release-quality gate", verifier)
        self.assertNotIn("NVIDIA_NIM_API_KEY", publisher)
        self.assertIn("create-github-app-token", publisher)
        self.assertIn("gh pr create", publisher)
        self.assertNotIn("cargo test", publisher)
        self.assertNotIn("pytest ", publisher)
        self.assertLess(
            text.index("Preserve trusted metadata parser"),
            text.index("Verify and apply immutable proposal without executing it"),
        )
        self.assertLess(
            text.index("Parse bounded untrusted pull-request metadata"),
            text.index("Mint dedicated maintainer App token only for publication"),
        )

