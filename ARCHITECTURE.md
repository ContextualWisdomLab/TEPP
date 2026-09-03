# TEPP Architecture

## Product definition

TEPP is a Temporal Event Psychometrics Platform. It measures multilingual semantic evidence, links documents and event mentions through typed temporal relations, estimates shared latent topic and higher-order psychometric structures, and renders the resulting evidence, uncertainty, trajectories, and networks.

```mermaid
flowchart LR
    A[Immutable documents and metadata] --> B[Evidence ingestion]
    B --> C[Temporal and event normalization]
    C --> D[Multilingual semantic units]
    D --> E[Shared-latent temporal topic measurement]
    C --> F[Typed document-event-entity graph]
    E --> G[Posterior topic coordinates]
    F --> G
    G --> H[Longitudinal Modeling]
    G --> I[Topic and event networks]
    H --> J[Evidence-grounded interpretation]
    I --> J
    J --> K[Accessible visual analytics and exports]
```

## Bounded services and Rust crates

| Boundary | Primary responsibility |
|---|---|
| `evidence_ingestion` | immutable source bytes, hashes, layout, exact spans, metadata, provenance |
| `temporal_core` | instants, intervals, uncertain dates, partial orders, bitemporal availability and leakage gates |
| `event_ontology` | event mentions, event instances, roles, subevents, products, factors, places, and evidence links |
| `relation_graph` | typed document, segment, event, entity, revision, translation, evidence, and transition edges |
| `membership_model` | time-varying multilevel, cross-classified, and multiple-membership assignments with auditable event-time validity |
| `semantic_preprocessor` | Unicode, segmentation, morphology, dependency phrases, LLM span contracts, validation |
| `concept_dictionary` | versioned multilingual concept alignment and unknown-concept review |
| `topic_measurement` | shared-latent temporal/relational topic estimation and uncertainty |
| `compute_backend` | CPU `f64`, fixed-pool multithreading, accelerator parity, sparse streaming, VRAM budgeting |
| `model_selection` | fitted candidate-K scoring from the CPU reference, predictive fit, coherence, exclusivity, stability, alignment, fairness, blinded LLM review |
| `psychometric_core` | posterior-aware non-temporal measurement inputs and structural measurement fitting; reusable generalized/static psychometric arithmetic is consumed from released fast-mlsirm contracts rather than re-owned here |
| `longitudinal_modeling` | event-time longitudinal state/trajectory composition, irregular-gap transitions, longitudinal invariance and DSEM/continuous-time mappings, time-varying membership composition, alignment, and recovery evidence |
| `event_intelligence` | TDT segmentation/link/detection/first-story/tracking and CHRONOS schema reasoning |
| `network_analysis` | log-ratio topic correlation, conditional networks, uncertainty, Leiden consensus clusters |
| `interpretation_gateway` | evidence-bounded LLM interpretation and independent verification; never numerical/scientific authority |
| `artifact_service` | model registry, manifests, JSON-LD, GraphML, Arrow/Parquet, tables, SVG/PDF exports |
| `visual_analytics` | bitemporal lens, event graph, topic river, drift, longitudinal builder, invariance and leakage audit |

Every boundary must be independently usable and expose versioned contracts for integration with organization repositories. Cross-boundary integration is through released/versioned contracts and explicit ACLs; direct cross-service application-table access and mutable sibling-head dependencies are prohibited.

The `analysis_engine` vertical slice is intentionally separate from `tepp_api`: the API owns wire contracts while the engine owns deterministic execution. It does not replace topic, measurement, or Longitudinal Modeling estimators and does not read another service's application tables.

### Longitudinal Modeling ownership

Temporal composition is a TEPP domain responsibility. `longitudinal_modeling` owns the meaning and composition of event-time state/trajectory changes, irregular gaps, time-varying hierarchy/cross-classification/multiple membership, temporal alignment, rolling-origin leakage control, and longitudinal recovery evidence. Its Rust implementation path is `longitudinal_core`.

Reusable static/generalized-mixed/dependence psychometric arithmetic is not duplicated in TEPP. fast-mlsirm is the canonical owner for that arithmetic and TEPP consumes only immutable released/versioned Published Language through an ACL. A TEPP temporal adapter may combine those static primitives with typed event-time state, membership, and evidence semantics, but it cannot copy the upstream implementation or make an open upstream PR head authoritative.

