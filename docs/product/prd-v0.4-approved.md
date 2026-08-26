# Temporal Event Psychometrics Platform — Approved PRD v0.4

**Status:** Approved design baseline  
**Approval date:** 2026-08-05  
**Product name:** Temporal Event Psychometrics Platform (TEPP)  
**Measurement family:** Temporal Relational Shared-Latent Topic Measurement (TRSL-TM)

## 1. Product thesis

TEPP measures multilingual documentary evidence as fallible observations of latent semantic, event, and psychological structures. It preserves document links, event chronology, multilevel membership, measurement error, and uncertainty rather than treating documents as independent exchangeable points.

The platform combines:

- multilingual evidence and exact source-span preservation;
- shared-latent topic measurement with structural covariates;
- document, segment, event, entity, revision, translation, and evidence relations;
- multiple temporal clocks and forward-only state transitions;
- TDT-style event detection and tracking;
- CHRONOS-style schema instantiation, prediction, and temporal-consistency reasoning;
- posterior-aware topic networks and consensus clusters;
- longitudinal ESEM, DSEM, and continuous-time structural modeling;
- evidence-grounded LLM interpretation and independent verification;
- coordinated visual analytics and accessible exact-value exports.

## 2. Target users and decisions

Primary users include organizational researchers, psychometricians, strategy and risk teams, project and opportunity governance teams, market-intelligence teams, auditors, and AI product architects. TEPP supports discovery and measurement; it does not automatically establish causality or authorize high-stakes decisions.

Representative questions include:

- Which latent topics and events appear, persist, split, merge, reactivate, or disappear over time?
- Which reports, passages, organizations, authors, projects, and evidence chains connect those changes?
- Are equivalent meanings measured on the same scale across language, time, template, source, and group?
- Which input states precede process or intervention states, and which outcomes follow?
- Are apparent changes substantive, lexical, semantic, compositional, reporting, or measurement drift?
- Which conclusions remain stable across seeds, posterior draws, model families, languages, and hardware backends?
- Which evidence-grounded predecessor branches form a project's journey when
  record creation order differs from uncertain event time?

## 3. Input contract

TEPP accepts reports and related documentary records with optional metadata and relations. Supported content may contain English, Korean, Japanese, Chinese, Vietnamese, Indonesian, French, German, Turkish, code switching, and additional long-tail languages.

Every source record preserves:

- immutable source bytes and SHA-256;
- content type, encoding, page/layout coordinates, and exact character spans;
- document identifier, version, source system, and tenant;
- author, department, organization, project, opportunity pool, and role hints when available;
- observed hyperlinks, citations, revision identifiers, translation identifiers, and attachment relations;
- event, assertion, document, system, and availability timestamps or bounded uncertain intervals;
- provenance and confidence for every inferred field.

## 4. Temporal semantics

A single `date` field is insufficient. TEPP distinguishes:

1. **event or valid time** — when a state or event occurred or held;
2. **assertion time** — when a claim was stated;
3. **document time** — creation, publication, revision, or reporting period;
4. **system time** — when the platform observed a record or change;
5. **availability time** — when an analyst could actually have used the evidence;
6. **knowledge cutoff** — the maximum availability time permitted for a model run.

Historical fitting enforces:

\[
\operatorname{available\_time}(d) \leq \operatorname{knowledge\_cutoff}.
\]

Forward transition relations such as `input_to`, `process_to`, `outcome_of`, `causes`, `enables`, and `transitions_to` require a valid partial temporal order. Citation, revision, translation, summary, support, contradiction, and retrospective-reporting relations may point to earlier events, but they never become reverse state transitions.

Instant, interval, duration, open boundary, uncertain boundary, before, after, meets, overlaps, starts, finishes, during, contains, and equality relations are represented explicitly.

## 5. Relational and multilevel structure

TEPP maintains separate but coordinated graphs:

- a document and passage graph;
- an event and event-schema graph;
- a time-varying entity-role graph;
- a topic association graph;
- a document-topic bipartite graph;
- a factor and structural-path graph;
- a complete evidence and provenance graph.

Observations may be cross-classified and multiply assigned to authors, departments, customers, partners, competitors, projects, opportunity pools, markets, templates, languages, locations, and event episodes. Membership weights, validity intervals, confidence, and evidence spans are retained. Customer, partner, and competitor are time-varying relation roles rather than permanent entity types.

