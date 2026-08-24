# Temporal Event Foundation — validation and release-readiness report

**Status:** Living validation ledger for the Temporal/Event foundation program
**Last reviewed:** 2026-08-21
**Authority:** ADR 0014 (claim promotion), ADR 0007 (quality gates), AGENTS.md scientific acceptance

## Scope

This report tracks exact-head scientific and engineering evidence required before promoting foundation slices to release claims. It is not itself a release.

## Capability ledger

| Capability | Owning crate / contract | Protected-main maturity | Open PR status | Required evidence | Notes |
|---|---|---|---|---|---|
| Immutable evidence + spans | `evidence_core` | implemented-main | — | unit + wire + coverage | Task 2 |
| Six-clock temporal | `temporal_core` | implemented-main | — | unit + wire | Task 3 / PR #8 |
| Allen path-consistency | `temporal_core` | implemented-main | — | unit + budget tests | Task 4 / PR #9 |
| Event mention/instance | `event_core` | partial | — | unit + fail-closed promotion | Task 5 / PR #13 |
| Multiple membership | `membership_core` | partial | — | unit + ESS weights | Task 7 / PR #12 + #25 |
| Forward transition DAG | `relation_graph` | implemented-main | — | unit + cycle rejection | Task 6 / PR #14 |
| Bitemporal persistence + live SQL port | `persistence_postgres` | partial | backup/restore integrity | migration contracts + recording transport + optional PgPool + live CI + tenant RLS + `0005`/`0006` + event relation/mention/instance + source-artifact + audit-event + concurrent-write (#37–#43 implemented-main) + restore integrity probes (active PR) | Task 8 / PR #16 + #23 + #26 + #27 + #29 + #30–#43 + restore integrity |
| Leakage-safe splits | `corpus_split` | implemented-main | — | cutoff + co-partition tests | Task 9 / PR #17 |
| Truth corpora / manifests | `tepp_simulation` | implemented-main | — | deterministic generator tests | Task 10 / PR #18 |
| Recovery metrics | `validation_core` | implemented-main | — | RMSE/bias/coverage/MC gates | Task 11 / PR #19 |
| Versioned API/export contracts | `tepp_api` | implemented-main | naruon HTTP interchange | unknown-field/version/limit + naruon HTTPS interchange tests | Task 12 / PR #21; live HTTP service remaining |
| Psychometric structural input gates | `psychometric_core` | partial | stacked psychometric PR | construct-class refusal + ALR/ILR boundary + true-loading RMSE + posterior-draw point-estimate mean + Rubin `T` + CWC within/between + CWC contextual effect + event-time log-rate + constant- and time-varying-predictor discrete effects + exact scalar discrete process noise + lagged latent covariance and unconditional latent variance + stationary within-subject variance + trait-plus-state variance + observed-indicator variance + discrete latent mean (`T0MEANS`/`CINT`) + evolved observed mean (`τ + λ μ_t`; `τ + λ μ_0` is not `E(y_t)`) + contemporaneous `TDPREDEFFECT` impulse (`m x`; not `CINT`, not `TIPREDEFFECT`, not Voelkle Eq. 14) + Eq. 5 of that contemporaneous impulse (`τ + λ(μ_t + m x)`; `τ + λ μ_t` is not that observed mean) + time-independent `TIPREDEFFECT` increment (`A^{-1}[e^{A Δt} − I] B z`; not `CINT`, not `M x`, not Voelkle Eq. 14, not the coefficient `B`) + Eq. 5 of that increment (`τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)`; `τ + λ μ_t` is not that observed mean) + within-interval `TDPREDEFFECT` carry (`e^{A(t−u)} M x` for `t0 < u < t`; not the contemporaneous Dirac, not `CINT`, not `TIPREDEFFECT`, not Voelkle Eq. 14) + Eq. 5 of that carry (`τ + λ(μ_t + e^{a(t−u)} m x)`; `τ + λ μ_t` is not that observed mean) + §7.2 level-change `CINT` (`κ = −a m x`; Eq. 3 increment `(1 − e^{a Δt}) m x`) + §7.2 extra-process contribution (`a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)`; not `κ`, not the increment, not the Dirac; `ε ≥ 0` fails closed) + Eq. 5 of that extra-process contribution (`τ + λ(μ_t + a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)`; extra `LAMBDA` is 0; `τ + λ μ_t` is not that observed mean) + after-t0 extra-process `TDPREDEFFECT` (`a_{ηξ} x (e^{ε(t−u)} − e^{a(t−u)}) / (ε − a)` for `t0 < u < t`; Eq. 5 `τ + λ(μ_t + contribution(t−u))`; not the first-occasion extra-process observed mean; not the impulse-carry Dirac) + §7.2 `asymTIPREDEFFECT` (`-B z / a` for `a < 0`; not `B`, not the finite-interval increment, not `CINT`, not `M x`) + §7.2 `addedTIPREDVAR` (`(B / a)² v`; not `TRAITVAR`, not `asymDIFFUSION`, not `-B z / a`) + Table 2 `asymCINT` (`-κ / a` for `a < 0`; not `κ`, not the finite-interval increment, not `T0MEANS`, not `-B z / a`) + p. 16 stationary `T0MEANS` (`-κ / a + −B z / a`; not free `T0MEANS`, not `asymCINT` alone, not `asymTIPREDEFFECT` alone, not the finite-interval discrete mean) + Eq. 5 of that constrained mean (`τ + λ(−κ / a + −B z / a)`; `τ + λ μ_0` is not that observed mean; `τ + λ(−κ / a)` is not that observed mean when `B z ≠ 0`; `τ + λ μ_t` is not that observed mean; `MANIFESTMEANS` is not `E(y_0)`; the constrained latent mean is not `E(y_0)`) + stationary `T0VAR` (`trait + −q / (2 a) + (B / a)² v`; not free `T0VAR`, not `asymDIFFUSION` alone, not `TRAITVAR` alone, not `addedTIPREDVAR` alone; Eq. 5 is `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ` (`λ² p_0` is not `Var(y_0)`; `λ²(−q / (2 a)) + θ` is not `Var(y_0)` when trait or TI is nonzero; `MANIFESTVAR` is not `Var(y_0)`; the constrained latent variance is not `Var(y_0)`)) + Eq. 5 of that constrained variance (`λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ`; `MANIFESTVAR` is not `Var(y_0)`) + p. 16 `TDPREDEFFECTstd` (`m · √v / √(-q / (2 a))` after strictly positive `asymDIFFUSION` and TD predictor variance; not `TIPREDEFFECTstd` even when `M = B`; not intercept-style `A^{-1}[e^{A Δt} − I] M · √v / √p`; not trait-contaminated) + Table 3 / p. 16 `T0TDPREDEFFECTstd` (`t0_m · √v / √p_0` after strictly positive free `T0VAR` and TD predictor variance; not `TDPREDEFFECTstd`; not `T0TIPREDEFFECTstd` even when `t0_m = t0_b`; not trait-contaminated; free `T0VAR` does not require `a < 0`) + p. 16 `T0VARstd` (`p_0 / p_0 = 1` after strictly positive free `T0VAR`; not unstandardised `T0VAR`; not `T0TDPREDEFFECTstd`; not `addedT0TIPREDVAR`; free `T0VAR` does not require `a < 0`) + p. 16 `TRAITVARstd` (`trait / trait = 1` after strictly positive `TRAITVAR`; no ridge addend; not unstandardised `TRAITVAR`; not `T0VARstd` even when both equal 1; not `addedT0TIPREDVAR`; `TRAITVAR` does not require `a < 0`) + p. 16 `MANIFESTTRAITVARstd` (`ψ / ψ = 1` after strictly positive `MANIFESTTRAITVAR`; 2017-era source adds ridging; default ridge is 0; not unstandardised `MANIFESTTRAITVAR`; not `TRAITVARstd` even when both equal 1; not `MANIFESTVAR`; `MANIFESTTRAITVAR` does not require `a < 0`) + p. 16 `MANIFESTVARstd` (`θ / θ = 1` after strictly positive `MANIFESTVAR`; 2017-era source adds ridging; default ridge is 0; 2017-era `dimnames` assignment to `latentNames` is a source bug; not unstandardised `MANIFESTVAR`; not `MANIFESTTRAITVARstd` even when both equal 1; not Equation 5 `Var(y)`; `MANIFESTVAR` does not require `a < 0`) + p. 16 `TIPREDVARstd` (`v / v = 1` after strictly positive `TIPREDVAR`; 2017-era source adds ridging; default ridge is 0; `dimnames` are `TIpredNames`; not unstandardised `TIPREDVAR`; not `MANIFESTVARstd` even when both equal 1; not §7.2 `addedTIPREDVAR`; `TIPREDVAR` does not require `a < 0`) + p. 16 `asymDIFFUSIONstd` (`p / p = 1` after strictly positive `asymDIFFUSION`; 2017-era source adds ridging; default ridge is 0; `dimnames` are `latentNames`; not unstandardised `asymDIFFUSION`; not `TIPREDVARstd` even when both equal 1; not `DIFFUSIONstd` `−2 a`; lasting `asymDIFFUSION` requires `a < 0`) + p. 16 `discreteCINTstd` (`A^{-1}[e^{A Δt} − I] κ / √p` after strictly positive `asymDIFFUSION`; not unstandardised `discreteCINT`; not `κ / √p`; not `(-κ / a) / √p`; lasting `asymDIFFUSION` requires `a < 0`) + exact scalar p. 16 `asymCINTstd` (`(-κ / a) / √p` after strictly positive `asymDIFFUSION`; not unstandardised `asymCINT`; not `κ / √p`; not `discreteCINTstd`; lasting `asymDIFFUSION` requires `a < 0`) + irregular already-centered residual lag + strong-gated latent means (n=2 residual variance is identically `0` and caps at strong/scalar; Putnick & Bornstein, 2016); full ESEM/DSEM remaining | ADR 0005; `docs/research/posterior-esem-input-gates.md`; `docs/research/multilevel-event-time-recovery.md`; `docs/research/rubin-total-variance.md`; `docs/research/strong-invariance-latent-means.md` |
| Purpose-bound provider payloads | `tepp_api` | implemented-main | provider-payload minimization | expired/not-yet-valid/inverted/cross-tenant/impossible-calendar grant, mapping refusal, audited elevated re-id replay | ADR 0009; `docs/research/provider-payload-minimization.md` |
| Adaptive orchestration router | `tepp_api` | accepted-target | active PR | mode selection, document-control denial, ablation, credential-free bind | ADR 0010; `docs/research/adaptive-orchestration-router.md` |
| CWL modular connectors | `docs/connectors/*` | implemented-main | — | contract docs + examples | PR #22; live HTTP ports remaining |
| Release SBOM/provenance generator | `scripts/release_evidence.py` | partial | — | generate+validate in CI | Task 13 partial / PR #28 |


## Scientific acceptance checklist (foundation)

- [ ] Parameter recovery studies with known truth (RMSE, bias, coverage) on exact production metrics
- [ ] Temporal ordering accuracy and no future-available evidence use
- [ ] Relation precision/recall on synthetic forward DAGs
- [ ] Membership multilevel/multiple-membership contracts exercised
- [ ] CPU `f64` reference path present for every estimator (future measurement crates)
- [ ] 100% production line and branch coverage on exact head
- [ ] Independent review approval on each merge-critical PR
- [x] Repository SBOM/provenance generator + CI validation (`scripts/release_evidence.py`)
- [ ] Full package/image SBOM/provenance/reproducibility package for release cut

## Release decision rule

A foundation release may be cut only when:

1. required capabilities for the release train are `implemented-main`;
2. exact-head CI, security, and independent review evidence are green on the release commit;
3. this ledger and `CHANGELOG.md` describe the released capability set without over-claiming;
4. no scientific, privacy, security, or supply-chain blocker remains.

## Non-claims

- This document does not certify CSAP/SOC 2/ISO/NIST.
- Graph recovery and recovery metrics do not establish causal identification.
- Connector contracts do not imply deployed HTTP services until service crates land.

## References

See `docs/research/` doctoring notes for Task 2–12 primary sources (APA 7), including `docs/research/multilevel-multiple-membership-measurement.md`.
