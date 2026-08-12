# Task 4 Qualitative Interval Algebra Foundations

## Purpose

This doctoring note traces TEPP's executable qualitative interval relation and bounded closure contracts to primary temporal-reasoning literature and the current published OWL-Time vocabulary. References use APA 7th style.

Task 4 is a storage-independent Rust domain slice. It implements Allen's thirteen elementary relations for proper bounded intervals, relation-set inverse and composition, and a resource-bounded path-consistency network with conservative provenance. It does not implement event ontology, transition-policy validation, bitemporal persistence, leakage-safe snapshots, metric temporal constraints, complete scenario search, or probabilistic uncertainty.

## Implemented relation contract

### Proper interval domain

Allen's interval calculus assumes proper intervals whose start precedes their end. TEPP therefore classifies only `TemporalInterval<T>` values that are:

- bounded on both sides;
- nonzero in duration;
- validated by one nominal clock type; and
- represented as `TemporalCertainty::Bounded`.

Exact instants, one-sided intervals, and explicitly unknown intervals fail closed with `RelationRequiresProperBoundedInterval`. Included versus excluded boundary metadata does not alter the endpoint ordering relation; TEPP's qualitative relation is defined over ordered endpoints rather than set-theoretic overlap at a boundary instant.

### Thirteen elementary relations

The executable enum contains the thirteen jointly exhaustive endpoint-order cases:

```text
before       after
meets        met_by
overlaps     overlapped_by
starts       started_by
during       contains
finishes     finished_by
equals
```

Every relation has an exact inverse. `equals` is self-inverse. The stable enum order is also the private bit index used by `RelationSet`; callers cannot construct relation bits outside the thirteen reviewed values.

### Composition table

For relations \(R_{xy}\) and \(R_{yz}\), composition returns every elementary relation \(R_{xz}\) admitted by at least one compatible triple of proper intervals:

```math
R_{xy} \circ R_{yz}
=
\{R_{xz}: \exists x,y,z\; R_{xy}(x,y) \land R_{yz}(y,z)\}.
```

The production table is generated once from all proper intervals over six ordered endpoint ranks. Six ranks are sufficient because three proper intervals contain at most six distinct endpoints; every equality and strict-order pattern can be order-preservingly mapped into those ranks. The result is deterministic and immutable after `OnceLock` initialization.

The implementation is independently checked against:

- classification examples for all thirteen elementary relations;
- known composition entries;
- the converse law
  ```math
  (R \circ S)^{-1}=S^{-1}\circ R^{-1};
  ```
- nonempty composition for every elementary pair; and
- a separate exhaustive oracle built from all proper intervals over eight endpoint ranks.

The eight-rank oracle is deliberately not the production table generator. It is a larger independent enumeration used to detect truncation or table-construction errors.

## Bounded path-consistency contract

For every distinct variable triple \((i,k,j)\), closure applies the monotone narrowing rule

```math
R_{ij}
\leftarrow
R_{ij}\cap(R_{ik}\circ R_{kj}).
```

The reverse cell is updated with the exact inverse relation set. Iteration continues until no relation set changes, a pair becomes empty, or the configured propagation budget is exhausted.

### What successful closure means

A successful `close()` result establishes a path-consistent local network under the implemented relation algebra. It does **not** certify global satisfiability for unrestricted disjunctive Allen networks. General interval-algebra consequence and satisfiability problems are computationally intractable, and local consistency is not a complete decision procedure for the full algebra (Vilain & Kautz, 1986).

Accordingly:

- an empty narrowed relation set is sound contradiction evidence;
- a nonempty path-consistent network is not described as a proof that a concrete global interval assignment exists;
- complete scenario search and tractable-fragment detection remain separate future work; and
- user-facing APIs and documentation must preserve this claim boundary.

### Resource and failure boundaries

Every reasoner instance has explicit nonzero maxima for:

- interval variables;
- accepted direct constraints; and
- propagation checks.

Closure snapshots the relation matrix before propagation. Contradiction or budget exhaustion restores the pre-closure matrix, so a failed closure cannot expose a partially narrowed network. Constraint capacity is consumed only by accepted assertions.

