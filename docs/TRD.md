# TEPP Technical Requirements Document

**Status:** Accepted technical baseline aligned to PRD v0.5  
**Last reviewed:** 2026-08-13

## 1. Technical objective

TEPP is a multilingual temporal-event psychometrics platform whose executable
core is a set of independently usable Rust crates and versioned service,
persistence, job, artifact, and connector contracts. Each boundary separates
source evidence, deterministic semantics, statistical estimation, psychometric
claims, LLM interpretation, and release authority so validity can be tested and
audited independently.

The canonical product requirements are
`docs/product/prd-v0.5.md`. Every implementation PR shall identify the affected
`FR-*` identifiers, owning ADRs, source/tests/migrations/schemas, failure modes,
rollback, and requested maturity change.

## 2. Current implementation maturity

Protected main at the PRD v0.5 authoring baseline contains:

- the Rust workspace and exact repository-quality gates;
- immutable evidence records and exact source spans;
- typed six-clock values and uncertain intervals;
- bounded Allen interval path consistency;
- forward-only state-transition relation graphs;
- event mention/instance separation;
- time-varying weighted multiple-membership and ESS helpers;
- leakage-safe knowledge-cutoff snapshots and relation-connected splits;
- deterministic temporal/event known-truth simulation;
- recovery metrics including RMSE, bias, interval coverage, relation recovery,
  temporal order, and Monte Carlo uncertainty;
- bitemporal PostgreSQL contracts, live SQL ports, and selected SQLx execution;
- versioned analysis-run, reproducibility, JSON-LD, and GraphML artifacts;
- modular naruon and contextual-orchestrator connector contracts;
- SBOM, provenance, checksum, and validation-ledger foundations.

Tenant FORCE RLS and runtime-role restrictions are active on PR #30 until exact
protected-main integration. `docs/TRACEABILITY.md` is the authoritative maturity
ledger.

Multilingual semantic measurement, the TRSL-TM estimator, candidate-K selection,
topic networks/clusters, GPU compute, TDT/CHRONOS intelligence, longitudinal
ESEM/DSEM, interpretation, coordinated visual analytics, and the complete
production job/API service remain separately gated target slices.

## 3. Module boundaries

| Module / port | Responsibility | PRD families |
|---|---|---|
| `evidence_core` | Immutable source identity, digest, exact spans, geometry, strict wire reconstruction | `FR-EVD-*` |
| `temporal_core` | Six clocks, strict instants, uncertain intervals, Allen relations and bounded closure | `FR-TMP-*` |
| `relation_graph` | Typed forward transition and provenance edges with acyclicity | `FR-TMP-005`, `FR-REL-*` |
| `event_core` | Event mentions, governed instances, roles, validity, promotion boundaries | `FR-EVT-001`, `FR-REL-*` |
| `membership_core` | Weighted time-varying cross-classified memberships and ESS/design effects | `FR-MEM-*` |
| `persistence_postgres` | Bitemporal storage, migrations, tenant/purpose controls, audit, replay | `FR-EVD-005`, `FR-SEC-*`, `FR-OPS-*` |
| `corpus_split` | Cutoff eligibility, relation grouping, rolling-origin and leakage-safe partitions | `FR-TMP-003`, `FR-REL-003` |
| `tepp_simulation` | Deterministic known-truth corpora and manifests | scientific acceptance |
| `validation_core` | Matching, bias/RMSE/coverage, graph/time recovery, Monte Carlo gates | all scientific families |
| `tepp_api` | Versioned DTOs, jobs/artifacts, errors, manifests, exports, compatibility | `FR-API-*`, `FR-EXP-*` |
| semantic measurement target | Segmentation, language profiles, semantic units, concepts, method sources | `FR-LNG-*`, `FR-SEM-*` |
| `topic_measurement` target | TRSL-TM CPU reference, posterior, covariates and drift | `FR-TOP-*` |
| `model_selection` target | Candidate plan, hard gates, Pareto frontier, blinded review | `FR-KSEL-*` |
| `network_analysis` target | Valid-coordinate associations and consensus clusters | `FR-NET-*` |
| `psychometric_core` target | Construct-role, ESEM/DSEM, invariance and continuous time | `FR-PSY-*` |
| `compute_backend` target | Bounded CPU pools, GPU kernels, VRAM admission and parity | `FR-CMP-*` |
| `event_intelligence` target | TDT tasks, CHRONOS schemas/prediction/calibration | `FR-EVT-002/003/004` |
| `interpretation_gateway` target | Evidence-bounded interpreter/verifier and claim states | `FR-LLM-*` |
| `visual_analytics` target | Accessible coordinated views and exact-value exports | `FR-EXP-*` |

Each module shall remain independently testable. Cross-module communication uses
public Rust types, versioned schemas, ports, artifacts, or events rather than
private storage access.

## 4. Evidence and data-boundary requirements

