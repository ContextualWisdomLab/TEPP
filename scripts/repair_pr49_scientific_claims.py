"""Apply PR 49 compositional-geometry and posterior-summary claim repairs."""

from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    """Replace exactly one fragment or fail closed."""
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)


def update_indicator_contract() -> None:
    """Distinguish valid structural coordinates from Aitchison isometries."""
    path = Path("crates/psychometric_core/src/indicator.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "//! Valid psychometric indicator coordinates.\n",
        "//! Valid structural indicator coordinates and compositional-geometry claims.\n",
        "indicator module docs",
    )
    old_block = """    /// Return whether the kind is a valid Euclidean psychometric input.
    #[must_use]
    pub const fn is_valid_psychometric_input(self) -> bool {
        !matches!(self, Self::RawProportion)
    }
"""
    new_block = """    /// Return whether the kind is an admissible unconstrained structural input.
    ///
    /// This does not claim that the coordinates are orthonormal or preserve
    /// Aitchison distance. ALR is reference-dependent; only ILR carries that
    /// orthonormal compositional-geometry claim.
    #[must_use]
    pub const fn is_valid_structural_input(self) -> bool {
        !matches!(self, Self::RawProportion)
    }

    /// Return whether the coordinate kind is an orthonormal Aitchison isometry.
    #[must_use]
    pub const fn preserves_aitchison_distance(self) -> bool {
        matches!(self, Self::IsometricLogRatio)
    }
"""
    text = replace_once(text, old_block, new_block, "indicator geometry methods")
    text = text.replace("kind.is_valid_psychometric_input()", "kind.is_valid_structural_input()")
    text = replace_once(
        text,
        "/// Pearson product-moment correlation on already-mapped coordinates.\n",
        """/// Pearson product-moment correlation on already-mapped coordinates.
///
/// For ALR this is a reference-dependent coordinate correlation, not an
/// Aitchison-distance-preserving statistic. Use an ILR basis when orthonormal
/// compositional geometry is part of the estimand.
""",
        "Pearson claim boundary",
    )
    path.write_text(text, encoding="utf-8")


def update_posterior_summary_contract() -> None:
    """Rename point-estimate averages so they cannot imply Rubin pooling."""
    path = Path("crates/psychometric_core/src/plausible.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "//! Plausible-value aggregation of posterior structural draws.\n",
        "//! Point-estimate aggregation across posterior structural draws.\n",
        "posterior module docs",
    )
    text = text.replace("plausible_value_mean", "posterior_draw_point_estimate_mean")
    text = text.replace(
        "recover_loading_from_plausible_values",
        "recover_loading_point_estimate_mean",
    )
    text = replace_once(
        text,
        "/// Arithmetic mean of finite plausible-value draws.\n",
        """/// Arithmetic mean of finite posterior-draw point estimates.
///
/// This helper does not pool within-draw and between-draw uncertainty and must
/// not be described as Rubin multiple-imputation variance pooling.
""",
        "point-estimate mean docs",
    )
    text = replace_once(
        text,
        """/// Recover a reflective loading by averaging OLS slopes across posterior
/// indicator draws (Rubin-style plausible values).
""",
        """/// Recover a reflective loading point estimate by averaging OLS slopes across
/// posterior indicator draws.
///
/// The result is a point-estimate summary only. It does not estimate within-draw
/// variance, between-draw variance, total variance, degrees of freedom, or a
/// confidence interval, and therefore is not Rubin-style uncertainty pooling.
""",
        "loading aggregation docs",
    )
    text = text.replace("plausible-value loading", "posterior-draw loading point estimate")
    text = text.replace("mean_of_two_draws", "mean_of_two_point_estimates")
    path.write_text(text, encoding="utf-8")


