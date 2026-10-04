# Refuse non-finite topic expectation sufficient statistics

- Validate each accumulated document-topic and topic-term count immediately after its write. A finite log-likelihood does not establish that the sufficient statistics are finite.
- Return the existing `NonFiniteEstimate` error on overflow without changing the responsibility formula, arithmetic order, model target, smoothing, or convergence tolerance.
- Add private crate regressions for document and topic accumulation overflow and an ordinary finite positive control. These exercise injected private states, not an observed public-fit success or a production incident.
- This correction reinforces the existing ADR0012 finite-intermediate contract. It does not implement or attest GPU execution, CPU/GPU parity, parameter recovery, release readiness, or protected-main integration.
