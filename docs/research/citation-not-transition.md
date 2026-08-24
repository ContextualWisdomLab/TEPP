# Citation edges are not state transitions (doctoring)

## Scope

`citation_edge` keeps citation, translation, revision, and retrospective
report edges out of the forward state-transition vocabulary. Recovery is
the computed share of recovered kinds that match known truth.

This slice does not persist the graph, implement Allen composition, or
replace `relation_graph`.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — forward
  state-transition and input-process-outcome edges never move backward
  in event time; citation and retrospective edges may point to the past
  but never become reverse state transitions.
- `docs/adr/0003-relational-event-multiple-membership.md` — typed
  relations distinguish transition from provenance.

### Supporting literature

Allen (1983) classifies interval relations; it does **not** authorize
treating a bibliographic citation as a `causes` or `intervenes_on`
transition.

Allen, J. F. (1983). Maintaining knowledge about temporal intervals.
*Communications of the ACM, 26*(11), 832–843.
https://doi.org/10.1145/182.358434