`psychometric_core` retains legacy compatibility surfaces for existing measurement code while they are migrated through explicit adapters. Those legacy APIs do not establish new ownership of temporal composition. New temporal/state-transition behavior belongs in `longitudinal_core`; new reusable static psychometric primitives belong in fast-mlsirm.

Detailed equations, literature claims, recovery fixtures, and exact implementation/test evidence belong in TRACEABILITY, doctoring/research documents, ADRs, and source tests. The responsibility tables below intentionally identify ownership rather than duplicate an equation catalogue.

## Implemented foundation topology

The crate names are implementation identifiers. A crate can be an adapter or an incremental slice without becoming a new bounded context or architectural authority.

| Rust crate | Initial responsibility |
|---|---|
| `evidence_core` | immutable evidence domain primitives |
| `semantic_core` | span-grounded semantic units; language is not identity |
| `location_membership` | location is not entity identity and not a language channel |
| `temporal_core` | typed clocks, intervals, and temporal reasoning |
| `event_core` | event instances, span-grounded `EventMention`, roles, provenance, and CHRONOS occurrence-prediction calibration |
| `relation_graph` | typed relations and forward-transition validation |
| `membership_core` | time-varying cross-classified/multiple membership, Kish ESS, nested ICC with non-nested refusal |
| `role_contradiction` | customer and competitor cannot occupy the same group |
| `relation_absence` | unobserved relation pairs are not evidence of no relationship |
| `persistence_postgres` | PostgreSQL repositories and migrations |
| `corpus_split` | cutoff-safe, relation-aware partitioning |
| `tepp_simulation` | known-truth temporal/event data generation |
| `validation_core` | RMSE, bias, coverage, graph, Monte Carlo, and exact-head claim-promotion metrics |
| `tepp_api` | versioned DTO, schema, terminal-result, and export contracts |
| `analysis_engine` | bounded cutoff-safe temporal evidence readiness execution and digest-bound terminal artifacts |
| `episode_membership` | event-time episode membership containment gate |
| `prompt_source` | prompt boilerplate is not unique latent content and not stopword deletion |
| `corpus_background` | corpus-background wording is not unique latent content and not stopword deletion |
| `modality_source` | non-lexical modality is not unique latent content and not stopword deletion |
| `copied_text` | copied-text residue is not unique latent content and not stopword deletion |
| `style_source` | house-voice style residue is not unique latent content and not stopword deletion |
| `stopword_deletion` | default stopword deletion is not a valid method for repeated report language |
| `copy_identity` | a template copy is not the source document and not a state transition |
| `intake_authorization` | untrusted intake fails closed without a grant; bounds are not authorization |
| `summarizes_edge` | a summary is not a state transition and not the source document |
| `outcome_order` | input-process-outcome edges cannot move backward in event time |
| `retrospective_edge` | retrospective reporting cannot become a transition or a translation |
| `payload_bound` | untrusted documents, records, checkpoints, and LLM outputs fail closed without identity, provenance, size, and depth |
| `inferred_status` | inferred relations cannot be promoted to observed evidence or transitions |
| `support_edge` | support, contradiction, summary, and outcome_of edges are not state transitions |
| `system_clock` | system time cannot be replaced by event, assertion, document, available, or cutoff time |
| `event_clock` | event time cannot be replaced by assertion, system, document, or available time |
| `assertion_clock` | assertion time cannot be replaced by event, system, document, or available time |
| `cutoff_clock` | knowledge cutoff cannot be replaced by event, system, or availability time |
| `available_clock` | availability time cannot be replaced by event or system time |
| `document_clocks` | document rows carry assertion time and document time |
| `revision_order` | later document revisions must have later system time |
| `encrypted_mapping` | purpose-bound in-memory AES-256-GCM identity mappings; no plaintext persistence or KMS claim |
| `citation_edge` | citation, revision, translation, and retrospective edges are not state transitions |
| `psychometric_fit` | CPU `f64` ESEM loading recovery and event-time admission at the measurement/Longitudinal ACL |
| `subevent_containment` | subevent event-time intervals stay inside the parent |
| `prediction_contradiction` | Allen promotion gate: contradictory and unsupported relations fail closed unless required evidence exists |
| `provider_receipt` | provider-disclosure field-code receipts; source text and identity are not disclosable |
| `operational_log` | operational logs; `try_record` is the recording gate and source text/source identity are not loggable |
| `service_tls` | production TLS bind gates and rustls server configuration |
| `derived_sensitivity` | derived topic/factor/relation outputs inherit source sensitivity |
| `longitudinal_core` | Longitudinal Modeling temporal composition and recovery: within/between decomposition, event-time lagged correlation using both marginals, scalar event-time mappings, irregular residual log-rate, and typed interval admission; reusable static psychometric arithmetic is consumed only through released fast-mlsirm ACLs |
| `topic_lineage` | global topic identity across active/dormant/reactivated states |
| `network_analysis` | compositional cluster-pair gates; raw simplex is not Euclidean |
| `interpretation_gateway` | evidence-bounded LLM interpretations; not estimators or observed facts |
| `orchestrator_live` | loopback interpretation HTTP/1.1 listener |
| `model_selection` | fitted candidate-`K` scoring from the CPU `f64` reference plus statistical/Pareto gates; LLM votes are not numerical authority |
| `checkpoint_authority` | a model checkpoint is not the CPU `f64` estimator |
| `compute_backend` | VRAM-budgeted streamed planning, executable OOM retry plans, and a compensated CPU `f64` reference |
| `membership_target` | language, episode, template, department, and opportunity-pool targets cannot collapse into entity or project |
| `topic_measurement` | logistic-normal ALR/ILR coordinates and the CPU `f64` TRSL-TM reference estimator |
| `psychometric_core` | posterior-aware structural measurement gates and existing non-temporal fitting/compatibility surfaces; it is not the owner for new event-time temporal composition or reusable static generalized psychometric arithmetic |

