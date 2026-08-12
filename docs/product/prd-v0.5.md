# Temporal Event Psychometrics Platform — Product Requirements Document v0.5

**Status:** Approved elaborated product baseline  
**Effective date:** 2026-08-13  
**Product name:** Temporal Event Psychometrics Platform (TEPP)  
**Measurement family:** Temporal Relational Shared-Latent Topic Measurement (TRSL-TM)  
**Supersedes:** `docs/product/prd-v0.4-approved.md` as the canonical product-requirements contract; v0.4 remains historical design evidence  
**Decision authority:** ADR 0001–0016; this version elaborates product behavior and acceptance criteria without changing the approved latent-variable, temporal, ontology, privacy, or service-authority decisions  
**Implementation baseline inspected:** protected `main` at `1832026121e7ad92d21e0592fdd0ad5a59f40cff`; capabilities on an open pull request remain `active-PR`, not `implemented-main`

## 0. How to read this PRD

This document is the executable product contract for TEPP. It specifies:

- the buyer problems and decisions the product supports;
- the exact data, modeling, workflow, evidence, security, privacy, and export boundaries;
- functional requirements with stable identifiers;
- fail-closed behavior and user-visible error semantics;
- scientific claim-promotion criteria;
- product tiers, scale profiles, release slices, and measurable acceptance evidence;
- traceability from product requirements to ADRs, technical documents, source crates, tests, and release artifacts.

The words **shall**, **must**, and **required** define release-gating obligations. **May** identifies optional behavior. **Target** identifies a measurable product objective that can be revised only through reviewed product evidence. A feature is not considered shipped because it appears in this document, an ADR, a branch, a pull request, a demonstration, or an LLM response.

## 1. Product thesis

TEPP turns multilingual reports and related documentary evidence into a reproducible temporal-event measurement system. It measures documents, passages, events, entity roles, topics, higher-order factors, and structural paths while preserving uncertainty, chronology, multilevel membership, provenance, and the distinction between observed evidence and model-derived claims.

TEPP is not:

- a bag-of-words dashboard;
- a keyword classifier presented as topic modeling;
- a generic embedding cluster explorer;
- an LLM summarizer whose prose is treated as statistical truth;
- a causal-discovery engine that promotes temporal precedence into causality;
- a database that silently erases personally identifiable information needed for valid authorship, linkage, or longitudinal analysis.

The product combines:

1. immutable evidence and exact source-span identity;
2. six-clock temporal semantics and historical knowledge-cutoff control;
3. document, passage, event, entity, role, membership, revision, translation, citation, and evidence relations;
4. multilingual semantic-unit measurement in one shared latent space with native lexical channels;
5. temporal and relational topic posteriors with structural covariates and method effects;
6. statistically gated candidate-topic-count selection with blinded LLM review;
7. posterior-aware topic networks and consensus clusters;
8. TDT-style detection/tracking and CHRONOS-style schema/prediction reasoning;
9. longitudinal ESEM, DSEM, and continuous-time structural modeling;
10. evidence-bounded interpretation with independent verification;
11. coordinated accessible visual analytics;
12. versioned APIs, artifacts, manifests, audit trails, and reproducible release evidence.

## 2. Buyer problems

### 2.1 Evidence fragmentation

Enterprise reports are distributed across languages, versions, templates, authors, departments, customers, partners, competitors, projects, opportunities, and reporting systems. Ordinary document clustering loses the connections among those records and treats translated or revised documents as independent observations.

### 2.2 Temporal leakage

A document can describe an old event while becoming available much later. Using event time or document time as the only date leaks future evidence into historical analyses and creates false predictive or structural validity.

### 2.3 Method contamination

Repeated headings, report forms, copied prior-period paragraphs, question prompts, boilerplate, style, modality, and document-source conventions can dominate lexical frequency and become false substantive topics.

### 2.4 Multilingual non-equivalence

Separate language models or post-hoc topic matching can assign different latent meanings to equivalent evidence. Architectural multilingual support does not prove scale alignment, measurement invariance, or equal error across languages.

### 2.5 Unqualified latent claims

A topic is a latent model component, but it is not automatically a validated psychological construct. Raw topic proportions are compositional, topic point estimates contain uncertainty, and input→process→outcome arrows are not causal merely because they are drawn.

### 2.6 Unverifiable LLM interpretation

Unbounded LLM summaries can reverse coefficient directions, omit uncertainty, invent evidence, treat correlations as causes, or generalize from minority groups. Buyers require every claim to point back to source evidence and model artifacts.

### 2.7 Compute and deployment constraints

Customers may have 4–24 GB GPUs, CPU-only environments, air-gapped or regional deployments, strict data residency, and reports containing PII that cannot be destructively masked. The product must remain usable and scientifically comparable across supported backends.

## 3. Product outcomes

A successful TEPP deployment enables a qualified user to:

- freeze a governed corpus and reproduce exactly which evidence was eligible at a chosen historical cutoff;
- inspect the relationship among source documents, passages, events, actors, projects, and topic/factor outputs;
- fit and compare multilingual temporal-relational topic candidates;
- understand why a candidate topic count was accepted, rejected, or escalated for human review;
- quantify topic prevalence, content, drift, covariance, network edges, clusters, and structural effects with uncertainty;
- test whether comparisons across language, time, template, source, or group are defensible;
- distinguish substantive change from lexical, semantic, method, reporting, and measurement drift;
- export accessible exact-value tables, graphs, manifests, and evidence bundles;
- replay the analysis using versioned inputs, software, model, prompts, seeds, and hardware metadata;
- enforce purpose-bound PII use, tenant isolation, selective disclosure, retention, and auditable privileged access.

## 4. Users, roles, and jobs to be done

| Role | Primary job | Decisions supported | Prohibited assumption |
|---|---|---|---|
| Psychometrician / methodologist | Define estimands, assess measurement structure, validate recovery and invariance | Whether topic/factor/structural claims are statistically defensible | A discovered topic is automatically a construct |
| Research analyst | Build corpora, compare candidate models, review evidence, publish analyses | Which model and interpretation best answer a research question | Highest likelihood or LLM preference alone determines the model |
| Domain expert | Review concepts, topics, event schemas, inclusion/exclusion rules | Whether model labels and evidence are substantively meaningful | Model fluency implies domain validity |
| Data engineer | Ingest evidence, map metadata, operate persistence and exports | Whether lineage, schemas, jobs, and replay are complete | A successful parser proves analytical validity |
| Model-risk / audit reviewer | Inspect evidence, claims, access, lineage, and release gates | Whether a result can be relied upon for a declared use | Green CI equals scientific validity |
| Enterprise administrator | Configure tenants, identities, providers, retention, and deployment controls | Who may access which evidence and which external model providers | Blanket masking is the only privacy control |
| Downstream application owner | Consume versioned TEPP analysis artifacts or APIs | Whether an artifact is compatible and sufficiently mature | Lexical heuristics are a substitute for TEPP inference |
| Executive / strategy consumer | Read approved reports and exact-value views | Which patterns warrant investigation or action | Associations or forecasts are causal facts |

## 5. Product surfaces

### 5.1 Evidence workspace

The evidence workspace accepts source files, text, metadata, relations, and access-policy information. It exposes immutable identities, exact spans, six clocks, lineage, parsing status, quarantine status, and provenance.

### 5.2 Corpus builder

The corpus builder defines eligibility, deduplication, relation-connected partition groups, method-source labels, language profiles, covariates, memberships, and historical cutoffs. It emits a frozen corpus manifest.

### 5.3 Semantic measurement workbench

The workbench displays deterministic segmentation, morphology, part of speech, dependency phrases, negation, modality, temporal expressions, LLM-proposed semantic units, concept mappings, unknown concepts, reviewer decisions, and language-profile diagnostics.

