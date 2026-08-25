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
| Interval-aware cutoff eligibility | `temporal_core` | active-PR | this PR | unknown/open-ended fail-closed + computed latest-instant agreement | ADR 0002 |
| Knowledge-cutoff identity | `cutoff_clock` | active-PR | this PR | recovered cutoff flags vs availability-time stand-in | ADR 0002 |
| Six-clock temporal | `temporal_core` | implemented-main | `document_clocks` omitted assertion/document time | unit + wire | Task 3 / PR #8; document-row clocks on this PR |
| Knowledge-cutoff identity | `cutoff_clock` | active-PR | this PR | recovered cutoff flags vs availability-time stand-in | ADR 0002 |
| Allen path-consistency | `temporal_core` | implemented-main | — | unit + budget tests | Task 4 / PR #9 |
| Span-grounded mentions | `event_core` | active-PR | this PR | `EventMention` is the only constructible mention type; exact-extent P/R + occupancy RMSE vs whole-document | ADR 0016; `docs/research/span-grounded-mentions.md` |
| Event mention/instance | `event_core` | partial | — | unit + fail-closed promotion | Task 5 / PR #13 |
| TDT link precision/recall | `event_core` | active-PR | this PR | computed precision/recall + RMSE vs always-link | ADR 0016; `docs/research/event-link-detection-calibration.md` |
| First-story FAR/miss | `event_core` | active-PR | this PR | computed FAR/miss + RMSE vs always-first | ADR 0016; `docs/research/first-story-detection-calibration.md` |
| Multiple membership | `membership_core` | partial | nested ICC + non-nested refusal | unit + ESS + nested ICC recovery | Task 7 / PR #12 + #25 + this increment |
| Episode membership containment | `episode_membership` | active-PR | event-time window containment | boundary, inversion, and recovery contracts | ADR 0003 / PR #146 |
| TDT tracking stability | `event_core` | active-PR | this PR | pair P/R + switch rate + RMSE vs always-one-track | ADR 0016; `docs/research/event-tracking-calibration.md` |
| CHRONOS schema-slot accuracy | `event_core` | active-PR | this PR | computed slot P/R + RMSE vs always-fill | ADR 0016; `docs/research/chronos-schema-slot-calibration.md` |
| TDT story segmentation | `event_core` | active-PR | this PR | computed `WindowDiff`/`Pk` + RMSE vs always-cut | ADR 0016; `docs/research/tdt-story-segmentation.md` |
| Event mention/instance | `event_core` | partial | CHRONOS prediction calibration | unit + fail-closed promotion | Task 5 / PR #13 |
| CHRONOS occurrence-prediction calibration | `event_core` | accepted-target | active PR | Brier vs later-observed truth; refuse prediction-as-instance | ADR 0016; `docs/research/chronos-prediction-calibration.md` |
| Multiple membership | `membership_core` | partial | — | unit + ESS weights | Task 7 / PR #12 + #25 |
| Episode membership containment | `episode_membership` | accepted-target | active PR | refuse membership outside episode + recovery vs accept-all | ADR 0003 |
| Customer/competitor role contradiction | `role_contradiction` | accepted-target | active PR | refuse overlap + recovery vs commercial collapse | ADR 0003 role assertions |
| Forward transition DAG | `relation_graph` | implemented-main | — | unit + cycle rejection | Task 6 / PR #14 |
| Copy-versus-source identity | `copy_identity` | accepted-target | active PR | refuse copy-as-source/transition + recovery vs source collapse | ADR 0003 |
| Summary-versus-source identity | `summarizes_edge` | accepted-target | active PR | refuse summary-as-transition/source + recovery vs source collapse | ADR 0003 |
| Input-process-outcome event-time order | `outcome_order` | accepted-target | active PR | refuse reverse/uncertain IPO order + outcome_of-is-not-transition + recovery vs input collapse | ADR 0002/0003 |
| Retrospective reporting identity | `retrospective_edge` | accepted-target | active PR | refuse retrospective-as-transition/translation + recovery vs forward collapse | ADR 0002/0003 |
| Inferred-versus-observed promotion | `inferred_status` | accepted-target | active PR | refuse inferred-as-observed/transition + recovery vs observed collapse | ADR 0003 |
| Evidential-vs-transition gate | `support_edge` | active-PR | this PR | recovered kind rate vs support collapse | ADR 0002/0003 |
| Relation absence is not negative evidence | `relation_absence` | accepted-target | active PR | refuse unobserved-as-negative + recovery vs observed collapse | ADR 0003 |
| Bitemporal persistence + live SQL port | `persistence_postgres` | partial | backup/restore integrity | migration contracts + recording transport + optional PgPool + live CI + tenant RLS + `0005`/`0006` + event relation/mention/instance + source-artifact + audit-event + concurrent-write (#37–#43 implemented-main) + restore integrity probes (active PR) | Task 8 / PR #16 + #23 + #26 + #27 + #29 + #30–#43 + restore integrity |
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
| Typed membership targets beyond entity/project | `membership_target` | active-PR | PR #131 | refuse collapse + recovery vs entity stand-in for language, episode, template, department, and opportunity pool | ADR 0003 |
| Bitemporal persistence + live SQL port | `persistence_postgres` | partial | entity/project target SQL on PR #131 | migration contracts + recording transport + session-affine optional PgPool + live CI + tenant RLS + `0005`/`0006` interval/membership contracts + event relation/mention/instance + source-artifact + audit-event + concurrent-write + restore integrity + typed `text_segment` SQL (#37–#44 implemented-main) | Task 8 / PR #16 + #23 + #26 + #27 + #29 + #30–#44 + entity/project SQL |
| Leakage-safe splits | `corpus_split` | implemented-main | — | cutoff + co-partition tests | Task 9 / PR #17 |
| Inferential TF-IDF/BM25/stopword refusal | `corpus_split` | active-PR | this PR | retrieval scores fail closed + `group_normalized_ess`-vs-TF-IDF RMSE + `refuse_default_stopword_deletion(TokenDeletionRule::GlobalStopwordList)` refusal | ADR 0004/0012; `docs/research/inferential-retrieval-weight-gate.md` |
| Unicode canonical identity | `corpus_split` | implemented-main | merged PR #59 | NFC/NFD and Hangul canonical-equivalence links, duplicate/empty refusal, connected-group co-partition | ADR 0004/0008/0013; `docs/research/unicode-canonical-identity.md` |
| Truth corpora / manifests | `tepp_simulation` | implemented-main | — | deterministic generator tests | Task 10 / PR #18 |
| Recovery metrics | `validation_core` | implemented-main | — | RMSE/bias/coverage/MC gates | Task 11 / PR #19 |
| Mention-confidence Brier score | `event_core` | active-PR | calibration vs binary truth | perfect 0 / half 0.25 RMSE | ADR 0003; `docs/research/mention-confidence-brier.md` |
| Checkpoint is not the estimator | `checkpoint_authority` | accepted-target | active PR | refuse checkpoint-as-estimator + unvalidated artifact + recovery vs estimator collapse | ADR 0001/0014 |
| Scientific claim promotion gates | `validation_core` | active-PR | this PR | exact-head SHA + computed RMSE SE gate | ADR 0014; full release bundle remaining |
| Causal-identification gate | `relation_graph` | active-PR | association ≠ cause | LeadsTo/References denied | ADR 0003; `docs/research/causal-identification-gate.md` |
| Versioned API/export contracts | `tepp_api` | implemented-main | naruon HTTP interchange | unknown-field/version/limit + naruon HTTPS interchange tests | Task 12 / PR #21; live HTTP service remaining |
| Simulation cutoff eligibility | `tepp_simulation` | accepted-target | `available_time <= knowledge_cutoff` on PR #62 | delayed-document exclusion, generated-count agreement, exact-boundary admission, and fail-closed `TemporalInvariantViolation` for late documents | ADR 0002; `crates/tepp_simulation/tests/cutoff_eligibility_contract.rs`; `docs/research/simulation-cutoff-eligibility.md` |
| Psychometric structural input gates | `psychometric_core` | partial | stacked psychometric PR | construct-class refusal + ALR/ILR boundary + true-loading RMSE + posterior-draw point-estimate mean + Rubin `T` + CWC within/between + CWC contextual effect + event-time log-rate + constant- and time-varying-predictor discrete effects + exact scalar discrete process noise + lagged latent covariance and unconditional latent variance + stationary within-subject variance + trait-plus-state variance + observed-indicator variance + discrete latent mean (`T0MEANS`/`CINT`) + evolved observed mean (`τ + λ μ_t`; `τ + λ μ_0` is not `E(y_t)`) + contemporaneous `TDPREDEFFECT` impulse (`m x`; not `CINT`, not `TIPREDEFFECT`, not Voelkle Eq. 14) + Eq. 5 of that contemporaneous impulse (`τ + λ(μ_t + m x)`; `τ + λ μ_t` is not that observed mean) + time-independent `TIPREDEFFECT` increment (`A^{-1}[e^{A Δt} − I] B z`; not `CINT`, not `M x`, not Voelkle Eq. 14, not the coefficient `B`) + Eq. 5 of that increment (`τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)`; `τ + λ μ_t` is not that observed mean) + within-interval `TDPREDEFFECT` carry (`e^{A(t−u)} M x` for `t0 < u < t`; not the contemporaneous Dirac, not `CINT`, not `TIPREDEFFECT`, not Voelkle Eq. 14) + Eq. 5 of that carry (`τ + λ(μ_t + e^{a(t−u)} m x)`; `τ + λ μ_t` is not that observed mean) + §7.2 level-change `CINT` (`κ = −a m x`; Eq. 3 increment `(1 − e^{a Δt}) m x`) + §7.2 extra-process contribution (`a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)`; not `κ`, not the increment, not the Dirac; `ε ≥ 0` fails closed) + Eq. 5 of that extra-process contribution (`τ + λ(μ_t + a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a))`; extra `LAMBDA` is 0; `τ + λ μ_t` is not that observed mean) + after-t0 extra-process `TDPREDEFFECT` (`a_{ηξ} x (e^{ε(t−u)} − e^{a(t−u)}) / (ε − a)` for `t0 < u < t`; Eq. 5 `τ + λ(μ_t + contribution(t−u))`; not the first-occasion extra-process observed mean; not the impulse-carry Dirac) + §7.2 `asymTIPREDEFFECT` (`-B z / a` for `a < 0`; not `B`, not the finite-interval increment, not `CINT`, not `M x`) + §7.2 `addedTIPREDVAR` (`(B / a)² v`; not `TRAITVAR`, not `asymDIFFUSION`, not `-B z / a`) + Table 2 `asymCINT` (`-κ / a` for `a < 0`; not `κ`, not the finite-interval increment, not `T0MEANS`, not `-B z / a`) + p. 16 stationary `T0MEANS` (`-κ / a + −B z / a`; not free `T0MEANS`, not `asymCINT` alone, not `asymTIPREDEFFECT` alone, not the finite-interval discrete mean) + Eq. 5 of that constrained mean (`τ + λ(−κ / a + −B z / a)`; `τ + λ μ_0` is not that observed mean; `τ + λ(−κ / a)` is not that observed mean when `B z ≠ 0`; `τ + λ μ_t` is not that observed mean; `MANIFESTMEANS` is not `E(y_0)`; the constrained latent mean is not `E(y_0)`) + stationary `T0VAR` (`trait + −q / (2 a) + (B / a)² v`; not free `T0VAR`, not `asymDIFFUSION` alone, not `TRAITVAR` alone, not `addedTIPREDVAR` alone; Eq. 5 is `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ` (`λ² p_0` is not `Var(y_0)`; `λ²(−q / (2 a)) + θ` is not `Var(y_0)` when trait or TI is nonzero; `MANIFESTVAR` is not `Var(y_0)`; the constrained latent variance is not `Var(y_0)`)) + Eq. 5 of that constrained variance (`λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ`; `MANIFESTVAR` is not `Var(y_0)`) + p. 16 `TDPREDEFFECTstd` (`m · √v / √(-q / (2 a))` after strictly positive `asymDIFFUSION` and TD predictor variance; not `TIPREDEFFECTstd` even when `M = B`; not intercept-style `A^{-1}[e^{A Δt} − I] M · √v / √p`; not trait-contaminated) + Table 3 / p. 16 `T0TDPREDEFFECTstd` (`t0_m · √v / √p_0` after strictly positive free `T0VAR` and TD predictor variance; not `TDPREDEFFECTstd`; not `T0TIPREDEFFECTstd` even when `t0_m = t0_b`; not trait-contaminated; free `T0VAR` does not require `a < 0`) + p. 16 `T0VARstd` (`p_0 / p_0 = 1` after strictly positive free `T0VAR`; not unstandardised `T0VAR`; not `T0TDPREDEFFECTstd`; not `addedT0TIPREDVAR`; free `T0VAR` does not require `a < 0`) + p. 16 `TRAITVARstd` (`trait / trait = 1` after strictly positive `TRAITVAR`; no ridge addend; not unstandardised `TRAITVAR`; not `T0VARstd` even when both equal 1; not `addedT0TIPREDVAR`; `TRAITVAR` does not require `a < 0`) + p. 16 `MANIFESTTRAITVARstd` (`ψ / ψ = 1` after strictly positive `MANIFESTTRAITVAR`; 2017-era source adds ridging; default ridge is 0; not unstandardised `MANIFESTTRAITVAR`; not `TRAITVARstd` even when both equal 1; not `MANIFESTVAR`; `MANIFESTTRAITVAR` does not require `a < 0`) + p. 16 `MANIFESTVARstd` (`θ / θ = 1` after strictly positive `MANIFESTVAR`; 2017-era source adds ridging; default ridge is 0; 2017-era `dimnames` assignment to `latentNames` is a source bug; not unstandardised `MANIFESTVAR`; not `MANIFESTTRAITVARstd` even when both equal 1; not Equation 5 `Var(y)`; `MANIFESTVAR` does not require `a < 0`) + p. 16 `TIPREDVARstd` (`v / v = 1` after strictly positive `TIPREDVAR`; 2017-era source adds ridging; default ridge is 0; `dimnames` are `TIpredNames`; not unstandardised `TIPREDVAR`; not `MANIFESTVARstd` even when both equal 1; not §7.2 `addedTIPREDVAR`; `TIPREDVAR` does not require `a < 0`) + p. 16 `asymDIFFUSIONstd` (`p / p = 1` after strictly positive `asymDIFFUSION`; 2017-era source adds ridging; default ridge is 0; `dimnames` are `latentNames`; not unstandardised `asymDIFFUSION`; not `TIPREDVARstd` even when both equal 1; not `DIFFUSIONstd` `−2 a`; lasting `asymDIFFUSION` requires `a < 0`) + p. 16 `discreteCINTstd` (`A^{-1}[e^{A Δt} − I] κ / √p` after strictly positive `asymDIFFUSION`; not unstandardised `discreteCINT`; not `κ / √p`; not `(-κ / a) / √p`; lasting `asymDIFFUSION` requires `a < 0`) + exact scalar p. 16 `asymCINTstd` (`(-κ / a) / √p` after strictly positive `asymDIFFUSION`; not unstandardised `asymCINT`; not `κ / √p`; not `discreteCINTstd`; lasting `asymDIFFUSION` requires `a < 0`) + exact scalar p. 16 `T0MEANSstd` (`μ_0 / √p_0` after strictly positive free `T0VAR`; not unstandardised `T0MEANS`; not `T0VARstd`; not `μ_0 / √asymDIFFUSION`; free `T0MEANS` does not require `a < 0`) + irregular already-centered residual lag + strong-gated latent means (n=2 residual variance is identically `0` and caps at strong/scalar; Putnick & Bornstein, 2016); full ESEM/DSEM remaining | ADR 0005; `docs/research/posterior-esem-input-gates.md`; `docs/research/multilevel-event-time-recovery.md`; `docs/research/rubin-total-variance.md`; `docs/research/strong-invariance-latent-means.md` |
| Prompt-versus-unique-content identity | `prompt_source` | accepted-target | active PR | refuse prompt-as-unique/stopword + recovery vs unique-content collapse | ADR 0004/0012 |
| Corpus-background-versus-unique-content identity | `corpus_background` | accepted-target | active PR | refuse background-as-unique/stopword + recovery vs unique-content collapse | ADR 0004/0012 |
| Modality-versus-unique-content identity | `modality_source` | accepted-target | active PR | refuse modality-as-unique/stopword + recovery vs unique-content collapse | ADR 0004/0012 |
| Copied-versus-unique-content identity | `copied_text` | accepted-target | active PR | refuse copied-text-as-unique/stopword + recovery vs unique-content collapse | ADR 0004/0012 |
| Style-versus-unique-content identity | `style_source` | accepted-target | active PR | refuse style-as-unique/stopword + recovery vs unique-content collapse | ADR 0004/0012 |
| Untrusted intake grant presence | `intake_authorization` | accepted-target | active PR | refuse missing grant + refuse bounds-as-authorization + recovery vs accept-all | ADR 0009; AGENTS.md |
| Untrusted payload identity/provenance/size/depth | `payload_bound` | accepted-target | active PR | refuse missing identity/provenance and oversize/over-deep payloads + recovery vs accept-all | AGENTS.md untrusted-boundary |
| System-clock identity | `system_clock` | active-PR | this PR | recovered system flags vs event-time stand-in | ADR 0002 |
| Event-clock identity | `event_clock` | active-PR | this PR | recovered event flags vs assertion-time stand-in | ADR 0002 |
| Assertion-clock identity | `assertion_clock` | active-PR | this PR | recovered assertion flags vs event-time stand-in | ADR 0002 |
| Availability-clock identity | `available_clock` | active-PR | this PR | recovered availability flags vs system-time stand-in | ADR 0002 |
| Revision system-time order | `revision_order` | active-PR | this PR | order-flag recovery vs accept-all | ADR 0002/0013 |
| Provenance-vs-transition gate | `citation_edge` | active-PR | this PR | recovered kind rate vs citation collapse | ADR 0002/0003 |
| ESEM/DSEM CPU fit | `psychometric_fit` | active-PR | this PR | loading RMSE vs zero-collapse; reverse-lag refusal | ADR 0005; does not recreate `psychometric_core` |
| Subevent parent containment | `subevent_containment` | active-PR | this PR | containment-flag recovery vs accept-all | ADR 0003 |
| Predicted-vs-observed contradiction | `prediction_contradiction` | active-PR | this PR | `refuse_promotion` requires observed coverage; `refuse_contradiction_or_adjacency` is not promotion authority; cutoff eligibility; label agreement is not RMSE recovery | ADR 0016 |
| Provider-disclosure receipts | `provider_receipt` | active-PR | this PR | recovered field-code rate vs collapsed set | ADR 0009 |
| Purpose-bound provider payloads | `tepp_api` | implemented-main | provider-payload minimization | expired/not-yet-valid/inverted/cross-tenant/impossible-calendar grant, mapping refusal, audited elevated re-id replay | ADR 0009; `docs/research/provider-payload-minimization.md` |
| Adaptive orchestration router | `tepp_api` | partial | — | mode selection, document-control denial, ablation, credential-free bind; live NIM execution remains future | ADR 0010; `docs/research/adaptive-orchestration-router.md` |
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
| Style-versus-unique-content identity | `style_source` | accepted-target | active PR | refuse style-as-unique/stopword + recovery vs unique-content collapse | ADR 0004/0012 |
| Modality-versus-unique-content identity | `modality_source` | accepted-target | active PR | refuse modality-as-unique/stopword + recovery vs unique-content collapse | ADR 0004/0012 |
| Corpus-background-versus-unique-content identity | `corpus_background` | accepted-target | active PR | refuse background-as-unique/stopword + recovery vs unique-content collapse | ADR 0004/0012 |
| Prompt-versus-unique-content identity | `prompt_source` | accepted-target | active PR | refuse prompt-as-unique/stopword + recovery vs unique-content collapse | ADR 0004/0012 |
| Purpose-bound provider payloads | `tepp_api` | implemented-main | provider-payload minimization | expired/not-yet-valid/inverted/cross-tenant/impossible-calendar grant, mapping refusal, audited elevated re-id replay | ADR 0009; `docs/research/provider-payload-minimization.md` |
| Adaptive orchestration router | `tepp_api` | implemented-main | live execution/production ablation | mode selection, document-control denial, ablation, credential-free bind | ADR 0010; `docs/research/adaptive-orchestration-router.md` |
| Operational log/source separation | `operational_log` + `persistence_postgres` | active-PR | this PR | replayed action recovery vs collapsed action; inspected `audit_event` insert refuses source text, source identity, and blanket-mask grants | ADR 0009 |
| Adaptive orchestration router | `tepp_api` | accepted-target | active PR | mode selection, document-control denial, ablation, credential-free bind | ADR 0010; `docs/research/adaptive-orchestration-router.md` |
| Embedded image source units | `evidence_core` | active-PR | active PR (#58) | exact data-URI span/media-type recovery, empty/incomplete/invalid/recovery cases, lexical refusal | ADR 0008; `docs/research/embedded-image-units.md` |
| Derived sensitivity inheritance | `derived_sensitivity` | active-PR | this PR | fixed 3×3 topic/factor/relation × Public/Internal/Restricted truth with kind-and-class recovery and public-collapse rates invariant to reordering; fail-closed unknown kinds, validated public constructors, empty/mismatched payloads, unauthorized derivation-as-public, unauthorized blanket PII masking | ADR 0009; GDPR Art. 4(1)/Recital 26; WP 136 |
| Evidence-bounded LLM interpretation | `interpretation_gateway` | active-PR | this PR | span citation + unsupported-claim rate | ADR 0010; live orchestration remaining |
| TDT/CHRONOS evidence-status gates | `event_core` | active-PR | PR #50 | admission + first-story rates | known-stream miss/FA; full tracking/calibration/schema extraction remains future; ADR 0016; `docs/research/event-intelligence-status-gates.md` |
| Encrypted identity mapping envelope | `encrypted_mapping` | active-PR | this PR | exact-head evidence with no unresolved security blocker; unauthorized purpose, wrong key identity/bytes, tampered ciphertext/tag, key redaction, empty input, persistence refusal, generated-nonce/reuse resistance, and recovered identity rate vs collapsed names | ADR 0009; persistence waits for later migration; promotion requires the complete fail-closed security matrix |
| Logistic-normal topic coordinates and CPU reference estimator | `topic_measurement` | active-PR | stable ALR + sequential ILR + lexical refusal + bounded sparse TRSL-TM fit | known-simplex ALR/ILR RMSE, Aitchison-distance ILR isometry, known-topic RMSE, exact line/branch coverage | ADR 0012; calibrated posterior, method effects, persistence, and accelerated backends remaining |
| CWL modular connectors | `docs/connectors/*` | implemented-main | — | contract docs + examples | PR #22; live HTTP ports remaining |
| Orchestrator loopback interpretation listener | `orchestrator_live` | active-PR | this PR | loopback bind + hypothetical claim + credential/table refusal | ADR 0010/0011; not TLS or model execution |
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
