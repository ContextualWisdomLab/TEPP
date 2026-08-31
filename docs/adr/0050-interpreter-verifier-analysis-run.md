# ADR 0050 — Interpreter/verifier composition as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0010 (adaptive LLM orchestration) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already owns `interpretation_gateway`: an interpretation must
cite at least one evidence span, remains hypothetical, cannot become an
estimator result or observed fact, and records an unsupported-claim rate from
known truth. Operators still cannot request that interpreter/verifier boundary
as a digest-bound analysis-run output.

Live contextual-orchestrator provider execution, committee/conductor
calibration, and scientific claim promotion remain later GAP-013 work and are
not this slice. An LLM completion must not define numerical authority.

## Decision

Add the `interpreter_verifier_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes an interpretation identity, cited evidence spans, and known-truth
  support labels;
- requires the request snapshot and knowledge cutoff to match the offered
  construction;
- invokes `EvidenceBoundInterpretation::propose`,
  `refuse_interpretation_as_estimator_result`,
  `refuse_interpretation_as_observed_fact`, and `unsupported_claim_rate`
  without reimplementing those gates;
- fails closed when numerical-authority refusal or observed-fact refusal would
  ever succeed;
- emits a canonical SHA-256-digested `tepp.interpreter_verifier.v1` artifact
  with cited-span count, unsupported-claim rate, interpretation status
  `hypothetical`, and inference status
  `hypothetical_interpretation_not_scientific_authority`;
- does not invent a live LLM provider, persist rows, or promote scientific
  truth.

This is evidence-bounded interpretation composition, not live orchestration
and not estimator authority.

## Alternatives considered

1. Bind another GAP-004 Bayesian sampler or topic birth/split/merge slice —
   rejected because fitted candidate-`K` is already a live analysis-run PR and
   this hour's unique gap is GAP-013.
2. Invent a live LLM provider inside `analysis_engine` — rejected because
   provider execution belongs behind contextual-orchestrator and would embed
   model names as scientific meaning.
3. Put interpreter/verifier composition into `tepp_api` — rejected because
   transport contracts and interpretation composition would become one service
   boundary.
4. Bind the existing `interpretation_gateway` refusals to ADR 0022's
   analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe interpreter/verifier composition as a
digest-bound terminal result. The artifact cannot become an estimator result
or observed fact. Snapshot/profile/cutoff mismatch, missing evidence spans,
and invalid support payloads fail closed. Live provider execution, committee
modes, and scientific promotion remain later work.

## Verification

The PR includes Rust unit and integration tests for cited-span hypothetical
output, uncited-promotion rate recording without scientific promotion, missing
span refusal, invalid support-payload refusal, snapshot/profile/cutoff
mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `interpreter_verifier_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps numerical
authority and observed-fact refusal fail-closed and distinct from live LLM
execution.