### 4.1 Trust boundary

Source documents, embedded metadata, LLM output, connector output, imported
schemas, and model artifacts are untrusted until validated. Active content is
not executed. Source bytes/text are copied into bounded owned storage, hashed,
and assigned opaque identity before downstream use.

### 4.2 Exact locations

Evidence locations use half-open UTF-8 byte and Unicode-scalar coordinates and
optional page/layout geometry. Coordinate conversions must remain deterministic
and reject invalid boundaries rather than repair them silently.

### 4.3 Version lineage

A revision creates a new document state with valid-time and system-time lineage.
No frozen corpus, model run, or published claim mutates its parent artifact.
Supersession is explicit and queryable.

## 5. Temporal and relation requirements

TEPP represents event/valid, assertion, document, system, availability, and
knowledge-cutoff time as nominally distinct types. Historical eligibility is
based on availability policy rather than event/document date.

Uncertain/open/unknown intervals retain certainty, precision, inclusion, and
provenance. Interval reasoning is resource-bounded and reports path consistency,
not unrestricted global satisfiability.

Transition and input→process→outcome edges require valid forward temporal order
and acyclicity. Citation, translation, revision, support, contradiction,
summary, and retrospective reporting remain provenance/evidence relations and
may point backward without becoming reverse transitions.

## 6. Event and membership requirements

Event mentions are source-bound fallible observations. Event instances require
explicit promotion with evidence and policy. Event schemas and forecasts remain
hypotheses until temporal, evidence, calibration, and claim-promotion gates pass.

Customers, partners, competitors, authors, departments, organizations, projects,
opportunities, templates, languages, locations, and episodes are represented as
contextual time-varying role/membership assignments. Observations may be
multiply assigned. Supplied, normalized, inferred, and estimated weights remain
distinguishable.

## 7. Persistence requirements

PostgreSQL is the reference relational store. Persistent objects use descriptive
two-or-more-word `snake_case` names. Tenant-scoped tables require explicit
tenant identity and, when exposed to application runtime roles, FORCE RLS or an
equivalent fail-closed boundary proven by live cross-tenant tests.

Persistence shall preserve:

- opaque identities and content digests;
- valid-time and system-time history;
- immutable corpus/model/artifact manifests;
- tenant, purpose, role, retention, and disclosure policy;
- relation, membership, evidence, and claim provenance;
- audit events and deletion/legal-hold receipts;
- migrations, rollback, backup/restore, and compatibility evidence.

In-memory adapters are test/reference implementations and cannot establish live
database, RLS, migration, or operational claims.

## 8. Multilingual semantic measurement target

The pipeline shall preserve original text, use NFC for analysis where declared,
limit NFKC to explicit auxiliary keys, and combine Unicode boundaries,
language/script tailoring, layout, headings, lists, tables, morphology, universal
part of speech, dependency phrases, negation, modality, quantity, and temporal
expressions.

Stopword deletion is not the default. POS is a soft source/model input rather
than a universal deletion rule. TF-IDF/BM25 may support retrieval but cannot
weight inferential topic, correlation, ESEM, or DSEM calculations.

LLM semantic-unit proposals must be exact-span and schema validated against a
versioned concept dictionary. Free-form model prose cannot enter statistical
input tables. Unknown meanings are preserved as governed unknowns.

All supported profiles share global topic identity and latent coordinates while
retaining native lexical/morphological channels. Language validity is promoted
per task/domain from benchmark and invariance evidence.

## 9. TRSL-TM estimator target

The CPU f64 Rust implementation is the numerical reference. Conceptually:

\[
\eta_d = \mu(t_d) + X_d\Gamma
+ \sum_{g\in G_d} w_{dg}u_g(t_d)
+ r_d + \epsilon_d,
\qquad
\theta_d=\operatorname{softmax}(\eta_d),
\]

where structural covariates, temporal state, multiple memberships, relations,
and method sources are explicit. The implementation shall expose objectives,
convergence, posterior coordinates, parameter uncertainty, diagnostics, and
artifacts sufficient for true-parameter recovery and an independent oracle.

The model distinguishes prevalence, semantic, lexical, measurement, method, and
reporting drift. P0 uses one global topic identity across the analysis window
with activation/dormancy/reactivation; explicit birth/split/merge/retirement is a
later versioned capability.

External topic backends are adapters, not substitutes for TEPP contracts. They
must pass posterior, temporal, relation, invariance, provenance, recovery, and
compatibility conformance before registration.

## 10. Candidate-K selection target

A candidate plan freezes topic counts/search rule, model families, seeds,
resamples, folds, time windows, cutoffs, partitions, compute budget, convergence,
collapse, and escalation policy.

Non-converged, collapsed, redundant, unstable, misaligned, unfair, infeasible, or
scientifically invalid candidates are rejected before LLM review. Survivors are
compared on a Pareto frontier across fit, posterior checks, coherence,
exclusivity, coverage, redundancy, stability, parsimony, alignment, fairness,
and compute feasibility.

