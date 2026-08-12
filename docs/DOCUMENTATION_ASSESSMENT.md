# TEPP Whole-Conversation Documentation Assessment

**Assessment date:** 2026-08-13  
**Scope:** Durable TEPP decisions from the full product conversation, PRD v0.4,
concrete PRD v0.5, protected `main` at
`1832026121e7ad92d21e0592fdd0ad5a59f40cff`, open persistence PR #30, and the
canonical ADR/TRD/Architecture/UML/ERD/security/privacy/test/operability graph.  
**Verdict:** **The documentation set is design-sufficient after PRD v0.5, but
protected-main sufficiency and product maturity remain separately gated by
integration, current-head checks, scientific evidence, and deployment evidence.**

## 1. Assessment method

A file existing is not evidence that the product contract is complete or that a
capability is shipped. Each documentation family is evaluated against:

1. durable user decisions;
2. current ADR authority and supersession;
3. protected-main source and migrations;
4. open pull requests and their exact heads;
5. scientific claim and recovery requirements;
6. security, privacy, operability, and release evidence;
7. whether a qualified reviewer can implement or audit the product without chat
   reconstruction.

Status vocabulary:

- **PRESENT-CURRENT** — canonical and semantically aligned to current product
  truth;
- **PRESENT-STALE** — exists but contradicts current decisions or source;
- **PARTIAL** — useful but missing a material contract;
- **MISSING** — no adequate canonical artifact;
- **NOT-APPLICABLE** — intentionally not required for the current boundary;
- **SUPERSEDED** — retained only as historical evidence;
- **deployment-owned** — requires a measured deployment;
- **external-assurance** — requires an independent assessor or authority.

Decision status and implementation maturity remain independent. An `Accepted`
ADR or PRD requirement is not automatically implemented or released.

## 2. Documentation family assessment

| Family | Status | Evidence / action |
|---|---|---|
| PRD | PRESENT-CURRENT | `docs/product/prd-v0.5.md` is the concrete canonical contract; v0.4 is historical evidence |
| Stable requirement identifiers | PRESENT-CURRENT | PRD v0.5 defines `FR-EVD-*` through `FR-OPS-*`, fail-closed behavior, and acceptance evidence |
| User/workflow/state contracts | PRESENT-CURRENT | PRD v0.5 defines product surfaces, deployment modes, input/output contracts, evidence/corpus/model/claim/release lifecycles, and error taxonomy |
| Scientific claim promotion | PRESENT-CURRENT | PRD v0.5 claim matrix plus ADR 0014 and Test Strategy |
| TRD | PRESENT-CURRENT | `docs/TRD.md` defines technical/scientific boundaries and implementation maturity |
| Architecture | PRESENT-CURRENT | root `ARCHITECTURE.md` owns service/crate and compute boundaries |
| UML/system flows | PRESENT-CURRENT | `docs/UML.md` covers temporal/event/membership/compute flows; future UI-specific flows remain event-driven |
| ERD/logical data model | PRESENT-CURRENT | `docs/ERD.md` distinguishes domain objects, planned persistence, and as-built claims |
| ADR index and policy | PRESENT-CURRENT | ADR 0001–0016 plus decision/maturity/supersession policy and ownership map |
| API and connectors | PRESENT-CURRENT | API contract plus naruon and contextual-orchestrator ports; production HTTP/jobs remain partial |
| Security and threat model | PRESENT-CURRENT | `SECURITY.md` and `docs/THREAT_MODEL.md` include scientific-integrity threats |
| Privacy/PII governance | PRESENT-CURRENT | purpose-bound PII controls without blanket masking, identity separation, provider disclosure, retention, and audit |
| Compliance readiness | PRESENT-CURRENT | CSAP/SOC 2/ISO/NIST readiness mapping without certification claims |
| Test/scientific validation | PRESENT-CURRENT | true-parameter recovery, Monte Carlo uncertainty, leakage, invariance, parity, event/network/cluster and LLM tests |
| Operability/recovery/release | PRESENT-CURRENT | job/recovery/release target contract; measured SLO/RPO/RTO remain deployment-owned |
| LLM orchestration | PRESENT-CURRENT | direct/verifier/committee/conductor boundaries and Fugu/Conductor/TRINITY-based ablation contract |
| Traceability | PRESENT-CURRENT | `docs/TRACEABILITY.md` maps PRD v0.5 families to ADRs, source, tests, PRs, and maturity |
| Research/APA 7 doctoring | PRESENT-CURRENT | primary standards and research register remains canonical; implementation-specific doctoring expands with each slice |
| AGENTS/CLAUDE/Governance | PRESENT-CURRENT | development, authority, quality, and naming contracts are discoverable |
| CHANGELOG | PARTIAL | implementation changes are recorded; PRD v0.5 entry is added by the same reviewed change |
| Figma interaction design | NOT-APPLICABLE now | PRD v0.5 specifies required surfaces/states; detailed Figma work begins when visual data contracts are stable |
| Physical persistence/RLS | active-PR / partial | base persistence exists; tenant FORCE RLS/runtime-role isolation remains PR #30 until integration |
| Production SLO/RPO/RTO evidence | deployment-owned | targets must be measured in a concrete environment |
| CSAP certification / SOC 2 attestation | external-assurance | repository controls cannot self-issue certification or attestation |

