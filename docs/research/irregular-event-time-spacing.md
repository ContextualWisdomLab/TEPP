# Irregular event-time spacing (doctoring)

## Scope

`irregular_time` computes consecutive lags from event/valid time. Equal
system-time sampling cannot stand in for irregular event lags. Recovery is the
computed RMSE of recovered lags against known-truth event lags.

This slice does not fit DSEM, claim a unique lag kernel, or collapse the six
TEPP clocks.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — event/valid time is
  distinct from system/record time.
- `docs/adr/0005-posterior-esem-dsem.md` — longitudinal models must handle
  irregular time rather than assume equally spaced system samples.

### Supporting literature

Asparouhov, Hamaker, and Muthén (2018) formulate DSEM for intensive
longitudinal data whose observation times need not be equally spaced. They do
**not** authorize substituting system-time cadence for event-time lags.

Asparouhov, T., Hamaker, E. L., & Muthén, B. (2018). Dynamic structural
equation models. *Structural Equation Modeling: A Multidisciplinary Journal,
25*(3), 359–388. https://doi.org/10.1080/10705511.2017.1406803