### 5.4 Candidate-model workbench

The workbench creates candidate topic-count and model-family plans, estimates resource needs, launches fits, displays convergence and posterior diagnostics, and builds the Pareto frontier used for model review.

### 5.5 Topic and network explorer

The explorer displays topic definitions, representative evidence, prevalence, content effects, temporal activity, drift, valid covariance coordinates, network edges, consensus clusters, and uncertainty.

### 5.6 Event intelligence console

The console supports story segmentation, link detection, event detection, first-story/onset detection, tracking, event schema instantiation, next-event candidates, temporal consistency, and promotion of model-derived relations only after evidence and policy gates.

### 5.7 Psychometric model builder

The builder maps topic posterior coordinates or balances to reflective, formative, network, ESEM, set-ESEM, DSEM, or continuous-time structures. It displays invariance tests, within/between decomposition, direct/indirect effects, identification warnings, and claim boundaries.

### 5.8 Interpretation and publication workspace

The workspace generates evidence-bounded topic, cluster, event, and structural interpretations; runs independent verification; removes or marks unsupported statements; records approvals; and exports accessible reports and machine-readable artifacts.

### 5.9 Administration and audit

The administration surface manages tenants, purposes, roles, identity mappings, model-provider disclosure, retention, deletion, exports, privileged access, compute profiles, model registries, and audit evidence.

## 6. Product editions and deployment modes

| Mode | Required capability | Data boundary | Intended use |
|---|---|---|---|
| Embedded library | Rust crates and versioned artifact contracts | Host application owns transport and persistence | Integration into another CWL or customer product |
| Standalone workstation | Local service, local object store/database, CPU and optional GPU | Single-user or controlled-team environment | Research and governed analyst workflows |
| Enterprise self-hosted | API/worker/persistence/object store, tenant isolation, KMS, audit, SSO integration | Customer-controlled region/network | Sensitive enterprise and regulated deployments |
| Managed service | Same logical contracts plus operated control plane | Contracted regional/provider controls | Organizations preferring managed operations |
| Air-gapped profile | Offline model/provider adapters, import/export receipts, no external fetch | Isolated environment | High-sensitivity or disconnected deployment |

All deployment modes shall use the same scientific artifact and claim-promotion contracts. A deployment mode may omit a provider or visual surface, but it may not silently weaken temporal, evidence, tenant, or statistical semantics.

## 7. Current implementation maturity

At the inspected protected-main baseline:

- `evidence_core`, typed temporal primitives, bounded interval reasoning, forward-only relation graph, leakage-safe corpus splitting, deterministic simulations, recovery metrics, and selected API/export contracts are implemented on protected main;
- `event_core`, `membership_core`, `persistence_postgres`, `tepp_api`, modular connector contracts, and release-evidence tooling are partial;
- tenant FORCE RLS and runtime-role restrictions exist on open PR #30 and remain `active-PR` until integrated;
- multilingual semantic measurement, TRSL-TM estimation, candidate-K selection, topic networks/clusters, GPU backends, TDT/CHRONOS intelligence, longitudinal ESEM/DSEM, interpretation, and coordinated visual analytics remain accepted targets.

This section is informational. `docs/TRACEABILITY.md` is the authoritative implementation-maturity ledger and must be updated when protected main changes.

## 8. Core domain concepts

| Concept | Product meaning |
|---|---|
| Source artifact | Immutable bytes or text accepted at the trust boundary |
| Document record | Versioned documentary unit derived from a source artifact |
| Text segment | Exact source-span unit used for measurement or evidence |
| Temporal interval | Typed instant/interval with boundary, precision, certainty, and provenance |
| Document relation | Typed observed or inferred link between documents |
| Event mention | A fallible textual observation of an event |
| Event instance | A governed analytical entity promoted from one or more mentions/evidence |
| Entity role assignment | Time-varying contextual role such as customer, partner, competitor, author, or department |
| Membership assignment | Weighted, validity-bounded cross-classified membership |
| Semantic unit | Span-grounded candidate meaning observation |
| Concept mapping | Versioned mapping from a semantic unit to a governed concept |
| Corpus snapshot | Immutable eligibility- and cutoff-bound analysis input |
| Model run | Versioned execution with complete configuration and provenance |
| Topic posterior | Distribution over document/topic latent coordinates and parameters |
| Factor solution | ESEM/DSEM or related measurement solution with identification and uncertainty |
| Scientific claim | Human- or model-authored statement linked to evidence and analysis artifacts |
| Release evidence | Exact-head bundle proving software, scientific, security, migration, and operational gates |

## 9. Input contracts

### 9.1 Source artifact

A source artifact shall include:

- `source_artifact_id` as an opaque RFC 9562 UUIDv7;
- canonical SHA-256 content digest;
- byte size and accepted content type;
- original filename or external identifier when supplied;
- tenant, purpose, and retention class;
- system-observed timestamp;
- ingestion method and producer identity;
- immutable raw payload reference;
- validation and quarantine state.

### 9.2 Document record

A document record shall include:

- `document_record_id`, `source_artifact_id`, and version lineage;
- exact content and encoding identity;
- document type, source system, jurisdiction/region when relevant;
- language/script posterior summaries;
- event, assertion, document, system, and availability times or typed intervals;
- template, section, prompt, copied-text, and source-method hints;
- author/department/organization/project/opportunity hints with provenance;
- relation and evidence references;
- access-policy bindings.

### 9.3 Relation input

Observed relations may originate from hyperlinks, reference lists, document-management metadata, version identifiers, translation identifiers, attachments, or explicit user input. Inferred relations shall carry:

- relation type and direction;
- confidence and calibrated status;
- source and target evidence spans;
- extractor/model/prompt/version identity;
- observed-versus-inferred classification;
- verification status;
- validity interval where applicable.

### 9.4 Covariates and memberships

Covariates shall declare:

- stable identifier and display label;
- data type and permissible values;
- missingness semantics;
- level of observation;
- time-varying status;
- whether used for prevalence, content, method, event, measurement, or structural modeling;
- transformation and reference category;
- access and sensitivity class.

Memberships shall permit multiple assignments with weights, validity intervals, source evidence, and confidence. Weights must declare whether they are supplied, normalized, inferred, or estimated.

### 9.5 Language profile input

Language and script labels shall use versioned BCP 47-compatible identifiers. A document may contain multiple language spans. The system shall preserve the posterior or uncertainty of language identification rather than force a single document-level code when evidence is mixed.

### 9.6 LLM semantic-unit proposal

An LLM proposal shall contain only approved structured fields, including:

- document and span identity;
- exact `source_start` and `source_end`;
- `surface_text` that exactly matches source evidence;
- candidate `concept_key` or governed unknown status;
- polarity, modality, quantity, temporal-expression, and negation attributes when applicable;
- language/script label;
- confidence and rationale code;
- provider/model, prompt hash, reasoning effort, workflow depth, role, access list, and version.

Free-form model prose cannot enter statistical input tables.

## 10. Output and artifact contracts

Every published artifact shall contain or reference:

- artifact identifier and semantic version;
- tenant and authorized purpose;
- corpus, relation, cutoff, and configuration hashes;
- source-code commit, dependency lock, build provenance, and model versions;
- estimator/backend/precision/seed metadata;
- prompt/provider/reasoning metadata when an LLM participated;
- scientific maturity and claim boundary;
- generated-at system time;
- checksum and media type;
- parent artifacts and supersession status.

The minimum artifact families are:

