# Standard-error acceptance scale-underflow repair

## Problem and scientific contract

TEPP's Validation bounded context exposes `accept_within_standard_errors(estimate, target, standard_error, k)` as the deterministic CPU `f64` reference for the decision

`|estimate - target| <= k * standard_error`.

The predecessor normalized every operand before comparison. That avoided overflow for opposite-sign full-range estimates, but the order of operations could erase a finite positive uncertainty allowance. With

- `estimate = 1.0e308`,
- `target = next_down(estimate)`,
- `standard_error = 2.2e-16`, and
- `k = 1.0e308`,

the represented direct residual is finite (`1.99584030953472e292`) and the represented direct bound is finite (`2.2e292`), so the recovery is admissible. The predecessor nevertheless computed `standard_error / scale == 0.0` before multiplying by `k`, converted the positive bound to zero, and rejected the result. This is an operation-order underflow in the decision implementation, not evidence that the scientific uncertainty is zero.

Public RED `76067efce44419c27687b8673cb76092c90fb5a5` fixes the admissible case and a nearby rejection (`standard_error = 1.8e-16`) so repairing the positive bound cannot widen the decision arbitrarily.

## Constraints and alternatives

The repair must retain the existing exact-recovery semantics for `standard_error == 0` or `k == 0`, must remain deterministic binary64, and must still handle an overflowing opposite-sign residual without accepting merely because both sides materialized as infinity.

Always evaluating the normalized expression was rejected because it is the demonstrated cause of the false rejection. Always comparing `abs(estimate - target)` with `k * standard_error` was also rejected because either side can overflow for finite inputs. Log-domain comparison was not selected because it introduces transcendental rounding into a gate whose operands already admit an arithmetic decision.

Causal repair `4ffdf3665b3bbafd3b0bbf06b599fe71498169ab` therefore uses direct represented subtraction and multiplication whenever those results are finite. If only the finite positive bound overflows, any finite residual is inside it; if only the residual overflows, a finite bound cannot cover it. Scale normalization remains only for the both-overflow case. The nearby rejection in the public contract preserves the original decision boundary. CHANGELOG trace: `d822c0ba7fa548c3283e462323c56d8f5705de31`.

## Ownership and risk

This is TEPP Validation decision semantics. It does not introduce reusable static psychometric estimation into TEPP, does not move arithmetic owned by `fast-mlsirm`, and does not depend on an unreleased contextual-orchestrator contract. The change is confined to the existing `accept_within_standard_errors` Domain Service and its public contract.

Residual risk remains in binary64 tie behavior when both the direct residual and direct bound overflow and the normalized fallback must decide the comparison. No broader correctly-rounded claim is made without a separate represented-input counterexample and RED.

## Standards trace

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://standards.ieee.org/ieee/754/6210/

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). https://www.iso.org/standard/80985.html

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

AERA, APA, and NCME announced the Joint Committee charged with revising the 2014 edition on June 12, 2024; as of 2026-09-04 the 2014 edition remains the published edition used by this trace. https://www.aera.net/Newsroom/Members-of-the-Joint-Committee-for-the-Revision-of-the-Standards-for-Educational-and-Psychological-Testing-Named
