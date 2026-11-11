# Refuse non-finite topic expectation sufficient statistics

- Validate each accumulated document-topic and topic-term count immediately after its write. A finite log-likelihood does not establish that the sufficient statistics are finite.
- Return the existing `NonFiniteEstimate` error on overflow without changing the responsibility formula, model target, smoothing, or convergence tolerance.
- Add private crate regressions for document and topic accumulation overflow and an ordinary finite positive control. These exercise injected private states, not an observed public-fit success or a production incident.
- This correction reinforces the existing ADR0012 finite-intermediate contract. It does not implement or attest GPU execution, CPU/GPU parity, parameter recovery, release readiness, or protected-main integration.

- Reject positive topic smoothing that underflows to a zero probability after beta normalization. Preserve the supplied floor and formulas; return `NonFiniteEstimate` rather than emitting a degenerate fit or treating an explicit zero sparse entry differently from its omission. Public CSR/CSC regressions include a representable tiny-floor control.

Reject observed topic transitions when the mapped document event time moves backward, even when the independently supplied relation intervals are forward-validated. Preserve equal-time pairs and document-row permutation.

- Reject CSR/CSC outer dimensions whose required offset count is not representable as `usize`, using `InvalidSparseMatrix` rather than overflowing during validation. Preserve tiny canonical storage with maximum inner dimensions. This closes an inherited checked-arithmetic constructor boundary; it does not establish a release-mode panic or change estimator arithmetic.


- Form each posterior responsibility before multiplying by its observed count. This arithmetic regrouping preserves representable tiny expected counts that a count-first intermediate product can round to zero. Public CSR/CSC fit-based regressions check aggregate rare-term count conservation against a normal-count control; this is CPU `f64` acceptance, not GPU parity or parameter-recovery evidence.

- When the mixture is normal but an individual topic mass underflows, divide its smaller simplex factor by that normal mixture before the final product, ordering the observed-count scaling around one. When the mixture itself is subnormal or rounds to zero, decompose all positive factors into binary mantissas/exponents and use one common scaled sum for both responsibilities and log-likelihood. Scale the observed count before final binary rounding so representable expected counts survive. Regressions retain the large-count/tiny-mass and public CSR/CSC cases and add subnormal-mixture posterior/count conservation, consistent likelihood, zero/small/large-count controls, and individually rounded-zero products. The new subnormal cases are injected private normalized states, not an observed public-fit trajectory or GPU acceptance.


- Use the same mixture normalization and log-likelihood helper in expectation and the public in-sample likelihood consumed by statistical K selection. Actual public likelihood regressions compare the same injected subnormal/rounded-zero model states; they do not claim a public fit reached those states.

- Preserve exact-zero topic factors as zero mass when another topic supplies a positive subnormal mixture. Keep the positive-factor decomposition strict, reject nonfinite or negative partners before skipping a zero product, and retain all-zero-mixture refusal. Private expectation/count and public likelihood/statistical-candidate regressions distinguish injected model states from the unchanged positive fitted-model path.


- Reject nonfinite ordinary mixture sums before selecting binary-scaled normalization. Product overflow and summation overflow must retain the existing `NonFiniteEstimate` refusal instead of becoming finite log scores; finite zero sums caused by positive-product underflow still use scaled recovery. Private expectation and public likelihood/statistical-candidate regressions cover overflowing injected model states alongside ordinary, subnormal and rounded-zero positive controls. This does not assert that a valid normalized fit produces overflowing probabilities.