1. corpus snapshot manifest;
2. semantic-unit and concept-mapping dataset;
3. candidate-model plan and resource estimate;
4. topic posterior and parameter artifact;
5. model-selection decision record;
6. topic network and consensus-cluster artifact;
7. event graph and event-intelligence artifact;
8. invariance and psychometric-model artifact;
9. interpretation claim bundle;
10. exact-value visual/export package;
11. validation report;
12. audit and release-evidence bundle.

## 11. Lifecycle state machines

### 11.1 Evidence lifecycle

```text
received
→ bounded
→ hashed
→ validated
→ accepted
→ versioned
→ retained / exported / deleted_by_policy
```

Alternative states are `quarantined`, `rejected`, and `superseded`. Quarantined evidence cannot enter a corpus snapshot.

### 11.2 Corpus lifecycle

```text
draft
→ eligibility_checked
→ relation_grouped
→ leakage_checked
→ frozen
→ analyzable
→ archived
```

Any source, relation, cutoff, concept-dictionary, policy, or membership change creates a new corpus snapshot rather than mutating the frozen snapshot.

### 11.3 Model-run lifecycle

```text
created
→ contract_validated
→ resources_admitted
→ fitting
→ diagnostics
→ candidate
→ reviewed
→ accepted / rejected / human_review_required
→ registered
→ archived
```

`failed`, `cancelled`, `resource_exhausted`, `invalid_scientific_contract`, and `inconclusive` are terminal or restartable states with distinct error records.

### 11.4 Claim lifecycle

```text
proposed
→ evidence_bound
→ independently_verified
→ human_approved_when_required
→ published
```

A claim may transition to `unsupported`, `ambiguous`, `superseded`, or `withdrawn`. Published claims retain immutable evidence and model references.

### 11.5 Release lifecycle

```text
candidate_head
→ software_gates_passed
→ scientific_gates_passed
→ security_privacy_gates_passed
→ migration_recovery_gates_passed
→ operational_gates_passed
→ release_approved
→ published
→ verified
```

## 12. Functional requirements

Each requirement below is independently testable. Acceptance evidence may combine deterministic tests, integration tests, simulation studies, current-head workflow results, signed manifests, and qualified human review.

### FR-EVD-001 — Immutable source acceptance

**Requirement.** The system shall copy accepted source bytes/text into immutable owned storage, calculate the canonical SHA-256 digest, assign an opaque UUIDv7 identity, and reject content that exceeds configured bounds.

**Acceptance evidence.** Digest vectors, mutation tests, size-limit tests, and round-trip reconstruction.

### FR-EVD-002 — Exact evidence spans

**Requirement.** The system shall preserve half-open UTF-8 byte and Unicode-scalar coordinates plus optional page/layout geometry and reject cross-document, mid-code-point, reversed, non-finite, or out-of-page spans.

**Acceptance evidence.** Multilingual hostile-span and geometry regression suite.

### FR-EVD-003 — No active-content execution

**Requirement.** The system shall parse documentary content without executing scripts, macros, embedded instructions, or external resource fetches unless a separately authorized connector contract explicitly permits a bounded retrieval.

**Acceptance evidence.** Security tests with active-content and external-fetch fixtures.

### FR-EVD-004 — Quarantine semantics

**Requirement.** Malformed, unsupported, untrusted, or policy-ineligible evidence shall enter a distinct quarantine/rejection state with redacted diagnostics and shall not enter a corpus snapshot.

**Acceptance evidence.** State-machine and corpus-eligibility integration tests.

### FR-EVD-005 — Evidence versioning

**Requirement.** A revision shall create a new document version linked to prior versions without overwriting the original artifact or historical system-time state.

**Acceptance evidence.** Bitemporal revision and as-known-at replay tests.

### FR-TMP-001 — Six nominal clocks

**Requirement.** Event/valid, assertion, document, system, availability, and knowledge-cutoff times shall remain distinct typed values through API, persistence, modeling, and export.

**Acceptance evidence.** Compile-time type tests, schema discrimination, persistence round trips.

### FR-TMP-002 — Uncertain interval preservation

**Requirement.** Exact, bounded, open, and unknown temporal evidence shall preserve inclusion/exclusion, precision, certainty, and provenance without coercion to a convenient point.

**Acceptance evidence.** Interval property tests and wire-schema parity.

### FR-TMP-003 — Historical eligibility

**Requirement.** A corpus snapshot shall include evidence only when the configured availability policy proves it eligible at the knowledge cutoff; uncertain availability crossing the cutoff fails closed by default.

**Acceptance evidence.** Retrospective-document and cutoff-crossing simulations.

### FR-TMP-004 — Partial-order reasoning

**Requirement.** The system shall support Allen-style interval relations with bounded path-consistency, contradiction witnesses, provenance, and resource budgets without claiming unrestricted global satisfiability.

**Acceptance evidence.** Relation algebra, converse/composition, rollback, and budget tests.

### FR-TMP-005 — Forward-only transitions

**Requirement.** State-transition and input→process→outcome edges shall obey valid temporal order and acyclicity; retrospective evidence relations may point backward but cannot become reverse transitions.

**Acceptance evidence.** Graph cycle, reverse-edge, and relation-class tests.

### FR-REL-001 — Typed relation graph

**Requirement.** Document, segment, event, entity-role, membership, topic, factor, and provenance relations shall use versioned typed edges with direction, source, confidence, validity, and evidence.

**Acceptance evidence.** Schema and graph reconstruction tests.

### FR-REL-002 — Observed/inferred separation

**Requirement.** The system shall never promote an inferred relation to observed fact merely because confidence is high or multiple models agree.

**Acceptance evidence.** Promotion-policy and artifact-label tests.

### FR-REL-003 — Relation-aware partitioning

**Requirement.** Translations, revisions, copied variants, and members of the same governed episode shall remain within one validation partition when separation would cause leakage.

**Acceptance evidence.** Connected-component split and adversarial leakage tests.

### FR-MEM-001 — Cross-classified membership

**Requirement.** One observation may belong simultaneously to multiple authors, departments, organizations, projects, opportunities, markets, templates, languages, locations, or episodes.

**Acceptance evidence.** Weighted membership construction and retrieval tests.

### FR-MEM-002 — Time-varying roles

**Requirement.** Customer, partner, competitor, author, department, and related roles shall be contextual assignments with validity intervals rather than immutable entity types.

**Acceptance evidence.** Role-overlap, role-change, and temporal-query tests.

### FR-MEM-003 — Membership weighting

**Requirement.** Supplied, normalized, inferred, and estimated membership weights shall be distinguishable and shall retain provenance and normalization policy.

**Acceptance evidence.** Weight-sum, ESS, design-effect, and provenance tests.

### FR-MEM-004 — Atomistic-fallacy warning

**Requirement.** Analyses that collapse materially clustered or multiply assigned observations into independent units shall fail or display an explicit scientific warning according to model policy.

**Acceptance evidence.** Simulation comparing naïve and multilevel estimates.

### FR-LNG-001 — Span-level language identification

**Requirement.** The system shall support language/script posterior estimates at segment or span level and preserve mixed-language uncertainty.

**Acceptance evidence.** Code-switching gold corpus and calibration tests.

### FR-LNG-002 — Shared latent identity

**Requirement.** Supported language profiles shall map equivalent evidence to the same global topic identities and latent coordinate system while retaining native lexical channels.

**Acceptance evidence.** Parallel/equivalent corpus alignment and invariance studies.

### FR-LNG-003 — Language support tiers

**Requirement.** Each language profile shall be labeled validated, calibrated, provisional, or unresolved from versioned evidence; configuration alone cannot mark a language validated.

**Acceptance evidence.** Profile-promotion contract and report tests.

### FR-LNG-004 — Launch language profiles

**Requirement.** The product shall define validation profiles for English, Korean, Japanese, Simplified Chinese, Traditional Chinese, Vietnamese, Indonesian, French, German, and Turkish; long-tail languages remain provisional until evidence promotes them.

