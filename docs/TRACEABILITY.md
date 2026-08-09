# TEPP Requirements, Research, and Evidence Traceability

**Status:** Accepted cross-cutting traceability baseline  
**Last reviewed:** 2026-08-09

The full APA 7th standards/literature register remains `docs/research/standards-and-literature.md`. This matrix links durable requirements to implementation/evidence maturity without duplicating the bibliography.

| Requirement / decision | Canonical basis | Source/evidence boundary | Maturity |
|---|---|---|---|
| immutable source evidence and exact spans | PRD; Architecture; ADR-0001 | `evidence_core`, Task 2 tests/doctoring | implemented-main |
| Rust workspace/quality foundation | ADR-0001 | workspace/CI/repository contract | implemented-main |
| six distinct clocks and uncertain intervals | PRD; ADR-0002 | PR #5 `temporal_core` + tests/doctoring | active-PR |
| Allen relation algebra/bounded closure | temporal architecture/research | PR #6 `temporal_core` + tests/doctoring | active-PR |
| forward-only transition subgraph | PRD; ADR-0002/0003 | future `relation_graph` validation | accepted-target |
| event ontology/evidence mentions | PRD; ADR-0003 | future `event_core` | accepted-target |
| time-varying cross-classified multiple membership | PRD; ADR-0003 | future `membership_core` | accepted-target |
| leakage-safe availability/cutoff snapshots | PRD; ADR-0002 | future `corpus_split` | accepted-target |
| PostgreSQL bitemporal/lineage persistence | Architecture/ERD | future `persistence_postgres` migrations | accepted-target |
| multilingual shared latent semantic space | PRD; ADR-0004 | future semantic/concept/topic crates | accepted-target |
| temporal/relational topic posterior | PRD; ADR-0004/0005 | future `topic_measurement` | accepted-target |
| CPU f64 reference + bounded multithreading | ADR-0001/0006 | future compute implementation | accepted-target |
| GPU/VRAM streaming + CPU parity | ADR-0006 | future `compute_backend` | accepted-target |
| candidate K statistical + blinded LLM review | PRD/research | future `model_selection` | accepted-target |
| compositional topic correlation / stable clustering | PRD/research | future `network_analysis` | accepted-target |
| posterior ESEM / longitudinal invariance / DSEM | ADR-0005 | future `psychometric_core` | accepted-target |
| TDT/CHRONOS event intelligence | PRD/research | future `event_intelligence` | accepted-target |
| evidence-bounded LLM interpretation | PRD; ADR-0006 | future `interpretation_gateway` | accepted-target |
| accessible bitemporal/network/drift/invariance views | PRD | future `visual_analytics` | accepted-target |
| 100% production line/branch/docs | AGENTS/quality architecture | CI/repository contracts | implemented-main and required for all future source |

## Scientific evidence promotion

A target becomes `implemented-main` only when its source is integrated on protected main and the relevant current-head tests, recovery/validation evidence, security/supply-chain checks, and independent review pass. Planning documents, simulations that do not exercise production code, queued checks, predecessor-head results, or LLM judgments cannot promote implementation maturity.

## Claim discipline

- PR #6 path consistency must not be promoted into global satisfiability proof.
- Language support is not promoted from architecture alone; each profile needs validation/invariance evidence.
- GPU support is not promoted if required GPU tests are skipped or only a software fallback ran when hardware parity is claimed.
- Dynamic ESEM/DSEM effects are not promoted to causal claims without an identified design and corresponding evidence.
- A `200억 달러` acquisition bar is a prioritization heuristic, not a valuation result.

## Documentation rule

When a scientific estimand, time meaning, ontology relation, membership structure, compute backend, persistence contract, or accepted evidence threshold changes, update the corresponding PRD version/ADR plus this matrix and the exact tests/doctoring in the same reviewed change.