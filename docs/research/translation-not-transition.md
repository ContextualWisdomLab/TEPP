# Translation edges are not state transitions (doctoring)

## Scope

`translation_edge` keeps translation, same-language copy, and revision
edges out of the forward state-transition vocabulary. A shared RFC 5646
primary language subtag cannot be classified as a translation. Recovery
is the computed share of recovered kinds that match known truth.

This slice does not persist the graph, implement Allen composition,
allocate migration `0008`, or replace `relation_graph`.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — forward
  state-transition and input-process-outcome edges never move backward
  in event time; citation, revision, translation, and retrospective
  edges may point to the past but never become reverse state
  transitions.
- `docs/adr/0003-relational-event-multiple-membership.md` — typed
  relations distinguish transition from provenance; translation, copy,
  and revision remain provenance.

### Supporting literature

Allen (1983) classifies interval relations; it does **not** authorize
treating a translation as `causes` or `transitions_to`. RFC 5646
primary subtags separate translation from same-language copy or
revision.

Allen, J. F. (1983). Maintaining knowledge about temporal intervals.
*Communications of the ACM, 26*(11), 832–843.
https://doi.org/10.1145/182.358434

Phillips, A., & Davis, M. (2009). *Tags for identifying languages*
(RFC 5646). Internet Engineering Task Force.
https://doi.org/10.17487/RFC5646
