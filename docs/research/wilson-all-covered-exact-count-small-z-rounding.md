# Wilson all-covered exact-count near-one rounding

## Problem and scientific boundary

TEPP's Validation Evidence owner computes the Wilson score interval from retained coverage counts and a finite positive standard-normal critical value. For an all-covered sample, Wilson's lower endpoint is algebraically

`L = n / (n + z²) = 1 - z² / (n + z²)`.

The estimator and sidedness do not change here. The defect is binary64 evaluation of the same endpoint when `n` is exactly representable and `z²` is small but not so small that the final endpoint should round to exact one.

With `n = 1` and represented `z = 0x1.0000000000001p-27`, binary64 multiplication produces represented `z² = 0x1.0000000000002p-54`. The correctly rounded represented-input endpoint is `0x1.fffffffffffffp-1` (`next_down(1.0)`). Evaluating `n + z²` first rounds that denominator back to `1.0`, so the predecessor direct expression `n / (n + z²)` emitted false exact `1.0` and erased representable finite-sample uncertainty.

This is not a new confidence-interval estimator. It is numerical conditioning of Wilson's all-covered score endpoint and remains owned by TEPP `validation_core` Validation Evidence. Reusable static psychometric estimation remains owned by fast-mlsirm.

## Causal repair

Public RED `e0c4ec81bb455d230259489dc71e23fe33704b1d` adds `crates/validation_core/tests/wilson_all_covered_exact_count_small_z_rounding_contract.rs` and fixes the expected lower endpoint to bits `0x3fefffffffffffff`.

Causal repair `c9dcb9df363999bbcbb6fffdc8b6a6d9ae5e762c` keeps the ordinary exact-count direct path. Only when that path returns exact `1.0` while represented `z² > 0` does it evaluate the algebraically equivalent miss mass `z² / (n + z²)` and subtract that from one. This closes the demonstrated false-one boundary without globally replacing the all-covered formula with a different rearrangement.

Boundary reinforcement `6140080d2257d0550be479d71371d70e2255c3d0` fixes `z = 2^-28`, `z² = 2^-56`, where the exact Wilson lower endpoint differs from one by less than half the binary64 spacing immediately below one. Exact `1.0` is therefore the correct represented endpoint, and the repair must not manufacture a lower representable value.

CHANGELOG trace: `09ebb482851fe7836e738a74395e6424621da9bf`.

## Decision record

Problem: exactly representable coverage denominators could still lose a representable all-covered miss mass through denominator absorption.

Constraint: preserve the existing Wilson estimator, standard-normal/two-sided semantics, `u64` count provenance, and `coverage.rs` single-writer ownership. Do not claim global correct rounding for every algebraic rearrangement.

Alternatives considered:

- Always evaluate `1 - z² / (n + z²)`. Rejected because large `z²/n` can make the miss fraction round to exact one and create the opposite false-zero boundary already addressed on the inexact-count path.
- Always evaluate `1 / (1 + z²/n)`. Rejected because forming `1 + z²/n` is exactly the near-one absorption mechanism exposed by this RED.
- Add a boundary-local complementary evaluation only after the direct exact-count result has collapsed to `1.0`. Selected because it is the minimum causal change for the demonstrated public contract and leaves ordinary direct evaluation unchanged.

Risk: the repair does not prove every possible exact-count Wilson endpoint is globally correctly rounded. Its claim is narrower: it removes the demonstrated false exact-one state while retaining the correctly rounded exact-one state below binary64 resolution.

## Traceability

- Bounded context: Validation Evidence
- Production writer: `crates/validation_core/src/coverage.rs`
- Public contract: `crates/validation_core/tests/wilson_all_covered_exact_count_small_z_rounding_contract.rs`
- Durable consumers: `WilsonCoverageEvidenceV1` and `ValidationEvidenceV1` continue to consume the canonical Wilson writer rather than duplicate endpoint arithmetic.
- RED: `e0c4ec81bb455d230259489dc71e23fe33704b1d`
- Causal fix: `c9dcb9df363999bbcbb6fffdc8b6a6d9ae5e762c`
- Edge reinforcement: `6140080d2257d0550be479d71371d70e2255c3d0`
- CHANGELOG: `09ebb482851fe7836e738a74395e6424621da9bf`

## References

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

IEEE. (2019). *IEEE Standard for Floating-Point Arithmetic (IEEE Std 754-2019).* https://standards.ieee.org/ieee/754/6210/

ISO/IEC. (2020). *ISO/IEC 60559:2020 Information technology—Microprocessor systems—Floating-point arithmetic.* https://www.iso.org/standard/80985.html

As checked in September 2026, IEEE 754-2019 and IEEE/ISO/IEC 60559-2020 are published active standards; IEEE P754 is an active revision project and is not treated as a published replacement.
