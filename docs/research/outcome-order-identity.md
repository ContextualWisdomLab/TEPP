# Input-process-outcome edges keep event-time order (doctoring)

## Scope

`outcome_order` keeps `input_to` and `process_to` forward in event-time
rank and keeps `outcome_of` out of the transition vocabulary. Recovery
is the computed share of recovered kinds that match known truth.

This slice does not persist the graph, allocate migration `0008`, or
replace `relation_graph`, `citation_edge`, `translation_edge`, or
`retrospective_edge`. Event-time ranks are opaque ordinals, not clock
identities.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — forward
  state-transition and input-process-outcome edges never move backward
  in event time; provenance edges may point to the past but never
  become reverse state transitions.
- `docs/adr/0003-relational-event-multiple-membership.md` — typed
  relations distinguish transition from provenance.

### Supporting literature

Allen (1983) classifies interval relations; it does **not** authorize
a later outcome to precede its input, or `outcome_of` to become
`input_to`.

Allen, J. F. (1983). Maintaining knowledge about temporal intervals.
*Communications of the ACM, 26*(11), 832–843.
https://doi.org/10.1145/182.358434