Foundation crates expose only tested contracts. Empty façades are not public APIs. No crate exposes placeholder production behavior merely to reserve an API.

## Immutable evidence boundary

Stable RFC 9562 `UUIDv7` identities are independent from canonical `SHA-256` content digests. Source bytes and UTF-8 document text are copied into immutable owned storage, bounded before allocation, and verified without exposing mutable fields.

A source span records an owning document, a half-open UTF-8 byte range, the matching half-open Unicode-scalar range, and optional page/layout geometry. It fails closed for empty or reversed ranges, byte or scalar overflow, mid-code-point boundaries, coordinate disagreement, cross-document use, nonfinite geometry, nonpositive dimensions, and rectangles outside the page. Scalar coordinates are evidence locations rather than grapheme, word, or sentence boundaries; language-tailored segmentation remains a separate boundary.

The evidence boundary exposes a strict versioned JSON wire contract without exposing private Rust fields. Reconstruction revalidates identifiers, canonical digests, content limits, exact text coordinates, ownership, and page geometry. Malformed/unsupported input and unknown nested fields fail closed with content-redacting errors.

Persistence, JSON Schema publication, JSON-LD, GraphML, source acquisition metadata, signatures, and W3C PROV remain outward adapters or later contracts. They depend inward on validated domain values rather than defining them.

## Quality architecture

The workspace centralizes package metadata and Rust/Clippy lints. Every member inherits `unsafe_code = "forbid"`, `missing_docs = "deny"`, and warning denial. Repository contract scripts independently verify the approved crate set, workspace inheritance, action SHA pinning, absence of execution credentials from ordinary CI, and complete Rust documentation.

Stable Rust is the compile/lint/test reference selected by repository toolchain policy. CPU `f64` is the scientific numerical reference. Branch/statement coverage, doctests, dependency policy, property/fuzz/security/concurrency tests, and exact missing-source evidence are enforced without retries or denominator manipulation. Accelerator claims require actual hardware execution and parity evidence rather than skipped tests.

## Temporal invariants

TEPP stores event/valid time, assertion time, document time, system time, available time, and knowledge cutoff independently. A historical analysis may include evidence only when:

\[
\operatorname{available\_time}(d) \leq \operatorname{knowledge\_cutoff}.
\]

When availability is an interval, every admissible instant must satisfy the cutoff. Unknown or open-ended availability that can extend past the cutoff fails closed; event time and document time cannot substitute for availability.

Forward transition/state edges require a temporally valid partial order. Retrospective, revision, translation, citation, support, contradiction, and provenance relations retain their direction and evidence but do not become reverse state transitions.

Longitudinal recovery preserves event-time spacing, available-time admission, irregular gaps, delayed/retrospective reports, missing occasions, changing memberships, language/source drift, and required alignment across translations/rotations/reflections or cluster labels. Scientific acceptance uses true-parameter state/trajectory recovery, RMSE/bias/coverage/convergence and Monte Carlo uncertainty; LLM output cannot activate a scientific candidate.

## Measurement invariants

