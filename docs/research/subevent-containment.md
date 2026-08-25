# Subevent parent-window containment (doctoring)

## Scope

`subevent_containment` requires a half-open subevent interval to lie inside
its parent event-time interval. Recovery is the computed share of
containment flags that match known truth.

This slice does not persist subevents, promote mentions to instances, or
implement Allen composition.

## Authority

### Normative TEPP contract

- `docs/adr/0003-relational-event-multiple-membership.md` — event instances
  own subevents, roles, and provenance; mentions remain fallible evidence.
- `docs/adr/0002-six-clock-temporal-semantics.md` — event/valid time is the
  clock for occurrence intervals.

### Supporting literature

Allen (1983) includes the `during` relation. Containment here is the
half-open special case used to refuse escaped subevents; the crate does
not implement the full Allen table.

Allen, J. F. (1983). Maintaining knowledge about temporal intervals.
*Communications of the ACM, 26*(11), 832–843.
https://doi.org/10.1145/182.358434