Blinded LLM review receives evidence bundles and cannot override hard gates. The
selection artifact records recommended and acceptable candidates, rejected
reasons, trade-offs, reviewer disagreement, human escalation, and decision
authority.

## 11. Network and cluster target

Raw topic proportions are compositional and cannot be passed directly to naïve
Pearson correlation. The product uses logistic-normal coordinates or declared
orthonormal balances and propagates posterior uncertainty.

Every edge records estimate, interval, selection probability, bootstrap/seed
stability, sample basis, coordinate transform, and correction policy. Positive
stable edges feed repeated Leiden or approved community detection and a
co-assignment consensus matrix. Negative associations are modeled/displayed as
tension rather than positive cluster membership.

## 12. Psychometric target

A construct-role gate separates reflective, formative/composite, network, and
unresolved structures. Topic posterior means are not error-free indicators;
ESEM/DSEM uses plausible values or a joint strategy.

Longitudinal comparison tests applicable configural, metric, scalar, residual,
method, partial, or time-varying invariance. Stable between-unit differences are
separated from within-unit change. Irregular observation gaps use a
continuous-time model or an explicit approximation justified by sensitivity
analysis.

Input→process/intervention→outcome paths require valid temporal order. Causal
wording requires an identified design and assumptions; otherwise output remains
associational or predictive.

## 13. Compute requirements

Production mathematical and psychometric arithmetic is Rust-first. CPU f64 is
the reference. CPU execution uses bounded worker pools, sparse matrices,
thread-local sufficient statistics, controlled reductions, and oversubscription
protection.

GPU execution is admitted by a backend-neutral compute profile. The VRAM
controller measures availability, reserves margin, predicts peak memory, tunes
micro-batches, streams responsibilities, accumulates stable sufficient
statistics, bounds OOM retries, and falls back safely. Real-device parity is
required for every marketed GPU profile.

Local LLM and topic-model weights are phase scheduled when their concurrent
residency would violate the VRAM profile.

## 14. API, job, and export requirements

Public contracts are semantic-versioned. Long analyses expose idempotent create,
status/progress, cancel, retry/replay, and artifact discovery. Typed errors
distinguish invalid evidence, temporal ineligibility, relation contradiction,
authorization, model unavailability, scientific invalidity, resource admission,
provider failure, inconclusive evidence, incompatibility, and release-gate
failure.

naruon and other consumers use versioned TEPP artifacts/ports and cannot
substitute keyword heuristics for unavailable topic inference.
`contextual-orchestrator` may execute bounded provider workflows, while TEPP
retains evidence, statistical, scientific, artifact, claim, and release
authority.

Published artifacts carry corpus/relation/cutoff/config/code/dependency/model/
prompt/seed/backend identities and checksums. SVG/PDF/CSV/JSON/JSON-LD/GraphML/
Arrow/Parquet outputs derive from the same approved source artifact.

## 15. Privacy and security requirements

PII required for valid authorship, linkage, multiple membership, deduplication,
or audit is protected through purpose-bound authorization, tenant/service
identity, least privilege, opaque analytical identifiers, separately protected
identity mapping, encryption/KMS, selective provider disclosure, retention,
deletion/legal hold, export controls, and immutable privileged audit rather than
blanket masking.

Before provider disclosure, TEPP records provider/region, disclosed fields/spans,
transformation, purpose/authorization, retention/training policy, model/version,
request/response digest, and audit identity.

Scientific integrity is a security property. Temporal leakage, relation or
membership poisoning, evidence substitution, unsupported equivalence,
uncalibrated confidence, backend divergence, causal overclaiming, or
unverifiable interpretation fails closed.

## 16. Verification requirements

Every estimator/reasoner has known-truth tests appropriate to its claim,
including parameter matching, bias, RMSE, interval coverage, convergence,
temporal order, relation/event/network/cluster recovery, invariance, alignment,
calibration, and CPU/GPU parity. Monte Carlo gates account for simulation error.

Owned production line and branch coverage is exactly 100%. Public APIs and
safety contracts have complete documentation. Required CI/security/review is
bound to the unchanged exact head. Skipped required hardware, predecessor-head,
local-only, status-only, queued, or synthetic-only evidence cannot promote a
claim.

## 17. Operability and release

Workers use bounded queues, cancellation, backpressure, idempotent state changes,
redacted observability, and crash-safe publication. Deployment-specific SLO,
RPO, RTO, capacity, regional, KMS, backup/restore, and incident claims require
measured evidence.

A release requires exact integrated protected-head software, scientific,
security/privacy, migration/rollback/recovery, accessibility, package, SBOM,
provenance, checksum, compatibility, and approval evidence. Version and
CHANGELOG are updated only when the integrated release candidate satisfies the
applicable PRD slice.
