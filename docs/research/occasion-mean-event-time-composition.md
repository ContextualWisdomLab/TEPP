# Occasion-mean event-time composition

Status: active PR evidence on #310; not protected-main or release evidence.

## Scientific claim

Hamaker, Kuiper, and Grasman (2015, Eq. 1a) write an observed score as an occasion-specific mean plus a deviation, `x_it = μ_t + p_it`. In TEPP, `μ_t` is the group mean for one numeric event-time occasion and `p_it = x_it - μ_t`. This is distinct from person/unit-mean centering (CWC), a sample-wide grand mean, and person-specific detrending.

The resulting `p_it` deviations are not, by themselves, within-person effects. Stable between-person differences can remain in them; Hamaker et al.'s random-intercept extension removes a person-specific stable component before interpreting within-person dynamics. TEPP therefore refuses promoting the occasion-mean residual log-rate to a within-person lag estimand.

Aligned occasions may be irregularly spaced. Each consecutive unit-specific pair retains its own positive finite `EventTimeInterval`; unequal event intervals are not pooled as one discrete coefficient.

## Admission and numerical invariants

`longitudinal_core::center_occasion_mean_event_lags` owns the temporal composition boundary.

- Occasion identity is numeric event time. IEEE-754 `-0.0` and `+0.0` are one occasion, not two bit-pattern identities.
- Exact zero Hamaker Eq. 1a residuals use one public encoding: canonical `+0.0`. A `-0.0` observed score equal to a canonical `+0.0` occasion mean must not leak a signed-zero public residual. IEEE-754 signed zeros are numerically equal but bit-distinct; exact zero occasion-mean deviation has no positive/negative measurement direction. Private numerical intermediates and caller-constructed already-centered pairs may still retain signed zero. This is the same public-identity contract as CWC and `decompose_within_between`; it is not a license to treat occasion-mean residuals as within-person or RI-CLPM lags.
- A unit may contribute at most one observation to one numeric occasion.
- Each admitted occasion contains at least two distinct units, and at least two units contribute consecutive lags. This prevents an atomistic one-unit series from manufacturing multilevel evidence.
- Scores and event times must be finite. Consecutive event intervals must be finite and strictly positive.
- A representable occasion mean is not rejected merely because a same-sign intermediate partial sum would overflow binary64. Mixed signs cancel before bounded same-sign averaging; same-sign values are normalized by a finite maximum magnitude, accumulated in deterministic total order with compensation, divided by count, and then rescaled.
- A representable subnormal occasion mean must retain IEEE-754 round-to-nearest, ties-to-even behavior. In particular, the mean of one and two minimum-subnormal ULPs is 1.5 ULPs and rounds to the even two-ULP representation rather than losing the half-ULP update in an incremental recurrence.
- Occasion means are bit-stable under permutation of the same admitted rows. Input arrival order is not scientific evidence.
- `recover_occasion_mean_centered_irregular_residual_log_rate` composes the centered pairs with the existing Longitudinal Modeling exact-log-rate boundary. It does not create a second static psychometric arithmetic authority.

## Test trace

Current #310 branch tests:

