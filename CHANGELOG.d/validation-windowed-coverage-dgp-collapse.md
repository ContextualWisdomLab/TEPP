### Validation

- Added `summarize_windowed_coverage_replications` so rolling-origin interval coverage is collapsed within each independently generated DGP replication before Monte Carlo uncertainty is estimated. The owner rejects singleton DGP designs, empty window sets, non-finite/out-of-range coverage, and invalid percentile configuration; repeated document/coordinate/window intervals are not promoted to an IID Monte Carlo denominator.
