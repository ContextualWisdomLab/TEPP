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
    G --> H[Longitudinal ESEM and DSEM]
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
| `membership_model` | time-varying cross-classified and multiple-membership assignments |
| `semantic_preprocessor` | Unicode, segmentation, morphology, dependency phrases, LLM span contracts, validation |
| `concept_dictionary` | versioned multilingual concept alignment and unknown-concept review |
| `topic_measurement` | shared-latent temporal/relational topic estimation and uncertainty |
| `compute_backend` | CPU `f64`, fixed-pool multithreading, CUDA/WGPU, sparse streaming, VRAM budgeting |
| `model_selection` | candidate K, predictive fit, coherence, exclusivity, stability, alignment, fairness, blinded LLM review |
| `psychometric_core` | posterior-plausible-value ESEM, longitudinal invariance, DSEM, continuous-time paths |
| `event_intelligence` | TDT segmentation/link/detection/first-story/tracking and CHRONOS schema reasoning |
| `network_analysis` | log-ratio topic correlation, conditional networks, uncertainty, Leiden consensus clusters |
| `interpretation_gateway` | evidence-bounded LLM interpretation, independent verification, routing and ablations |
| `artifact_service` | model registry, manifests, JSON-LD, GraphML, Arrow/Parquet, tables, SVG/PDF exports |
| `visual_analytics` | bitemporal lens, event graph, topic river, drift, ESEM/DSEM builder, invariance and leakage audit |

Every boundary must be independently usable and expose versioned contracts for integration with organization repositories, `naruon`, and `contextual-orchestrator`.

## Implemented foundation topology

Task 1 materializes the first storage-independent workspace boundaries. The
crate names are stable implementation identifiers, while the broader service
boundaries above remain the target modular MSA architecture.

| Rust crate | Initial responsibility |
|---|---|
| `evidence_core` | immutable evidence domain primitives |
| `temporal_core` | typed clocks, intervals, and temporal reasoning |
| `event_core` | event instances, mentions, roles, and provenance |
| `relation_graph` | typed relations and forward-transition validation |
| `membership_core` | time-varying cross-classified multiple membership |
| `persistence_postgres` | PostgreSQL repositories and migrations |
| `corpus_split` | cutoff-safe, relation-aware partitioning |
| `tepp_simulation` | known-truth temporal/event data generation |
| `validation_core` | RMSE, bias, coverage, graph, and Monte Carlo metrics |
| `tepp_api` | versioned DTO, schema, and export contracts |
| `psychometric_core` | posterior-aware structural input gates, CWC within/between OLS plus the contextual effect, event-time log-rate, unequal-interval discrete-lag remapping, constant-predictor discrete effect, time-varying-predictor discrete effect (Eq. 14), exact scalar discrete process noise (Driver et al., 2017, Eq. 3), lagged latent covariance and unconditional latent variance (Driver et al., 2017, Eq. 3–4), stationary within-subject variance (Driver et al., 2017, Eq. 4 as `Δt → ∞`; `asymDIFFUSION`), trait-plus-state variance (Driver et al., 2017, §4.3 `TRAITVAR`; not process noise), observed-indicator variance and lagged observed covariance (Driver et al., 2017, Eq. 5; Table 2 `MANIFESTVAR` is `Θ`, not `Var(y)`; `MANIFESTTRAITVAR` is not `MANIFESTVAR`; `Θ` does not enter lagged observed covariance; observed-indicator mean is `τ + λ μ`; `MANIFESTMEANS` is not `E(y)`; `CINT` is not `MANIFESTMEANS`; discrete latent mean is `exp(a Δt) μ_0 + (exp(a Δt) − 1)/a κ`; `T0MEANS` is not `μ_t`; `CINT` is not the discrete increment; evolved observed mean is `τ + λ μ_t`; `τ + λ μ_0` is not `E(y_t)`; contemporaneous `TDPREDEFFECT` impulse is `m x`, not `CINT`, not `TIPREDEFFECT`, and not Voelkle Eq. 14; Eq. 5 of that contemporaneous impulse is `τ + λ(μ_t + m x)`, and `τ + λ μ_t` is not that observed mean; time-independent `TIPREDEFFECT` increment is `A^{-1}[e^{A Δt} − I] B z`, not `CINT`, not `M x`, not Voelkle Eq. 14, and not the coefficient `B`; within-interval `TDPREDEFFECT` carry is `e^{A(t−u)} M x` for `t0 < u < t`, not the contemporaneous Dirac, not `CINT`, not `TIPREDEFFECT`, and not Voelkle Eq. 14; Eq. 5 of that carry is `τ + λ(μ_t + e^{a(t−u)} m x)`, and `τ + λ μ_t` is not that observed mean; `τ + λ(μ_t + m x)` is not that carried observed mean when `u ≠ t`), irregular already-centered residual lag, Rubin `T` on OLS loadings, and strong-gated latent means (two-observation residual variance is identically `0` and caps at strong/scalar; Putnick & Bornstein, 2016) |

