"""Apply PR 48 stable log-ratio arithmetic and documentation repairs."""

from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    """Replace exactly one fragment or fail closed."""
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)


def update_coordinates() -> None:
    """Use stable log differences and max-shifted inverse softmax."""
    path = Path("crates/topic_measurement/src/coordinates.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        """/// For a `K`-part composition `θ` the image is the `K-1` vector
/// `y_k = ln(θ_k / θ_K)`. This is the logistic-normal coordinate system used
/// by correlated topic models and required before Euclidean or ESEM/DSEM work.
""",
        """/// For a `K`-part composition `θ` the image is the `K-1` vector
/// `y_k = ln(θ_k / θ_K)`. This reference-dependent, full-rank coordinate map
/// supports logistic-normal regression and ESEM/DSEM interfaces. It is not an
/// orthonormal isometry for Aitchison distance; use ILR coordinates when that
/// Euclidean geometry is the estimand.
""",
        "coordinate documentation",
    )
    text = replace_once(
        text,
        ".map(|part| (part / last).ln())",
        ".map(|part| part.ln() - last.ln())",
        "stable forward ALR",
    )
    old_inverse = """pub fn from_additive_log_ratio(coordinates: &[f64]) -> Result<Vec<f64>, TopicMeasurementError> {
    if coordinates.is_empty() {
        return Err(TopicMeasurementError::InvalidLogRatioDimension);
    }
    let mut exponentiated = Vec::with_capacity(coordinates.len());
    let mut denom = 1.0_f64;
    for &value in coordinates {
        if !value.is_finite() {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
        let exp = value.exp();
        if !exp.is_finite() {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
        denom += exp;
        exponentiated.push(exp);
    }
    if !denom.is_finite() || denom <= 0.0 {
        return Err(TopicMeasurementError::InvalidLogRatioDimension);
    }
    let mut simplex = Vec::with_capacity(coordinates.len() + 1);
    for exp in exponentiated {
        simplex.push(exp / denom);
    }
    simplex.push(1.0 / denom);
    Ok(simplex)
}
"""
    new_inverse = """pub fn from_additive_log_ratio(coordinates: &[f64]) -> Result<Vec<f64>, TopicMeasurementError> {
    if coordinates.is_empty() {
        return Err(TopicMeasurementError::InvalidLogRatioDimension);
    }
    let mut maximum = 0.0_f64;
    for &value in coordinates {
        if !value.is_finite() {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
        maximum = maximum.max(value);
    }

    let reference_weight = (-maximum).exp();
    if reference_weight == 0.0 {
        return Err(TopicMeasurementError::InvalidLogRatioDimension);
    }
    let mut shifted_weights = Vec::with_capacity(coordinates.len());
    let mut denominator = reference_weight;
    for &value in coordinates {
        let weight = (value - maximum).exp();
        if weight == 0.0 {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
        denominator += weight;
        shifted_weights.push(weight);
    }

    let mut simplex = Vec::with_capacity(coordinates.len() + 1);
    for weight in shifted_weights {
        simplex.push(weight / denominator);
    }
    simplex.push(reference_weight / denominator);
    Ok(simplex)
}
"""
    text = replace_once(text, old_inverse, new_inverse, "stable inverse ALR")
    text = replace_once(
        text,
        """/// Returns [`TopicMeasurementError::InvalidLogRatioDimension`] when the
/// coordinate vector is empty or contains a non-finite value.
""",
        """/// Returns [`TopicMeasurementError::InvalidLogRatioDimension`] when the
/// coordinate vector is empty, non-finite, or would underflow a part to zero
/// in the strictly positive `f64` simplex representation.
""",
        "inverse error documentation",
    )
    text = replace_once(
        text,
        "fn two_part_equal_shares_are_zero_and_overflow_fails_closed()",
        "fn two_part_equal_shares_are_zero_and_unrepresentable_extremes_fail_closed()",
        "internal test name",
    )
    path.write_text(text, encoding="utf-8")


def update_error_and_crate_docs() -> None:
    """Describe representability and ALR geometry without overclaiming isometry."""
    error_path = Path("crates/topic_measurement/src/error.rs")
    error_text = error_path.read_text(encoding="utf-8")
    error_text = replace_once(
        error_text,
        "    /// Log-ratio vector is empty or non-finite.\n",
        "    /// Log-ratio vector is empty, non-finite, or not representable as a strictly positive `f64` simplex.\n",
        "error variant documentation",
    )
    error_path.write_text(error_text, encoding="utf-8")

    lib_path = Path("crates/topic_measurement/src/lib.rs")
    lib_text = lib_path.read_text(encoding="utf-8")
    lib_text = replace_once(
        lib_text,
        """//! Raw topic proportions are not Euclidean indicators. Downstream network and
//! psychometric analysis must use additive log-ratio (logistic-normal) maps
//! rather than TF-IDF, BM25, or keyword scores as inferential coordinates.
""",
        """//! Raw topic proportions are compositional rather than unconstrained Euclidean
//! indicators. ALR supplies a reference-dependent full-rank logistic-normal map
//! for regression and psychometric interfaces; it is not an orthonormal
//! Aitchison-distance isometry. Distance-based Aitchison geometry requires ILR.
//! TF-IDF, BM25, and keyword scores remain forbidden inferential coordinates.
""",
        "crate geometry documentation",
    )
    lib_path.write_text(lib_text, encoding="utf-8")


def update_research_and_adr() -> None:
    """Clarify ALR versus ILR and record stable arithmetic evidence."""
    research_path = Path("docs/research/topic-logratio-coordinates.md")
    research = research_path.read_text(encoding="utf-8")
    research = replace_once(
        research,
        """1. raw topic proportions are not Euclidean indicators;
2. additive log-ratio coordinates implement the logistic-normal map used by correlated topic models;
3. inverse ALR recovers a known simplex with a computed RMSE;
4. TF-IDF, BM25, and keyword scores are refused as inferential coordinates.
""",
        """1. raw topic proportions are compositional rather than unconstrained Euclidean indicators;
2. additive log-ratio coordinates implement the reference-dependent logistic-normal map used by correlated topic models;
3. ALR is full rank but not an orthonormal Aitchison-distance isometry; ILR is required when that Euclidean geometry is the estimand;
4. max-shifted inverse ALR and log-difference forward ALR recover representable extreme coordinates without overflow;
5. TF-IDF, BM25, and keyword scores are refused as inferential coordinates.
""",
        "research scope",
    )
    research = replace_once(
        research,
        """Aitchison and Shen (1980) define the logistic-normal family via the additive log-ratio map; Aitchison (1982) is the compositional-data authority that forbids treating parts of a whole as unconstrained Euclidean coordinates. Blei and Lafferty (2007) use that same map for correlated topic models. TEPP therefore converts a strictly positive unit simplex through `additive_log_ratio` before any Euclidean or psychometric operation, and recovers the simplex with `from_additive_log_ratio` (Aitchison & Shen, 1980; Aitchison, 1982; Blei & Lafferty, 2007).
""",
        """Aitchison and Shen (1980) define the logistic-normal family via the additive log-ratio map; Aitchison (1982) is the compositional-data authority that forbids treating parts of a whole as unconstrained Euclidean coordinates. Blei and Lafferty (2007) use that same reference-dependent map for correlated topic models. TEPP therefore uses `additive_log_ratio` for logistic-normal regression and psychometric interfaces, but does not claim that ALR preserves Aitchison distance. Analyses whose estimand is orthonormal Euclidean Aitchison geometry must use ILR. `from_additive_log_ratio` uses a max-shifted inverse softmax and the forward map subtracts logarithms, avoiding avoidable exponential and ratio overflow while failing closed when an `f64` simplex part would underflow to zero (Aitchison & Shen, 1980; Aitchison, 1982; Blei & Lafferty, 2007).
""",
        "research application",
    )
    research = replace_once(
        research,
        """- closed-form simplex `(2,3,1)/6` maps to `(ln 2, ln 3)` and inverts with computed RMSE below `1e-15`;
- equal shares map to a zero ALR vector;
""",
        """- closed-form simplex `(2,3,1)/6` maps to `(ln 2, ln 3)` and inverts with computed RMSE below `1e-15`;
- representable coordinates `(710, 709)` round-trip through the max-shifted inverse without exponential overflow;
- extremes that would underflow a strictly positive `f64` simplex part fail closed;
- equal shares map to a zero ALR vector;
""",
        "research verification",
    )
    research_path.write_text(research, encoding="utf-8")

    adr_path = Path("docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md")
    adr = adr_path.read_text(encoding="utf-8")
    adr = replace_once(
        adr,
        """- topic proportions are compositional and downstream network/psychometric analysis uses logistic-normal coordinates or valid orthonormal log-ratio coordinates;
""",
        """- topic proportions are compositional and downstream network/psychometric analysis uses logistic-normal coordinates or valid orthonormal log-ratio coordinates;
- ALR is a reference-dependent full-rank logistic-normal map, not an Aitchison-distance isometry; distance-based Euclidean Aitchison geometry uses an orthonormal ILR basis;
""",
        "ADR ALR/ILR boundary",
    )
    adr_path.write_text(adr, encoding="utf-8")


def restore_conflict_resolved_ledgers() -> None:
    """Reapply the topic slice to main-owned shared ledgers after the merge."""
    changelog_path = Path("CHANGELOG.md")
    changelog = changelog_path.read_text(encoding="utf-8")
    item = (
        "- `topic_measurement` logistic-normal additive log-ratio coordinates: "
        "fail-closed simplex validation, max-shifted stable ALR/inverse maps with "
        "true-parameter round-trip RMSE, explicit ALR-versus-ILR geometry boundary, "
        "and refusal of TF-IDF/BM25/keyword scores as inferential topic coordinates "
        "(ADR 0012 first production slice; no new migration).\n"
    )
    if item not in changelog:
        changelog = replace_once(changelog, "### Added\n\n", "### Added\n\n" + item, "changelog marker")
        changelog_path.write_text(changelog, encoding="utf-8")

    trace_path = Path("docs/TRACEABILITY.md")
    trace = trace_path.read_text(encoding="utf-8")
    trace = replace_once(
        trace,
        "| TRSL-TM temporal/relational topic posterior and backend compatibility | ADR 0012; ADR 0004 | future `topic_measurement` | accepted-target |",
        "| TRSL-TM temporal/relational topic posterior and backend compatibility | ADR 0012; ADR 0004 | `topic_measurement` stable ALR coordinates on the active PR; temporal STM backend remaining | partial |",
        "trace topic backend",
    )
    trace = replace_once(
        trace,
        "| no default stopword deletion / no TF-IDF-BM25 inferential weighting | ADR 0004/0012; PRD/TRD | future semantic/method-source model | accepted-target |",
        "| no default stopword deletion / no TF-IDF-BM25 inferential weighting | ADR 0004/0012; PRD/TRD | `topic_measurement::refuse_lexical_inferential_weight` on the active PR; preprocessing pipeline remaining | partial |",
        "trace lexical refusal",
    )
    trace_path.write_text(trace, encoding="utf-8")

    validation_path = Path("docs/validation/temporal-event-foundation.md")
    validation = validation_path.read_text(encoding="utf-8")
    row = (
        "| Logistic-normal topic coordinates | `topic_measurement` | active-PR | stable ALR + lexical refusal | "
        "known-simplex and extreme-coordinate RMSE, ALR/ILR boundary | ADR 0012; "
        "`docs/research/topic-logratio-coordinates.md` |\n"
    )
    if row not in validation:
        marker = (
            "| Versioned API/export contracts | `tepp_api` | implemented-main | naruon HTTP interchange | "
            "unknown-field/version/limit + naruon HTTPS interchange tests | Task 12 / PR #21; live HTTP service remaining |\n"
        )
        validation = replace_once(validation, marker, marker + row, "validation API row")
        validation_path.write_text(validation, encoding="utf-8")


update_coordinates()
update_error_and_crate_docs()
update_research_and_adr()
restore_conflict_resolved_ledgers()