def update_public_api_and_tests() -> None:
    """Align exports and recovery tests with the narrower scientific claims."""
    lib_path = Path("crates/psychometric_core/src/lib.rs")
    lib = lib_path.read_text(encoding="utf-8")
    lib = replace_once(
        lib,
        """//! Raw topic proportions are not Euclidean indicators. This crate classifies
//! constructs, admits only log-ratio or logistic-normal coordinates, aggregates
//! plausible-value loadings on a CPU `f64` path, and refuses causal language
//! from temporal precedence, document linkage, event tracking, or prediction.
""",
        """//! Raw topic proportions are not unconstrained structural indicators. This
//! crate classifies constructs, admits mapped log-ratio/logistic-normal inputs,
//! distinguishes ALR from orthonormal ILR geometry, averages loading point
//! estimates across posterior draws on a CPU `f64` path without claiming Rubin
//! uncertainty pooling, and refuses causal language from non-identifying cues.
""",
        "crate claim boundary",
    )
    lib = lib.replace("plausible_value_mean", "posterior_draw_point_estimate_mean")
    lib = lib.replace(
        "recover_loading_from_plausible_values",
        "recover_loading_point_estimate_mean",
    )
    lib = lib.replace(
        "/// Arithmetic mean of plausible-value draws.",
        "/// Arithmetic mean of posterior-draw point estimates.",
    )
    lib = lib.replace(
        "/// Average OLS loadings across posterior indicator draws.",
        "/// Average OLS loading point estimates across posterior indicator draws.",
    )
    lib_path.write_text(lib, encoding="utf-8")

    test_path = Path("crates/psychometric_core/tests/esem_input_recovery_contract.rs")
    tests = test_path.read_text(encoding="utf-8")
    tests = tests.replace("plausible_value_mean", "posterior_draw_point_estimate_mean")
    tests = tests.replace(
        "recover_loading_from_plausible_values",
        "recover_loading_point_estimate_mean",
    )
    tests = tests.replace(
        "plausible_value_mean_recovers_true_loading_under_symmetric_draw_noise",
        "posterior_draw_point_estimate_mean_recovers_under_symmetric_draw_noise",
    )
    tests = tests.replace("plausible-value loading", "posterior-draw point-estimate loading")
    tests = tests.replace("plausible-value RMSE", "posterior-draw point-estimate RMSE")
    tests = tests.replace(
        "IndicatorKind::AdditiveLogRatio.is_valid_psychometric_input()",
        "IndicatorKind::AdditiveLogRatio.is_valid_structural_input()",
    )
    tests = tests.replace(
        "IndicatorKind::IsometricLogRatio.is_valid_psychometric_input()",
        "IndicatorKind::IsometricLogRatio.is_valid_structural_input()",
    )
    tests = tests.replace(
        "IndicatorKind::LogisticNormal.is_valid_psychometric_input()",
        "IndicatorKind::LogisticNormal.is_valid_structural_input()",
    )
    tests = tests.replace(
        "IndicatorKind::RawProportion.is_valid_psychometric_input()",
        "IndicatorKind::RawProportion.is_valid_structural_input()",
    )
    test_path.write_text(tests, encoding="utf-8")


def update_architecture_and_research() -> None:
    """Describe the implemented slice without claiming ESEM or Rubin pooling."""
    architecture_path = Path("ARCHITECTURE.md")
    architecture = architecture_path.read_text(encoding="utf-8")
    architecture = architecture.replace(
        "posterior-aware ESEM/DSEM input gates and CPU `f64` loading recovery",
        "posterior-aware structural input gates and CPU `f64` loading point-estimate recovery",
    )
    architecture_path.write_text(architecture, encoding="utf-8")

    readme_path = Path("README.md")
    readme = readme_path.read_text(encoding="utf-8")
    readme = readme.replace(
        "crates/psychometric_core",
        "crates/psychometric_core  # construct/input gates; not a full ESEM/DSEM estimator",
        1,
    )
    readme_path.write_text(readme, encoding="utf-8")

    research_path = Path("docs/research/posterior-esem-input-gates.md")
    research = research_path.read_text(encoding="utf-8")
    research = replace_once(
        research,
        """3. admit only additive log-ratio, isometric log-ratio, or logistic-normal coordinates;
4. recover a reflective loading by ordinary least squares on a CPU `f64` path;
5. average recovered loadings across posterior indicator draws (plausible values);
""",
        """3. admit ALR, ILR, or logistic-normal coordinates as unconstrained structural inputs while reserving orthonormal Aitchison-distance claims for ILR;
4. recover a reflective loading point estimate by ordinary least squares on a CPU `f64` path;
5. average recovered loading point estimates across posterior indicator draws without claiming Rubin within/between uncertainty pooling;
""",
        "research scope",
    )
    research = replace_once(
        research,
        """- **Plausible-value loading** is the arithmetic mean of \\(\\hat\\lambda_d\\) across posterior indicator draws (Mislevy, 1991).
""",
        """- **Posterior-draw loading point estimate** is the arithmetic mean of \\(\\hat\\lambda_d\\) across draws. This narrow slice does not compute within-draw variance, between-draw variance, total variance, degrees of freedom, or Rubin-style pooled uncertainty; Mislevy (1991) motivates the future full posterior-propagation contract rather than validating this point-estimate shortcut.
""",
        "research formula claim",
    )
    research = research.replace(
        "symmetric plausible-value draw noise cancels in the pooled loading",
        "symmetric posterior-draw point-estimate noise cancels in the arithmetic mean",
    )
    research_path.write_text(research, encoding="utf-8")

    adr_path = Path("docs/adr/0005-posterior-esem-dsem.md")
    adr = adr_path.read_text(encoding="utf-8")
    adr = adr.replace(
        "CPU `f64` OLS and plausible-value loading recovery",
        "CPU `f64` OLS and posterior-draw loading point-estimate averaging (not Rubin variance pooling)",
    )
    decision_anchor = (
        "Topic proportions are not treated as error-free ordinary indicators. TEPP uses "
        "logistic-normal latent coordinates or valid orthonormal log-ratio coordinates and "
        "propagates topic posterior uncertainty through plausible values or a joint "
        "text-measurement/structural model.\n"
    )
    decision_replacement = decision_anchor + (
        "The current executable slice only averages loading point estimates across posterior "
        "draws. It does not yet pool within-draw and between-draw uncertainty and therefore "
        "does not satisfy the full posterior-propagation decision by itself.\n"
    )
    adr = replace_once(adr, decision_anchor, decision_replacement, "ADR current-slice boundary")
    adr_path.write_text(adr, encoding="utf-8")

    adr_index_path = Path("docs/adr/README.md")
    adr_index = adr_index_path.read_text(encoding="utf-8")
    adr_index = adr_index.replace(
        "Input gates, plausible-value loading recovery, and causal-refusal are on the active PR",
        "Input gates, posterior-draw loading point-estimate averaging, and causal-refusal are on the active PR; Rubin uncertainty pooling remains target work",
    )
    adr_index_path.write_text(adr_index, encoding="utf-8")


