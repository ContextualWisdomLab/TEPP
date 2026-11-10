"""Guard the generated-proposal verifier against writable cache persistence."""

from pathlib import Path
import unittest
from unittest.mock import patch


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
        self.assertNotIn("actions/cache", verifier)
        for version in (
            "cargo-nextest --locked --version 0.9.140",
            "cargo-deny --locked --version 0.19.7",
            "cargo-llvm-cov --locked --version 0.8.6",
        ):
            with self.subTest(version=version):
                self.assertIn(version, verifier)


    def test_verifier_rejects_split_cache_actions(self) -> None:
        """The real verifier contract must reject restore and save subactions."""

        workflow = WORKFLOW.read_text(encoding="utf-8")
        marker = "  package_product_increment:\n"
        self.assertEqual(workflow.count(marker), 1)
        prefix, verifier_and_publisher = workflow.split(marker, 1)
        for action in ("actions/cache@", "actions/cache/restore@", "actions/cache/save@"):
            changed = verifier_and_publisher.replace(
                "    steps:\n",
                "    steps:\n      - name: Synthetic cache permission fixture\n"
                f"        uses: {action}" + "0" * 40 + "\n",
                1,
            )
            mutated = prefix + marker + changed
            with self.subTest(action=action):
                with patch.object(Path, "read_text", return_value=mutated):
                    with self.assertRaises(AssertionError):
                        self.test_verifier_does_not_persist_cache_after_applying_proposal()

    def test_setup_python_never_restores_or_saves_a_pip_cache(self) -> None:
        """Neither the proposal job nor the verifier may share a pip cache."""

        workflow = WORKFLOW.read_text(encoding="utf-8")
        setup_steps = workflow.split("uses: actions/setup-python@")[1:]
        self.assertEqual(len(setup_steps), 2)
        for index, step in enumerate(setup_steps):
            with_block = step.split("\n      - name:", 1)[0]
            with self.subTest(step=index):
                self.assertIn("python-version:", with_block)
                self.assertNotIn("cache:", with_block)
                self.assertNotIn("cache-dependency-path:", with_block)


if __name__ == "__main__":
    unittest.main()
