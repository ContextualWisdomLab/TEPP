# Event-relation vocabulary persistence (doctoring)

## Scope

`event_relation` already exists on the foundation schema. This slice adds the
fail-closed SQL insert contract that binds `relation_type_code` to
`transition_edge` using the closed ERD vocabulary. Forward state-transition
types (`causes`, `enables`, `intervenes_on`, `leads_to`, `produces`,
`transitions_to`, `input_to`, `process_to`) must set `transition_edge=true`.
Provenance types (`references`, `summarizes`, `revises`, `translates`,
`retrospectively_reports`, `supports`, `contradicts`, `outcome_of`) must set
`transition_edge=false` and may point backward in event time. Unknown types and
transition self-loops fail closed.

A later physical CHECK can encode the same vocabulary in the catalog. This
slice does not add a new migration number so it can land independently of
stacked `0005`/`0006` PRs.

## Authority

### Normative TEPP contract (vocabulary and transition flags)

The closed `relation_type_code` list, `transition_edge` pairing, forward versus
provenance direction rules, and the prohibition on reverse state transitions are
TEPP product/measurement contracts. Normative sources:

- `docs/ERD.md` — Typed event-relation contract (forward state-transition types,
  evidence/provenance types, `transition_edge` validation, temporal-order
  admission before transition-subgraph use).
- `docs/adr/0013-bitemporal-persistence-reproducibility-and-split-authority.md` —
  persistence must preserve relation-aware provenance without collapsing
  temporal/event semantics; relation components fail closed when inconsistent.

### Supporting temporal literature (semantics only)

Allen (1983) and Jensen and Snodgrass (1999) support interval ordering and
bitemporal data-management context. They do **not** define TEPP's closed
relation vocabulary or `transition_edge` mapping.

Allen, J. F. (1983). Maintaining knowledge about temporal intervals.
*Communications of the ACM, 26*(11), 832–843.
https://doi.org/10.1145/182.358434

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE
Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

## Verification

- `crates/persistence_postgres/tests/event_relation_sql_contract.rs` requires
  matching transition flags, rejects unknown types, and rejects transition
  self-loops before SQL is rendered;
- recording-session / unit coverage for `insert_event_relation_sql` and
  `EventRelationRecord::validate`;
- no live PostgreSQL insert of transition/provenance rows is claimed in this
  slice (live CI remains the shared foundation gate; catalog CHECK and live
  relation-row admission are deferred).
