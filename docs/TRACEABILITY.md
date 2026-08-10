# TEPP Requirements, Research, and Evidence Traceability

**Status:** Accepted cross-cutting traceability baseline  
**Last reviewed:** 2026-08-10

The full APA 7th standards/literature register remains `docs/research/standards-and-literature.md`. This matrix links durable requirements to implementation/evidence maturity without duplicating the bibliography.

| Requirement / decision | Canonical basis | Source/evidence boundary | Maturity |
|---|---|---|---|
| immutable source evidence and exact spans | PRD; Architecture; ADR-0001/0008 | `evidence_core`, Task 2 tests/doctoring | implemented-main |
| Rust workspace/quality foundation | ADR-0001/0007 | workspace/CI/repository contract | implemented-main |
| six distinct clocks and uncertain intervals | PRD; ADR-0002 | PR #5 `temporal_core` + tests/doctoring | active-PR |
| Allen relation algebra/bounded closure | temporal architecture/research | PR #6 `temporal_core` + tests/doctoring | active-PR |
| forward-only transition subgraph | PRD; ADR-0002/0003 | future `relation_graph` validation | accepted-target |
| event ontology/evidence mentions | PRD; ADR-0003 | future `event_core` | accepted-target |
| time-varying cross-classified multiple membership | PRD; ADR-0003 | future `membership_core` | accepted-target |
| leakage-safe availability/cutoff snapshots | PRD; ADR-0002 | future `corpus_split` | accepted-target |
| PostgreSQL bitemporal/lineage persistence | Architecture/ERD | future `persistence_postgres` migrations | accepted-target |
| multilingual shared latent semantic space | PRD; ADR-0004 | future semantic/concept/topic crates | accepted-target |
| temporal/relational topic posterior | PRD; ADR-0004/0005 | future `topic_measurement` | accepted-target |
| no default stopword deletion / no TF-IDF-BM25 inferential weighting | PRD; TRD; Architecture | future `semantic_preprocessor` / method-source model | accepted-target |
| report template/section/copied/style method effects | PRD; TRD | future preprocessing/topic model | accepted-target |
| CPU f64 reference + bounded multithreading | ADR-0001/0006 | future compute implementation | accepted-target |
| GPU/VRAM streaming + CPU parity | ADR-0006 | future `compute_backend` | accepted-target |
| candidate K statistical + blinded LLM review | PRD/research | future `model_selection` | accepted-target |
| compositional topic correlation / stable clustering | PRD/research | future `network_analysis` | accepted-target |
| posterior ESEM / longitudinal invariance / DSEM | ADR-0005 | future `psychometric_core` | accepted-target |
| TDT/CHRONOS event intelligence | PRD/research | future `event_intelligence` | accepted-target |
| evidence-bounded LLM interpretation | PRD; ADR-0006/0010 | future `interpretation_gateway` | accepted-target |
| adaptive direct/multi-agent test-time compute | ADR-0010; `docs/LLM_ORCHESTRATION.md` | future contextual-orchestrator integration + ablation evidence | accepted-target |
| purpose-bound PII handling without blanket masking | ADR-0009; `docs/PRIVACY_DATA_GOVERNANCE.md` | future authorization/persistence/export/provider adapters | accepted-target |
| tenant/purpose/role/lifetime access and identity separation | ADR-0009; Threat Model | future service/persistence boundaries | accepted-target |
| standalone + modular CWL MSA / no cross-service DB coupling | ADR-0011; `docs/API_CONTRACT.md` | future service ports + consumer contract tests | accepted-target |
| naruon consumer boundary | ADR-0011; API contract | versioned TEPP artifacts/API; no lexical heuristic substitution | accepted-target |
| contextual-orchestrator execution boundary | ADR-0010/0011 | provider-neutral orchestration port; TEPP retains scientific authority | accepted-target |
| CSAP/SOC 2/ISO/NIST assurance readiness | `docs/COMPLIANCE_READINESS.md`; research register | repository controls + future deployment evidence | accepted-target / deployment-owned |
| threat-model controls and scientific-integrity security | `SECURITY.md`; `docs/THREAT_MODEL.md` | deterministic security/privacy/scientific validation gates | accepted-target; foundation portions implemented-main |
| accessible bitemporal/network/drift/invariance views | PRD | future `visual_analytics`; Figma in approved visual phase | accepted-target |
| 100% production line/branch/docs | AGENTS/quality architecture | CI/repository contracts | implemented-main and required for all future source |
| SBOM/provenance/reproducible release | AGENTS; Operability; Compliance | future exact-release evidence bundle | accepted-target |

## Scientific evidence promotion

A target becomes `implemented-main` only when its source is integrated on protected main and the relevant current-head tests, recovery/validation evidence, security/supply-chain checks, and qualifying independent review pass. Planning documents, simulations that do not exercise production code, queued checks, predecessor-head results, or LLM judgments cannot promote implementation maturity.

## Claim discipline

- PR #6 path consistency must not be promoted into global satisfiability proof.
- Language support is not promoted from architecture alone; each profile needs validation/invariance evidence.
- GPU support is not promoted if required GPU tests are skipped or only a software fallback ran when hardware parity is claimed.
- Dynamic ESEM/DSEM effects are not promoted to causal claims without an identified design and corresponding evidence.
- LLM or multi-agent agreement does not establish measurement truth; source evidence and scientific gates remain authoritative.
- CSAP readiness, SOC 2 readiness, ISO/NIST alignment, and repository controls are not certification or attestation.
- A `200억 달러` acquisition bar is a prioritization heuristic, not a valuation result.

## Documentation fitness trace

`docs/DOCUMENTATION_ASSESSMENT.md` evaluates the full canonical documentation graph and distinguishes design sufficiency from protected-main sufficiency. New documentation or accepted architecture does not become protected-main authority until the exact documentation PR is integrated under repository policy.

## Documentation rule

When a scientific estimand, time meaning, ontology relation, membership structure, compute backend, privacy/authorization model, service authority, persistence contract, orchestration policy, or accepted evidence threshold changes, update the corresponding PRD version/ADR where required plus this matrix and the exact tests/doctoring in the same reviewed change.