**Acceptance evidence.** Per-profile benchmark manifests and maturity reports.

### FR-SEM-001 — Non-destructive normalization

**Requirement.** Original text shall remain immutable; NFC may be used for analysis and NFKC only for explicitly declared auxiliary keys where compatibility folding is acceptable.

**Acceptance evidence.** Unicode normalization and source-span identity tests.

### FR-SEM-002 — Boundary tailoring

**Requirement.** Segmentation shall combine Unicode boundary rules, language/script tailoring, layout, headings, lists, tables, and code-switching behavior rather than assuming whitespace tokenization.

**Acceptance evidence.** Multilingual segmentation gold tests.

### FR-SEM-003 — POS as soft source information

**Requirement.** Part of speech, dependency structure, negation, modality, quantity, and temporal expressions shall inform source/model priors without default hard deletion.

**Acceptance evidence.** Ablation showing retained negation/modality and method-source behavior.

### FR-SEM-004 — No default stopword deletion

**Requirement.** The default inferential pipeline shall not delete stopwords; any exclusion must be versioned, justified, reversible, and evaluated as an ablation.

**Acceptance evidence.** Configuration and reproducibility tests.

### FR-SEM-005 — No TF-IDF/BM25 inferential weighting

**Requirement.** TF-IDF and BM25 may support retrieval but shall not weight topic estimation, topic correlation, ESEM, DSEM, or substantive importance.

**Acceptance evidence.** Pipeline configuration and artifact provenance tests.

### FR-SEM-006 — Method-source decomposition

**Requirement.** Template, section, copied text, prompt, corpus background, style, modality, metadata, and substantive-topic sources shall be represented separately where supported.

**Acceptance evidence.** Synthetic method-effect recovery and false-topic tests.

### FR-SEM-007 — Governed concept dictionary

**Requirement.** Concept identifiers, labels, aliases, merges, splits, language mappings, and unknown concepts shall be versioned and reviewable; an LLM cannot silently mutate the dictionary.

**Acceptance evidence.** Dictionary versioning and unauthorized-mutation tests.

### FR-SEM-008 — LLM proposal validation

**Requirement.** LLM semantic units shall be accepted only when exact spans, surface text, schema, concept policy, size bounds, and injection controls pass deterministic validation.

**Acceptance evidence.** Hostile JSON, span substitution, concept, and prompt-injection tests.

### FR-TOP-001 — CPU f64 reference estimator

**Requirement.** The reference TRSL-TM estimator shall use Rust CPU f64 arithmetic and expose objective, convergence, posterior, and diagnostics sufficient for independent validation.

**Acceptance evidence.** R/independent oracle comparisons and true-parameter recovery.

### FR-TOP-002 — Structural covariates

**Requirement.** The estimator shall support prevalence, content, method, temporal, relational, and multilevel covariates with declared design matrices and reference categories.

**Acceptance evidence.** Known-covariate simulation recovery.

### FR-TOP-003 — Posterior uncertainty

**Requirement.** Document coordinates, topic parameters, covariate effects, drift, and relevant derived quantities shall retain posterior or resampling uncertainty; point estimates alone are insufficient.

**Acceptance evidence.** Coverage and plausible-value tests.

### FR-TOP-004 — Drift decomposition

**Requirement.** The product shall distinguish prevalence, semantic, lexical, measurement, method, and reporting drift to the extent identified by the selected model and evidence.

**Acceptance evidence.** Known-drift simulation and non-identifiability warnings.

### FR-TOP-005 — Global topic identity P0

**Requirement.** The initial production family shall use one global topic identity across the declared analysis window and represent activation, dormancy, and reactivation without fitting unrelated topic sets per time slice.

**Acceptance evidence.** Temporal identity and reactivation simulations.

### FR-TOP-006 — Backend compatibility gate

**Requirement.** A Polylingual, GloCTM-aligned, neural, or external adapter may be registered only when it satisfies TEPP posterior, temporal, relation, invariance, provenance, and recovery contracts.

**Acceptance evidence.** Adapter conformance suite.

### FR-TOP-007 — Model unavailable fail-closed

**Requirement.** A downstream request requiring a fitted model shall return a typed unavailable/incompatible error rather than substitute lexical or keyword pseudo-analysis.

**Acceptance evidence.** API and connector integration tests.

### FR-KSEL-001 — Candidate plan

**Requirement.** The user shall define or approve candidate topic counts, seeds, resamples, time windows, model families, compute budgets, and stopping rules before fitting.

**Acceptance evidence.** Immutable candidate-plan manifest.

### FR-KSEL-002 — Statistical hard gates

**Requirement.** Non-converged, collapsed, highly redundant, unstable, misaligned, scientifically invalid, or infeasible candidates shall be rejected before LLM review.

**Acceptance evidence.** Known-bad candidate fixture suite.

### FR-KSEL-003 — Pareto frontier

**Requirement.** Surviving candidates shall be compared across predictive fit, posterior checks, coherence, exclusivity, coverage, redundancy, stability, parsimony, fairness, multilingual alignment, and compute feasibility.

**Acceptance evidence.** Deterministic frontier calculation tests.

### FR-KSEL-004 — Blinded LLM review

**Requirement.** LLM reviewers shall see blinded candidate identities and evidence bundles rather than raw topic-count prestige or developer labels.

**Acceptance evidence.** Prompt/schema snapshots and leakage tests.

### FR-KSEL-005 — Decision output

**Requirement.** Selection shall output a recommended count, acceptable set, rejected candidates with reasons, statistical trade-offs, reviewer disagreement, confidence, and human-review status.

**Acceptance evidence.** Golden decision-record tests.

### FR-KSEL-006 — Human escalation

**Requirement.** Large reviewer disagreement, statistical/semantic conflict, weak evidence coverage, or policy-defined high-stakes use shall return `human_review_required`.

**Acceptance evidence.** Escalation-rule tests.

### FR-NET-001 — Valid topic coordinates

**Requirement.** Topic association shall use logistic-normal latent coordinates or declared orthonormal log-ratio coordinates, never naïve Pearson correlation of raw proportions.

**Acceptance evidence.** Compositional-data regression tests.

### FR-NET-002 — Posterior edge evidence

**Requirement.** Every edge shall include estimate, interval, selection probability, bootstrap/seed stability, sample basis, coordinate transform, and threshold/correction policy.

**Acceptance evidence.** Network artifact schema and recovery tests.

### FR-NET-003 — Conditional network gate

**Requirement.** Sparse conditional networks shall operate only in a valid coordinate space with tuning, stability, and uncertainty evidence.

**Acceptance evidence.** Known-precision-matrix recovery.

### FR-NET-004 — Consensus clustering

**Requirement.** Stable positive associations shall feed repeated Leiden or approved community detection and a co-assignment consensus matrix; unstable topics may remain unassigned.

**Acceptance evidence.** Known-community ARI/NMI and stability tests.

### FR-NET-005 — Negative association semantics

**Requirement.** Negative associations shall be displayed as opposition/tension and shall not be used as ordinary positive cluster membership.

**Acceptance evidence.** Graph encoding and accessibility tests.

### FR-EVT-001 — Mention/instance separation

**Requirement.** Event mentions shall remain fallible evidence observations; event instances require an explicit promotion record and supporting evidence.

**Acceptance evidence.** Promotion refusal and provenance tests.

### FR-EVT-002 — TDT task separation

**Requirement.** Story segmentation, link detection, event detection, first-story/onset detection, and tracking shall have separate metrics, thresholds, and error reports.

**Acceptance evidence.** Task-specific benchmark suite.