Relation-aware training splits keep translations, revisions, copied variants, and members of the same event episode within one partition. Group-normalized likelihood and duplicate-aware effective sample size prevent pseudo-replication.

## 6. Multilingual evidence measurement

All languages share global topic identities, concept prototypes, and document latent coordinates. Language-specific lexical emission, morphology, script, syntax, and content deviation may vary.

The processing pipeline is:

1. immutable source preservation;
2. Unicode normalization without overwriting original text;
3. language and script posterior estimation at segment or span level;
4. Unicode and language-tailored sentence/word boundaries;
5. morphology, universal part of speech, dependency phrases, negation, modality, quantity, and temporal-expression analysis;
6. optional LLM semantic-unit proposal using exact source spans;
7. deterministic schema, span, security, and concept-dictionary validation;
8. versioned shared-concept and native-lexical channels.

Stopword deletion is not the default. Part of speech is a soft source prior, not a hard deletion rule. TF-IDF and BM25 do not weight inferential topic, correlation, ESEM, or DSEM estimation.

Repeated report language is modeled through substantive-topic, corpus-background, template, section, copied-text, style, prompt, modality, and metadata sources. This prevents report forms and boilerplate from becoming false substantive topics while retaining auditable evidence.

Language profiles are reported as validated, calibrated, provisional, or unresolved. Equivalent meanings must demonstrate alignment and measurement invariance; architectural support alone does not establish validity.

## 7. Topic measurement model

The reference family is a temporal, relational, shared-latent extension of logistic-normal structural topic modeling. Polylingual and global-context neural topic models may be used through an adapter when they provide the same posterior, temporal, relation, and invariance contracts.

A conceptual prevalence layer is:

\[
\eta_d = \mu(t_d) + X_d\Gamma + \sum_{g \in G_d}w_{dg}u_g(t_d) + \epsilon_d,
\qquad
\theta_d=\operatorname{softmax}(\eta_d).
\]

The model separates:

- prevalence drift — stable meaning with changing frequency;
- semantic drift — changing shared concept prototype;
- lexical drift — changing language-specific expression;
- measurement drift — changing relation between topics and higher-order factors;
- method drift — changing template, section, source, or reporting behavior.

The initial release selects one global topic count across the analysis window and permits activation, dormancy, and reactivation. Explicit topic birth, split, merge, lineage, and retirement are later extensions.

## 8. Topic-count selection

No single metric or LLM determines the topic count. Candidate models are evaluated across:

- held-out predictive fit and posterior predictive checks;
- semantic coherence, exclusivity, coverage, and redundancy;
- topic prevalence and collapse;
- seed, bootstrap, split, and time-window stability;
- multilingual alignment and group fairness;
- covariate-effect stability;
- compute and memory feasibility;
- blinded evidence-bounded LLM review.

Statistically invalid candidates are rejected before LLM review. The remaining candidates form a Pareto frontier across fit, interpretability, stability, parsimony, and cross-language alignment. Results report a recommended count, acceptable set, trade-offs, disagreement, and human-review status.

## 9. Topic association and clusters

Raw topic proportions are compositional and are not passed directly to ordinary Pearson correlation or ESEM. TEPP uses logistic-normal latent coordinates or orthonormal log-ratio coordinates and propagates posterior uncertainty.

Each network edge reports effect size, interval, posterior selection probability, bootstrap/seed stability, sample basis, and threshold/correction policy. Conditional networks operate in a valid log-ratio space.

Stable positive associations feed repeated Leiden community detection and a co-assignment consensus matrix. Unstable topics may remain unclustered. Negative associations represent opposition or tension rather than cluster membership.

## 10. Psychometric layer

A topic model is latent-variable modeling, but a discovered topic is not automatically a validated psychological construct. TEPP requires construct-role assessment:

- reflective topic indicators may enter ESEM or set-ESEM;
- formative topic composites use composite or formative SEM;
- mutually interacting topics use latent-network models;
- uncertain structures are compared through multiple plausible models and external validation.

Posterior topic coordinates enter ESEM/DSEM through plausible values or a joint model. Point estimates are not treated as error-free observations.

Longitudinal comparison evaluates configural, approximate metric, scalar when means are compared, residual/method, partial, and time-varying measurement invariance. Stable between-episode differences are separated from within-episode change.