Opaque variable and constraint identifiers include a reasoner-instance UUIDv7. Identifiers from another reasoner fail closed even when their numeric indices happen to match.

### Provenance contract

Each ordered relation cell records:

- the current possible relation set;
- whether at least one direct assertion was accepted for that pair; and
- a conservative ordered set of accepted assertion identifiers supporting its current narrowing.

Derived support is the union of the current pair support and the two composing path supports. This is intentionally conservative: it preserves sufficient accepted evidence for audit and contradiction reporting but does not claim a minimal proof core.

Direct contradictions report the rejected attempted relation set separately and do not fabricate an accepted `ConstraintId`. Propagation contradictions report the conservative accepted-assertion support that led to the empty pair.

## OWL-Time interoperability boundary

OWL-Time publishes interval relation vocabulary aligned with Allen-style temporal topology, including before, after, meets, met by, overlaps, overlapped by, starts, started by, during, contains, finishes, finished by, and equals. TEPP can map its reviewed relations to that outward vocabulary in a later JSON-LD or RDF adapter.

OWL-Time does not define TEPP's in-memory bit layout, resource limits, provenance semantics, atomic rollback, or Rust API. Those are TEPP engineering decisions. The latest published OWL-Time document is a W3C Candidate Recommendation Draft, so its status must be recorded accurately rather than described as a final Recommendation.

## Security and operational behavior

Adversarial relation graphs can cause high propagation cost or intentionally inconsistent cycles. Task 4 therefore:

- rejects zero resource limits;
- rejects empty direct relation sets;
- scopes identifiers to one reasoner instance;
- bounds variables, constraints, and propagation work;
- restores state after failed closure;
- returns content-redacting stable errors;
- avoids recursion in closure; and
- keeps the production relation set closed over thirteen reviewed values.

The implementation does not deserialize arbitrary relation bit masks and does not accept user-defined relation semantics.

## Verification mapping

| Evidence | Representative verification |
|---|---|
| relation partition | concrete proper intervals classify all thirteen relations |
| inverse | every inverse pair round-trips exactly |
| composition | known entries, all-pair converse law, and eight-rank exhaustive oracle |
| proper-interval boundary | exact, open-ended, and unknown intervals fail closed |
| inverse propagation | accepted and derived reverse cells are exact inverses |
| path consistency | multi-edge networks narrow until stable |
| idempotence | a second closure on a stable network performs no revision |
| contradiction | impossible cycles and direct conflicts return conservative support |
| atomicity | propagation-budget failure restores the pre-closure network |
| provenance | observed and derived relations remain distinguishable |
| identity isolation | foreign variables fail closed despite matching indices |
| resource limits | variable, constraint, and propagation maxima are enforced |

Production line and branch coverage remain merge gates, but coverage is not treated as proof of global satisfiability or completeness.

## Deferred research boundary

Later slices must separately doctor and implement, where required:

- tractable Allen subalgebra recognition;
- complete backtracking scenario search;
- minimal contradiction cores;
- metric temporal constraints and continuous durations;
- event-transition edge policies;
- persistence and historical snapshots;
- uncertainty-weighted or probabilistic relation evidence; and
- performance benchmarks for large sparse networks.

## References

Allen, J. F. (1983). Maintaining knowledge about temporal intervals. *Communications of the ACM, 26*(11), 832–843. https://doi.org/10.1145/182.358434

Cox, S., & Little, C. (Eds.). (2022). *Time ontology in OWL* (W3C Candidate Recommendation Draft). World Wide Web Consortium. https://www.w3.org/TR/owl-time/

Dechter, R., Meiri, I., & Pearl, J. (1991). Temporal constraint networks. *Artificial Intelligence, 49*(1–3), 61–95. https://doi.org/10.1016/0004-3702(91)90006-6

Vilain, M. B., & Kautz, H. A. (1986). Constraint propagation algorithms for temporal reasoning. In *Proceedings of the Fifth National Conference on Artificial Intelligence* (pp. 377–382). American Association for Artificial Intelligence. https://www.aaai.org/Papers/AAAI/1986/AAAI86-063.pdf
