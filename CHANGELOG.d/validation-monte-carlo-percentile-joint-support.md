### Fixed

- Reject `MonteCarloSummary` artifacts whose distinct nearest-rank percentile endpoints individually fit the recorded spread but jointly exceed the sample squared-deviation budget `(n - 1) * SD^2`.
