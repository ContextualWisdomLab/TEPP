# TEPP PRD v0.4 amendment — Longitudinal event-time ownership

**Parent product baseline:** `docs/product/prd-v0.4-approved.md`  
**Amendment:** v0.4-LONGITUDINAL-TIME-1  
**Status:** Approved clarification; protected-main implementation remains pending  
**Recorded:** 2026-09-01

This amendment narrows implementation authority without changing TEPP's approved product thesis or scientific claims.

TEPP Longitudinal Modeling owns temporal/event composition for longitudinal psychometrics. Product APIs that interpret a duration as substantive event time must accept an event-time domain value rather than a bare numeric duration. This preserves the PRD's distinction among event/valid, assertion, document, system, availability time, and knowledge cutoff and prevents measurement occasion or another clock from being silently substituted for substantive event time.

For event-time lagged association, the product contract requires lagged covariance, both occasion-specific marginal variances, and an admitted positive finite event-time interval. Nonstationary correlation must not replace the later marginal variance with the earlier one.

CWC-then-irregular residual log-rate is Longitudinal Modeling composition, not a generic psychometric kernel. Consecutive unit-mean-centered pairs keep typed event-time intervals. The pairwise mean of the Driver, Oud, and Voelkle (2017, Eq. 3) scalar inverse on nonzero same-sign residuals is not raw-process drift (Curran & Bauer, 2011, Eq. 36). Already-centered irregular pairs may recover that inverse, including `ln(0.5)`. This is not DSEM and not Newton least-squares.

Occasion-mean event-time composition is also Longitudinal Modeling composition. For Hamaker, Kuiper, and Grasman (2015, Eq. 1a), the product may form `p_it = x_it - μ_t` only from an occasion-specific group mean on numeric event time. Numeric `-0.0` and `+0.0` are one occasion, a unit may contribute at most one observation to an occasion, each admitted occasion must retain multilevel evidence, and consecutive unit-specific pairs keep their own positive finite event-time interval. Occasion means must be deterministic under row permutation and must remain recoverable when the final mean is representable even if a naive intermediate same-sign sum would overflow. These deviations are not person-mean CWC residuals and are not within-person effects by themselves; stable between-person differences may remain. TEPP therefore refuses promoting the resulting occasion-residual log-rate to RI-CLPM, DSEM, causal, or within-person authority.

For the scalar stationary Driver, Oud, and Voelkle (2017) p. 16 `discreteDRIFTstd` mapping, TEPP may expose a deterministic event-time transform only when stable drift and positive stationary within-person variance are established. Numerical equality with `exp(a Δt)` in that scalar stationary special case does not make unstandardised `discreteDRIFT` and `discreteDRIFTstd` the same estimand. When the stationary variance cancels algebraically from a standardized scalar map, positive real-valued stationarity is established from finite positive diffusion and stable finite drift; the cancelled variance is not required to fit in binary64 if the final standardized result is representable. Conversely, a nonzero exact transition that rounds to a false endpoint such as `1.0` or `0.0` remains fail-closed rather than being reported as no change or complete dissipation.

Reusable static/generalized-mixed/dependence-aware psychometric arithmetic remains owned by `ContextualWisdomLab/fast-mlsirm`; TEPP consumes versioned contracts through an anti-corruption layer and does not retain duplicate production kernels. LLM execution remains `contextual-orchestrator`-owned and cannot substitute for numerical estimation or scientific acceptance.

Acceptance for this amendment requires typed event-time admission in the public longitudinal boundary, regression coverage for wrong-clock-shaped numeric input and extreme finite binary64 cases, explicit occasion-vs-CWC claim-boundary recovery on the same longitudinal panel, current-head Rust/documentation/security evidence, and normal protected-main integration. Branch-local implementation does not constitute released product capability.
