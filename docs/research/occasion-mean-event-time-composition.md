# Occasion-mean event-time composition

Status: active PR evidence on #310; not protected-main or release evidence.

## Scientific claim

Hamaker, Kuiper, and Grasman (2015, Eq. 1a) write an observed score as an occasion-specific mean plus a deviation, `x_it = μ_t + p_it`. In TEPP, `μ_t` is the group mean for one numeric event-time occasion and `p_it = x_it - μ_t`. This is distinct from person/unit-mean centering (CWC), a sample-wide grand mean, and person-specific detrending.

The resulting `p_it` deviations are not, by themselves, within-person effects. Stable between-person differences can remain in them; Hamaker et al.'s random-intercept extension removes a person-specific stable component before interpreting within-person dynamics. TEPP therefore refuses promoting the occasion-mean residual log-rate to a within-person lag estimand.

Aligned occasions may be irregularly spaced. Each consecutive unit-specific pair retains its own positive finite `EventTimeInterval`; unequal event intervals are not pooled as one discrete coefficient.

## Admission and numerical invariants

`longitudinal_core::center_occasion_mean_event_lags` owns the temporal composition boundary.

- Occasion identity is numeric event time. IEEE-754 `-0.0` and `+0.0` are one occasion, not two bit-pattern identities.
- A unit may contribute at most one observation to one numeric occasion.
- Each admitted occasion contains at least two distinct units, and at least two units contribute consecutive lags. This prevents an atomistic one-unit series from manufacturing multilevel evidence.
- Scores and event times must be finite. Consecutive event intervals must be finite and strictly positive.
- A representable occasion mean is not rejected merely because a same-sign intermediate partial sum would overflow binary64. The mean path preserves cancellation before bounded same-sign averaging and still fails closed if the final mean or centered residual is non-representable.
- Occasion means are bit-stable under permutation of the same admitted rows. Same-sign retained values are put in a deterministic total order before incremental averaging; input arrival order is not scientific evidence.
- `recover_occasion_mean_centered_irregular_residual_log_rate` composes the centered pairs with the existing Longitudinal Modeling exact-log-rate boundary. It does not create a second static psychometric arithmetic authority.

## Test trace

Current #310 branch tests:

- `crates/longitudinal_core/tests/occasion_mean_event_time_contract.rs::signed_zero_is_one_numeric_occasion` — `-0.0`/`+0.0` share one occasion and the known scalar event-time log-rate is recovered.
- `signed_zero_duplicate_unit_is_rejected_as_one_occasion` — the same unit cannot use signed zero to bypass duplicate occasion admission.
- `representable_occasion_mean_is_not_rejected_for_intermediate_sum_overflow` — `[0.75·MAX, 0.75·MAX, -0.5·MAX]` at one occasion retains a finite representable mean and finite centered residuals rather than failing on the naive partial sum.
- `occasion_mean_is_bit_stable_under_row_permutation` — the same three-unit occasion with `{1, nextafter(1,+∞), MAX/2}` yields bit-identical centered pairs when row arrival order changes.

The initial occasion-composition RED is `75b0184d2f6341ef23cf14fc84398c68d8d95d22`. Deterministic-order RED `8a59019ed3112a3e27dd0dcd1b6b86d8d45e5435` exposes order-dependent same-sign averaging; causal repair `465d139dce6101c4958c8b0827b6ef5d674b54c2` orders same-sign values before averaging. The owner-correct implementation remains in `crates/longitudinal_core/src/occasion_mean.rs`. Only the current exact branch head may be used for merge evidence.

## Claim boundary

This slice is an occasion-mean residual event-time association. It is not RI-CLPM, DSEM, a causal effect, or proof of within-person dynamics. LLM output does not estimate or activate this quantity. Reusable static psychometric arithmetic remains fast-mlsirm-owned and may be consumed only through an immutable released contract.

## Primary source

Hamaker, E. L., Kuiper, R. M., & Grasman, R. P. P. P. (2015). A critique of the cross-lagged panel model. *Psychological Methods, 20*(1), 102–116. https://doi.org/10.1037/a0038889

Repository evidence was cross-checked on 2026-09-03 against the Utrecht University/UvA-DARE final-published-version record and PubMed record (PMID 25822208). The repository record identifies the DOI, journal, volume, issue, pages, and peer-reviewed publication. No timezone-suffixed access timestamp is used as scientific evidence.