- `crates/longitudinal_core/tests/occasion_mean_event_time_contract.rs::signed_zero_is_one_numeric_occasion` — `-0.0`/`+0.0` share one occasion and the known scalar event-time log-rate is recovered.
- `crates/longitudinal_core/tests/occasion_mean_signed_zero_contract.rs` — a `-0.0` observed score equal to a canonical `+0.0` occasion mean cannot keep a signed-zero public residual; nonzero mixed-sign residuals stay bit-identical.
- `occasion_mean_residual_rate_is_not_cwc_rate_on_the_same_panel` — the same time-varying-group-mean panel recovers the known Hamaker Eq. 1a occasion-residual rate while person-mean CWC does not recover that rate. This preserves the valid scientific distinction from Draft #486 without retaining its wrong `psychometric_core` ownership.
- `signed_zero_duplicate_unit_is_rejected_as_one_occasion` — the same unit cannot use signed zero to bypass duplicate occasion admission.
- `representable_occasion_mean_is_not_rejected_for_intermediate_sum_overflow` — `[0.75·MAX, 0.75·MAX, -0.5·MAX]` at one occasion retains a finite representable mean and finite centered residuals rather than failing on the naive partial sum.
- `representable_subnormal_occasion_mean_preserves_round_to_even` — `[1 ULP, 2 ULP]` at one occasion requires the 1.5-ULP mathematical mean to round ties-to-even to 2 ULP; the first centered residual is therefore exactly `-1 ULP` rather than zero.
- `occasion_mean_is_bit_stable_under_row_permutation` — the same three-unit occasion with `{1, nextafter(1,+∞), MAX/2}` yields bit-identical centered pairs when row arrival order changes.
- `sparse_unaligned_and_nonfinite_occasion_inputs_fail_closed` — empty/singleton payloads, unaligned one-unit occasions, and non-finite event-time input fail closed at the Longitudinal boundary.
- `singleton_wave_unit_does_not_manufacture_or_block_lag_evidence` — a one-wave unit may contribute to an admitted occasion mean but cannot count toward the two-unit lag-evidence floor; two genuine lag-contributing units still recover the known rate.

The initial occasion-composition RED is `75b0184d2f6341ef23cf14fc84398c68d8d95d22`. Deterministic-order RED `8a59019ed3112a3e27dd0dcd1b6b86d8d45e5435` exposes order-dependent same-sign averaging; causal repair `465d139dce6101c4958c8b0827b6ef5d674b54c2` orders same-sign values before averaging. Successor-evidence commit `b9e952bb8a893f62aaead59cdf825b5e3c6251c6` ports #486's valid same-panel occasion-vs-CWC scientific claim boundary into the owner-correct Longitudinal test surface. Admission-evidence commit `aad56b502bbdfab08ba896b7d3560884c87fc589` preserves #486's sparse/unaligned/non-finite and singleton-wave cases without retaining its wrong crate ownership. Subnormal-rounding RED `9aff817f9e0f82b9cdb2077f3f62bb3e6a987103` exposes the incremental same-sign mean's loss of a half-ULP update at the binary64 floor; causal repair `40e057b83980a0cc501ad936c02e2d59f90a6fe9` replaces that recurrence with deterministic normalized compensated averaging. Signed-zero residual RED `crates/longitudinal_core/tests/occasion_mean_signed_zero_contract.rs` drives `center_occasion_mean_event_lags` with a T=3 mixed-sign panel whose middle score is `-0.0`; causal repair `238c4def7a5a5f19963d2e5cc936dd750d55c5a1` canonicalizes only a validated exact-zero occasion-mean residual to public `+0.0`. IEEE Std 754-2019 remains the active published floating-point standard while IEEE P754 is the active revision project as of 2026-09-03. The owner-correct implementation remains in `crates/longitudinal_core/src/occasion_mean.rs`. Only the current exact branch head may be used for merge evidence.

## Claim boundary

This slice is an occasion-mean residual event-time association. It is not RI-CLPM, DSEM, a causal effect, or proof of within-person dynamics. LLM output does not estimate or activate this quantity. Reusable static psychometric arithmetic remains fast-mlsirm-owned and may be consumed only through an immutable released contract.

## Primary source

Hamaker, E. L., Kuiper, R. M., & Grasman, R. P. P. P. (2015). A critique of the cross-lagged panel model. *Psychological Methods, 20*(1), 102–116. https://doi.org/10.1037/a0038889

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE.

Repository evidence was cross-checked on 2026-09-03 against the Utrecht University/UvA-DARE final-published-version record and PubMed record (PMID 25822208). The repository record identifies the DOI, journal, volume, issue, pages, and peer-reviewed publication. No timezone-suffixed access timestamp is used as scientific evidence.