### FR-EVT-003 — Schema hypothesis boundary

**Requirement.** CHRONOS-style schemas, arguments, subevents, and next-event candidates shall remain hypotheses until deterministic temporal and evidence gates pass.

**Acceptance evidence.** Hypothesis/promotion state tests.

### FR-EVT-004 — Forecast calibration

**Requirement.** Event prediction shall report horizon, base rate, calibration, discrimination, uncertainty, and abstention; it shall not expose uncalibrated confidence as probability.

**Acceptance evidence.** Calibration and OOD tests.

### FR-PSY-001 — Construct-role gate

**Requirement.** Before SEM, the user shall classify or compare topic indicators as reflective, formative/composite, network, or unresolved; the product shall prevent an unqualified reflective default.

**Acceptance evidence.** Model-builder contract tests.

### FR-PSY-002 — Posterior propagation

**Requirement.** ESEM/DSEM shall consume plausible values or a joint posterior strategy and shall not treat topic posterior means as error-free observations.

**Acceptance evidence.** Simulation comparing naïve and propagated uncertainty.

### FR-PSY-003 — Invariance workflow

**Requirement.** Language, time, template, source, and group comparisons shall report configural, metric, scalar when means are compared, residual/method, partial, or time-varying invariance as applicable.

**Acceptance evidence.** Known non-invariance simulations.

### FR-PSY-004 — Within/between separation

**Requirement.** Longitudinal models shall distinguish stable between-unit differences from within-unit change and account for cross-classified/multiple membership.

**Acceptance evidence.** Multilevel true-parameter recovery.

### FR-PSY-005 — Irregular time

**Requirement.** When observation gaps are materially irregular, the product shall support or recommend a continuous-time formulation rather than pretend equal spacing.

**Acceptance evidence.** Irregular-time simulation recovery.

### FR-PSY-006 — Temporal path validity

**Requirement.** Input→process/intervention→outcome paths shall obey temporal order and display direct, indirect, total, and lag-dependent effects with uncertainty.

**Acceptance evidence.** Known-path simulation and reverse-time rejection.

### FR-PSY-007 — Causal-language gate

**Requirement.** Causal wording shall require a declared identification design and supporting assumptions/evidence; otherwise outputs use associational or predictive language.

**Acceptance evidence.** Claim verifier tests.

### FR-LLM-001 — Untrusted model output

**Requirement.** All LLM output shall be schema-bound, size-bounded, evidence-linked, provenance-recorded, and treated as untrusted data.

**Acceptance evidence.** Malformed/output-injection tests.

### FR-LLM-002 — Interpreter/verifier separation

**Requirement.** Interpretation shall be independently checked for evidence support, direction, uncertainty, causal language, group generalization, and omission.

**Acceptance evidence.** Supported/unsupported claim gold tests.

### FR-LLM-003 — Unsupported claim handling

**Requirement.** Unsupported clauses shall be removed, marked, or escalated; they shall not be silently published in an approved report.

**Acceptance evidence.** Publication state tests.

### FR-LLM-004 — Adaptive orchestration evidence

**Requirement.** Direct, verifier, fixed committee, and adaptive conductor strategies shall be compared under declared budgets with stages, decomposition, recursion, access lists, roles, and role-specific reasoning effort recorded.

**Acceptance evidence.** Ablation manifest and score comparison.

### FR-LLM-005 — Credential boundary

**Requirement.** Approved live model work shall use `NVIDIA_NIM_API_KEY` or an explicitly approved provider credential; `COPILOT_GITHUB_TOKEN` is prohibited and reviewer credentials remain separate.

**Acceptance evidence.** Workflow static and secret-boundary tests.

### FR-LLM-006 — No deterministic-authority substitution

**Requirement.** LLMs shall not replace time algebra, hashing, authorization, schema validation, numerical estimation, coverage, migration, or release authority.

**Acceptance evidence.** Architecture and integration tests.

### FR-CMP-001 — Backend-neutral compute plan

**Requirement.** Every run shall record CPU/GPU backend, device, precision, memory budget, concurrency, deterministic mode, and fallback policy.

**Acceptance evidence.** Compute-profile manifest tests.

### FR-CMP-002 — Bounded CPU parallelism

**Requirement.** CPU work shall use bounded worker pools and avoid uncontrolled BLAS/runtime oversubscription and per-request thread creation.

**Acceptance evidence.** Concurrency and oversubscription benchmarks.

### FR-CMP-003 — VRAM admission control

**Requirement.** The GPU controller shall predict peak memory, reserve safety margin, tune micro-batches, bound retries, and reject or route infeasible work before uncontrolled OOM.

**Acceptance evidence.** 4/6/8/12/24-GB profile tests.

### FR-CMP-004 — Streaming sufficient statistics

**Requirement.** Document×topic responsibilities shall be batch-local and released after stable sufficient-statistic accumulation rather than globally materialized.

**Acceptance evidence.** Peak-memory and leak tests.

### FR-CMP-005 — CPU/GPU parity

**Requirement.** GPU results shall be compared with the CPU f64 reference for objectives, parameters/posteriors, convergence, validation metrics, and artifacts under versioned tolerances.

**Acceptance evidence.** Real-hardware parity suite.

### FR-CMP-006 — Small-GPU phase scheduling

**Requirement.** Local LLM and topic-model tensors shall not remain concurrently resident when the configured VRAM profile cannot support both safely.

**Acceptance evidence.** Resource-pressure integration tests.

### FR-API-001 — Versioned contracts

**Requirement.** Public commands, APIs, schemas, events, and artifacts shall use explicit semantic versions and typed compatibility errors.

**Acceptance evidence.** Contract and backward-compatibility tests.

### FR-API-002 — Asynchronous jobs

**Requirement.** Long-running analyses shall expose create, status, progress, cancel, retry/replay, and artifact-discovery operations with idempotency.

**Acceptance evidence.** Job-state and idempotency tests.

### FR-API-003 — No cross-service table coupling

**Requirement.** naruon, contextual-orchestrator, and other services shall use versioned TEPP ports/artifacts rather than direct application-table access.

**Acceptance evidence.** Connector architecture tests.

### FR-API-004 — naruon consumer truthfulness

**Requirement.** naruon may consume fitted TEPP artifacts but shall not label keyword or lexical heuristics as TEPP topic inference when a compatible model is unavailable.

**Acceptance evidence.** Consumer contract tests.

### FR-API-005 — Contextual-orchestrator authority

**Requirement.** contextual-orchestrator may execute bounded provider workflows, while TEPP retains evidence, statistical, scientific, artifact, and claim authority.

**Acceptance evidence.** Port conformance and credential tests.

### FR-EXP-001 — Accessible exact-value companion

**Requirement.** Every quantitative visualization shall have an exact-value semantic table with units, uncertainty, missingness, sample basis, and provenance.

**Acceptance evidence.** Keyboard, screen-reader, no-JS, print/PDF tests.

### FR-EXP-002 — Source-consistent exports

**Requirement.** SVG, PDF, CSV, JSON, JSON-LD, GraphML, Arrow, and Parquet outputs shall derive from the same approved data artifact and preserve identifiers and checksums.

**Acceptance evidence.** Cross-format consistency tests.

### FR-EXP-003 — Knowledge-cutoff audit

**Requirement.** A user shall be able to inspect why each evidence record was included or withheld at a historical cutoff.

**Acceptance evidence.** Leakage-audit golden scenarios.

### FR-EXP-004 — Reproducibility manifest

**Requirement.** Every published analysis shall include corpus/relation/cutoff/config/code/dependency/model/prompt/seed/backend/artifact identities.

**Acceptance evidence.** Manifest completeness tests.

### FR-SEC-001 — Tenant isolation

