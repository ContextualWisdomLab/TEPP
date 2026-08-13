# Event-relation vocabulary persistence (doctoring)

## Scope

`event_relation` already exists on the foundation schema. This slice adds the
fail-closed SQL insert contract that binds `relation_type_code` to
`transition_edge` using the closed ERD vocabulary. Forward state-transition
types (`causes`, `enables`, `intervenes_on`, `leads_to`, `produces`,
`transitions_to`, `input_to`, `process_to`) must set `transition_edge=true`.
Provenance types (`references`, `summarizes`, `revises`, `translates`,
`retrospectively_reports`, `supports`, `contradicts`, `outcome_of`) must set
`transition_edge=false` and may point backward (Allen, 1983; Jensen &
Snodgrass, 1999). Unknown types and transition self-loops fail closed.

A later physical CHECK can encode the same vocabulary in the catalog. This
slice does not add a new migration number so it can land independently of
stacked `0005`/`0006` PRs.

## Authority

Allen, J. F. (1983). Maintaining knowledge about temporal intervals.
*Communications of the ACM, 26*(11), 832–843.
https://doi.org/10.1145/182.358434

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE
Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

## Verification

- contract tests require matching flags, reject unknown types, and reject
  transition self-loops;
- recording-session coverage for insert SQL;
- live PostgreSQL CI inserts one transition and one provenance row when
  `TEPP_LIVE_POSTGRES=1`.
