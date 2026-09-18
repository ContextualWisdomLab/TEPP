"""Guard the generated-proposal verifier against writable cache persistence."""

from pathlib import Path
import unittest


WORKFLOW = Path(".github/workflows/hourly-nim-product-development.yml")


class HourlyNimCacheBoundaryTests(unittest.TestCase):
    """Keep untrusted proposal execution outside cache-write authority."""

    def test_verifier_does_not_persist_cache_after_applying_proposal(self) -> None:
        """Reject cache actions from the verifier that executes proposal code."""

        workflow = WORKFLOW.read_text(encoding="utf-8")
        verifier = workflow.split("package_product_increment:", 1)[1].split(
            "publish_product_increment:", 1
        )[0]

        self.assertIn("Verify immutable artifact identity and apply proposal", verifier)
        self.assertIn("Run every release-quality gate", verifier)
        self.assertNotIn("actions/cache@", verifier)
        self.assertNotIn("ACTIONS_CACHE_URL", verifier.split("Run every release-quality gate", 1)[1])


if __name__ == "__main__":
    unittest.main()
