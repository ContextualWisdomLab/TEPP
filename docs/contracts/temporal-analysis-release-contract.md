# Temporal-analysis release contract

This document records the TEPP-owned consumer boundary needed by downstream workforce-validation adapters such as Orgmetra. It does not declare a released artifact. GitHub release, package, provenance, SBOM, rollback, reproducibility, exact-head checks, and independent review remain separate release gates.

## Existing owner surface

The release path reuses `tepp_api::AnalysisRunRequest`, `AnalysisRunAccepted`, `AnalysisRunStatus`, and `AnalysisRunTerminalResult`; it does not create an Orgmetra-specific analysis schema or copy TEPP source into a consumer repository. Request identity already binds the immutable snapshot, knowledge cutoff, model/backend contract version, and requested output profile. A successful terminal result binds the run and request identity to an opaque result artifact, canonical lowercase SHA-256, result-schema version, completion time, and bounded summary.

For the first immutable consumer release, `AnalysisResultSummary.validation_status` has exactly one successful wire value: `validated`. `not_verifiable`, `non_converged`, `insufficient_evidence`, and other non-success scientific states must use `AnalysisRunTerminalResult::failed` and therefore carry no measurement artifact, result digest, result schema, or success summary. This prevents a consumer from treating a provider-authored non-empty status string as scientific success.

This change does not make `run_state = succeeded` a general scientific-validity claim outside the exact result contract. The result artifact's method/configuration, temporal/window/cohort semantics, cross-classified or multiple-membership structure, estimator provenance, uncertainty, and scientific recovery remain owned by the versioned result schema and the producing TEPP domain boundary. Downstream adapters must consume those released fields without flattening temporal or membership coordinates.

## Release acceptance still open

Issue #638 remains open until a protected TEPP head publishes an immutable versioned package/artifact for this boundary and records the artifact digest, SBOM/provenance, reproducibility and compatibility evidence, rollback path, exact-head tests/security/scientific acceptance, and qualifying independent review. Orgmetra must then pin that immutable owner contract rather than the historical mutable TEPP commit currently named by its adapter documentation.

The current source repair is only one prerequisite: it closes the success/failure semantic ambiguity before an immutable temporal-analysis contract is cut. It is not itself a release, attestation, compatibility guarantee, or authorization for consumer activation.