**Requirement.** Persistent and service boundaries shall enforce tenant isolation, including FORCE RLS where PostgreSQL tables are tenant-scoped and runtime roles cannot bypass policy.

**Acceptance evidence.** Live PostgreSQL cross-tenant tests.

### FR-SEC-002 — Purpose-bound PII access

**Requirement.** PII required for valid linkage or modeling shall use purpose, role, tenant, retention, disclosure, and audit controls rather than blanket masking.

**Acceptance evidence.** Authorization/export/retention tests.

### FR-SEC-003 — Identity separation

**Requirement.** Analytical opaque identifiers shall be separable from protected identity mappings; model providers receive only the minimum approved disclosure.

**Acceptance evidence.** Identity-vault and provider payload tests.

### FR-SEC-004 — Encryption and keys

**Requirement.** Sensitive evidence, mappings, model artifacts, and exports shall support encryption in transit and at rest with customer- or tenant-scoped key policy where deployed.

**Acceptance evidence.** Deployment-owned control evidence.

### FR-SEC-005 — Audit completeness

**Requirement.** Evidence access, privileged actions, exports, model-provider disclosure, dictionary changes, model promotion, claims, and releases shall create immutable audit events.

**Acceptance evidence.** Audit-event integration tests.

### FR-SEC-006 — Retention and deletion

**Requirement.** Retention, legal hold, deletion, and export policies shall produce durable state and receipts without silently corrupting reproducibility claims.

**Acceptance evidence.** Lifecycle and tombstone tests.

### FR-OPS-001 — Observability

**Requirement.** The service shall emit structured logs, metrics, traces, job state, queue depth, latency, memory, GPU, error, retry, and scientific-gate telemetry without exposing protected content.

**Acceptance evidence.** OpenTelemetry and redaction tests.

### FR-OPS-002 — Recovery

**Requirement.** Workers and services shall recover or fail closed after interruption, preserving immutable inputs and avoiding duplicate publication.

**Acceptance evidence.** Crash/restart and idempotency tests.

### FR-OPS-003 — Backup and restore

**Requirement.** Enterprise persistence shall define backup, restore, integrity verification, and point-in-time recovery procedures with measured evidence before operational claims.

**Acceptance evidence.** Deployment restore rehearsal.

### FR-OPS-004 — Capacity profiles

**Requirement.** The product shall publish tested corpus, concurrency, CPU, RAM, GPU, storage, and run-time profiles for each release rather than universal scale claims.

**Acceptance evidence.** Benchmark and capacity report.

### FR-OPS-005 — Release evidence

**Requirement.** A release shall include exact-head CI/security, 100% owned-code coverage/docstrings, scientific validation, migrations/rollback, SBOM, provenance, checksums, compatibility, accessibility, and operational evidence.

**Acceptance evidence.** Signed release-evidence bundle.

## 13. Scientific claim-promotion matrix

| Claim | Minimum evidence before publication | Required output wording when evidence is incomplete |
|---|---|---|
| “Topic K exists” | Identified candidate model, recovery/stability, representative evidence, non-collapse, uncertainty, and model-selection record | “Model-derived topic candidate” |
| “Topic prevalence changed” | Posterior/interval evidence, cutoff-safe corpus, method/reporting drift checks, and stable topic identity | “Estimated prevalence difference under the fitted model” |
| “Meaning is equivalent across languages” | Parallel/equivalent evidence, alignment, error analysis, and applicable invariance | “Provisional cross-language alignment” |
| “Topics are correlated” | Valid latent/log-ratio coordinates, uncertainty, sample basis, and stability | “Posterior association estimate” |
| “Topics form a cluster” | Repeated community detection, co-assignment stability, and evidence review | “Consensus cluster candidate” |
| “Factor means differ” | At least scalar or defensible partial/time-varying invariance plus uncertainty | Means are not compared |
| “Input affects outcome through process” | Identified longitudinal model, ordered measurements, indirect-effect uncertainty, and confounding assumptions | “Temporally ordered associational path” |
| “Event is the first occurrence” | Bounded evidence universe, historical cutoff, onset false-alarm evaluation | “First observed eligible mention” |
| “Next event probability is p” | Held-out forecast calibration, horizon/base rate, OOD and abstention evidence | “Uncalibrated model score” |
| “GPU result is equivalent” | Real-device parity across objectives, parameters, diagnostics, and artifacts | “GPU result not parity-qualified” |
| “Language is supported” | Versioned validated/calibrated profile evidence | “Provisional/unresolved language profile” |
| “TEPP is CSAP/SOC 2/ISO certified” | Independent applicable certification or attestation | Claim prohibited; use “readiness/alignment evidence” |

## 14. Language-profile release gates

A language profile is promoted separately for each declared task family: segmentation, semantic-unit mapping, concept mapping, topic measurement, event intelligence, interpretation, and structural comparison.

### 14.1 Validated

A validated profile requires:

- representative gold data for the declared domain and writing styles;
- span and concept error analysis with uncertainty;
- calibration evidence;
- cross-language alignment where comparisons are offered;
- measurement invariance appropriate to the claim;
- no unresolved high-severity systematic error;
- qualified reviewer approval and immutable benchmark manifest.

### 14.2 Calibrated

A calibrated profile has reliable task-level error and confidence calibration but lacks one or more domain, invariance, or expert-review dimensions required for validated comparison.

### 14.3 Provisional

A provisional profile may use the common pipeline and shared model, but outputs visibly lower the claim boundary and may require human review. It cannot be included in aggregate comparisons that assume equivalence.

### 14.4 Unresolved

An unresolved profile preserves source evidence and records failure/unknown status. The product shall not silently translate, drop, or force the evidence into another language profile.

## 15. Topic-count selection contract

### 15.1 Candidate specification

A candidate plan shall declare:

- allowed topic counts or adaptive search rule;
- model families and backend versions;
- seeds, resamples, folds, and time windows;
- training/validation cutoff and partition manifest;
- maximum fit count and compute budget;
- convergence and collapse rules;
- mandatory diagnostics;
- LLM review budget and blindness policy;
- human escalation criteria.

### 15.2 Candidate evidence vector

For each candidate, the product records:

\[
S_K = \left\{
L_{\text{heldout}},
P_{\text{posterior-check}},
C_{\text{coherence}},
E_{\text{exclusivity}},
R_{\text{redundancy}},
S_{\text{stability}},
A_{\text{alignment}},
F_{\text{fairness}},
M_{\text{memory}},
Q_{\text{review}}
\right\}.
\]

Metrics shall not be reduced to a single unexplained score. The accepted model decision shall preserve the complete evidence vector and dominance/trade-off relationships.

### 15.3 LLM evidence bundle

The blinded reviewer receives:

- probability, exclusivity/FREX-like, lift, and coverage concepts;
- representative exact source spans across languages and groups;
- redundancy and uncovered-document summaries;
- prevalence and stability summaries;
- covariate effects with uncertainty;
- research question and declared use;
- no model label, developer label, or unblinded topic count unless policy explicitly permits it after independent scoring.

### 15.4 Selection result

The result schema includes:

- `recommended_candidate_id`;
- `acceptable_candidate_ids`;
- `rejected_candidate_reasons`;
- `pareto_dimensions`;
- `statistical_tradeoffs`;
- `reviewer_scores`;
- `reviewer_disagreement`;
- `human_review_required`;
- `decision_authority`;
- `evidence_artifact_ids`;
- `selection_policy_version`.

## 16. Psychometric modeling contract

### 16.1 Measurement input

Raw \(\theta\) proportions shall not be treated as unconstrained Gaussian indicators. The product uses the model's unconstrained logistic-normal coordinates or declared orthonormal balances and propagates their uncertainty.

