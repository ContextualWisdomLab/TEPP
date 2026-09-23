# Fit-bound document marginal covariance from joint precision

Issue: #704  
Owner: `topic_measurement::JointCoordinatePrecision`  
Consumer frontier: #703 → #680

## Scientific question

TEPP's fitted document-coordinate summary stores diagonal Laplace quantities of the form `1 / diag(P)`. Those values are useful local curvature diagnostics, but they are not the marginal covariance block of the relation-coupled joint Gaussian approximation. When observed transition structure contributes off-diagonal precision, the marginal covariance for document `d` is the `d` principal block of `P^-1`, not the reciprocal diagonal of `P` and not the inverse of the isolated within-document precision block.

For one requested document, the bounded CPU reference path therefore solves

`P X = E_d`

where the columns of `E_d` are the unit vectors for that document's `K-1` ALR coordinates. The returned rows at the same coordinates form the document marginal covariance block. TEPP factors `P = L Lᵀ` once for the request and performs forward/back substitutions for only those `K-1` right-hand sides; it does not materialize the complete dense inverse.

## Numerical contract

`JointCoordinatePrecision::document_marginal_covariance(document_id)`:

- requires a document identity retained by the fit-bound precision;
- preserves the retained topic order: ALR numerator topics followed by the reference topic;
- uses the full relation-coupled precision, so candidate covariance cannot be manufactured from `ReferenceTopicModel::document_coordinate_variances`;
- rejects inconsistent retained geometry before factorization;
- rejects non-finite factorization or triangular-solve states;
- reconciles only symmetry differences within a documented `1e-10 × (1 + scale)` floating-point tolerance, then requires the returned principal covariance block to remain positive-definite;
- is bounded by `MAX_JOINT_COORDINATES = 4096`. With joint dimension `N` and requested document coordinate count `C = K-1`, the current dense reference path is `O(N^3 + C N^2)` time and `O(N^2)` memory.

The contract test uses a relation-coupled fitted precision and requires the full-inverse marginal to differ from both the reciprocal precision diagonal and the inverse of the isolated one-coordinate document block. Internal edge tests cover missing identity, retained-dimension mismatch, non-finite triangular solve, symmetry roundoff, material asymmetry, and non-positive-definite output.

## Statistical interpretation

This is a deterministic Gaussian/Laplace validation primitive. A positive-definite observed-information or generalized-Gauss-Newton precision defines the covariance of the corresponding local Gaussian approximation by inversion. Off-diagonal precision terms encode dependence, so marginal uncertainty for a coordinate or subvector generally requires the relevant block of the full inverse rather than reciprocal diagonal curvature.

Tierney and Kadane (1986) describe Laplace approximations for posterior moments, variances, and marginal densities from local likelihood geometry. Rue, Martino, and Chopin (2009) treat latent Gaussian models in precision form and emphasize computation of posterior marginals and subvector marginals from the joint Gaussian structure. TEPP uses the same linear-algebra distinction while keeping a narrower claim: the returned matrix is the marginal covariance of TEPP's retained local Gaussian approximation, not proof that the approximation is empirically calibrated.

## Claim boundary

The #704 solve does **not** establish empirical interval coverage, calibrated posterior probability, Evidence source/vocabulary provenance, source/event-time authenticity, Membership provenance, relation activation authority, release topic identity, or CPU/MLX/CUDA/OpenCL parity. #703 may re-express this covariance into the truth-reference ALR basis as `A Σ Aᵀ`; #680 still requires repeated leakage-safe rolling-origin recovery with Monte Carlo uncertainty before a scientific recovery claim can be promoted.

## References

Rue, H., Martino, S., & Chopin, N. (2009). Approximate Bayesian inference for latent Gaussian models by using integrated nested Laplace approximations. *Journal of the Royal Statistical Society: Series B (Statistical Methodology), 71*(2), 319–392. https://doi.org/10.1111/j.1467-9868.2008.00700.x

Tierney, L., & Kadane, J. B. (1986). Accurate approximations for posterior moments and marginal densities. *Journal of the American Statistical Association, 81*(393), 82–86. https://doi.org/10.1080/01621459.1986.10478240

## Code traceability

- Owner value: `crates/topic_measurement/src/reference.rs` → `JointCoordinatePrecision`
- Marginal solve: `crates/topic_measurement/src/marginal_covariance.rs`
- Public export: `crates/topic_measurement/src/lib.rs`
- Fit-bound RED/contract: `crates/topic_measurement/tests/reference_document_marginal_covariance_contract.rs`
- Truth-basis covariance consumer: #703
- Scientific recovery acceptance: #680
