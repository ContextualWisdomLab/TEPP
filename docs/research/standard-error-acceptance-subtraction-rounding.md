# Standard-error acceptance subtraction rounding

## Problem and represented-input contract

TEPP Validation exposes `accept_within_standard_errors(estimate, target, standard_error, k)` for the deterministic decision `|estimate - target| <= k * standard_error`.

GAP-081 corrected a finite rounded tie when subtraction was exact and multiplication alone rounded across the decision boundary. A fresh review found the complementary finite case: subtraction itself can round onto the finite bound and erase a strict rejection.

Public RED `58cbc03253997865b7c8ec19fb501fc89c22c851` uses `estimate = 1`, `target = -2^-54` (`0xbc90_0000_0000_0000`), `standard_error = 1`, and `k = 1`. The exact dyadic residual represented by the inputs is `1 + 2^-54`, strictly greater than the exact unit bound, while binary64 subtraction rounds the residual to `1.0`. The predecessor therefore fell through to `1.0 <= 1.0` and falsely accepted. The existing positive-min-subnormal target remains an acceptance control because its exact residual is below the unit bound even though its subtraction also rounds to `1.0`.

## Causal repair

Causal repair `68a6fd98b4f661ad5d4c3c35dc8b9074f2c59281` remains inside the existing `validation_core` writer. On a nonzero finite rounded equality only, it obtains the error-free low term of `estimate - target`, changes that low-term sign when the rounded difference is negative so it describes the absolute residual, and compares it with the FMA low term of `k * standard_error`. If those two correction projections differ, their order determines the represented-input decision. If they are equal, TEPP preserves the predecessor rounded decision rather than claiming a global exact comparator.

This preserves the earlier finite-direct, scale-underflow, one-sided overflow, and both-overflow paths. It also preserves the GAP-081 multiplication-only boundary as the special case whose residual correction is zero. Edge-coverage commit `b2c06512b6d795555269862c1c3d7e4bc67f3f18` adds the sign-symmetric negative rounded-difference case so the absolute-residual correction sign is exercised explicitly rather than left to branch inference.

Alternatives rejected: blanket rejection of rounded ties would break valid below-bound cases; replacing all finite comparisons with arbitrary precision would widen the owner and implementation surface beyond the demonstrated defect; scale or logarithmic comparison would introduce another rounded approximation at a boundary already expressible with binary floating-point error terms.

CHANGELOG trace: `c4f752f30121901bfbf4a96b03fdef0ccd9f4bf3`.

## Restack and ownership

Before this RED, protected `main` advanced through #489 to `b18bca1c69ef8d1799fcd3af6bf4412498e007c3`. The Validation branch was one commit behind and diverged from that protected head. It was repaired without force push or destructive rebase by merge-restack commit `a2754592987be9e88941d220565c45e34f621a4e`, retaining all Validation ancestry while inheriting #489's workflow-load repair.

This remains TEPP Validation decision semantics. It is not reusable static psychometric estimation owned by `fast-mlsirm`, does not alter Longitudinal Modeling composition, and does not consume mutable `contextual-orchestrator` source.

## Standards trace

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE.

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). ISO.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

The published standards above govern the current trace. Unpublished revisions are not treated as current normative authority.
