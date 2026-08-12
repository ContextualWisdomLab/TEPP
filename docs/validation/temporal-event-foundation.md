# Temporal Event Foundation — validation and release-readiness report

**Status:** Living validation ledger for the Temporal/Event foundation program  
**Last reviewed:** 2026-08-12  
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
| Bitemporal persistence + live SQL port | `persistence_postgres` | partial | active-PR (PR #29 live PG CI) | migration contracts + recording transport + optional PgPool + live CI service | Task 8 / PR #16 + #23 + #26 + #27 + #29 |
| Leakage-safe splits | `corpus_split` | implemented-main | — | cutoff + co-partition tests | Task 9 / PR #17 |
| Truth corpora / manifests | `tepp_simulation` | implemented-main | — | deterministic generator tests | Task 10 / PR #18 |
| Recovery metrics | `validation_core` | implemented-main | — | RMSE/bias/coverage/MC gates | Task 11 / PR #19 |
| Versioned API/export contracts | `tepp_api` | implemented-main | — | unknown-field/version/limit tests | Task 12 / PR #21; HTTP service remaining |
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
