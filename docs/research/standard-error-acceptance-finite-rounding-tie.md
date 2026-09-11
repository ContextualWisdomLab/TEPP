# Standard-error acceptance finite rounding tie

## Problem and scientific contract

TEPP Validation exposes `accept_within_standard_errors(estimate, target, standard_error, k)` for the deterministic decision

`|estimate - target| <= k * standard_error`.

The preceding GAP-079/GAP-080 repairs covered scale-underflow and both-overflow failures. Fresh represented-input review found a separate finite/finite boundary: both direct operations can remain finite yet round to the same binary64 value even when the exact dyadic values represented by the original inputs are strictly ordered.

Public RED `23c3262c824609d79dc14d45bb0acb5a54e99a51` uses `estimate = 1`, `target = 0`, `standard_error = 0x1.ffffffc000000p-1 = 1 - 2^-27`, and `k = 0x1.0000002000000p+0 = 1 + 2^-27`. The subtraction is exact, so the residual is exactly 1. The exact represented product is

`(1 - 2^-27)(1 + 2^-27) = 1 - 2^-54`,

which is strictly smaller than the residual. Binary64 multiplication rounds that midpoint to the even value `1.0`, so the predecessor compared `1.0 <= 1.0` and falsely accepted.

The adjacent multiplier `0x1.0000002000001p+0` remains accepted. A second control uses factors whose exact product lies slightly above 1 while the rounded product is still 1, ensuring the discriminator does not turn rounded ties into blanket rejection.

## Causal repair and constraints

Causal repair `6a2add488cfa6bb5ac3cc854a107f287b61bbed5` keeps every non-tie finite comparison and both-overflow exact comparator unchanged. On a nonzero finite rounded tie only, TEPP computes the error-free low term of `estimate - target`. If subtraction was exact, it uses fused multiply-add `k.mul_add(SE, -rounded_bound)` to expose the sign of multiplication roundoff. A negative nonzero product residual proves the exact represented bound is below the tied residual and rejects; a positive residual accepts. A zero product residual or an inexact subtraction stays on the predecessor rounded comparison rather than pretending that this bounded repair proves a more general exact inequality.

This is deliberately narrower than replacing all finite comparisons with arbitrary-precision arithmetic. A general sparse-dyadic comparator was rejected for this finding because the demonstrated defect needs only the product-rounding sign when subtraction itself is exact. Log-domain comparison was rejected because transcendental rounding would replace an exact binary boundary with another approximation. Treating every rounded tie as rejection was rejected because exact products can lie on either side of the same rounded value.

Coverage/edge contract `3656deb2f2abc0a83027618a823497a829d9227e` adds the strict-rejection RED, the adjacent accepted multiplier, a positive product-roundoff tie, an exactly represented tie, and an inexact-subtraction tie that must remain on the conservative rounded path. CHANGELOG trace: `090f5124588cb184fc2ef644d91355f2b2c2082b`.

## Ownership and non-claims

This is TEPP Validation decision semantics in `validation_core`. It is not reusable static psychometric estimation owned by `fast-mlsirm`, does not alter Longitudinal Modeling composition, and does not consume mutable `contextual-orchestrator` source.

The public Rust contract is the production authority. Exact dyadic arithmetic was used to establish the counterexample and controls; it does not replace scientific acceptance, hosted CI, or current-head review evidence.

This repair does not claim globally exact comparison for every finite rounded tie. In particular, when subtraction itself rounded or the FMA product residual is zero, the function preserves the existing finite rounded decision. A future widening requires an independent represented-input counterexample and a bounded exact method that does not create a second writer.

## Standards trace

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE. https://standards.ieee.org/ieee/754/6210/

IEEE 754-2019 was rechecked on 2026-09-04 and remains an Active Standard. IEEE P754 remains an Active PAR approved 2024-06-06 to supersede 754-2019; it is not treated as a published replacement. https://standards.ieee.org/ieee/754/11684/

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). ISO. https://www.iso.org/standard/80985.html

ISO/IEC 60559:2020 was rechecked on 2026-09-04 and remains Published at stage 60.60.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

AERA continues to publish the 2014 edition, while the Joint Committee announced in 2024 remains charged with revising that edition. TEPP therefore does not treat an unpublished revision as current normative authority.
