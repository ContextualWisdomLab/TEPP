# Support edges are not state transitions (doctoring)

## Scope

`support_edge` keeps support, contradiction, summary, and `outcome_of`
edges out of the forward state-transition vocabulary. Recovery is the
computed share of recovered kinds that match known truth.

This slice does not persist the graph, implement Allen composition, or
replace `relation_graph`.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — forward
  state-transition and input-process-outcome edges never move backward
  in event time; citation, revision, translation, and retrospective
  edges may point to the past but never become reverse state
  transitions. Support, contradiction, summary, and `outcome_of` follow
  the same provenance rule.
- `docs/adr/0003-relational-event-multiple-membership.md` — typed
  relations distinguish transition from provenance; not every relation
  is a transition or causal edge.

### Supporting literature

Allen (1983) classifies interval relations; it does **not** authorize
treating supportive, contradicting, summarizing, or inverse-production
edges as `causes` or `transitions_to`.

Allen, J. F. (1983). Maintaining knowledge about temporal intervals.
*Communications of the ACM, 26*(11), 832–843.
https://doi.org/10.1145/182.358434
