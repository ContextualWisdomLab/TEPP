# TEPP Requirements, Research, and Evidence Traceability

**Status:** Accepted cross-cutting traceability baseline aligned to PRD v0.5  
**Last reviewed:** 2026-08-13

The full APA 7 standards/literature register remains
`docs/research/standards-and-literature.md`. This matrix links the stable PRD v0.5
requirement families to their owning decisions, implementation boundaries, and
claim maturity without duplicating the bibliography.

## Product requirement families

| PRD family | Product obligation | Owning authority | Current source/evidence boundary | Maturity |
|---|---|---|---|---|
| `FR-EVD-*` | Immutable evidence, exact spans, quarantine, version lineage, no active-content execution | ADR 0008; Architecture; TRD | `evidence_core` and evidence wire/security tests | implemented-main |
| `FR-TMP-*` | Six clocks, uncertain intervals, historical eligibility, interval reasoning, forward-only transitions | ADR 0002; ADR 0013 | `temporal_core`, `relation_graph`, `corpus_split` | implemented-main |
| `FR-REL-*` | Typed relations, observed/inferred separation, relation-aware partitions | ADR 0003; ADR 0016 | `relation_graph`, `corpus_split`; broader evidence graph remains partial | partial |
| `FR-MEM-*` | Cross-classified multiple membership, time-varying roles, ESS/design effects, atomistic-fallacy control | ADR 0003; ADR 0005 | `membership_core`; multilevel estimators remain accepted-target | partial |
| `FR-LNG-*` | Span-level language profiles, shared latent identity, profile promotion | ADR 0004; ADR 0012 | language/semantic/topic crates not yet integrated | accepted-target |
| `FR-SEM-*` | Non-destructive Unicode, language-tailored boundaries, POS/dependency/method sources, governed concepts, LLM proposal validation | ADR 0004; ADR 0012 | simulation method factors exist; semantic measurement remains accepted-target | partial |
| `FR-TOP-*` | Rust CPU f64 TRSL-TM, structural covariates, posterior uncertainty, drift, global topic identity, adapter conformance | ADR 0001; ADR 0012 | `topic_measurement` not yet integrated | accepted-target |
| `FR-KSEL-*` | Candidate plans, statistical hard gates, Pareto frontier, blinded LLM review, escalation | ADR 0010; ADR 0012; ADR 0014 | `model_selection` not yet integrated | accepted-target |
| `FR-NET-*` | Valid compositional coordinates, posterior edges, conditional networks, consensus clusters | ADR 0005; ADR 0012 | `network_analysis` not yet integrated | accepted-target |
| `FR-EVT-*` | Mention/instance separation, TDT tasks, schema-hypothesis authority, forecast calibration | ADR 0003; ADR 0016 | `event_core` mention/instance separation implemented; intelligence stack remaining | partial |
| `FR-PSY-*` | Construct-role gate, posterior propagation, invariance, within/between, irregular time, ordered paths, causal-language control | ADR 0005; ADR 0014 | `psychometric_core` not yet integrated | accepted-target |
| `FR-LLM-*` | Untrusted output, interpreter/verifier, unsupported claims, orchestration ablation, credential and authority boundaries | ADR 0010; ADR 0015 | connector and workflow contracts exist; analytical interpretation port remains partial | partial |
| `FR-CMP-*` | Compute profiles, bounded CPU pools, VRAM admission, streaming statistics, CPU/GPU parity, phase scheduling | ADR 0001; ADR 0006; ADR 0007 | reference workspace/quality gates exist; numerical/GPU backends remain accepted-target | partial |
| `FR-API-*` | Versioned contracts, async jobs, no cross-service table coupling, naruon/contextual-orchestrator authority | ADR 0011; ADR 0013 | `tepp_api` DTO/export contracts and connector documents; HTTP/jobs remaining | partial |
| `FR-EXP-*` | Accessible exact values, source-consistent exports, cutoff audit, reproducibility manifests | ADR 0011; ADR 0013; ADR 0014 | JSON-LD/GraphML/reproducibility contracts exist; visual/export suite remains partial | partial |
| `FR-SEC-*` | Tenant isolation, purpose-bound PII, identity separation, keys, audit, retention/deletion | ADR 0009; ADR 0013 | persistence contracts are partial; FORCE RLS/runtime-role work is active on PR #30 | active-PR |
| `FR-OPS-*` | Observability, crash recovery, backup/restore, capacity profiles, release evidence | ADR 0014; Operability | release-evidence tooling exists; deployment evidence remains partial/deployment-owned | partial |

