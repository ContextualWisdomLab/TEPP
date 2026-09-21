### Fixed

- Reject generic `MonteCarloSummary` payloads where exactly two retained replications expose two distinct nearest-rank percentile endpoint values but the recorded mean or sample standard deviation cannot be produced by those two values. With two replications, distinct endpoint values exhaust the retained sample, so looser individual/joint moment-budget checks are not sufficient evidence of a realizable summary.
