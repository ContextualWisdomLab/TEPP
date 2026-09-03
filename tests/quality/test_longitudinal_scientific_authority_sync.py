"""Keep Longitudinal Modeling numerical guidance synchronized with production contracts."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CLAUDE = ROOT / "CLAUDE.md"


def test_stationary_variance_overflow_guidance_matches_source_contract() -> None:
    """Reject the retired ratio-first overflow rewrite and require the current fallback."""
    guidance = CLAUDE.read_text(encoding="utf-8")

    assert "When `2 a` overflows, form `(q / a) * -0.5`." not in guidance
    assert "Do not form `0.5 q` first (`q = from_bits(1)` underflows)." not in guidance
    assert "When `2 a` overflows, form `(q * 0.5) / |a|`." in guidance
