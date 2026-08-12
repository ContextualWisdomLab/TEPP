# TEPP Requirements, Research, and Evidence Traceability

**Status:** Accepted cross-cutting traceability baseline  
**Last reviewed:** 2026-08-12

The full APA 7th standards/literature register remains `docs/research/standards-and-literature.md`. This matrix links durable requirements to their owning decisions and implementation/evidence maturity without duplicating the bibliography.

| Requirement / decision | Canonical basis | Source/evidence boundary | Maturity |
|---|---|---|---|
| immutable source evidence and exact spans | PRD; Architecture; ADR 0008 | `evidence_core`, Task 2 tests/doctoring | implemented-main |
| Rust numerical authority / CPU `f64` reference | ADR 0001 | current workspace foundation; future estimators | partial |
| Rust workspace/quality foundation | ADR 0007 | workspace/CI/repository contract | implemented-main |
| six distinct clocks and uncertain intervals | PRD; ADR 0002 | PR #8 replacement `temporal_core` + tests/doctoring; PR #5 historical only | active-PR |
| Allen relation algebra/bounded closure | ADR 0002; temporal research | legacy PR #6 unique Task 4 work must be replayed on PR #8/current-main lineage before promotion | accepted-target / legacy active work |
| forward-only transition subgraph | PRD; ADR 0002/0003 | future `relation_graph` validation | accepted-target |
| event ontology/evidence mentions | PRD; ADR 0003 | future `event_core` | accepted-target |
| time-varying cross-classified multiple membership | PRD; ADR 0003 | future `membership_core` | accepted-target |
| leakage-safe availability/cutoff snapshots | PRD; ADR 0002/0013 | future `corpus_split` | accepted-target |
| PostgreSQL bitemporal/lineage persistence | ADR 0013; Architecture/ERD | future `persistence_postgres` migrations | accepted-target |
| immutable split/run/reproducibility manifests | ADR 0013; ERD | future persistence/model-run artifact chain | accepted-target |
| multilingual shared latent semantic space | PRD; ADR 0004 | future semantic/concept/topic crates | accepted-target |
| TRSL-TM temporal/relational topic posterior and backend compatibility | ADR 0012; ADR 0004 | future `topic_measurement` | accepted-target |
| global P0 topic identity with activity/dormancy/reactivation | ADR 0012 | future topic lineage/activity state | accepted-target |
| no default stopword deletion / no TF-IDF-BM25 inferential weighting | ADR 0004/0012; PRD/TRD | future semantic/method-source model | accepted-target |
| report template/section/copied/style/modality method effects | ADR 0004/0012 | future preprocessing/topic model | accepted-target |
| candidate K statistical/Pareto gates + blinded LLM review | ADR 0012; research | future `model_selection` | accepted-target |
| compositional topic correlation / stable clustering | ADR 0005/0012; research | future `network_analysis` | accepted-target |
| posterior ESEM / longitudinal invariance / DSEM | ADR 0005 | future `psychometric_core` | accepted-target |
| CPU bounded multithreading + GPU/VRAM streaming/parity | ADR 0001/0006 | future `compute_backend` | accepted-target |
| TDT detection/tracking vs CHRONOS schema/prediction/temporal consistency | ADR 0016; PRD/research | future `event_intelligence` | accepted-target |
| evidence-bounded LLM interpretation | ADR 0010/0012; PRD | future `interpretation_gateway` | accepted-target |
| adaptive direct/verify/committee/conductor test-time compute | ADR 0010; `docs/LLM_ORCHESTRATION.md` | future contextual-orchestrator integration + ablation evidence | accepted-target |
| purpose-bound PII handling without blanket masking | ADR 0009; `docs/PRIVACY_DATA_GOVERNANCE.md` | future authorization/persistence/export/provider adapters | accepted-target |
| tenant/purpose/role/lifetime access and identity separation | ADR 0009; Threat Model | future service/persistence boundaries | accepted-target |
| standalone + modular CWL MSA / no cross-service DB coupling | ADR 0011; `docs/API_CONTRACT.md` | current standalone crates; future service ports | partial |
| naruon consumer boundary | ADR 0011/0012; API contract | versioned TEPP artifacts/API; no lexical heuristic substitution | accepted-target |
| contextual-orchestrator execution boundary | ADR 0010/0011 | provider-neutral orchestration port; TEPP retains scientific authority | accepted-target |
| autonomous model proposal separated from verification/publication/review/merge | ADR 0015 | future safe OpenCode/NVIDIA autonomous-development workflow | accepted-target |
| scientific claim promotion separated from design/implementation/release | ADR 0014; ADR policy | documentation/CI/domain validation/release evidence | partial |
| CSAP/SOC 2/ISO/NIST assurance readiness | `docs/COMPLIANCE_READINESS.md`; research register | repository controls + future deployment evidence | accepted-target / deployment-owned |
| threat-model controls and scientific-integrity security | `SECURITY.md`; `docs/THREAT_MODEL.md` | deterministic security/privacy/scientific validation gates | partial |
| accessible bitemporal/network/drift/invariance views | PRD/UML | future `visual_analytics`; Figma in approved visual phase | accepted-target |
| 100% production line/branch/public docs | ADR 0007; AGENTS | CI/repository contracts | implemented-main and required for future source |
| SBOM/provenance/reproducible release | ADR 0014; Operability/Compliance | future exact-release evidence bundle | accepted-target |

