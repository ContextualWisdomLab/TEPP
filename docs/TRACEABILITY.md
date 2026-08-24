# TEPP Requirements, Research, and Evidence Traceability

**Status:** Accepted cross-cutting traceability baseline  
**Last reviewed:** 2026-08-16

The full APA 7th standards/literature register remains `docs/research/standards-and-literature.md`. This matrix links durable requirements to their owning decisions and implementation/evidence maturity without duplicating the bibliography.

| Requirement / decision | Canonical basis | Source/evidence boundary | Maturity |
|---|---|---|---|
| immutable source evidence and exact spans | PRD; Architecture; ADR 0008 | `evidence_core`, Task 2 tests/doctoring; `persistence_postgres` source-artifact SQL insert/lookup plus idempotent retry (#40 implemented-main); `payload_bound` inbound identity/provenance/size/depth on the active PR | partial |
| Rust numerical authority / CPU `f64` reference | ADR 0001 | current workspace foundation; future estimators | partial |
| Rust workspace/quality foundation | ADR 0007 | workspace/CI/repository contract | implemented-main |
| six distinct clocks and uncertain intervals | PRD; ADR 0002 | PR #8 `temporal_core` on protected main; `system_clock` system-vs-other-clock identity on the active PR | active-PR |
| Allen relation algebra/bounded closure | ADR 0002; temporal research | PR #9 `temporal_core` path-consistency on protected main | implemented-main |
| forward-only transition subgraph | PRD; ADR 0002/0003 | `relation_graph` on protected main; `copy_identity` copy-versus-source identity on the active PR | partial |
| event ontology/evidence mentions | PRD; ADR 0003 | `event_core` mention/instance separation on protected main; `persistence_postgres` mention SQL implemented-main refuses mention-as-instance; event-instance SQL (#39 implemented-main) refuses inverted windows; full intelligence stack remaining | partial |
| time-varying cross-classified multiple membership | PRD; ADR 0003 | `membership_core` network on protected main; `inferred_status` inferred-versus-observed identity on the active PR; multilevel estimators remaining | partial |
| leakage-safe availability/cutoff snapshots | PRD; ADR 0002/0013 | `corpus_split` on protected main | implemented-main |
| recovery metrics (RMSE, bias, coverage, graph, temporal order, Monte Carlo SE gates) | PRD; Test Strategy; ADR 0007/0014 | `validation_core` on protected main (PR #19); SE-aware Monte Carlo gates included | implemented-main |
| PostgreSQL bitemporal/lineage persistence | ADR 0013; Architecture/ERD | `persistence_postgres` on protected main as before; `revision_order` later-revision system-time gate on the active PR; remaining physical ERD constraints | partial |
| known-truth temporal/event simulation manifests | PRD; TRD; Test Strategy | `tepp_simulation` on protected main; recovery metrics in `validation_core` | implemented-main |
| versioned service/API contracts and exports | PRD; API contract; ADR 0011/0013 | `tepp_api` analysis-run/export/JSON-LD/GraphML contracts on protected main (PR #21); HTTP service remaining accepted-target | partial |
| immutable split/run/reproducibility manifests | ADR 0013; ERD | `tepp_api` reproducibility manifest and corpus-split leakage-audit wire (`CorpusSplitManifest` v1) on this PR; `persistence_postgres` append-only SQL insert/lookup for `reproducibility_manifest`, `corpus_split_manifest`, `model_run`, and `model_artifact` (migration `0003`); full physical ERD constraints remaining | partial |
| versioned service/API contracts and exports | PRD; API contract; ADR 0011/0013 | `tepp_api` analysis-run/export/JSON-LD/GraphML contracts on protected main (PR #21); merged PR #158 supplies the LineageWeave cutoff-safe temporal-context DTO and PR #155 carries its current loopback consumer boundary; production TLS remaining | partial |
| immutable split/run/reproducibility manifests | ADR 0013; ERD | `tepp_api` reproducibility manifest and corpus-split leakage-audit wire (`CorpusSplitManifest` v1) on the active stack; `persistence_postgres` append-only SQL insert/lookup for `reproducibility_manifest`, `corpus_split_manifest`, `model_run`, and `model_artifact` (migration `0003`); full physical ERD constraints remaining | partial |
| multilingual shared latent semantic space | PRD; ADR 0004 | future semantic/concept/topic crates | accepted-target |
| TRSL-TM temporal/relational topic posterior and backend compatibility | ADR 0012; ADR 0004 | future `topic_measurement` | accepted-target |
| global P0 topic identity with activity/dormancy/reactivation | ADR 0012 | future topic lineage/activity state | accepted-target |
| no default stopword deletion / no TF-IDF-BM25 inferential weighting | ADR 0004/0012; PRD/TRD | `stopword_deletion` default-list refusal on the active PR; TF-IDF/BM25 inferential-weight refusal remains accepted-target | partial |
| report template/section/copied/style/modality method effects | ADR 0004/0012; PRD/TRD | simulation truth factors implemented; estimator-side method model remains future | partial |
| candidate K statistical/Pareto gates + blinded LLM review | ADR 0012; research | future `model_selection` | accepted-target |
| compositional topic correlation / stable clustering | ADR 0005/0012; research | future `network_analysis` | accepted-target |
| posterior ESEM / longitudinal invariance / DSEM | ADR 0005 | `psychometric_fit` ESEM loading and DSEM lag gates on the active PR; `psychometric_core` input gates remain #49; invariance/multilevel remain accepted-target | active-PR |
| CPU bounded multithreading + GPU/VRAM streaming/parity | ADR 0001/0006 | future `compute_backend` | accepted-target |
| TDT detection/tracking vs CHRONOS schema/prediction/temporal consistency | ADR 0016; PRD/research | future `event_intelligence` | accepted-target |
| evidence-bounded LLM interpretation | ADR 0010/0012; PRD | future `interpretation_gateway` | accepted-target |
| adaptive direct/verify/committee/conductor test-time compute | ADR 0010; `docs/LLM_ORCHESTRATION.md` | `tepp_api::route_orchestration` + ablation record on the active PR; live contextual-orchestrator execution remaining | partial |
| purpose-bound PII handling without blanket masking | ADR 0009; `docs/PRIVACY_DATA_GOVERNANCE.md` | `tepp_api` export authorization plus provider-payload minimization / elevated re-identification implemented-main; `intake_authorization` grant-presence gate on the active PR; persistence retention/deletion remaining | partial |
| tenant/purpose/role/lifetime access and identity separation | ADR 0009; Threat Model | `tepp_api` time-bounded `PurposeGrant` + cross-tenant denial implemented-main; `intake_authorization` grant-presence gate on the active PR; persistent `access_grant` storage remaining | partial |
| standalone + modular CWL MSA / no cross-service DB coupling | ADR 0011; `docs/API_CONTRACT.md` | current standalone crates; future service ports | partial |
| naruon modular artifact consumer boundary | ADR 0011/0012; API contract | `docs/connectors/naruon-artifact-consumer.md` + PR #22 versioned consumer contract on protected main; `tepp_api` HTTP interchange (PR #42 implemented-main); loopback live listener on the active PR; production TLS remaining | partial |
| contextual-orchestrator interpretation port boundary | ADR 0010/0011; LLM orchestration | `docs/connectors/contextual-orchestrator-interpretation-port.md`; live port remaining | partial |
| Actions registry identities bound to protected-main tree (orphan disable) | Operability; GitHub Actions REST | `scripts/actions_workflow_fleet.py` + issue #20 tests/doctoring; live disable remains operator-authorized | active-PR |
| autonomous model proposal separated from verification/publication/review/merge | ADR 0015 | future safe OpenCode/NVIDIA autonomous-development workflow | accepted-target |
| contextual-orchestrator execution boundary | ADR 0010/0011 | credential-free `bind_contextual_orchestrator` on the active PR; live HTTP remaining | partial |
| foundation validation / release-readiness ledger | ADR 0014; Test Strategy | PR #24 `docs/validation/temporal-event-foundation.md` on protected main | implemented-main |
| scientific claim promotion separated from design/implementation/release | ADR 0014; ADR policy | documentation/CI/domain validation/release evidence | partial |
| CSAP/SOC 2/ISO/NIST assurance readiness | `docs/COMPLIANCE_READINESS.md`; research register | repository controls + future deployment evidence | accepted-target / deployment-owned |
| threat-model controls and scientific-integrity security | `SECURITY.md`; `docs/THREAT_MODEL.md` | deterministic security/privacy/scientific validation gates | partial |
| accessible bitemporal/network/drift/invariance views | PRD/UML | future `visual_analytics`; Figma in approved visual phase | accepted-target |
| 100% production line/branch/public docs | ADR 0007; AGENTS | CI/repository contracts | implemented-main and required for future source |
| SBOM/provenance/reproducible release | ADR 0014; Operability/Compliance | `scripts/release_evidence.py` CycloneDX SBOM + exact-head provenance + checksums in CI; full package/image release bundle remaining | partial |

## Scientific evidence promotion

Promotion rules are governed by ADR 0014 and `docs/adr/ADR_POLICY.md`. A decision can be `Accepted` while implementation remains `accepted-target`. A target becomes `implemented-main` only when its source is integrated on protected main and the relevant exact-head tests, scientific/recovery/validation evidence, security/supply-chain checks, and qualifying review required by live policy pass. Planning documents, simulations that do not exercise production code, queued checks, predecessor-head results, or LLM judgments cannot promote implementation maturity.

Replacement or replay PRs inherit source/test lineage only for auditability; they do **not** inherit current-head CI, security, review, or approval evidence. PRs #8 and #9 reacquired those gates before their protected-main merges. Every active PR listed above must independently reacquire the same exact-head evidence before its capability can be promoted.

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
