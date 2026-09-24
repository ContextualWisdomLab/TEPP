# Topic-lineage diagonal Laplace curvature consistency

- The CPU reference estimator now derives retained per-document diagonal-Laplace variances from the same generalized-Gauss-Newton relation Jacobian blocks used by `JointCoordinatePrecision`, rather than treating every incident transition as a full `relation_strength` contribution.
- `document_coordinate_variances[d][j]` is the reciprocal of the corresponding owner GGN precision diagonal. It remains a diagonal approximation and is not the marginal variance from the inverse full precision matrix.
- Objective, convergence, deterministic seeds, fitted topic basis, Evidence provenance, and joint-covariance semantics are unchanged. This closes #669's source-level arithmetic inconsistency; backend parity and calibration/recovery remain required before interval-quality claims.
