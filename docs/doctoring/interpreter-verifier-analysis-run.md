# Interpreter/verifier analysis-run composition

**Active slice:** ADR 0050 / `interpreter_verifier_v1`
**Decision state:** Proposed / active-PR
**Protected-main status:** not implemented-main

Protected main already owns `interpretation_gateway`: it requires cited
evidence spans, refuses promotion to estimator result or observed fact, and
computes unsupported-claim rates from known truth. This slice proposes an
Analysis Run adapter around that owner contract; it does not make
`interpretation_gateway` itself branch-local.

The original adapter was not cutoff-safe. Evidence spans carried only UUIDs,
claim labels carried no identity or interpretation link, raw input was
unbounded before downstream cloning, and request/executor cutoff equality used
RFC 3339 text. The current branch requires per-record immutable snapshot and
`AvailableTime` provenance, identified claim assessments bound to the offered
interpretation, `MAX_EVIDENCE_UNITS` admission, and parsed-instant cutoff
binding. Same-snapshot records unavailable at the cutoff are excluded before
duplicate or claim-link checks; cross-snapshot evidence remains a fail-closed
provenance violation. Terminal provider validation is `validated`; the
hypothetical/non-authoritative domain claim remains in artifact
`inference_status`.

Historical replay is therefore part of the contract: adding a future span or
claim, including a future record that reuses an otherwise visible identity,
must not change the earlier artifact or terminal result. Artifact cutoff text
is canonical UTC so equivalent timestamp spellings cannot mint different
canonical digests.

The executor does not call a live LLM provider and cannot promote scientific
truth. Live committee/conductor execution remains later GAP-013 work and must
consume contextual-orchestrator only through its released contract.

The current organization ruleset requires one qualifying current-head approval,
stale approvals are dismissed after pushes, review-thread resolution is
required, and organization required workflows must pass. Thread resolution or
predecessor checks are not approval or exact-head merge evidence.