## Scientific evidence promotion

Promotion rules are governed by ADR 0014 and `docs/adr/ADR_POLICY.md`. A decision can be `Accepted` while implementation remains `accepted-target`. A target becomes `implemented-main` only when its source is integrated on protected main and the relevant exact-head tests, scientific/recovery/validation evidence, security/supply-chain checks, and qualifying review required by live policy pass. Planning documents, simulations that do not exercise production code, queued checks, predecessor-head results, or LLM judgments cannot promote implementation maturity.

A replacement PR inherits source/test lineage only for auditability; it does **not** inherit current-head CI, security, review, or approval evidence. PR #8 therefore reacquires all merge evidence even though its initial implementation blobs preserve Task 3's prior RED→GREEN history. Legacy PR #6 remains non-promotable until its unique Task 4 behavior is replayed onto the canonical Task 3 lineage and independently revalidated.

## Claim discipline

- Allen path consistency must not be promoted into unrestricted global satisfiability proof.
- Language support is not promoted from architecture alone; each profile needs validation/invariance evidence.
- GPU support is not promoted if required hardware tests are skipped or only a software fallback ran when hardware parity is claimed.
- Dynamic ESEM/DSEM effects are not promoted to causal claims without an identified design and corresponding evidence.
- Topic clusters, event links, TDT tracking, CHRONOS predictions, and LLM agreement are not observed fact or causal evidence by identity.
- LLM or multi-agent agreement does not establish measurement truth; source evidence and scientific gates remain authoritative.
- CSAP readiness, SOC 2 readiness, ISO/NIST alignment, and repository controls are not certification or attestation.
- A `200억 달러` acquisition bar is a prioritization heuristic, not a valuation result.

## Documentation fitness trace

`docs/DOCUMENTATION_ASSESSMENT.md` evaluates the full canonical graph and distinguishes design sufficiency from protected-main sufficiency. `docs/adr/README.md` is the decision ownership/supersession map; `docs/adr/ADR_POLICY.md` defines decision status independently from implementation maturity. New documentation or accepted architecture does not become protected-main implementation evidence until the exact documentation/implementation PR is integrated under repository policy.

## Documentation rule

When a scientific estimand, time meaning, ontology relation, membership structure, topic/backend identity, compute backend, privacy/authorization model, service authority, persistence/reproducibility contract, orchestration policy, autonomous-development authority, event-intelligence claim, implementation-lineage authority, or accepted evidence threshold changes, update the owning ADR/PRD where required plus this matrix, affected architecture/data/API documents, exact tests/validation evidence, and standards/research documentation in the same reviewed change.
