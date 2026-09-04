# SE-aware acceptance: equal nonzero correction projection

## Problem

`accept_within_standard_errors` decides the Validation Evidence predicate

`|estimate - target| <= k * standard_error`

from finite binary64 inputs. GAP-081 and GAP-082 recover first-order rounding terms when the directly rounded residual and bound are equal; GAP-083 falls back to an exact dyadic product comparison when both correction projections are zero. A remaining case existed when the subtraction correction and the FMA product correction both rounded to the same **nonzero** binary64 value even though the exact product correction lay strictly on one side of the exact subtraction correction. Falling through to the directly rounded equality could therefore change the scientific decision.

This is Validation Evidence admission policy in TEPP. It is not a reusable psychometric estimator and does not move arithmetic ownership from `fast-mlsirm`.

## Exact represented-input RED

Public RED commit `35ea85ba5c049e3736e8549445bc799638cc6555` adds `crates/validation_core/tests/standard_error_acceptance_equal_nonzero_correction_projection_contract.rs`.

Strict-rejection payload:

- `estimate = f64::from_bits(0x0210_2814_5144_3c99)`
- `target = -f64::from_bits(0x0000_0000_6398_c737)`
- `k = f64::from_bits(0x20d9_5434_7757_68c7)`
- `standard_error = f64::from_bits(0x2124_696e_33e2_baaa)`

The direct subtraction and direct product round to the same finite binary64 value. The error-free subtraction low term and `mul_add` product low term also project to the same positive subnormal. However, the represented-input exact product correction is slightly smaller than the represented subtraction correction; therefore the exact represented-input residual is strictly greater than `k * standard_error` and the correct decision is rejection.

The same contract contains an adjacent acceptance control with `estimate=0x01c3_f43e_c52b_4312`, subtraction correction `0x0000_0000_003b_9da9`, `k=0x20c8_7ace_8d72_9746`, and `standard_error=0x20ea_1585_cc49_24ca`. There the exact product correction lies above the subtraction correction, so a blanket rejection of equal nonzero projections would also be wrong.

Edge commit `b463a991adc6ff98fa09f91eb428cda9e0ff1255` adds the sign-complementary projection boundary: both first-order corrections can be the same negative subnormal while the exact represented product remains slightly farther below the rounded bound than the exact residual. This keeps the comparator's signed ordering executable rather than testing only positive product roundoff.

## Causal repair

Commit `7d597a18e043f3619b893981823b9be15ddb823c` keeps the existing decision hierarchy and changes only the unresolved equal-nonzero finite-tie branch. `represented_correction_le_exact_product_roundoff` decodes the represented factors and rounded product into integer significands and powers of two, forms the exact represented product and its exact signed roundoff in `u128`, and compares that exact roundoff with the represented subtraction correction.

The chosen repair avoids a second public decision authority, arbitrary-precision runtime dependency, decimal conversion, scale normalization, or a source copy from another CWL repository. The product significand is at most 106 bits, so the exact product and alignment required by this branch fit the existing `u128` numerical boundary.

## Alternatives rejected

Treating equal projected corrections as equality was rejected because the RED proves that projection equality does not imply exact represented-input equality. Rejecting every equal nonzero correction pair was rejected because the companion control proves that a valid acceptance exists on the other side of the same projection boundary. Replacing every finite comparison with a general exact-rational engine was rejected as unnecessary scope expansion: ordinary unequal direct results and the existing GAP-080/081/082/083 branches already have narrower causal rules.

## Scope and residual risk

The repair claims only the finite case where the directly rounded residual and bound are equal and the two nonzero correction projections are also equal. It does not claim globally correctly rounded Monte Carlo estimation, does not alter numerical estimation, and does not authorize scientific acceptance without the surrounding TEPP Validation Evidence contract and exact-head gates. Independent counterexamples remain the criterion for expanding the exact comparator into another branch.

## Traceability

- Bounded context: Validation Evidence
- Module/API: `crates/validation_core/src/monte_carlo.rs` / `accept_within_standard_errors`
- Public RED: `35ea85ba5c049e3736e8549445bc799638cc6555`
- Causal repair: `7d597a18e043f3619b893981823b9be15ddb823c`
- Signed edge coverage: `b463a991adc6ff98fa09f91eb428cda9e0ff1255`
- Contract test: `crates/validation_core/tests/standard_error_acceptance_equal_nonzero_correction_projection_contract.rs`
- CHANGELOG fragment: `CHANGELOG.d/validation-standard-error-acceptance-equal-nonzero-correction-projection.md`
- Landing vehicle: PR #488; only its current exact head after documentation commits is authoritative for hosted checks and review.

## Normative and methodological references

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE Standards Association. https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization, International Electrotechnical Commission, & Institute of Electrical and Electronics Engineers. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

As of 2026-09-05, IEEE 754-2019 remains an active published standard and P754 remains an active revision project rather than a published replacement. ISO/IEC 60559:2020 remains published. The AERA/APA/NCME Joint Committee is revising the 2014 testing Standards; the unpublished revision is not treated as current normative authority.