### 16.2 Reflective ESEM

A reflective solution is permitted only when a higher-order latent variable is theorized to generate covariance among topic indicators. Cross-loadings are estimated or regularized rather than set to zero without substantive justification.

### 16.3 Formative/composite model

When topics jointly define a construct rather than reflect it, the product uses a composite/formative specification and reports identification and weighting assumptions.

### 16.4 Dynamic model

For episode \(c\), the target decomposition is:

\[
f_{c,t} = \bar f_c + \widetilde f_{c,t},
\]

where \(\bar f_c\) captures stable between-episode differences and \(\widetilde f_{c,t}\) captures within-episode change. Input→process→outcome paths operate on appropriately timed components and declare lag assumptions.

For irregular gaps, the product may fit:

\[
f(t_j)=\exp(F\Delta t)f(t_i)
+\int_0^{\Delta t}\exp(F(\Delta t-s))Bx(t_i+s)\,ds
+\omega_{ij}, \qquad \Delta t>0.
\]

### 16.5 Uncertainty and missingness

The product shall declare missingness assumptions, plausible-value count or joint-estimation strategy, posterior combination rules, convergence, effective sample size, and sensitivity analyses.

## 17. Compute and scale profiles

Release documentation shall publish measured profiles. Initial acceptance testing covers:

| Profile | GPU VRAM | Expected behavior |
|---|---:|---|
| CPU reference | none | Full correctness/reference path; slower fit permitted |
| Small GPU | 4 GB | Reduced micro-batches; remote/offloaded LLM; bounded fallback |
| Entry GPU | 6 GB | Streamed topic fitting for bounded corpus tiers |
| Standard GPU | 8 GB | Default workstation target |
| Professional GPU | 12 GB | Larger K/vocabulary/batch profiles |
| High-memory GPU | 24 GB | Large enterprise/research profile |

### 17.1 Corpus tiers

The product shall benchmark at least these non-normative planning tiers and publish actual release limits:

| Tier | Documents | Semantic units / nonzeros | Intended workflow |
|---|---:|---:|---|
| S | up to 10,000 | up to 5 million | Interactive research pilot |
| M | up to 100,000 | up to 50 million | Department or portfolio corpus |
| L | up to 1,000,000 | up to 500 million | Enterprise batch analysis |
| XL | above L | deployment-specific | Distributed/partitioned execution requiring measured capacity evidence |

The system must reject or queue work that exceeds the admitted resource profile rather than beginning an uncontrolled run.

## 18. Product performance and reliability objectives

The following are product objectives, not current implementation claims:

- evidence and corpus state transitions are idempotent;
- all published artifacts are checksum-verifiable;
- historical replay produces identical eligibility and deterministic artifacts when deterministic mode is selected;
- non-deterministic estimators record all seeds and return statistically equivalent recovery within the declared acceptance region;
- cancellation produces no partially published approved artifact;
- worker retry does not duplicate model-run, claim, or export publication;
- user-visible errors distinguish invalid input, scientific invalidity, incompatibility, authorization, resource exhaustion, provider failure, and internal failure;
- service-level latency and availability objectives are deployment-profile-specific and are published only after measurement.

## 19. Security, privacy, and compliance requirements

### 19.1 PII without blanket masking

TEPP preserves PII when it is necessary for valid authorship, temporal linkage, multiple membership, deduplication, or audit. The alternative controls are:

- purpose-bound access grants;
- tenant and service identity;
- least-privilege roles;
- opaque analytical identifiers;
- separately protected identity mappings;
- selective field/provider disclosure;
- encryption and KMS policy;
- bounded retention, export, deletion, and legal hold;
- audited privileged access;
- regional/provider policy;
- reproducibility-safe tombstones and receipts.

### 19.2 Provider disclosure

Before any external LLM or embedding provider receives data, TEPP records:

- provider and region;
- fields/spans disclosed;
- transformation/redaction applied;
- purpose and authorization;
- retention/training policy reference;
- model/version;
- request/response digest;
- decision and audit identity.

### 19.3 Assurance readiness

The repository and deployment evidence may support CSAP, SOC 2, ISO/IEC 42001, ISO/IEC 23894, and NIST AI RMF readiness/alignment. TEPP shall not claim certification or attestation without independent applicable evidence.

## 20. Visual analytics product requirements

### 20.1 Bitemporal Lens

Displays event/valid time on one axis and document/availability/system time on another. Users can inspect late reports, retrospective evidence, revisions, and cutoff eligibility.

### 20.2 Temporal Event Graph

Displays document, passage, event, actor, role, evidence, and transition edges with clear observed/inferred/proposed/promoted styling.

### 20.3 Topic River and Lineage

Displays prevalence and activity with uncertainty, dormancy/reactivation, and later topic birth/split/merge/retirement lineage when that capability is promoted.

### 20.4 Drift Comparison

Separates prevalence, semantic, lexical, measurement, method, and reporting drift and provides source examples.

### 20.5 Candidate-K Workbench

Compares candidate evidence vectors, Pareto frontier, topic coverage/redundancy, resource profiles, blinded review, and human escalation.

### 20.6 Topic Network and Cluster Explorer

Displays valid-coordinate associations, edge uncertainty/stability, consensus clusters, unassigned topics, and negative tensions.

### 20.7 Dynamic ESEM/DSEM Builder

Displays indicators, cross-loadings, factors, lags, within/between components, invariance, direct/indirect effects, identification, and claim warnings.

### 20.8 Invariance Dashboard

Displays language×time×template×source×group comparability and identifies which comparisons are prohibited or qualified.

### 20.9 Leakage Audit

Explains every inclusion/withholding decision at a knowledge cutoff and shows relation-connected partition groups.

### 20.10 Accessibility

All high-complexity interactions are designed in Figma before implementation. Every view supplies keyboard/touch paths, screen-reader semantics, no-JavaScript exact-value data, print/PDF behavior, units, uncertainty, missingness, and source-consistent export.

## 21. Error taxonomy

| Error family | Example | User-visible behavior |
|---|---|---|
| `invalid_evidence` | malformed content or span mismatch | Quarantine/reject with redacted reason |
| `temporal_ineligible` | availability after cutoff | Withhold and explain cutoff evidence |
| `relation_contradiction` | impossible interval/path | Reject proposed relation or corpus freeze |
| `authorization_denied` | tenant/purpose/role mismatch | Deny without revealing protected existence |
| `model_unavailable` | no compatible fitted model | Fail closed; no lexical substitute |
| `scientific_contract_invalid` | no invariance or invalid coordinate use | Block comparison/claim |
| `resource_not_admitted` | predicted memory exceeds profile | Queue, down-profile, or require approval |
| `provider_failure` | LLM/embedding provider unavailable | Retry/failover only under recorded policy |
| `inconclusive` | diagnostics disagree | Require human review or additional evidence |
| `artifact_incompatible` | schema/model version mismatch | Return compatibility details without coercion |
| `release_gate_failed` | current-head evidence incomplete | Block release and preserve failure evidence |

Error messages shall be stable, content-redacting, and machine-readable. Internal details are available only to authorized operators through audit evidence.

## 22. Non-functional requirements

### 22.1 Correctness and reproducibility

- Production mathematical and psychometric arithmetic is Rust-first.
- CPU f64 is the numerical reference.
- Owned production line and branch coverage is exactly 100%.
- Public APIs and safety contracts have complete documentation.
- Deterministic runs are byte-reproducible where the contract declares determinism.
- Statistical runs publish uncertainty and recovery evidence rather than bitwise claims where nondeterminism is intrinsic.

### 22.2 Security

- No unsafe implicit code execution from documents or LLM output.
- Full-commit GitHub Action pins and locked dependencies.
- SBOM, provenance, checksums, and artifact signatures as defined by release policy.
- Tenant, purpose, role, lifetime, and provider boundaries fail closed.

