# Temporal-analysis release contract

This document records the TEPP-owned consumer boundary needed by downstream workforce-validation adapters such as Orgmetra. It does not declare a released artifact. GitHub release, package, provenance, SBOM, rollback, reproducibility, exact-head checks, and independent review remain separate release gates.

## Existing owner surface

The release path reuses `tepp_api::AnalysisRunRequest`, `AnalysisRunAccepted`, `AnalysisRunStatus`, and `AnalysisRunTerminalResult`; it does not create an Orgmetra-specific analysis schema or copy TEPP source into a consumer repository. Request identity already binds the immutable snapshot, knowledge cutoff, model/backend contract version, and requested output profile. A successful terminal result binds the run and request identity to an opaque result artifact, canonical lowercase SHA-256, result-schema version, completion time, and bounded summary.

`AnalysisRunRequest::to_json` treats the knowledge cutoff as a structural wire coordinate. The cutoff must use TEPP's canonical UTC RFC 3339 spelling; a semantically equivalent offset spelling is rejected rather than permitted to create a second valid byte identity for the same instant. Serialization does not consult the current wall clock. Live `AnalysisRunRequest::from_json` remains the admission boundary and additionally rejects a canonical cutoff that has not happened yet. The split is deliberate: immutable request evidence must preserve one stable byte identity during later replay, while a new command still cannot claim evidence availability from the future.

For the first immutable consumer release, `AnalysisResultSummary.validation_status` has exactly one successful wire value: `validated`. `not_verifiable`, `non_converged`, `insufficient_evidence`, and other non-success scientific states must use `AnalysisRunTerminalResult::failed` and therefore carry no measurement artifact, result digest, result schema, or success summary. This prevents a consumer from treating a provider-authored non-empty status string as scientific success.

This change does not make `run_state = succeeded` a general scientific-validity claim outside the exact result contract. The result artifact's method/configuration, temporal/window/cohort semantics, cross-classified or multiple-membership structure, estimator provenance, uncertainty, and scientific recovery remain owned by the versioned result schema and the producing TEPP domain boundary. Downstream adapters must consume those released fields without flattening temporal or membership coordinates.

## Release acceptance still open

Issue #638 remains open until a protected TEPP head publishes an immutable versioned package/artifact for this boundary and records the artifact digest, SBOM/provenance, reproducibility and compatibility evidence, rollback path, exact-head tests/security/scientific acceptance, and qualifying independent review. Orgmetra must then pin that immutable owner contract rather than the historical mutable TEPP commit currently named by its adapter documentation.

The current source repairs close three prerequisites: successful results fail closed unless their validation status is exactly `validated`; request serialization no longer depends on the current wall clock; and equivalent RFC 3339 offset aliases cannot produce multiple valid request byte identities for one cutoff instant. They do not yet bind the full method/configuration, time/window/cohort, multilevel or multiple-membership projection required by #638, and they are not themselves a release, attestation, compatibility guarantee, or authorization for consumer activation.
