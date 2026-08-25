# Knowledge-cutoff clock identity (doctoring)

## Scope

`cutoff_clock` keeps knowledge cutoff distinct from event time, system
time, and availability time. Recovery is the computed share of cutoff
stamps that match known truth.

This slice does not persist clocks, replace `temporal_core`, or recreate
`available_clock`, `document_clocks`, or `membership_cutoff`.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — knowledge cutoff is
  the latest availability time permitted in one historical analysis. It
  is not event time, system time, or availability time.
- Historical analyses may not treat the moment an event occurred, the
  moment TEPP recorded a row, or the moment evidence became available as
  the analysis cutoff.

### Supporting literature

Snodgrass (2000) separates valid time from transaction time. Jensen and
Snodgrass (1999) treat those clocks as independently governed. TEPP's
knowledge cutoff is a third analysis-bound clock: the latest
availability an estimator may use. Availability of one document is not
the cutoff of the run.

Tashman (2000) reviews out-of-sample evaluation that must freeze the
information set. A cutoff that collapses onto event time, record time,
or a single document's availability fabricates a later information set.

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management.
*IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

Snodgrass, R. T. (2000). *Developing time-oriented database applications
in SQL*. Morgan Kaufmann.

Tashman, L. J. (2000). Out-of-sample tests of forecasting accuracy: An
analysis and review. *International Journal of Forecasting, 16*(4),
437–450. https://doi.org/10.1016/S0169-2070(00)00065-0
