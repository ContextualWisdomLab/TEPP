# Temporal Event Foundation — validation and release-readiness report

**Status:** Living validation ledger for the Temporal/Event foundation program  
**Last reviewed:** 2026-08-20
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
| Multiple membership | `membership_core` | partial | nested ICC + non-nested refusal | unit + ESS + nested ICC recovery | Task 7 / PR #12 + #25 + this increment |
| Forward transition DAG | `relation_graph` | implemented-main | — | unit + cycle rejection | Task 6 / PR #14 |
| Bitemporal persistence + live SQL port | `persistence_postgres` | partial | typed `text_segment` SQL | migration contracts + recording transport + optional PgPool + live CI + tenant RLS + `0005`/`0006` + event relation/mention/instance + source-artifact + audit-event + concurrent-write + restore integrity (#37–#44 implemented-main) + typed `text_segment` insert/cutoff lookup (active PR) | Task 8 / PR #16 + #23 + #26 + #27 + #29 + #30–#44 + text-segment SQL |
| Bitemporal persistence + live SQL port | `persistence_postgres` | partial | — | migration contracts + recording transport + optional PgPool + live CI + tenant RLS + `0005`/`0006` interval and membership contracts + event relation/mention/instance + source-artifact + audit-event + concurrent-write + restore-integrity contracts implemented-main | Task 8 / PR #16 + #23 + #26 + #27 + #29 + #30–#44 |
| Leakage-safe splits | `corpus_split` | implemented-main | — | cutoff + co-partition tests | Task 9 / PR #17 |
| Truth corpora / manifests | `tepp_simulation` | implemented-main | — | deterministic generator tests | Task 10 / PR #18 |
| Recovery metrics | `validation_core` | implemented-main | — | RMSE/bias/coverage/MC gates | Task 11 / PR #19 |
| Mention-confidence Brier score | `event_core` | active-PR | calibration vs binary truth | perfect 0 / half 0.25 RMSE | ADR 0003; `docs/research/mention-confidence-brier.md` |
| Checkpoint is not the estimator | `checkpoint_authority` | accepted-target | active PR | refuse checkpoint-as-estimator + unvalidated artifact + recovery vs estimator collapse | ADR 0001/0014 |
| Versioned API/export contracts | `tepp_api` | implemented-main | naruon HTTP interchange | unknown-field/version/limit + naruon HTTPS interchange tests | Task 12 / PR #21; live HTTP service remaining |
| Compositional cluster-pair gates | `network_analysis` | active-PR | this PR | raw-simplex refusal + known-truth pair precision/recall + cluster-label permutation invariance | ADR 0005/0012; `crates/network_analysis/tests/compositional_cluster_contract.rs`; graphical model remaining |
| Candidate-K statistical/Pareto gates | `model_selection` | active-PR | this PR | known-K RMSE + LLM-vote refusal | ADR 0012; estimator/backend remaining |
| Purpose-bound provider payloads | `tepp_api` | implemented-main | provider-payload minimization | expired/not-yet-valid/inverted/cross-tenant/impossible-calendar grant, mapping refusal, audited elevated re-id replay | ADR 0009; `docs/research/provider-payload-minimization.md` |
| Adaptive orchestration router | `tepp_api` | accepted-target | active PR | mode selection, document-control denial, ablation, credential-free bind | ADR 0010; `docs/research/adaptive-orchestration-router.md` |
| Evidence-bounded LLM interpretation | `interpretation_gateway` | active-PR | this PR | span citation + unsupported-claim rate | ADR 0010; live orchestration remaining |
| TDT/CHRONOS evidence-status gates | `event_core` | active-PR | PR #50 | admission + first-story rates | known-stream miss/FA; full tracking/calibration/schema extraction remains future; ADR 0016; `docs/research/event-intelligence-status-gates.md` |
| Purpose-bound provider payloads | `tepp_api` | implemented-main | — | expired/not-yet-valid/inverted/cross-tenant/impossible-calendar grant, mapping refusal, audited elevated re-id replay | ADR 0009; `docs/research/provider-payload-minimization.md` |
| Adaptive orchestration router | `tepp_api` | partial | — | mode selection, document-control denial, ablation, credential-free bind; live NIM execution remains future | ADR 0010; `docs/research/adaptive-orchestration-router.md` |
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
