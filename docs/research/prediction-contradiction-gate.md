# Predicted-versus-observed temporal contradiction (doctoring)

## Scope

`prediction_contradiction` compares half-open event-time intervals. A
predicted interval that is disjoint from later-observed evidence cannot be
promoted to fact. Recovery is the computed share of contradiction flags that
match known truth.

This slice does not run the `temporal_core` path-consistency reasoner, fit
CHRONOS schemas, or claim a unique interval algebra.

## Authority

### Normative TEPP contract

- `docs/adr/0016-tdt-chronos-event-intelligence-boundary.md` — predictions
  remain hypothetical until supported by later evidence; temporal
  contradiction can reject a proposed promotion.
- `docs/adr/0002-six-clock-temporal-semantics.md` — event/valid time is
  the clock for occurrence intervals.

### Supporting literature

Allen (1983) defines thirteen interval relations, including disjoint
`before`/`after`. Disjointness is sufficient to refuse promotion; this
crate does not implement the full composition table.

Allen, J. F. (1983). Maintaining knowledge about temporal intervals.
*Communications of the ACM, 26*(11), 832–843.
https://doi.org/10.1145/182.358434
