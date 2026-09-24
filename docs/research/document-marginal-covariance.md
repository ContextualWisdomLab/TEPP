# Fit-bound document marginal covariance from joint precision

Issues: #704, #705, #721  
Owner: `topic_measurement` for fit-bound covariance; `validation_core` for validation-coordinate interval construction  
Consumer frontier: #703 → #721 → #680

## Scientific question

TEPP's fitted document-coordinate summary stores diagonal Laplace quantities of the form `1 / diag(P)`. Those values are useful local curvature diagnostics, but they are not the marginal covariance block of the relation-coupled joint Gaussian approximation. When observed transition structure contributes off-diagonal precision, the marginal covariance for document `d` is the `d` principal block of `P^-1`, not the reciprocal diagonal of `P` and not the inverse of the isolated within-document precision block.

For one requested document, the bounded CPU reference path therefore solves

`P X = E_d`

where the columns of `E_d` are the unit vectors for that document's `K-1` ALR coordinates. The returned rows at the same coordinates form the document marginal covariance block. TEPP factors `P = L Lᵀ` once for the request and performs forward/back substitutions for only those `K-1` right-hand sides; it does not materialize the complete dense inverse.

A second coordinate question is distinct from that linear algebra. `ReferenceTopicFit::build_joint_coordinate_precision(...)` still accepts a provisional caller UUID vector for compatibility, while `FittedTopicBasisIdentity` is derived from the actual fitted topic-term rows. The #705 fit-bound covariance projection therefore retains the fitted basis identity from the same owner-issued `ReferenceTopicFit`; caller topic UUIDs remain visible compatibility coordinates but cannot substitute for fit-local numerical basis identity.

A third question is interval construction. #703 can re-express both the fitted ALR location and its marginal covariance into the same truth-reference topic basis, but empirical coverage still requires interval bounds to be generated from that paired geometry. #721 therefore adds a `validation_core` owner that constructs marginal normal/Wald bounds only after validating the complete covariance matrix rather than consuming detached diagonal variances.

## Numerical contract

`JointCoordinatePrecision::document_marginal_covariance(document_id)`:

- requires a document identity retained by the fit-bound precision;
- preserves the retained topic order: ALR numerator topics followed by the reference topic;
- uses the full relation-coupled precision, so candidate covariance cannot be manufactured from `ReferenceTopicModel::document_coordinate_variances`;
- rejects inconsistent retained geometry before factorization;
- rejects non-finite factorization or triangular-solve states;
- reconciles only symmetry differences within a documented `1e-10 × (1 + scale)` floating-point tolerance, then requires the returned principal covariance block to remain positive-definite;
- is bounded by `MAX_JOINT_COORDINATES = 4096`. With joint dimension `N` and requested document coordinate count `C = K-1`, the current dense reference path is `O(N^3 + C N^2)` time and `O(N^2)` memory.

`FitBoundDocumentMarginalCovariance::from_bound_fit(...)` composes that numerical result with `FittedTopicBasisIdentity::from_bound_fit(...)` from the same `ReferenceTopicFit`. It does not recalculate `P`, invert another matrix, or accept a caller-supplied basis identity. The resulting value retains document identity, EventTime, provisional topic UUID order, covariance values, and the content-bound fitted basis identity.

`validation_core::normal_marginal_interval_bounds(...)` requires a non-empty finite location vector, an exact square covariance of matching width, a finite positive normal critical value `z`, and finite symmetric positive-semidefinite covariance geometry. It then emits closed marginal bounds `location_i ± z * sqrt(covariance_ii)`. Zero marginal variance is allowed as a degenerate interval; malformed covariance or non-finite interval arithmetic fails closed. The function does not estimate coverage or assume independence among coordinates/documents.

