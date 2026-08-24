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
| Interval-aware cutoff eligibility | `temporal_core` | active-PR | this PR | unknown/open-ended fail-closed + computed latest-instant agreement | ADR 0002 |
| Six-clock temporal | `temporal_core` | implemented-main | `document_clocks` omitted assertion/document time | unit + wire | Task 3 / PR #8; document-row clocks on this PR |
| Knowledge-cutoff identity | `cutoff_clock` | active-PR | this PR | recovered cutoff flags vs availability-time stand-in | ADR 0002 |
| Allen path-consistency | `temporal_core` | implemented-main | — | unit + budget tests | Task 4 / PR #9 |
| Event mention/instance | `event_core` | partial | — | unit + fail-closed promotion | Task 5 / PR #13 |
| Multiple membership | `membership_core` | partial | nested ICC + non-nested refusal | unit + ESS + nested ICC recovery | Task 7 / PR #12 + #25 + this increment |
| TDT tracking stability | `event_core` | active-PR | this PR | pair P/R + switch rate + RMSE vs always-one-track | ADR 0016; `docs/research/event-tracking-calibration.md` |
| CHRONOS schema-slot accuracy | `event_core` | active-PR | this PR | computed slot P/R + RMSE vs always-fill | ADR 0016; `docs/research/chronos-schema-slot-calibration.md` |
| TDT story segmentation | `event_core` | active-PR | this PR | computed `WindowDiff`/`Pk` + RMSE vs always-cut | ADR 0016; `docs/research/tdt-story-segmentation.md` |
| Event mention/instance | `event_core` | partial | CHRONOS prediction calibration | unit + fail-closed promotion | Task 5 / PR #13 |
| CHRONOS occurrence-prediction calibration | `event_core` | accepted-target | active PR | Brier vs later-observed truth; refuse prediction-as-instance | ADR 0016; `docs/research/chronos-prediction-calibration.md` |
| Multiple membership | `membership_core` | partial | — | unit + ESS weights | Task 7 / PR #12 + #25 |
| Episode membership containment | `episode_membership` | accepted-target | active PR | refuse membership outside episode + recovery vs accept-all | ADR 0003 |
| Forward transition DAG | `relation_graph` | implemented-main | — | unit + cycle rejection | Task 6 / PR #14 |
| Bitemporal persistence + live SQL port | `persistence_postgres` | partial | backup/restore integrity | migration contracts + recording transport + optional PgPool + live CI + tenant RLS + `0005`/`0006` + event relation/mention/instance + source-artifact + audit-event + concurrent-write (#37–#43 implemented-main) + restore integrity probes (#44 implemented-main) | Task 8 / PR #16 + #23 + #26 + #27 + #29 + #30–#44 + restore integrity |
| Bitemporal persistence + live SQL port | `persistence_postgres` | partial | typed `text_segment` SQL | migration contracts + recording transport + optional PgPool + live CI + tenant RLS + `0005`/`0006` + event relation/mention/instance + source-artifact + audit-event + concurrent-write + restore integrity (#37–#44 implemented-main) + typed `text_segment` insert/cutoff lookup (active PR) | Task 8 / PR #16 + #23 + #26 + #27 + #29 + #30–#44 + text-segment SQL |
| Bitemporal persistence + live SQL port | `persistence_postgres` | partial | — | migration contracts + recording transport + optional PgPool + live CI + tenant RLS + `0005`/`0006` interval and membership contracts + event relation/mention/instance + source-artifact + audit-event + concurrent-write + restore-integrity contracts implemented-main | Task 8 / PR #16 + #23 + #26 + #27 + #29 + #30–#44 |
| Bitemporal persistence + live SQL port | `persistence_postgres` | partial | entity/project target SQL | migration contracts + recording transport + optional PgPool + live CI + tenant RLS + `0005`/`0006` + event relation/mention/instance + source-artifact + audit-event + concurrent-write + restore integrity (#37–#44 implemented-main) + entity/project SQL (this PR) | Task 8 / PR #16 + #23 + #26 + #27 + #29 + #30–#44 + entity/project SQL |
| Evidential-vs-transition gate | `support_edge` | active-PR | this PR | recovered kind rate vs support collapse | ADR 0002/0003 |
| Inferred-versus-observed promotion | `inferred_status` | accepted-target | active PR | refuse inferred-as-observed/transition + recovery vs observed collapse | ADR 0003 |
| Input-process-outcome event-time order | `outcome_order` | accepted-target | active PR | refuse reverse/uncertain IPO order + outcome_of-is-not-transition + recovery vs input collapse | ADR 0002/0003 |
| Summary-versus-source identity | `summarizes_edge` | accepted-target | active PR | refuse summary-as-transition/source + recovery vs source collapse | ADR 0003 |
| Copy-versus-source identity | `copy_identity` | accepted-target | active PR | refuse copy-as-source/transition + recovery vs source collapse | ADR 0003 |
| Bitemporal persistence + live SQL port | `persistence_postgres` | partial | backup/restore integrity | migration contracts + recording transport + optional PgPool + live CI + tenant RLS + `0005`/`0006` + event relation/mention/instance + source-artifact + audit-event + concurrent-write (#37–#43 implemented-main) + restore integrity probes (active PR) | Task 8 / PR #16 + #23 + #26 + #27 + #29 + #30–#43 + restore integrity |
| Leakage-safe splits | `corpus_split` | implemented-main | — | cutoff + co-partition tests | Task 9 / PR #17 |
| Truth corpora / manifests | `tepp_simulation` | implemented-main | — | deterministic generator tests | Task 10 / PR #18 |
| Recovery metrics | `validation_core` | implemented-main | — | RMSE/bias/coverage/MC gates | Task 11 / PR #19 |
| Mention-confidence Brier score | `event_core` | active-PR | calibration vs binary truth | perfect 0 / half 0.25 RMSE | ADR 0003; `docs/research/mention-confidence-brier.md` |
| Checkpoint is not the estimator | `checkpoint_authority` | accepted-target | active PR | refuse checkpoint-as-estimator + unvalidated artifact + recovery vs estimator collapse | ADR 0001/0014 |
| Scientific claim promotion gates | `validation_core` | active-PR | this PR | exact-head SHA + computed RMSE SE gate | ADR 0014; full release bundle remaining |
| Causal-identification gate | `relation_graph` | active-PR | association ≠ cause | LeadsTo/References denied | ADR 0003; `docs/research/causal-identification-gate.md` |
| Versioned API/export contracts | `tepp_api` | implemented-main | naruon HTTP interchange | unknown-field/version/limit + naruon HTTPS interchange tests | Task 12 / PR #21; live HTTP service remaining |
| Availability-clock identity | `available_clock` | active-PR | this PR | recovered availability flags vs system-time stand-in | ADR 0002 |
| Revision system-time order | `revision_order` | active-PR | this PR | order-flag recovery vs accept-all | ADR 0002/0013 |
| Provenance-vs-transition gate | `citation_edge` | active-PR | this PR | recovered kind rate vs citation collapse | ADR 0002/0003 |
| ESEM/DSEM CPU fit | `psychometric_fit` | active-PR | this PR | loading RMSE vs zero-collapse; reverse-lag refusal | ADR 0005; does not recreate `psychometric_core` |
| Subevent parent containment | `subevent_containment` | active-PR | this PR | containment-flag recovery vs accept-all | ADR 0003 |
| Predicted-vs-observed contradiction | `prediction_contradiction` | active-PR | this PR | `refuse_promotion` requires observed coverage; `refuse_contradiction_or_adjacency` is not promotion authority; cutoff eligibility; label agreement is not RMSE recovery | ADR 0016 |
| Provider-disclosure receipts | `provider_receipt` | active-PR | this PR | recovered field-code rate vs collapsed set | ADR 0009 |
 | Purpose-bound provider payloads | `tepp_api` | implemented-main | provider-payload minimization | expired/not-yet-valid/inverted/cross-tenant/impossible-calendar grant, mapping refusal, audited elevated re-id replay | ADR 0009; `docs/research/provider-payload-minimization.md` |
 | Adaptive orchestration router | `tepp_api` | accepted-target | active PR | mode selection, document-control denial, ablation, credential-free bind | ADR 0010; `docs/research/adaptive-orchestration-router.md` |
 | Production TLS bind gates | `service_tls` | accepted-target | active PR | plaintext production, table-access host, mismatched PEM, and orchestrator loopback refusal plus recovery computed from `authorize_production_tls` / `authorize_orchestrator_live_port` | ADR 0011; rustls config is not a deployed listener |
| Longitudinal within/between | `longitudinal_core` | active-PR | this PR | known-truth component recovery, computed component RMSE, grand-mean pooling baseline comparison, and between-as-within refusal | ADR 0005 |
| Global topic activity identity | `topic_lineage` | active-PR | this PR | dormancy/reactivation identity recovery | ADR 0012; birth/split/merge remaining |
| Compositional cluster-pair gates | `network_analysis` | active-PR | this PR | raw-simplex refusal + known-truth pair precision/recall + cluster-label permutation invariance | ADR 0005/0012; `crates/network_analysis/tests/compositional_cluster_contract.rs`; graphical model remaining |
| Candidate-K statistical/Pareto gates | `model_selection` | active-PR | this PR | known-K RMSE + LLM-vote refusal | ADR 0012; estimator/backend remaining |
| Assertion-clock identity | `assertion_clock` | active-PR | this PR | recovered assertion flags vs event-time stand-in | ADR 0002 |
| Event-clock identity | `event_clock` | active-PR | this PR | recovered event flags vs assertion-time stand-in | ADR 0002 |
| System-clock identity | `system_clock` | active-PR | this PR | recovered system flags vs event-time stand-in | ADR 0002 |
| Untrusted payload identity/provenance/size/depth | `payload_bound` | accepted-target | active PR | refuse missing identity/provenance and oversize/over-deep payloads + recovery vs accept-all | AGENTS.md untrusted-boundary |
| Untrusted intake grant presence | `intake_authorization` | accepted-target | active PR | refuse missing grant + refuse bounds-as-authorization + recovery vs accept-all | ADR 0009; AGENTS.md |
| Purpose-bound provider payloads | `tepp_api` | implemented-main | provider-payload minimization | expired/not-yet-valid/inverted/cross-tenant/impossible-calendar grant, mapping refusal, audited elevated re-id replay | ADR 0009; `docs/research/provider-payload-minimization.md` |
| Adaptive orchestration router | `tepp_api` | implemented-main | live execution/production ablation | mode selection, document-control denial, ablation, credential-free bind | ADR 0010; `docs/research/adaptive-orchestration-router.md` |
| Operational log/source separation | `operational_log` + `persistence_postgres` | active-PR | this PR | replayed action recovery vs collapsed action; inspected `audit_event` insert refuses source text, source identity, and blanket-mask grants | ADR 0009 |
| Adaptive orchestration router | `tepp_api` | accepted-target | active PR | mode selection, document-control denial, ablation, credential-free bind | ADR 0010; `docs/research/adaptive-orchestration-router.md` |
| Derived sensitivity inheritance | `derived_sensitivity` | active-PR | this PR | fixed 3×3 topic/factor/relation × Public/Internal/Restricted truth with kind-and-class recovery and public-collapse rates invariant to reordering; fail-closed unknown kinds, validated public constructors, empty/mismatched payloads, unauthorized derivation-as-public, unauthorized blanket PII masking | ADR 0009; GDPR Art. 4(1)/Recital 26; WP 136 |
| Evidence-bounded LLM interpretation | `interpretation_gateway` | active-PR | this PR | span citation + unsupported-claim rate | ADR 0010; live orchestration remaining |
| TDT/CHRONOS evidence-status gates | `event_core` | active-PR | PR #50 | admission + first-story rates | known-stream miss/FA; full tracking/calibration/schema extraction remains future; ADR 0016; `docs/research/event-intelligence-status-gates.md` |
| Encrypted identity mapping envelope | `encrypted_mapping` | active-PR | this PR | exact-head evidence with no unresolved security blocker; unauthorized purpose, wrong key identity/bytes, tampered ciphertext/tag, key redaction, empty input, persistence refusal, generated-nonce/reuse resistance, and recovered identity rate vs collapsed names | ADR 0009; persistence waits for later migration; promotion requires the complete fail-closed security matrix |
| Purpose-bound provider payloads | `tepp_api` | implemented-main | — | expired/not-yet-valid/inverted/cross-tenant/impossible-calendar grant, mapping refusal, audited elevated re-id replay | ADR 0009; `docs/research/provider-payload-minimization.md` |
| Adaptive orchestration router | `tepp_api` | partial | — | mode selection, document-control denial, ablation, credential-free bind; live NIM execution remains future | ADR 0010; `docs/research/adaptive-orchestration-router.md` |
| CWL modular connectors | `docs/connectors/*` | implemented-main | — | contract docs + examples | PR #22; live HTTP ports remaining |
| Release SBOM/provenance generator | `scripts/release_evidence.py` | partial | — | generate+validate in CI | Task 13 partial / PR #28 |
| Default stopword deletion refusal | `stopword_deletion` | accepted-target | active PR | refuse default/global stopword lists + recovery vs stopword collapse | ADR 0004/0012 |


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
