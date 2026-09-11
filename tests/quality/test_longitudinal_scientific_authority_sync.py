"""Keep Longitudinal Modeling numerical guidance synchronized with production contracts."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CLAUDE = ROOT / "CLAUDE.md"
PRD_AMENDMENT = ROOT / "docs/product/prd-v0.4-amendment-longitudinal-time-ownership.md"
TRD = ROOT / "docs/TRD.md"
OWNERSHIP_ADR = ROOT / "docs/adr/longitudinal-modeling-ownership-addendum.md"
LITERATURE_REGISTER = ROOT / "docs/research/standards-and-literature.md"


def test_stationary_variance_overflow_guidance_matches_source_contract() -> None:
    """Reject the retired ratio-first overflow rewrite and require the current fallback."""
    guidance = CLAUDE.read_text(encoding="utf-8")

    assert "When `2 a` overflows, form `(q / a) * -0.5`." not in guidance
    assert "Do not form `0.5 q` first (`q = from_bits(1)` underflows)." not in guidance
    assert "When `2 a` overflows, form `(q * 0.5) / |a|`." in guidance


def test_prd_names_irregular_rate_estimands_and_weighting_population() -> None:
    """Keep the product target explicit when follow-up multiplicity changes pair weights."""
    prd = PRD_AMENDMENT.read_text(encoding="utf-8")

    assert "`tepp.irregular_rate.lag_pair_average.v1`" in prd
    assert "`tepp.irregular_rate.unit_average.v1`" in prd
    assert "candidate/contributing units" in prd
    assert "candidate/admitted/refused pairs" in prd
    assert "fail closed" in prd


def test_trd_requires_versioned_irregular_rate_weighting_and_denominators() -> None:
    """Keep the technical contract aligned with the public estimand API."""
    trd = TRD.read_text(encoding="utf-8")

    assert "`tepp.irregular_rate.lag_pair_average.v1`" in trd
    assert "`tepp.irregular_rate.unit_average.v1`" in trd
    assert "candidate/contributing units" in trd
    assert "candidate/admitted/refused pairs" in trd
    assert "attempted/recovered/failed" in trd


def test_ownership_adr_keeps_pair_and_unit_estimands_distinct() -> None:
    """Prevent the implemented pair target from silently becoming an equal-unit target."""
    adr = OWNERSHIP_ADR.read_text(encoding="utf-8")

    assert "`tepp.irregular_rate.lag_pair_average.v1`" in adr
    assert "`tepp.irregular_rate.unit_average.v1`" in adr
    assert "occasion count" in adr
    assert "membership weight" in adr
    assert "fail closed" in adr


def test_irregular_rate_weighting_sources_are_doctored_in_canonical_register() -> None:
    """Keep claim-specific informative-size sources complete enough for APA traceability."""
    literature = LITERATURE_REGISTER.read_text(encoding="utf-8")

    assert (
        "Huang, Y., & Leroux, B. (2011). Informative cluster sizes for subcluster-level "
        "covariates and weighted generalized estimating equations. *Biometrics, 67*(3), "
        "843–851." in literature
    )
    assert (
        "Kahan, B. C., Li, F., Blette, B., Jairath, V., Copas, A., & Harhay, M. O. "
        "(2023). Informative cluster size in cluster-randomised trials: A case study from "
        "the TRIGGER trial. *Clinical Trials, 20*(6), 661–669." in literature
    )
    assert (
        "Wang, M., Kong, M., & Datta, S. (2011). Inference for marginal linear models for "
        "clustered longitudinal data with potentially informative cluster sizes. "
        "*Statistical Methods in Medical Research, 20*(4), 347–367." in literature
    )
    assert "irregular-rate estimand" in literature
    assert "record multiplicity" in literature