## Protected-main implementation trace

| Capability | Canonical basis | Protected-main evidence | Maturity |
|---|---|---|---|
| immutable source evidence and exact spans | PRD `FR-EVD-*`; ADR 0008 | `evidence_core`, Task 2 tests and doctoring | implemented-main |
| Rust workspace and exact quality gates | PRD `FR-CMP-*`; ADR 0007 | workspace/CI/repository contracts | implemented-main |
| six distinct clocks and uncertain intervals | PRD `FR-TMP-001/002`; ADR 0002 | `temporal_core` | implemented-main |
| Allen relation algebra and bounded path consistency | PRD `FR-TMP-004`; ADR 0002 | `temporal_core` relation/reasoner contracts | implemented-main |
| forward-only transition subgraph | PRD `FR-TMP-005`; ADR 0002/0003 | `relation_graph` | implemented-main |
| event mention/instance separation | PRD `FR-EVT-001`; ADR 0003 | `event_core` promotion boundary | partial |
| weighted multiple membership and ESS helpers | PRD `FR-MEM-*`; ADR 0003 | `membership_core` | partial |
| leakage-safe cutoff snapshots and relation-connected splits | PRD `FR-TMP-003`, `FR-REL-003`; ADR 0002/0013 | `corpus_split` | implemented-main |
| known-truth temporal/event simulation | PRD scientific acceptance; Test Strategy | `tepp_simulation` | implemented-main |
| RMSE, bias, coverage, graph, temporal-order, and Monte Carlo SE-aware gates | PRD acceptance; ADR 0007/0014 | `validation_core` | implemented-main |
| bitemporal persistence contracts and live SQL port | PRD `FR-EVD-005`, `FR-SEC-*`; ADR 0013 | `persistence_postgres`, SQLx/live PostgreSQL CI | partial |
| tenant FORCE RLS and runtime-role restrictions | PRD `FR-SEC-001`; ADR 0009/0013 | PR #30 at exact head until protected-main integration | active-PR |
| versioned analysis-run DTOs and exports | PRD `FR-API-*`, `FR-EXP-*`; ADR 0011/0013 | `tepp_api`, JSON-LD/GraphML/reproducibility contracts | partial |
| naruon modular artifact consumer | PRD `FR-API-004`; ADR 0011/0012 | connector contract and protected-main consumer artifacts | partial |
| contextual-orchestrator interpretation boundary | PRD `FR-API-005`, `FR-LLM-*`; ADR 0010/0011 | connector and credential-separation documents | partial |
| release SBOM/provenance/checksum evidence | PRD `FR-OPS-005`; ADR 0014 | `scripts/release_evidence.py` and CI artifacts | partial |
| foundation validation ledger | PRD release slices; ADR 0014 | `docs/validation/temporal-event-foundation.md` | implemented-main |

## Accepted target trace