### 22.3 Operability

- Structured OpenTelemetry-compatible logs, metrics, and traces.
- Bounded queues, retries, cancellation, and backpressure.
- Backup/restore, migration/rollback, and incident runbooks before production claims.
- Deployment-specific SLO/RPO/RTO measured rather than invented.

### 22.4 Compatibility

- Versioned schemas and semantic compatibility policy.
- Standalone crates remain usable independently.
- Modular services communicate through ports/artifacts, not shared private tables.
- Breaking scientific semantics require ADR and PRD version increase.

## 23. Release slices and exit criteria

### P0-A — Governed temporal-event foundation

Exit requires immutable evidence, six clocks, interval reasoning, forward-only transitions, event mention/instance separation, multiple membership, persistence/RLS, leakage-safe snapshots, simulations, recovery metrics, API/export manifests, and exact release evidence.

### P0-B — Multilingual semantic measurement

Exit requires deterministic segmentation, language profiles, span-grounded semantic units, governed concept dictionary, method-source labels, prompt-injection controls, and human-gold validation for declared profiles.

### P0-C — TRSL-TM CPU reference

Exit requires Rust CPU f64 fitting, structural covariates, temporal/relational/multiple-membership inputs, posterior uncertainty, synthetic recovery, independent oracle comparison, and reproducibility manifests.

### P0-D — Candidate selection, network, and clusters

Exit requires candidate planning, hard gates, Pareto selection, blinded review, valid-coordinate associations, edge uncertainty, and consensus-cluster recovery.

### P0-E — Longitudinal psychometrics

Exit requires construct-role gate, plausible-value/joint uncertainty, invariance, within/between decomposition, irregular-time support where applicable, ESEM/DSEM true-parameter recovery, and causal-language controls.

### P0-F — Evidence-bounded publication

Exit requires interpreter/verifier separation, unsupported-claim handling, accessible coordinated visualizations, source-consistent exports, audit, and claim-promotion workflow.

### P1 — GPU and advanced event intelligence

Exit requires real GPU parity and VRAM profiles, TDT task benchmarks, CHRONOS schema/prediction boundaries, forecast calibration, and deployment capacity evidence.

### P2 — Topic lineage and advanced research adapters

Includes explicit topic birth/split/merge/retirement, approved neural/external backend adapters, advanced continuous-time/joint models, and additional language/domain profiles after separate validation.

## 24. Commercial success metrics

Commercial metrics are measured per deployment and do not replace scientific validity.

| Metric | Definition |
|---|---|
| Time to first governed corpus | Time from accepted source access to a frozen leakage-safe corpus |
| Time to first reviewable model | Time from corpus freeze to a candidate with complete diagnostics |
| Evidence review burden | Qualified reviewer minutes per accepted topic/claim |
| Reproducibility success | Share of replay attempts reproducing the declared artifact contract |
| Unsupported claim rate | Share of generated claim clauses rejected by the independent verifier |
| Language-profile error | Task-specific error/calibration by language and domain |
| Model-selection escalation | Share of runs requiring human selection and reasons |
| Analysis reuse | Number of downstream artifacts/consumers per approved run |
| Operational recovery | Share of interrupted jobs restored without duplicate publication |
| Buyer workflow completion | Share of pilots reaching approved evidence-backed report/export |
| Pilot-to-production conversion | Governed customer pilots moving to production deployment |
| Retention and repeat analysis | Organizations repeating analyses across reporting periods |

No valuation claim follows directly from these metrics. The `200억 달러` bar remains a prioritization standard requiring evidence of differentiated product utility, scientific trust, repeatable deployment, and durable commercial adoption.

## 25. Out of scope

The following are outside the P0 product boundary unless a later reviewed PRD/ADR adds them:

- unrestricted causal discovery;
- autonomous high-stakes decisions without qualified human authority;
- biometric, clinical-diagnostic, credit, employment, or legal determinations without domain-specific validation and governance;
- silent translation as a substitute for multilingual measurement;
- arbitrary document code execution;
- CAPTCHA, access-control, paywall, or anti-bot bypass;
- unrestricted LLM tool access;
- direct cross-service database coupling;
- claims of universal language validity;
- claims of global satisfiability from bounded path consistency;
- certification/attestation self-issuance;
- treating translated, inferred, clustered, or LLM-generated objects as observed fact.

## 26. Requirement-to-authority trace

| Requirement family | Owning ADRs / canonical technical contract |
|---|---|
| Evidence identity and spans | ADR 0008; Architecture; TRD |
| Temporal semantics | ADR 0002; ADR 0013 |
| Event/relation/membership | ADR 0003; ADR 0016 |
| Multilingual shared latent space | ADR 0004; ADR 0012 |
| Topic measurement and K selection | ADR 0012; TRD; Test Strategy |
| Psychometric modeling | ADR 0005; ADR 0014 |
| Rust/GPU/VRAM | ADR 0001; ADR 0006; ADR 0007 |
| LLM orchestration | ADR 0010; ADR 0015 |
| Privacy/PII | ADR 0009; Privacy/Data Governance; Threat Model |
| Persistence/manifests/splits | ADR 0013; ERD; API Contract |
| MSA/connectors | ADR 0011; API Contract; connector contracts |
| Event intelligence | ADR 0016 |
| Claim promotion/release | ADR 0014; Operability; Traceability |

## 27. Acceptance checklist for the first implementation release

A first implementation release is eligible only when all applicable items are evidenced on one exact protected head:

- P0 release slice declared and fully traceable;
- every applicable functional requirement has test/evidence identifiers;
- no unresolved high-severity scientific, security, privacy, or data-integrity finding;
- full required CI and required workflow success;
- zero valid unresolved review thread;
- 100% owned production line and branch coverage;
- complete public/safety docstrings;
- true-parameter recovery and Monte Carlo uncertainty report;
- language-profile validation report for every marketed profile;
- CPU/GPU parity report for every marketed accelerator profile;
- migration, rollback, backup/restore, and failure-recovery evidence;
- API/schema compatibility report;
- accessibility and source-consistent export evidence;
- SBOM, provenance, checksums, and reproducibility manifest;
- version and CHANGELOG consistency;
- signed/authorized release approval;
- post-publication artifact verification.

## 28. Change-control rule

A change requires a new PRD version and owning ADR update when it changes any of the following:

- latent variable, estimand, or topic identity;
- time meaning, cutoff, relation, or transition authority;
- event ontology or promotion authority;
- language equivalence or invariance claim;
- reflective/formative/network/structural interpretation;
- CPU/GPU numerical authority or acceptable tolerance;
- PII purpose/identity/provider-disclosure authority;
- persistence, split, or reproducibility identity;
- LLM, verifier, autonomous-development, review, merge, or release authority;
- public compatibility or artifact semantics.

A requirement may be clarified without a new ADR when its owning decision is unchanged, but the PRD version, traceability, tests, and affected technical documents shall be updated together.

## 29. Supersession and historical record

This v0.5 document is the canonical detailed product-requirements baseline after integration. The approved v0.4 document remains immutable historical evidence of the prior design baseline. Source planning packs remain under `docs/archive/source-material/` and `docs/source-pack/`. Historical documents do not override current ADR authority or implementation maturity.

## 30. Approval basis

The product direction was approved in PRD v0.4. Version 0.5 responds to the explicit request to make the PRD materially more concrete. It preserves the approved TEPP architecture and adds stable requirement identifiers, workflows, state machines, input/output contracts, fail-closed behavior, acceptance evidence, language and scientific claim gates, scale profiles, visual product requirements, commercial metrics, and release slices.