The covariance contract test uses a relation-coupled fitted precision and requires the full-inverse marginal to differ from both the reciprocal precision diagonal and the inverse of the isolated one-coordinate document block. The same real fitted state must also reproduce its `FittedTopicBasisIdentity` through the #705 projection. Internal edge tests cover missing identity, retained-dimension mismatch, non-finite triangular solve, symmetry roundoff, material asymmetry, and non-positive-definite output. #721 adds interval-owner contracts for square geometry, symmetry/PSD, zero variance, invalid critical values, non-finite inputs, and overflowed bounds.

## Statistical interpretation

This is a deterministic Gaussian/Laplace validation primitive. A positive-definite observed-information or generalized-Gauss-Newton precision defines the covariance of the corresponding local Gaussian approximation by inversion. Off-diagonal precision terms encode dependence, so marginal uncertainty for a coordinate or subvector generally requires the relevant block of the full inverse rather than reciprocal diagonal curvature.

Tierney and Kadane (1986) describe Laplace approximations for posterior moments, variances, and marginal densities from local likelihood geometry. Rue, Martino, and Chopin (2009) treat latent Gaussian models in precision form and emphasize computation of posterior marginals and subvector marginals from the joint Gaussian structure. TEPP uses the same linear-algebra distinction while keeping a narrower claim: the returned matrix is the marginal covariance of TEPP's retained local Gaussian approximation, not proof that the approximation is empirically calibrated.

For #680, coverage must be estimated over independent known-truth DGP replications. Expanding rolling-origin windows repeatedly estimate earlier documents, and coordinates within a joint fit are dependent. Those repeated intervals therefore must not be flattened into an IID Bernoulli sample or fed to Wilson bounds as though each document-coordinate-window observation were independent. The scientific harness must first compute window-level coverage, collapse window metrics within each DGP replication, and then quantify between-replication Monte Carlo uncertainty. Morris, White, and Crowther (2019) likewise frame simulation studies as empirical experiments with known truth and recommend reporting Monte Carlo standard errors for finite-replication performance estimates; TEPP keeps that uncertainty at the independent DGP-replication level rather than treating dependent rolling-window observations as extra replications.

## Claim boundary

The #704 solve, #705 basis binding, #703 truth-basis transform, and #721 interval construction do **not** establish empirical interval coverage, calibrated posterior probability, Evidence source/vocabulary provenance, source/event-time authenticity, Membership provenance, relation activation authority, released semantic topic identity, or CPU/MLX/CUDA/OpenCL parity. `FittedTopicBasisIdentity` is fit-local numerical identity only; #658/#663 remain the source/vocabulary/release authority path. #680 still requires repeated leakage-safe rolling-origin coverage/calibration with Monte Carlo uncertainty before a scientific recovery claim can be promoted.

## References

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

Rue, H., Martino, S., & Chopin, N. (2009). Approximate Bayesian inference for latent Gaussian models by using integrated nested Laplace approximations. *Journal of the Royal Statistical Society: Series B (Statistical Methodology), 71*(2), 319–392. https://doi.org/10.1111/j.1467-9868.2008.00700.x

Tierney, L., & Kadane, J. B. (1986). Accurate approximations for posterior moments and marginal densities. *Journal of the American Statistical Association, 81*(393), 82–86. https://doi.org/10.1080/01621459.1986.10478240

## Code traceability

- Joint precision owner: `crates/topic_measurement/src/reference.rs` → `JointCoordinatePrecision`
- Marginal solve: `crates/topic_measurement/src/marginal_covariance.rs`
- Fit-basis binding: `crates/topic_measurement/src/fit_bound_marginal_covariance.rs`
- Covariance public export: `crates/topic_measurement/src/lib.rs`
- Fit-bound RED/contract: `crates/topic_measurement/tests/reference_document_marginal_covariance_contract.rs`
- Truth-basis location/covariance consumer: #702/#703 → `crates/validation_core/src/topic_alignment.rs`
- Marginal interval owner: #721 → `crates/validation_core/src/marginal_interval.rs`
- Marginal interval RED/contract: `crates/validation_core/tests/marginal_interval_bounds_contract.rs`
- Scientific rolling-origin coverage/calibration acceptance: #680
