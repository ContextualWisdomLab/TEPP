"""Keep Longitudinal Modeling numerical guidance synchronized with production contracts."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CLAUDE = ROOT / "CLAUDE.md"
CHANGELOG = ROOT / "CHANGELOG.md"


def test_stationary_variance_overflow_guidance_matches_source_contract() -> None:
    """Reject the retired ratio-first overflow rewrite and require the current fallback."""
    guidance = CLAUDE.read_text(encoding="utf-8")

    assert "When `2 a` overflows, form `(q / a) * -0.5`." not in guidance
    assert "Do not form `0.5 q` first (`q = from_bits(1)` underflows)." not in guidance
    assert "When `2 a` overflows, form `(q * 0.5) / |a|`." in guidance


def test_hamaker_occasion_mean_changelog_uses_longitudinal_owner() -> None:
    """Keep the Hamaker occasion-mean temporal composition under Longitudinal Modeling."""
    changelog = CHANGELOG.read_text(encoding="utf-8")

    assert "`longitudinal_core` recovers Hamaker, Kuiper, and Grasman (2015, Eq. 1a" in changelog
    assert "`psychometric_core` recovers Hamaker, Kuiper, and Grasman (2015, Eq. 1a" not in changelog
    assert "2026-09-03T06:07Z KST" not in changelog
