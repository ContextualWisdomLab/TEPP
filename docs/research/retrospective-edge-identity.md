# Retrospective reporting is not a transition or a translation (doctoring)

## Scope

`retrospective_edge` keeps later reports about earlier events out of the
forward state-transition vocabulary and out of the translation
vocabulary. Recovery is the computed share of recovered reporting kinds
that match known truth.

This slice does not persist the graph, allocate migration `0008`, or
replace `relation_graph`, `citation_edge`, or `translation_edge`.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — citation, revision,
  translation, and retrospective-reporting edges may point to the past
  but never become reverse state transitions.
- `docs/adr/0003-relational-event-multiple-membership.md` — typed
  relations distinguish transition from provenance.

### Supporting literature

Allen (1983) classifies interval relations; it does **not** authorize
treating a later report as a `causes` edge or as a translation of the
earlier event.

Allen, J. F. (1983). Maintaining knowledge about temporal intervals.
*Communications of the ACM, 26*(11), 832–843.
https://doi.org/10.1145/182.358434
