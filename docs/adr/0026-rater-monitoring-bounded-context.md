# ADR-0026: Temporal rater monitoring bounded context

> Status: Proposed
> Date: 2026-08-29

## Context

The governed-rater program separates observation creation, numerical
calibration, hosted assessment operations, and temporal monitoring. TEPP already
owns six-clock temporal semantics, leakage-safe analysis cutoffs, event and
relation time, multiple membership, posterior longitudinal analysis, and
immutable analysis artifacts. It is therefore the natural owner of temporal
questions about rater parameters, but it must not become a second scoring engine
or mutate results published by Psychometrics Commons.

A single `drift` flag would collapse different phenomena:

- repeated-invocation variation within one exact rater configuration;
- gradual movement in severity, thresholds, discrimination, or interaction
  parameters;
- a discontinuity caused by a new model, prompt, rubric, procedure, or other
  configuration identity;
- loss or preservation of longitudinal measurement invariance.

Those phenomena have different causes, models, consequences, and review paths.
Conflating them would allow ordinary stochastic invocation variation to be
reported as operational drift, or a known configuration change to be hidden as
continuous parameter movement.

The downstream numerical source is the versioned parameter snapshot published
by `fast-mlsirm`. The hosted product source is immutable publication and
adjudication evidence from Psychometrics Commons. TEPP must consume those
artifacts without importing their databases or internal aggregate types.

## Decision

Create a `Temporal Measurement Monitoring` bounded context in a dedicated
`rater_monitoring` Rust crate.

### Aggregate roots

#### `RaterMonitoringRun`

Owns one leakage-safe, reproducible input set:

- one exact monitoring-run identity;
- one distinct `knowledge_cutoff` clock;
- zero or more draft `RaterParameterObservation` entities;
- `Draft -> Sealed` lifecycle.

A parameter observation preserves three different clocks:

- `effective_at`: when the parameter state applied in the measured domain;
- `available_at`: when an analysis could have used the artifact;
- `recorded_at`: when TEPP recorded the artifact.

A parameter observation is admissible only when:

```text
available_at <= knowledge_cutoff
```

The aggregate does not manufacture a total ordering between effective,
availability, and recording clocks. Duplicate parameter-snapshot references are
rejected. Sealing requires at least one input and freezes the set.

#### `RaterMonitoringArtifact`

Owns one immutable conclusion over one sealed run. Every artifact has exactly
one of the following kinds:

- `InvocationNoise`
- `GradualDrift`
- `ConfigurationChange`
- `MeasurementInvariance`

The source snapshot references must be non-empty, unique, and contained in the
sealed run. The conclusion is a versioned external reference rather than a free
text field or an instruction to rewrite a score.

## Context map

```text
fast-mlsirm
  Measurement Calibration
        | parameter snapshots
        v
TEPP
  Temporal Measurement Monitoring
        | monitoring artifacts
        v
psychometrics-commons
  Assessment Operations and governance review
```

All inputs and outputs are versioned published artifacts pinned by version and
digest. `semantic-data-portal` may resolve contextual revision metadata, but it
does not own parameter or monitoring artifacts.

## Ubiquitous language

| Term | Meaning |
|---|---|
| Parameter observation | A temporal reference to one immutable numerical parameter snapshot. |
| Knowledge cutoff | The latest availability time admitted by a monitoring run. |
| Invocation noise | Variation among repeated executions of one exact rater configuration. |
| Gradual drift | Continuous parameter movement without asserting a configuration identity change. |
| Configuration change | A discontinuity associated with a different exact rater configuration. |
| Measurement invariance | Evidence about whether the measurement structure remains comparable over time. |
| Monitoring artifact | An immutable typed conclusion over a sealed run. |

## Invariants

- effective, available, recorded, and knowledge-cutoff clocks remain distinct;
- evidence unavailable at the knowledge cutoff is rejected;
- duplicate parameter-snapshot identities are rejected;
- only draft runs accept inputs;
- only non-empty runs may be sealed;
- only sealed runs may publish artifacts;
- artifact evidence is non-empty, unique, and contained in its source run;
- monitoring kinds remain mutually distinct domain concepts;
- artifacts never rewrite source parameters, source invocations, or published
  scores;
- no placement, certification, employment, or other product decision is made in
  this context.

## Consequences

### Benefits

- future-information leakage becomes an aggregate violation rather than a
  reporting convention;
- known model/prompt/rubric changes cannot be silently interpreted as natural
  drift;
- repeated model calls can be monitored as nested invocation variation;
- longitudinal invariance remains distinct from parameter movement;
- downstream products receive auditable artifacts rather than mutable flags.

### Costs

- numerical monitoring estimators and simulation recovery remain follow-up
  work;
- producers must publish exact configuration and parameter-snapshot identities;
- persistence needs bitemporal indexes and immutable artifact receipts;
- product policy must decide how each artifact kind affects review, without
  asking TEPP to make the operational decision.

## Alternatives considered

1. **Store a Boolean drift flag in Psychometrics Commons.** Rejected because the
   hosted product does not own temporal estimands and the flag collapses distinct
   phenomena.
2. **Re-estimate rater parameters in TEPP.** Rejected because production
   calibration belongs to `fast-mlsirm`; TEPP consumes immutable snapshots.
3. **Use configuration name as a covariate without preserving exact identity.**
   Rejected because model, prompt, rubric, procedure, schema, workflow, and
   modality revisions must remain distinguishable.
4. **Use event/effective time as the analysis cutoff.** Rejected because a later
   artifact can describe an earlier state and would leak future knowledge.

## Follow-up implementation

1. released parameter-snapshot input schema and digest pinning;
2. PostgreSQL bitemporal persistence and transactional outbox;
3. true-state simulation for invocation noise, gradual drift, and change points;
4. Rust estimators for time-varying severity, thresholds, discrimination, and
   differential rater functioning;
5. longitudinal invariance diagnostics;
6. cross-classified and multiple-membership context effects;
7. consumer contract tests with Psychometrics Commons;
8. buyer-facing monitoring evidence and exact-value exports.

## Verification

The first crate tests every aggregate transition, cutoff violation, duplicate
identity, evidence-containment rule, monitoring kind, unsafe reference, and
error branch. Later numerical PRs must add true-parameter recovery, bias, RMSE,
interval coverage, change-point recovery, false-alarm rate, deterministic CPU
multithreading, and CPU/GPU parity.

## References

American Educational Research Association, American Psychological Association,
& National Council on Measurement in Education. (2014). *Standards for
educational and psychological testing*. American Educational Research
Association.

Evans, E. (2003). *Domain-driven design: Tackling complexity in the heart of
software*. Addison-Wesley.

Vernon, V. (2013). *Implementing domain-driven design*. Addison-Wesley.