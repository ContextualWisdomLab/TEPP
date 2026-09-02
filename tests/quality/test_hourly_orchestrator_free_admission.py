"""Regression tests for released contextual-orchestrator routing ownership.

The predecessor tests pinned TEPP-side provider discovery and explicit-zero price
filtering. That was useful RED evidence for issue #479, but it also proved the
consumer repository owned routing logic that belongs to contextual-orchestrator.
These tests preserve the defect boundary by requiring the wrong-owner bootstrap
to stay retired and the hourly workflow to consume only the released free route.
"""

from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/hourly-nim-product-development.yml"
RETIRED_BOOTSTRAP = ROOT / "scripts/run_contextual_orchestrator.py"


class HourlyOrchestratorFreeAdmissionTests(unittest.TestCase):
    """Keep provider/model/free-policy authority outside the TEPP consumer."""

    def test_consumer_side_provider_routing_bootstrap_is_retired(self) -> None:
        """Do not keep a second provider discovery or price-ranking authority."""

        self.assertFalse(
            RETIRED_BOOTSTRAP.exists(),
            "provider/model/free-route discovery belongs to released contextual-orchestrator",
        )

    def test_hourly_workflow_uses_only_released_free_route(self) -> None:
        """Require immutable owner release, HTTPS gateway, and orchestrator/free."""

        text = WORKFLOW.read_text(encoding="utf-8")
        for required in (
            "CONTEXTUAL_ORCHESTRATOR_RELEASE",
            "CONTEXTUAL_ORCHESTRATOR_BASE_URL",
            "secrets.CONTEXTUAL_ORCHESTRATOR_GATEWAY_TOKEN",
            "orchestrator/free",
            ".immutable == true",
            "contextual_orchestrator_release_unavailable",
            "contextual_orchestrator_gateway_unavailable",
            "--proto '=https'",
        ):
            self.assertIn(required, text)
        for forbidden in (
            "run_contextual_orchestrator.py",
            "select_top_n_cheapest_discovered_agents",
            "prompt_price_per_1k",
            "completion_price_per_1k",
            "http://127.0.0.1",
        ):
            self.assertNotIn(forbidden, text)

    def test_authenticated_gateway_probes_allow_only_https_redirects(self) -> None:
        """Prevent authenticated gateway probes from following an HTTP downgrade."""

        text = WORKFLOW.read_text(encoding="utf-8")
        secure_redirect_probe = (
            "curl --fail --silent --show-error --location --proto '=https' "
            "--proto-redir '=https' --tlsv1.2"
        )
        self.assertGreaterEqual(
            text.count(secure_redirect_probe),
            2,
            "both gateway probes must restrict redirect protocols to HTTPS",
        )

    def test_opencode_archive_download_allows_only_https_redirects(self) -> None:
        """Keep the checksum-pinned CLI download from following an HTTP downgrade."""

        text = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn(
            "curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' \\",
            text,
            "OpenCode archive download must restrict redirect protocols to HTTPS",
        )

    def test_canonical_llm_authority_requires_released_orchestrator_contract(self) -> None:
        """Keep normative product/technical docs from re-authorizing provider keys."""

        canonical_paths = (
            ROOT / "AGENTS.md",
            ROOT / "docs/product/prd-v0.4-approved.md",
            ROOT / "docs/TRD.md",
            ROOT / "docs/LLM_ORCHESTRATION.md",
            ROOT / "ARCHITECTURE.md",
        )
        for path in canonical_paths:
            with self.subTest(path=path.relative_to(ROOT)):
                text = path.read_text(encoding="utf-8")
                self.assertNotIn("NVIDIA_NIM_API_KEY", text)
                self.assertIn("contextual-orchestrator", text)
                self.assertIn("released", text.lower())


if __name__ == "__main__":
    unittest.main()