def restore_shared_ledgers() -> None:
    """Reapply the PR 49 slice to main-owned conflict-resolved ledgers."""
    changelog_path = Path("CHANGELOG.md")
    changelog = changelog_path.read_text(encoding="utf-8")
    item = (
        "- `psychometric_core` posterior-aware structural input gates: construct classification, "
        "refusal of raw-proportion Pearson/OLS, explicit ALR-versus-ILR geometry boundaries, CPU "
        "`f64` OLS recovery, posterior-draw loading point-estimate averaging without Rubin "
        "uncertainty claims, invariance-gated latent-mean comparison, and causal-heuristic refusal "
        "(ADR 0005 first production slice; no new migration).\n"
    )
    if item not in changelog:
        changelog = replace_once(changelog, "### Added\n\n", "### Added\n\n" + item, "CHANGELOG marker")
        changelog_path.write_text(changelog, encoding="utf-8")

    trace_path = Path("docs/TRACEABILITY.md")
    trace = trace_path.read_text(encoding="utf-8")
    trace = replace_once(
        trace,
        "| posterior ESEM / longitudinal invariance / DSEM | ADR 0005 | future `psychometric_core` | accepted-target |",
        "| posterior ESEM / longitudinal invariance / DSEM | ADR 0005 | `psychometric_core` construct/input gates, true-loading OLS recovery, and posterior-draw point-estimate averaging on the active PR; full ESEM/DSEM and Rubin/joint uncertainty propagation remaining | partial |",
        "trace psychometric row",
    )
    trace_path.write_text(trace, encoding="utf-8")

    validation_path = Path("docs/validation/temporal-event-foundation.md")
    validation = validation_path.read_text(encoding="utf-8")
    row = (
        "| Psychometric structural input gates | `psychometric_core` | accepted-target | active PR | "
        "construct-class refusal + ALR/ILR boundary + true-loading RMSE + posterior-draw point-estimate "
        "mean; full ESEM/DSEM/Rubin uncertainty remaining | ADR 0005; "
        "`docs/research/posterior-esem-input-gates.md` |\n"
    )
    if row not in validation:
        marker = (
            "| Versioned API/export contracts | `tepp_api` | implemented-main | naruon HTTP interchange | "
            "unknown-field/version/limit + naruon HTTPS interchange tests | Task 12 / PR #21; live HTTP service remaining |\n"
        )
        validation = replace_once(validation, marker, marker + row, "validation API row")
        validation_path.write_text(validation, encoding="utf-8")


update_indicator_contract()
update_posterior_summary_contract()
update_public_api_and_tests()
update_architecture_and_research()
restore_shared_ledgers()
