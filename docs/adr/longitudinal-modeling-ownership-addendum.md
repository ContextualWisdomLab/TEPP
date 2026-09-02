# ADR 0005 ownership addendum — Longitudinal Modeling

**Parent decision:** ADR 0005 — Posterior-aware ESEM/DSEM and structural interpretation

**Decision identity:** ADR 0005; this addendum does not mint a new ADR number

**Status:** Accepted clarification

**Recorded:** 2026-09-01

## Decision clarification

Temporal/event composition of longitudinal psychometric quantities belongs to the TEPP Longitudinal Modeling bounded context. Public operations whose meaning depends on substantive event time must therefore expose an event-time domain type at the context boundary rather than accepting an unqualified numeric duration.

`longitudinal_core` owns this TEPP temporal composition. `fast-mlsirm` remains the canonical owner of reusable static/generalized-mixed/dependence-aware psychometric kernels. A numerically reusable primitive must migrate through the fast-mlsirm Published Language/ACL boundary rather than turning `longitudinal_core` into a second static psychometric kernel.

For the scalar Driver, Oud, and Voelkle (2017) p. 16 `discreteDRIFTstd` special case, TEPP owns only the event-time composition and admissibility policy. The function requires stable negative drift, positive stationary within-person variance, and an admitted `EventTimeInterval`. In this scalar standardisation the stationary variance cancels algebraically: finite positive diffusion plus stable finite drift establish a positive real-valued stationary variance even when its materialized `f64` value would underflow or overflow. The cancelled intermediate therefore must not reject a representable final standardized map. An exact nonzero transition that itself collapses to a false binary64 endpoint remains fail-closed.

The same numerical rule applies to TEPP's research-candidate scalar diffusion-standardisation compositions: a cancelled stationary variance is not a second admission gate, while the final standardized quantity must remain representable. This does not promote those named diffusion extensions to canonical ctsem output and does not move reusable static arithmetic ownership out of fast-mlsirm.

The lagged-correlation boundary similarly requires both occasion-specific marginal variances and an `EventTimeInterval`; a covariance divided only by the earlier variance is not exposed as an autocorrelation.

CWC-then-irregular residual log-rate is also Longitudinal Modeling composition. Person-mean centering of a time-related series is not raw-process drift (Curran & Bauer, 2011, pp. 583–619; PMC3059070 XML opened 2026-09-02; Eq. 36). The unique pairwise-mean-after-CWC evidence from Draft #327 is folded here with typed `EventTimeInterval` rather than grown on `psychometric_core`. Already-centered irregular pairs `(1, 0.5)` recover `ln(0.5)` only for `Δt = 1`; for a general admitted interval the exact map is `ln(0.5) / Δt`. CWC of a raw AR path does not recover raw-process drift.

Occasion-mean event-time composition is likewise Longitudinal Modeling authority. Hamaker, Kuiper, and Grasman (2015, Eq. 1a) write `x_it = μ_t + p_it`, where `μ_t` is the occasion-specific group mean. TEPP forms those deviations only across numeric event-time occasions, treats IEEE-754 `-0.0` and `+0.0` as one numeric occasion, admits at most one observation per unit and occasion, and preserves each consecutive unit-specific interval as an `EventTimeInterval`. The resulting `p_it` still contains stable between-person differences and therefore does not become a within-person lag merely because it is lagged. It is distinct from person-mean CWC, a sample-wide grand mean, RI-CLPM, and DSEM. The occasion-specific mean must remain deterministic under row permutation and must not reject a representable final mean merely because a naive same-sign intermediate sum would overflow.

## DDD consequences

- `psychometric_core` is not the authority for temporal transforms merely because an earlier branch placed them there.
- `EventTimeInterval` is a value object of Longitudinal Modeling. Assertion-, document-, system-, availability-, and method-occasion intervals require explicit owning-context conversion before they can be admitted as substantive event time.
- Occasion identity is numeric event time, not the raw binary encoding of an otherwise equal numeric zero.
- One transform, route, clock, or refusal does not create a bounded context or a new ADR identity.
- Compatibility adapters may preserve public callers during a landed migration, but domain ownership and dependency direction must remain explicit.

## Verification

PR #310 is the current landing vehicle for this clarification. Its RED lineage includes an extreme stable-drift case that failed because `-2a` overflowed despite a representable stationary variance, a cancelled-stationary case in which the final scalar standardisation is representable even though `q / (-2a)` itself lies outside binary64 range, and a typed event-time contract that could not compile before the value object existed. The repaired source keeps actual stationary-variance recovery fail-closed when `p` itself is requested, but standardized scalar maps validate stationarity algebraically and avoid materializing an intermediate that cancels. The #327 fold adds CWC-then-pairwise-mean residual log-rate with Curran refusal and known-truth already-centered recovery of `ln(0.5)` at `Δt = 1`; arbitrary admitted intervals recover the exact `ln(0.5) / Δt` map.

The #486 fold is verified against the same landing vehicle rather than retained as a second `psychometric_core` authority. Its Hamaker claim boundary is implemented by `longitudinal_core::center_occasion_mean_event_lags` and `recover_occasion_mean_centered_irregular_residual_log_rate`. Regressions cover numeric signed-zero identity, duplicate-unit admission, intermediate-sum overflow with a representable occasion mean, permutation stability, known drift recovery, and the same-panel distinction between occasion-mean residuals and person-mean CWC residuals. The dedicated research note records the primary-source trace and explicitly refuses promotion to a within-person effect.

Protected-main maturity is not claimed until the exact landing head passes the live ruleset and is merged.
