# Interval-aware historical eligibility (doctoring)

## Scope

This note doctors the `temporal_core` historical-eligibility contract:

1. evidence may enter a historical analysis only when its governed availability interval is fully at or before the knowledge cutoff;
2. unknown availability and open-ended upper bounds fail closed because they can extend past the cutoff;
3. event time and document time cannot be substituted for availability.

Point-instant `available_time <= knowledge_cutoff` remains the exact special case. This crate owns the interval decision; persistence adapters and corpus snapshots continue to apply the same inequality to stored instants. The change allocates no database migration.

## Authoritative sources

Tashman, L. J. (2000). Out-of-sample tests of forecasting accuracy: An analysis and review. *International Journal of Forecasting, 16*(4), 437–450. https://doi.org/10.1016/S0169-2070(00)00065-0

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

Allen, J. F. (1983). Maintaining knowledge about temporal intervals. *Communications of the ACM, 26*(11), 832–843. https://doi.org/10.1145/182.358434

## Application

Tashman (2000) requires that evaluation origins use only information available at the origin. Jensen and Snodgrass (1999) separate valid time from transaction/availability time so a later report about an earlier event cannot leak into an earlier analysis. When availability is an interval rather than a point, Allen (1983) interval bounds are the representation: if any possible availability instant is after the cutoff, the evidence is not fully eligible.

TEPP therefore computes the latest representable availability instant and admits the interval only when that instant is `<= knowledge_cutoff`. An unknown interval or an unbounded upper bound has no such instant and fails closed.

## Verification

- exact availability on or before the cutoff is eligible; one second later is not;
- a closed interval whose included upper bound is after the cutoff is ineligible;
- an exclusive upper bound one nanosecond after the cutoff remains eligible because the latest representable instant is the cutoff;
- unknown and open-ended-upper availability return `UncertainAvailability`;
- the gate agrees with an independently computed latest-instant comparison on a fixture suite.