| Product slice | Required PRD evidence before promotion | Planned implementation boundary |
|---|---|---|
| Multilingual semantic measurement | `FR-LNG-*`, `FR-SEM-*`; human-gold span/concept/calibration/profile evidence | semantic preprocessing, concept dictionary, language-profile artifacts |
| TRSL-TM CPU reference | `FR-TOP-*`; objective/convergence/posterior/recovery/independent oracle | `topic_measurement` Rust estimator |
| Candidate-K selection | `FR-KSEL-*`; candidate manifests, hard gates, Pareto and blinded review | `model_selection` |
| Topic network and clusters | `FR-NET-*`; valid-coordinate recovery, edge uncertainty, ARI/NMI/stability | `network_analysis` |
| Longitudinal ESEM/DSEM | `FR-PSY-*`; invariance, within/between, irregular time, path recovery | `psychometric_core` |
| GPU and VRAM control | `FR-CMP-*`; real-device parity and 4/6/8/12/24-GB profiles | `compute_backend` |
| TDT and CHRONOS event intelligence | `FR-EVT-002/003/004`; task metrics, hypothesis authority, calibration | `event_intelligence` |
| Evidence-bounded interpretation | `FR-LLM-*`; support, verifier, disagreement, abstention and injection evidence | `interpretation_gateway` |
| Coordinated visual analytics | `FR-EXP-*`; Figma interaction contract, exact-value and accessibility evidence | `visual_analytics` |
| Production API/job service | `FR-API-001/002`; idempotent job lifecycle, backpressure and compatibility evidence | API server and workers |
| Enterprise operations | `FR-SEC-*`, `FR-OPS-*`; deployment, KMS, audit, backup/restore, measured SLO evidence | deployment-owned adapters and runbooks |

## Scientific evidence promotion

Promotion rules are governed by ADR 0014 and `docs/adr/ADR_POLICY.md`.
A PRD requirement or accepted decision may remain `accepted-target`. A capability
becomes `implemented-main` only when its source is integrated on protected main
and the applicable exact-head software, scientific, security/privacy,
supply-chain, migration/recovery, accessibility, and qualifying review gates
pass.

Planning documents, local-only results, simulations that do not exercise the
production path, skipped hardware tests, queued or predecessor-head checks,
status-only bot output, or LLM agreement cannot promote implementation maturity.
An open PR is evidence of an `active-PR` only.

## Claim discipline

- Allen path consistency is not unrestricted global satisfiability.
- Language support requires versioned task/domain validation and applicable
  invariance; architecture alone is insufficient.
- GPU support requires real-device execution and CPU-reference parity.
- Dynamic ESEM/DSEM paths are not causal without an identified design and
  assumptions.
- Topic clusters, event links, TDT tracking, CHRONOS predictions, and LLM
  agreement are model-derived claims rather than observed fact.
- Raw topic proportions are not passed to naïve Pearson correlation or ordinary
  Gaussian ESEM as unconstrained observations.
- LLMs do not establish measurement truth, deterministic authority, merge
  authority, or release authority.
- CSAP/SOC 2/ISO/NIST readiness and repository controls are not certification or
  attestation.
- The `200억 달러` acquisition bar is a prioritization standard, not a valuation
  result.

## PRD v0.5 requirement evidence rule

Each implementation PR that claims a PRD v0.5 requirement shall identify:

1. the exact `FR-*` requirement identifiers;
2. the owning ADR and affected technical documents;
3. RED→GREEN or equivalent pre-fix evidence for changed behavior;
4. exact source, tests, migrations, schemas, and artifacts;
5. scientific acceptance metrics where applicable;
6. current-head CI/security/review evidence;
7. rollback, compatibility, and failure-mode evidence;
8. maturity change requested after protected-main integration.

## Documentation fitness trace

`docs/DOCUMENTATION_ASSESSMENT.md` evaluates the canonical graph and distinguishes
design sufficiency from protected-main sufficiency. `docs/adr/README.md` is the
decision ownership/supersession map; `docs/adr/ADR_POLICY.md` defines decision
status independently from implementation maturity. `docs/product/prd-v0.5.md`
is the current detailed product contract; v0.4 remains historical evidence.

## Documentation rule

When a scientific estimand, clock meaning, relation authority, ontology,
membership structure, language-equivalence claim, topic/backend identity,
compute backend, privacy/authorization model, service authority,
persistence/reproducibility contract, orchestration policy, autonomous
development authority, event-intelligence claim, implementation-lineage
authority, or accepted evidence threshold changes, update the owning ADR and PRD
version where required, this matrix, affected architecture/data/API documents,
exact tests/validation evidence, and standards/research documentation in the same
reviewed change.