Input, process/intervention, and outcome paths obey temporal order. Temporal precedence alone is not described as causation; causal language requires an identified experimental, quasi-experimental, or defensible observational design.

## 11. TDT, CHRONOS, and event ontology

TEPP maps TDT functions to measurement questions:

- story segmentation — reliability of measurement-unit and occasion boundaries;
- link detection — convergent and discriminant validity of event identity;
- topic/event detection — discovery of candidate latent events and constructs;
- first-story detection — onset and change-point sensitivity and false-alarm control;
- tracking — longitudinal stability, responsiveness, and state trajectories.

The event ontology treats events as first-class entities with time, place, agents, factors, products, subevents, arguments, mentions, source evidence, uncertainty, and typed relations.

CHRONOS-style neural/symbolic extraction proposes event schemas, arguments, complex-event instances, and next-event candidates. A separate temporal reasoner derives implied interval relations, rejects contradictions, and preserves evidence provenance. Psychometric validation evaluates mention reliability, schema structure, tracking reliability, and forecast calibration.

Project Journey is a posterior event DAG, not a fixed lifecycle or earliest-row
timeline. It may include prior-project, customer-request, procurement-notice,
direct/negotiated-bid, external-sensing, internal-discussion, and lead evidence;
record time and event time remain distinct, and multiple predecessors,
branches, transitions, exact ties, and uncertainty are retained.

## 12. LLM responsibilities

LLMs may propose semantic units, concept mappings, candidate-model reviews, topic/cluster labels, explanations, and event-schema hypotheses. They do not perform authoritative numerical estimation or bypass statistical gates.

Every output is untrusted and must include approved structured fields, exact evidence identifiers, source spans, confidence, model/provider, prompt hash, reasoning effort, workflow depth, access list, and version.

Interpretation is produced by an evidence-bounded interpreter and checked by an independent verifier for unsupported claims, direction reversals, causal overclaiming, group generalization, and omitted uncertainty.

`contextual-orchestrator` integration allocates test-time compute between direct single-model routing and deeper role-based orchestration. Evaluation varies decomposition, recursion, workflow stages, role-specific reasoning effort, and tool/access lists, with ablations informed by Fugu-, Conductor-, and TRINITY-style research directions. Speed is not the primary objective; calibrated quality and evidence are.

Approved live tests use `NVIDIA_NIM_API_KEY`. `COPILOT_GITHUB_TOKEN` is prohibited.

## 13. Rust and compute requirements

Production mathematical and psychometric arithmetic is implemented in Rust. Python and R may serve validation, interoperability, and independent-oracle roles only.

The CPU `f64` estimator is the numerical reference. CPU acceleration uses bounded fixed worker pools, sparse CSR/CSC data, thread-local sufficient statistics, deterministic reductions where required, and controls to prevent BLAS/thread-pool oversubscription.

The GPU layer exposes backend-neutral operations with NVIDIA CUDA as the primary performance path and WGPU/CubeCL or equivalent portable acceleration where justified. Sparse topic-specific kernels may be custom implemented.

On Apple Silicon, MLX Metal executes only in the macOS-native Rust-owned
service governed by ADR 0024. Compose connects through an authenticated local
Unix socket or host-gateway boundary; Colima/Linux never claims Metal. Linux
portability uses `rust_cpu`, `mlx_cpu`, `mlx_cuda`, or `rust_opencl` only when
that backend actually executes; `mlx_opencl` is not a valid backend. Every
accelerated result carries numerical-parity provenance against CPU f64.

The VRAM controller:

- measures total and available memory;
- reserves a safety margin;
- predicts peak memory from topic count, vocabulary, batch nonzeros, precision, and workspace;
- autotunes micro-batches;
- streams document responsibilities and immediately releases them;
- accumulates sufficient statistics in stable precision;
- reduces batches after OOM with bounded retries;
- falls back to CPU safely;
- records peak allocation, transfer, kernel, retry, and fallback telemetry.

Local LLM weights and topic-model tensors are not concurrently resident on small GPUs. A phase scheduler unloads one workload before loading the other.

## 14. Verification and acceptance

Every estimator and product layer requires realistic tests. Scientific acceptance includes:

