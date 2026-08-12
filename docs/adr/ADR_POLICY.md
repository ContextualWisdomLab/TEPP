# TEPP ADR Policy

This file defines how architecture decisions are recorded and interpreted. It prevents an accepted design decision from being confused with implemented or released behavior.

## Two independent status axes

Every current numbered ADR and the ADR index carry two independent statuses.

### Decision status

- **Proposed** — under architectural review; not yet authoritative.
- **Accepted** — the decision is authoritative for future implementation unless superseded.
- **Superseded** — replaced wholly or partly by a later ADR; the superseding scope must be explicit.
- **Rejected** — considered and deliberately not adopted.

### Implementation maturity

- **implemented-main** — source is integrated on protected `main` and the relevant exact-head validation, security/supply-chain evidence, and review gates passed.
- **active-PR** — implementation exists only on an open PR.
- **partial** — only an explicitly identified subset is implemented on protected `main`.
- **accepted-target** — accepted architecture without an integrated implementation.
- **research-only** — evaluated direction that has not been accepted as production architecture.
- **out-of-scope** — deliberately excluded from TEPP ownership.

**Accepted never means shipped.** Implementation maturity is the only axis that describes as-built state.

## Required ADR structure

A decision is considered clear only when the ADR records, as applicable:

1. context and decision drivers;
2. precise owned decision and non-goals;
3. alternatives considered and why they were rejected;
4. consequences and operational trade-offs;
5. failure and recovery behavior;
6. security, privacy, scientific-integrity, and governance impact;
7. compatibility, migration, and standalone/MSA implications;
8. verification and falsifiable acceptance evidence;
9. rollback and supersession conditions; and
10. links to the canonical PRD/TRD/Architecture/ERD/UML/Traceability/research authority.

A later ADR may partially supersede an earlier one. In that case both the index and the affected ADR must state exactly which decision clauses moved and which remain authoritative.

## Decision ownership map

TEPP deliberately separates adjacent authorities:

- Rust/numerical implementation authority is not service-integration authority.
- GPU/VRAM scheduling is not LLM orchestration policy.
- event ontology/storage semantics are not TDT/CHRONOS event-intelligence semantics.
- multilingual semantic alignment is not the full topic-estimator/backend contract.
- immutable evidence identity is not persistence/reproducibility-manifest authority.
- scientific claim promotion is not ordinary CI success.
- an LLM development agent is not review, merge, statistical, or release authority.

The ADR index is the canonical map for these boundaries.

## Machine-checkable ADR contract

`scripts/validate_documentation.py` verifies that:

- every numbered ADR file appears exactly once in the ADR index and vice versa;
- every current ADR declares a valid `Decision status` and `Implementation maturity`;
- every ADR states explicit supersession/supersedes metadata;
- core Context, Decision, Alternatives, Consequences, Verification, and Rollback sections remain present; and
- the root documentation map links both this policy and the ADR index.

This turns ADR clarity into a repository gate rather than a prose convention.

## Change rule

A change to a latent-variable estimand, temporal meaning, event/role semantics, multilingual invariance, topic-estimator identity, compute authority, PII authority, persistence/reproducibility model, LLM orchestration policy, cross-service ownership, scientific claim-promotion rule, autonomous-development authority, or event-intelligence claim requires an ADR update or a new superseding ADR. Material changes to the approved measurement target also require a PRD version change.