All languages share global topic identities and latent document coordinates. Language-specific lexical emissions, morphology, script, and content deviations are modeled rather than forced to be identical. Validated, calibrated, provisional, and unresolved language profiles are reported separately.

Repeated report vocabulary is modeled through corpus-background, template, section, style, copied-text, prompt, modality, and substantive-topic sources. It is not silently removed by stopword lists, TF-IDF, or BM25.

Topic proportions are compositional (Aitchison, 1982). ESEM and network analysis consume logistic-normal latent coordinates or orthonormal log-ratio coordinates, with posterior uncertainty propagated through plausible values or a joint model (Asparouhov & Muthén, 2009; Asparouhov et al., 2018; Marsh et al., 2014). The product topic-estimator contract is TRSL-TM (ADR 0012); an STM-style logistic-normal family is a reference formulation, not a shipped-backend claim (Blei & Lafferty, 2006; Roberts et al., 2014, 2019). TDT/CHRONOS event intelligence remains evidence-gated (Allan, 2002; Anagnostopoulos et al., 2013).

Rasch identity is distinct from generic 1PL. Cross-classification and multiple membership are distinct. Known hierarchy/testlet/rater/method/item-family effects precede residual latent-space dependence. Temporal candidate activation requires exact formulation identity, identification/alignment, Rust estimator, primary citations, required data support, and passing recovery; automatic enumeration is never automatic activation.

## Compute architecture

The CPU `f64` implementation is the numerical reference. Fixed worker pools and thread-local sufficient statistics minimize context switching and oversubscription. Accelerator work is streamed and parity-verified; temporary responsibilities are not retained for the full corpus. OOM is an expected state with bounded retry/fallback behavior, not a hidden sample-size reduction.

## Persistence

PostgreSQL is the reference relational store. Database objects use two-or-more-word `snake_case` names. Temporal and membership persistence is normalized, interval/bitemporal constrained, idempotent where commands can be replayed, and immutable for evidence/provenance records. `audit_event` inserts call the operational logging gate before SQL is rendered so source text and source identity cannot enter the row. Cross-service SQL is prohibited.

## Security and trust boundaries

Documents and LLM outputs are untrusted. Exact spans, schema validation, size/depth limits, Unicode validity, prompt-injection isolation, tenant/purpose authorization, immutable audit evidence, dependency pinning, SBOM, provenance, and reproducible releases are mandatory. LLM live tests use `NVIDIA_NIM_API_KEY`; `COPILOT_GITHUB_TOKEN` is forbidden. The contextual-orchestrator migration is owned by its separate consumer-integration vehicle and is not silently folded into this Longitudinal Modeling branch.

## References

The full APA 7th register is [`docs/research/standards-and-literature.md`](docs/research/standards-and-literature.md). Detailed Longitudinal Modeling equations and exact implementation/recovery evidence are traced in TRACEABILITY/doctoring and source tests rather than duplicated in responsibility rows.

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B, 44*(2), 139–177. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A reasoning engine for qualitative temporal information in OWL. *Procedia Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

Asparouhov, T., Hamaker, E. L., & Muthén, B. (2018). Dynamic structural equation models. *Structural Equation Modeling, 25*(3), 359–388. https://doi.org/10.1080/10705511.2017.1406803

Asparouhov, T., & Muthén, B. (2009). Exploratory structural equation modeling. *Structural Equation Modeling, 16*(3), 397–438. https://doi.org/10.1080/10705510903008204

Blei, D. M., & Lafferty, J. D. (2006). Dynamic topic models. In *Proceedings of the 23rd International Conference on Machine Learning* (pp. 113–120). ACM. https://doi.org/10.1145/1143844.1143859

Marsh, H. W., Morin, A. J. S., Parker, P. D., & Kaur, G. (2014). Exploratory structural equation modeling: An integration of the best features of exploratory and confirmatory factor analysis. *Annual Review of Clinical Psychology, 10*, 85–110. https://doi.org/10.1146/annurev-clinpsy-032813-153700

Roberts, M. E., Stewart, B. M., Tingley, D., Lucas, C., Leder-Luis, J., Gadarian, S. K., Albertson, B., & Rand, D. G. (2014). Structural topic models for open-ended survey responses. *American Journal of Political Science, 58*(4), 1064–1082. https://doi.org/10.1111/ajps.12103

Roberts, M. E., Stewart, B. M., & Tingley, D. (2019). stm: An R package for structural topic models. *Journal of Statistical Software, 91*(2), 1–40. https://doi.org/10.18637/jss.v091.i02