- true topic, prevalence, content, relation, time, factor, and structural-path recovery;
- Hungarian or otherwise identified topic/factor matching;
- RMSE, bias, interval coverage, convergence, false-positive/negative, and calibration metrics;
- known topic-count recovery across overlap, sparsity, imbalance, language, time, and hierarchy conditions;
- temporal partial-order and future-information-leakage tests;
- relation, event, transition, topic-network, and cluster recovery;
- multilingual span, concept, alignment, invariance, and fairness tests;
- LLM rater agreement, calibration, unsupported-claim, and prompt-injection tests;
- CPU/GPU parity, real GPU execution, 4/6/8/12/24-GB memory profiles, and fallback tests;
- production line and branch coverage at 100%;
- complete public and safety-contract docstrings;
- fuzz, property, package, migration, SBOM, provenance, and reproducibility checks.

Monte Carlo pass criteria incorporate Monte Carlo standard error or confidence intervals instead of requiring a finite observed rate to equal or exceed its nominal population target mechanically.

## 15. Visual analytics

The product renders coordinated, accessible views:

1. bitemporal lens;
2. temporal document/event graph;
3. topic river and lineage;
4. semantic and lexical drift comparison;
5. TDT detection/tracking console;
6. CHRONOS event-schema canvas;
7. cross-classified membership graph;
8. dynamic ESEM/DSEM builder;
9. invariance dashboard;
10. knowledge-cutoff and leakage audit.

Every chart has an exact-value semantic table and exports source-consistent SVG, PDF, CSV, JSON, JSON-LD, GraphML, Arrow, or Parquet as appropriate. Keyboard, touch, screen-reader, no-JavaScript, print, and PDF states are designed in Figma before implementation of high-complexity interactions.

## 16. Persistence and audit

Reference database objects use two-or-more-word `snake_case` names, including:

- `document_record`, `document_covariate`, `text_segment`, `semantic_unit`;
- `temporal_interval`, `event_instance`, `event_mention`, `event_relation`;
- `document_relation`, `segment_relation`, `relation_evidence`;
- `entity_record`, `entity_role_assignment`, `membership_assignment`;
- `concept_dictionary`, `concept_mapping`, `model_run`, `model_artifact`;
- `topic_definition`, `topic_prevalence`, `topic_correlation`, `topic_cluster`;
- `factor_solution`, `structural_path`, `validation_metric`, `compute_profile`, `audit_event`.

Every model run records corpus and relation hashes, cutoff, preprocessing/concept/model versions, backend, precision, seeds, LLM metadata, dependency lock, Git commit, calibration status, and artifact checksums.

## 17. Security and governance

Documents and model outputs are untrusted. TEPP enforces tenant isolation, immutable evidence, authorization, size/depth limits, hostile Unicode and archive protection, prompt-injection isolation, no implicit tool/network execution, secret redaction, least-privilege workflows, action SHA pins, dependency locking, SBOM, provenance, and reproducible releases.

Scientific integrity is a security property. Silent temporal leakage, unsupported cross-language equivalence, failed uncertainty coverage, group bias, numerical backend divergence, or causal overclaiming fails closed.

Changes to latent-variable meaning, temporal semantics, event ontology, multilingual invariance, or estimator targets require an ADR and PRD version increase.

## 18. Delivery phases

1. Temporal and event foundation.
2. Multilingual evidence and semantic units.
3. Shared-latent temporal topic CPU reference.
4. GPU and VRAM-adaptive compute.
5. TDT and CHRONOS event intelligence.
6. Multilevel longitudinal ESEM/DSEM.
7. Coordinated visual analytics and Figma design.
8. LLM interpretation and commercial hardening.

## 19. Initial release gate

The first release requires:

- validated temporal/event data contracts and leakage-safe storage;
- a CPU `f64` reference estimator with true-parameter recovery;
- multilingual shared-space evidence for the declared language profiles;
- CPU/GPU parity and bounded VRAM behavior;
- posterior-aware topic network and cluster stability;
- longitudinal measurement and structural-model validation;
- evidence-grounded interpretations with verifier checks;
- accessible exact-value visualizations and exports;
- 100% production line/branch coverage and public docstrings;
- current-head CI/security approval, clean migrations, SBOM, provenance, rollback, version, and CHANGELOG evidence.

## 20. Approved baseline

This document is the approved v0.4 design baseline. The complete source PRD, preceding v0.2 and v0.3 designs, roadmap, implementation plan, validation report, instruction sources, and reproducibility manifest are retained in `docs/archive/source-material/`.
