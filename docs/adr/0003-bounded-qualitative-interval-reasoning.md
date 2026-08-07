# ADR 0003: Bounded qualitative interval reasoning

- **Status:** Accepted
- **Date:** 2026-08-07
- **Decision owners:** TEPP maintainers
- **Supersedes:** None
- **Related:** ADR 0002 six-clock temporal semantics

## Context

TEPP must express partial temporal order without forcing uncertain events into a single total timeline. Task 3 established six nominal clocks and validated exact, bounded, open-ended, and unknown intervals. The next inward dependency is a qualitative relation layer that can describe and narrow relations among proper bounded intervals before event ontology, transition policies, persistence, or longitudinal modeling are added.

A production design must also resist adversarial relation graphs, preserve accepted evidence for audit, and avoid overstating what local constraint propagation proves.

## Decision

TEPP will implement Allen's thirteen elementary proper-interval relations in Rust as a closed enum and a private thirteen-bit `RelationSet`.

The relation layer will:

- classify only validated, two-sided, nonzero bounded intervals;
- reject exact, open-ended, and explicitly unknown intervals;
- preserve all exact inverse pairs;
- compute complete relation-set composition from endpoint orderings;
- validate composition independently with exhaustive endpoint and converse-law tests; and
- reserve OWL-Time mapping for an outward adapter.

TEPP will provide a resource-bounded path-consistency reasoner using

\[
R_{ij}\leftarrow R_{ij}\cap(R_{ik}\circ R_{kj}).
\]

The reasoner will:

- maintain exact inverse cells;
- distinguish direct observations from derived narrowing;
- retain conservative accepted-assertion provenance;
- scope opaque identifiers to one reasoner instance;
- enforce explicit variable, constraint, and propagation limits;
- return contradiction evidence when a relation set becomes empty; and
- restore the pre-closure matrix after contradiction or budget exhaustion.

Successful path consistency will be documented as local consistency only. It will not be presented as a complete satisfiability proof for unrestricted disjunctive Allen networks.

## Rationale

Allen's algebra gives TEPP a standard qualitative endpoint vocabulary that can represent partial order, overlap, containment, and shared boundaries without inventing a project-specific relation system. A compact closed bitset makes inverse, intersection, union, and composition deterministic and allocation-free.

Path consistency provides useful monotone narrowing and early contradiction detection. Explicit limits and atomic rollback are necessary because unrestricted interval reasoning can be computationally intractable and because partially applied closure would be unsafe for downstream audit or decision logic.

Conservative provenance supports explainability without claiming that Task 4 already computes minimal proof cores.

## Consequences

### Positive

- Event and graph layers can depend on one reviewed temporal relation contract.
- All thirteen relations and their inverses have stable Rust identities.
- Contradictory cycles can fail closed with accepted-assertion evidence.
- Resource exhaustion cannot leave a partially narrowed public state.
- OWL-Time and JSON-LD adapters can be added without defining core semantics.

### Negative

- Successful closure does not prove global satisfiability.
- Conservative support may include assertions that are not part of a minimal contradiction core.
- The dense relation matrix has quadratic memory growth in the number of variables.
- Complete scenario search, metric time, and tractable-fragment detection remain future work.

### Neutral

- Boundary inclusion metadata does not change the Allen endpoint relation.
- Exact instants remain outside the proper-interval classifier.
- The reasoner is monotone: constraints may be added and relations narrowed, but accepted assertions are not removed in Task 4.

## Rejected alternatives

### Treat all temporal values as instants

Rejected because intervals, overlap, containment, and uncertain duration are first-class TEPP requirements.

### Encode relations as free-form strings or public bit masks

Rejected because misspellings, unknown bits, and unstable semantics would cross the domain boundary.

### Run unbounded closure

Rejected because hostile or unexpectedly large networks could monopolize compute and leave callers without deterministic failure behavior.

### Claim path consistency as a complete solver

Rejected because the full disjunctive interval algebra is not decided completely by local path consistency.

### Persist relation graphs in Task 4

Rejected because storage schema, bitemporal history, migration policy, and leakage-safe snapshot semantics require separate ADRs and tests.

## Verification obligations

Any change to this decision must preserve or deliberately version:

- all thirteen elementary classifications;
- inverse involution;
- composition correctness against an independent oracle;
- local closure idempotence;
- atomic rollback;
- identifier isolation;
- conservative provenance;
- stable redacting errors; and
- 100% production line and branch coverage.

## References

Allen, J. F. (1983). Maintaining knowledge about temporal intervals. *Communications of the ACM, 26*(11), 832–843. https://doi.org/10.1145/182.358434

Cox, S., & Little, C. (Eds.). (2022). *Time ontology in OWL* (W3C Candidate Recommendation Draft). World Wide Web Consortium. https://www.w3.org/TR/owl-time/

Dechter, R., Meiri, I., & Pearl, J. (1991). Temporal constraint networks. *Artificial Intelligence, 49*(1–3), 61–95. https://doi.org/10.1016/0004-3702(91)90006-6

Vilain, M. B., & Kautz, H. A. (1986). Constraint propagation algorithms for temporal reasoning. In *Proceedings of the Fifth National Conference on Artificial Intelligence* (pp. 377–382). American Association for Artificial Intelligence. https://www.aaai.org/Papers/AAAI/1986/AAAI86-063.pdf
