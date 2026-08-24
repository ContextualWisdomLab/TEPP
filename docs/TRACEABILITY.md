# TEPP Requirements, Research, and Evidence Traceability

**Status:** Accepted cross-cutting traceability baseline  
**Last reviewed:** 2026-08-19

The full APA 7th standards/literature register remains `docs/research/standards-and-literature.md`. This matrix links durable requirements to their owning decisions and implementation/evidence maturity without duplicating the bibliography.

| Requirement / decision | Canonical basis | Source/evidence boundary | Maturity |
|---|---|---|---|
| immutable source evidence and exact spans | PRD; Architecture; ADR 0008 | `evidence_core`, Task 2 tests/doctoring; `persistence_postgres` source-artifact SQL insert/lookup plus idempotent retry (#40 implemented-main); typed `text_segment` byte-span SQL (active PR) | implemented-main |
| Rust numerical authority / CPU `f64` reference | ADR 0001 | current workspace foundation; future estimators | partial |
| Rust workspace/quality foundation | ADR 0007 | workspace/CI/repository contract | implemented-main |
| six distinct clocks and uncertain intervals | PRD; ADR 0002; ISO 24617-1:2012; Hobbs & Pan (2017) | merged PR #8 `temporal_core` on protected main; PR #5 historical lineage only | implemented-main |
| Allen relation algebra/bounded closure | ADR 0002; Allen (1983) | merged PR #9 `temporal_core` path-consistency on protected main | implemented-main |
| forward-only transition subgraph | PRD; ADR 0002/0003 | `relation_graph` on protected main | implemented-main |
| event ontology/evidence mentions | PRD; ADR 0003 | `event_core` mention/instance separation on protected main; `persistence_postgres` mention SQL implemented-main refuses mention-as-instance; event-instance SQL (#39 implemented-main) refuses inverted windows; Brier calibration on the active PR; full intelligence stack remaining | partial |
| time-varying cross-classified multiple membership | PRD; ADR 0003 | `membership_core` network on protected main; multilevel estimators remaining | partial |
| leakage-safe availability/cutoff snapshots | PRD; ADR 0002/0013 | `corpus_split` on protected main | implemented-main |
| recovery metrics (RMSE, bias, coverage, graph, temporal order, Monte Carlo SE gates) | PRD; Test Strategy; ADR 0007/0014 | `validation_core` on protected main (PR #19); SE-aware Monte Carlo gates included | implemented-main |
| PostgreSQL bitemporal/lineage persistence | ADR 0013; Architecture/ERD | `persistence_postgres` migration contracts, in-memory adapters, live SQL session/document SQL port, tenant RLS (`0002` + session GUC/role helpers), `DATABASE_URL` SQLx gate, optional `live-sqlx` `PgPool` driver, exact-head live PostgreSQL CI with isolation proof, append-only immutability triggers (`0004`), temporal interval ordering CHECKs (`0005`), typed membership assignment (`0006` implemented-main), event-relation/mention/instance SQL (#37–#39 implemented-main), source-artifact SQL (#40 implemented-main), audit-event SQL (#41 implemented-main), concurrent document-write stress (#43 implemented-main), backup/restore integrity revalidation (#44 implemented-main), typed `text_segment` SQL insert/cutoff lookup (active PR); remaining physical ERD constraints including `document_record` FK on `text_segment` | partial |
| known-truth temporal/event simulation manifests | PRD; TRD; Test Strategy | `tepp_simulation` on protected main; recovery metrics in `validation_core` | implemented-main |
| analysis-run/export DTO and artifact contracts | PRD; API contract; ADR 0011/0013 | `tepp_api` analysis-run/export/JSON-LD/GraphML contracts on protected main (merged PR #21) | implemented-main |
| production HTTP service routing | PRD; API contract; ADR 0011 | No deployed HTTP service; endpoint shapes remain a future service target | partial |
| immutable split/run/reproducibility manifests | ADR 0013; ERD | `tepp_api` reproducibility manifest contract on protected main; `persistence_postgres` append-only SQL insert/lookup for `reproducibility_manifest`, `corpus_split_manifest`, `model_run`, and `model_artifact` (migration `0003`); full physical ERD constraints remaining | partial |
| multilingual shared latent semantic space | PRD; ADR 0004; Mimno et al. (2009); Blei & Lafferty (2006); Roberts et al. (2014, 2019) | future semantic/concept/topic crates | accepted-target |
| TRSL-TM temporal/relational topic posterior and backend compatibility | ADR 0012; ADR 0004; Chang & Blei (2009); Blei & Lafferty (2006); Roberts et al. (2014, 2019) | future `topic_measurement`; TRSL-TM is the product contract, STM-style logistic-normal the reference family | accepted-target |
| global P0 topic identity with activity/dormancy/reactivation | ADR 0012 | future topic lineage/activity state | accepted-target |
| no default stopword deletion / no TF-IDF-BM25 inferential weighting | ADR 0004/0012; PRD/TRD | future semantic/method-source model | accepted-target |
| report template/section/copied/style/modality method effects | ADR 0004/0012; PRD/TRD | simulation truth factors implemented; estimator-side method model remains future | partial |
| candidate K statistical/Pareto gates + blinded LLM review | ADR 0012; research | future `model_selection` | accepted-target |
| compositional topic correlation / stable clustering | ADR 0005/0012; research | `network_analysis` simplex refusal and pair precision/recall on the active PR; graphical lasso/Leiden remaining | active-PR |
| posterior ESEM / longitudinal invariance / DSEM | ADR 0005 | future `psychometric_core` | accepted-target |
| CPU bounded multithreading + GPU/VRAM streaming/parity | ADR 0001/0006 | future `compute_backend` | accepted-target |
| TDT detection/tracking | ADR 0016; Allan (2002) | future `event_intelligence` | accepted-target |
| neural event-schema induction and prediction | ADR 0016; Li et al. (2021) | future `event_intelligence` | accepted-target |
| symbolic qualitative temporal consistency | ADR 0016; Anagnostopoulos et al. (2013) | future `event_intelligence` | accepted-target |
| evidence-bounded LLM interpretation | ADR 0010/0012; PRD | `tepp_api` router plus future `interpretation_gateway` | partial |
| adaptive direct/verify/committee/conductor test-time compute | ADR 0010; `docs/LLM_ORCHESTRATION.md` | `tepp_api::route_orchestration`, ablation record, and credential-free contextual-orchestrator binding on protected main; live execution and learned conductor calibration remain future | partial |
| purpose-bound PII handling without blanket masking | ADR 0009; `docs/PRIVACY_DATA_GOVERNANCE.md` | `tepp_api` export authorization, elevated re-identification, and provider-payload minimization are implemented-main; migration `0007` retention/deletion/legal-hold SQL contracts are implemented-main; deployment/provider evidence remains accepted-target | partial |
| tenant/purpose/role/lifetime access and identity separation | ADR 0009; Threat Model | `tepp_api` time-bounded `PurposeGrant` + cross-tenant denial implemented-main; persistent `access_grant` storage remaining | partial |
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