## 3. Why PRD v0.4 was not concrete enough

PRD v0.4 correctly fixed the product direction but was primarily a design
baseline. It did not provide enough detail for product delivery and acquisition
diligence in these areas:

- stable requirement identifiers for PR/test/issue traceability;
- complete buyer problems, roles, product surfaces, and deployment modes;
- input, output, artifact, and lifecycle state contracts;
- requirement-specific fail-closed behavior and acceptance evidence;
- language-profile promotion criteria;
- model-selection candidate plan and decision schema;
- scientific claim-promotion wording;
- compute admission and corpus scale profiles;
- purpose-bound PII and model-provider disclosure behavior at product level;
- accessible visual product states;
- error taxonomy, phased release slices, commercial metrics, and exact release
  acceptance checklist.

PRD v0.5 fills these gaps without changing the approved TEPP estimands or
architecture decisions.

## 4. Durable conversation decisions covered

The canonical documentation graph now explicitly preserves:

- TEPP as the product and TRSL-TM as the topic-measurement family;
- immutable source evidence and exact spans;
- event/valid, assertion, document, system, availability, and knowledge-cutoff
  clocks;
- partial-order interval reasoning and forward-only transitions;
- document, passage, event, entity-role, membership, topic, factor, and evidence
  graphs;
- time-varying cross-classified multiple membership;
- one shared multilingual latent topic identity with native lexical channels;
- launch validation profiles for English, Korean, Japanese, Simplified and
  Traditional Chinese, Vietnamese, Indonesian, French, German, and Turkish;
- no default stopword deletion and no TF-IDF/BM25 inferential weighting;
- template/section/copied/prompt/style/modality/background method sources;
- Rust CPU f64 numerical authority, bounded CPU parallelism, GPU/VRAM streaming,
  and parity evidence;
- statistically gated candidate-K selection plus blinded LLM review;
- valid compositional coordinates, posterior networks, and consensus clusters;
- reflective/formative/network construct-role gates and posterior-aware
  longitudinal ESEM/DSEM;
- TDT and CHRONOS task/claim separation;
- LLMs as fallible semantic/interpreter raters rather than statistical or
  release authority;
- standalone and modular MSA behavior with no cross-service table coupling;
- purpose-bound PII access and separately protected identity mappings rather
  than destructive blanket masking;
- CSAP/SOC 2/AI-governance readiness without false certification claims;
- 100% owned production line/branch coverage, complete public docs, realistic
  recovery tests, SBOM, provenance, rollback, and reproducible release gates.

## 5. Implementation truth

Documentation completeness must not be confused with product completeness.

### Implemented-main

- Rust workspace and exact quality gates;
- immutable evidence and exact source spans;
- typed six-clock values and uncertain intervals;
- bounded Allen path consistency;
- forward-only transition graph;
- leakage-safe snapshots and relation-connected splits;
- deterministic temporal/event simulations;
- core recovery metrics;
- selected API/export/reproducibility contracts;
- foundation validation and release-evidence tooling.

### Partial

- event mention/instance and membership surfaces;
- PostgreSQL bitemporal and live SQL port;
- API/service contracts and modular connectors;
- privacy/security/operability implementation beyond repository contracts.

### Active PR

- PR #30 tenant FORCE RLS, runtime role restrictions, session helpers, and live
  cross-tenant PostgreSQL tests until exact-head integration.

### Accepted target

- multilingual semantic units and governed concepts;
- TRSL-TM CPU estimator and posterior;
- candidate-K selection;
- topic networks and clusters;
- GPU/VRAM compute;
- TDT/CHRONOS intelligence;
- longitudinal ESEM/DSEM;
- evidence-bounded interpretation;
- coordinated accessible visual analytics;
- production job/API and deployment operations.

### Deployment-owned / external-assurance

- measured service SLO, RPO, RTO, capacity, regional controls, and backup/restore;
- CSAP certification, SOC 2 attestation, and jurisdiction-specific legal
  conclusions.

## 6. Remaining documentation work is implementation-triggered

No second parallel documentation authority is required. Future changes occur in
the canonical graph when implementation advances:

- semantic measurement → concept/language profile schemas and benchmark reports;
- topic estimator → model/data cards, posterior schemas, recovery reports, and
  compatibility artifacts;
- candidate selection → candidate plan, Pareto, review, and decision schemas;
- GPU → measured backend, precision, memory, and parity profiles;
- psychometrics → estimand, invariance, missingness, and recovery artifacts;
- visual analytics → Figma design system, interaction/state diagrams,
  accessibility, and exact-value export contracts;
- deployment → OpenAPI/AsyncAPI, measured SLO/RPO/RTO, KMS, backup/restore,
  incident, and assurance evidence.

## 7. Sufficiency conclusion

The TEPP documentation graph is **design-sufficient** after PRD v0.5 because a
qualified reviewer can reconstruct the buyer problem, product workflows, data
and model boundaries, fail-closed semantics, scientific claims, security/privacy
controls, visual surfaces, acceptance evidence, phased release scope, and
implementation maturity without chat history.

It becomes **protected-main-sufficient** only after this exact PRD/documentation
change integrates under repository rules and remains current with protected-main
source. Product completeness still requires implementation and scientific,
security/privacy, operational, accessibility, and commercial evidence for the
remaining PRD slices.
