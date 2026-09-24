### Topic measurement

- Added batch document marginal covariance solving for a retained joint ALR precision. Requested document blocks now share one Cholesky factorization while preserving requested identity order, EventTime, topic order, symmetry, and positive-definiteness checks; empty, duplicate, and missing document requests fail closed. The fit-bound wrapper derives the fitted topic basis and joint precision once for the batch, while the scalar APIs delegate to the same owner arithmetic.
