# Irregular event-time analysis-run composition

**Active slice:** ADR 0040 / `irregular_event_time_v1`
**Protected-main status:** not implemented-main

`psychometric_core` already recovers Voelkle et al. (2012) local log-rates on
event time. This slice binds that recovery to a cutoff-safe analysis-run
profile so an operator can request a digest-bound terminal result.

The executor maps discrete lags through `a` and refuses pooled coefficients
across unequal intervals. It is not DSEM, not a Driver p.16 `std`-family
restore, and not GAP-003A scientific-acceptance wiring.

Exact-head Checks and two independent approvals are required before any
implemented-main claim.
