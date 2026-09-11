# Wilson boundary peer-root admission

## Problem

`ValidationReport` stores empirical coverage `p` and the two Wilson endpoints, but not the producer's sample count or critical value. The existing admission check therefore eliminates the unrecorded `a = z² / n` and checks the Wilson root identity. That identity is a necessary interior-pair condition, but its complement form can become numerically insensitive when a minority-coverage lower endpoint is replaced by exact zero.

A concrete represented-input case is one covered observation out of `n = 100_000_000` with `z = 10_000`. The canonical producer gives:

- `p = 0x3e45_798e_e230_8c3a` (`1.0e-8` as represented binary64);
- lower `L = 0x3c9c_d2b2_8e2c_a873` (`9.999999800000005e-17`);
- upper `U = 0x3fe0_0000_055e_63b8` (`0.5000000099999999`).

Replacing only `L` with `0.0` leaves the old complement-form comparison numerically equal on both sides, so the forged pair was admitted even though the peer endpoint implies a positive, ordinarily representable lower root. This is an artifact-admission defect: it does not change the Wilson producer or estimate a new psychometric quantity.

## Necessary boundary relation

Wilson's two roots obey

`L U = p² / (1 + a)`

and

`L + U = 1 + (2p - 1) / (1 + a)`.

Eliminating `a` and solving directly for the minority lower root gives

`L = p²(1 - U) / [p² + (1 - 2p)U]`, for `0 < p < 0.5`.

The implementation evaluates the same relation as

`p * ((p / denominator) * (1 - U))`

so `p²` is not forced to underflow before the final lower root itself becomes unrepresentable. If a stored lower endpoint is exact zero while this implied peer root remains positive in binary64, the pair cannot be admitted. For `p > 0.5`, TEPP applies the same check to the complement interval `(1-U, 1-L)`.

The check remains deliberately boundary-local. It does not reconstruct `n` or `z`, does not claim globally correctly rounded Wilson endpoints, and does not replace the existing ordinary interior-pair tolerance. An extreme `[0, 1]` pair remains admissible when the reconstructed minority peer root also rounds to zero; the durable report lacks enough producer provenance to make a stronger claim there.

## Decision and rejected alternatives

The selected repair is a necessary peer-root representability gate inside the existing Validation Evidence single writer. Tightening the global absolute tolerance was rejected because the counterexample is cancellation-conditioned rather than merely too loosely tolerated, and a global tolerance change would alter ordinary interior admission without a demonstrated defect. Requiring every nonzero empirical coverage to have a strictly positive lower endpoint was rejected because a finite extreme Wilson configuration can legitimately project a positive mathematical root to binary64 zero. Reconstructing a unique sample count or critical value from the durable report was rejected because those values are not fields of `ValidationReport`; inventing them would create false provenance.

## Executable traceability

- Public RED: `c9d612f2964eb89ae6070e707c77115a12be84b9`, `crates/validation_core/tests/validation_report_wilson_boundary_pair_coherence_contract.rs`.
- Minimal causal repair: `88512417d931e54ec4eb41581caa0ef20b71df5b`, `crates/validation_core/src/report.rs`.
- Branch-complete boundary implementation: `6a4f89da52cf18f63b2818545f15f72f43099a3f`.
- Symmetry, representable-control, extreme-boundary, and serde contract hardening: `0ce7639bd9f0ab5df2ecd6b24cd57a467e975d26`.
- CHANGELOG evidence: `407b3645233e8855cf30487250155ac903d6a4cb`.

Hosted exact-head GREEN, independent review, protected-main merge, release, and downstream release evidence remain separate delivery gates.

## References

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE Standards Association. https://doi.org/10.1109/IEEESTD.2019.8766229

ISO/IEC/IEEE. (2020). *Floating-point arithmetic* (ISO/IEC/IEEE 60559:2020). International Organization for Standardization, International Electrotechnical Commission, & IEEE.

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953