No crate exposes placeholder production behavior in Task 1. This prevents an
empty façade from becoming a de facto public API before its invariants and tests
exist.

## Immutable evidence boundary

Task 2 begins the executable `evidence_core` boundary. Stable RFC 9562 `UUIDv7`
identities are independent from canonical `SHA-256` content digests. Source
bytes and UTF-8 document text are copied into immutable owned storage, bounded
before allocation, and verified without exposing mutable fields.

A source span records an owning document, a half-open UTF-8 byte range, the
matching half-open Unicode-scalar range, and optional page/layout geometry. It
fails closed for empty or reversed ranges, byte or scalar overflow,
mid-code-point boundaries, coordinate disagreement, cross-document use,
nonfinite geometry, nonpositive dimensions, and rectangles outside the page.
Scalar coordinates are evidence locations rather than grapheme, word, or
sentence boundaries; language-tailored segmentation remains a later module.

The boundary now exposes a strict JSON wire version `1` without exposing private
Rust fields. Artifacts, documents, spans, and nested page locations are serialized
through explicit DTOs with unknown-field rejection. Reconstruction parses and
revalidates RFC 9562 identifiers, canonical digests, content limits, exact text
coordinates, document ownership, and page geometry. Artifact bytes and document
text are rehashed during reconstruction, and digest substitution fails closed.
Malformed JSON, unsupported versions, invalid byte values, and unknown nested
fields produce stable content-redacting errors.

Persistence, JSON Schema publication, JSON-LD, GraphML, source acquisition
metadata, signatures, and W3C PROV remain outward adapters or later contracts.
They must depend inward on these validated domain values rather than defining
them.

## Quality architecture

The workspace centralizes package metadata and Rust/Clippy lints. Every member
inherits `unsafe_code = "forbid"`, `missing_docs = "deny"`, and warning denial.
Repository contract scripts independently verify the approved crate set,
workspace inheritance, action SHA pinning, absence of LLM credentials from
ordinary CI, and complete Rust documentation.

Stable Rust 1.97.1 is the compile, lint, test, and line-coverage reference.
Branch coverage runs in a pinned nightly lane because LLVM branch coverage
remains unstable in Rust. `cargo-nextest` runs tests without retries, while
doctests remain a separate `cargo test --doc` gate. `cargo-deny` enforces
advisory, license, ban, and source policy. Failed Rust coverage gates print the
exact missing source locations from the same instrumented run without weakening
the 100% contract.

## Temporal invariants

TEPP stores event/valid time, assertion time, document time, system time, available time, and knowledge cutoff independently. A historical analysis may include a document only when:

\[
\operatorname{available\_time}(d) \leq \operatorname{knowledge\_cutoff}.
\]

Forward transition edges require a temporally valid partial order. Retrospective, revision, translation, citation, support, and contradiction relations retain their direction and provenance but do not create reverse state transitions.

## Measurement invariants

All languages share global topic identities and latent document coordinates. Language-specific lexical emissions, morphology, script, and content deviations are modeled rather than forced to be identical. Validated, calibrated, provisional, and unresolved language profiles are reported separately.

Repeated report vocabulary is modeled through corpus-background, template, section, style, copied-text, prompt, modality, and substantive-topic sources. It is not silently removed by stopword lists, TF-IDF, or BM25.

Topic proportions are compositional. ESEM and network analysis consume logistic-normal latent coordinates or orthonormal log-ratio coordinates, with posterior uncertainty propagated through plausible values or a joint model.

## Compute architecture

The CPU `f64` implementation is the numerical reference. Rayon-style fixed worker pools and thread-local sufficient statistics minimize context switching and oversubscription. GPU work is streamed; temporary responsibilities are never retained for the full corpus. The VRAM controller estimates peak allocation, reserves a safety margin, autotunes micro-batches, records telemetry, reduces batches after OOM, and falls back to CPU safely.

## Persistence

PostgreSQL is the reference relational store. Database objects use two-or-more-word `snake_case` names, including `document_record`, `temporal_interval`, `event_instance`, `event_mention`, `document_relation`, `segment_relation`, `entity_role_assignment`, `model_run`, `topic_definition`, `topic_correlation`, `topic_cluster`, `factor_solution`, `validation_metric`, and `audit_event`.

## Security and trust boundaries

Documents and LLM outputs are untrusted. Exact spans, JSON Schema, size/depth limits, Unicode validity, prompt-injection isolation, provider allowlists, no-tool execution, tenant isolation, immutable audit events, dependency pinning, SBOM, provenance, and reproducible releases are mandatory. LLM live tests use `NVIDIA_NIM_API_KEY`; `COPILOT_GITHUB_TOKEN` is forbidden.
