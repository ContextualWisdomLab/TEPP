# ADR 0050 — Interpreter/verifier composition as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0010 (adaptive LLM orchestration) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already owns `interpretation_gateway`: an interpretation must
cite at least one evidence span, remains hypothetical, cannot become an
estimator result or observed fact, and records an unsupported-claim rate from
known truth. What is not implemented on protected main is this
`interpreter_verifier_v1` Analysis Run composition. The original branch also
lacked record-level snapshot/availability provenance and attached support-label
vectors to the run without a claim identity or interpretation binding. That
made the advertised cutoff-safe profile unable to prove that cited spans and
claim assessments belonged to the requested historical snapshot and
interpretation.

Live contextual-orchestrator provider execution, committee/conductor
calibration, and scientific claim promotion remain later GAP-013 work and are
not this slice. An LLM completion must not define numerical authority.

## Decision

Propose the `interpreter_verifier_v1` analysis-run output profile in
`analysis_engine`. The executor:

- consumes an interpretation identity plus evidence-span records carrying
  immutable snapshot identity and `AvailableTime`;
- consumes identified claim assessments carrying the same snapshot and
  availability provenance plus the interpretation identity they assess;
- bounds both offered populations by `MAX_EVIDENCE_UNITS` before downstream
  cloning or identity-set work;
- binds request and executor cutoff by parsed `KnowledgeCutoff::instant()`
  rather than RFC 3339 text equality;
- rejects cross-snapshot records and excludes same-snapshot records with
  `AvailableTime > knowledge_cutoff` before duplicate or claim-link admission,
  so future evidence cannot change an earlier historical result;
- keeps duplicate identities among cutoff-visible evidence fail closed;
- invokes `EvidenceBoundInterpretation::propose`,
  `refuse_interpretation_as_estimator_result`,
  `refuse_interpretation_as_observed_fact`, and `unsupported_claim_rate`
  without reimplementing those owner gates;
- emits a canonical SHA-256-digested `tepp.interpreter_verifier.v1` artifact
  with canonical UTC cutoff spelling, cutoff-admitted cited-span count,
  unsupported-claim rate, interpretation status `hypothetical`, and inference
  status `hypothetical_interpretation_not_scientific_authority`;
- keeps terminal provider validation status `validated` separate from the
  domain inference claim;
- does not invent a live LLM provider, persist rows, or promote scientific
  truth.

The historical replay invariant is explicit: adding evidence or claim records
that were unavailable at the requested cutoff must not alter the earlier
artifact or terminal result. Cross-snapshot evidence is a provenance violation
and is rejected rather than censored.

This is evidence-bounded interpretation composition, not live orchestration
and not estimator authority. `Accepted` is premature until the implementation
lands on protected main with its required evidence.

## Alternatives considered

1. Keep raw span UUIDs and parallel truth/decision vectors — rejected because
   neither snapshot/availability provenance nor claim-to-interpretation
   identity can be established at the Analysis Run boundary.
2. Compare RFC 3339 strings directly — rejected because distinct legal
   spellings can denote the same instant and must not change admission.
3. Treat future-unavailable records as invalid evidence — rejected because
   their mere existence would then perturb a historical replay; they must be
   excluded before identity/domain admission.
4. Invent a live LLM provider inside `analysis_engine` — rejected because
   provider execution belongs behind contextual-orchestrator and would create
   a second routing authority.
5. Put interpreter/verifier composition into `tepp_api` — rejected because
   transport contracts and interpretation composition would become one service
   boundary.
6. Bind the protected-main `interpretation_gateway` refusals through the
   existing Analysis Run application boundary — selected.

## Consequences

Operators can eventually request cutoff-safe interpreter/verifier composition
as a digest-bound terminal result after this proposal lands. The artifact
cannot become an estimator result or observed fact. Historical replay now has
an explicit record-level provenance contract; support-rate labels cannot be
silently borrowed from another interpretation; raw populations are bounded
before downstream cloning; and canonical artifact digests cannot vary only
because of an equivalent cutoff spelling. Live provider execution, committee
modes, and scientific promotion remain later work.

`InterpreterVerifierArtifact::from_json` validates the artifact contract but is
not standalone provenance authentication. Consumers must still bind the
artifact to the expected terminal digest, run identity, snapshot, and request
context.

## Verification

The PR includes Rust unit and integration tests for cited-span hypothetical
output, uncited-promotion rate recording without scientific promotion,
equivalent-instant cutoff binding, future-unavailable replay invariance,
cross-snapshot and wrong-interpretation refusal, population bounds, missing
span refusal, invalid support-payload refusal, snapshot/profile/cutoff
mismatch, canonical cutoff serialization, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

Hosted exact-head results and authored line/branch coverage are required on the
unchanged surviving head; predecessor receipts do not transfer.

## Rollback and supersession

Rollback removes the `interpreter_verifier_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps record-level
cutoff provenance, numerical-authority and observed-fact refusal fail closed,
and live LLM execution distinct from scientific authority.
