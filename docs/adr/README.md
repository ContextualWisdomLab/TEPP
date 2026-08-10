# Architecture Decision Records

TEPP uses numbered ADRs for decisions that constrain latent-variable meaning, temporal semantics, event ontology, multilingual measurement, numerical backends, privacy/security, orchestration, and modular service boundaries.

| ADR | Decision |
|---|---|
| 0001 | Rust-first modular MSA and CPU `f64` numerical reference |
| 0002 | Six-clock temporal semantics and fail-closed historical leakage prevention |
| 0003 | Relational event ontology and time-varying cross-classified multiple membership |
| 0004 | One shared multilingual latent space with explicit invariance status |
| 0005 | Posterior-aware ESEM/DSEM and valid compositional coordinates |
| 0006 | VRAM-adaptive GPU compute and NVIDIA NIM/OpenCode orchestration boundary |
| 0007 | Explicit Rust workspace, pinned toolchains, and exact quality gates |
| 0008 | Immutable evidence identities, `SHA-256` digests, and exact UTF-8/page spans |
| 0009 | Purpose-bound PII governance without blanket masking |
| 0010 | Adaptive LLM orchestration and test-time compute |
| 0011 | Standalone operation and modular CWL MSA boundary |

ADR status changes require a pull request, source traceability, tests for affected invariants, and corresponding PRD/architecture updates where the approved measurement target changes. Decisions that materially alter privacy authority, orchestration authority, service ownership, temporal semantics, ontology, or estimands require a superseding ADR rather than silent drift.
