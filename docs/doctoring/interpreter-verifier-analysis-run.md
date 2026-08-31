# Interpreter/verifier analysis-run composition

**Active slice:** ADR 0050 / `interpreter_verifier_v1`
**Protected-main status:** not implemented-main

`interpretation_gateway` already refuses to treat an interpretation as an
estimator result or observed fact, requires cited evidence spans, and records
unsupported-claim rates from known truth. This slice binds those gates to a
cutoff-safe analysis-run profile so an operator can request a digest-bound
terminal result.

The executor does not call a live LLM provider and cannot promote scientific
truth. Live committee/conductor execution remains later GAP-013 work.

Exact-head Checks and two independent approvals are required before any
implemented-main claim.
