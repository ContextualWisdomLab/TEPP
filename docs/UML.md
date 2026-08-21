# TEPP UML and Scientific Runtime Views

**Status:** Accepted diagrams aligned to PRD v0.4; as-built versus target maturity is explicit.  
**Last reviewed:** 2026-08-12

## Platform component view

```mermaid
flowchart LR
    SRC[Immutable source evidence]
    TEMP[Temporal core]
    EVT[Event ontology]
    REL[Relation graph]
    MEM[Multiple-membership model]
    SEM[Multilingual semantic preprocessing]
    TOPIC[Shared-latent topic measurement]
    PSY[Longitudinal ESEM/DSEM]
    NET[Topic/event network]
    INT[Evidence-bounded interpretation]
    ART[Artifacts/visual analytics]

    SRC --> TEMP
    SRC --> SEM
    TEMP --> EVT
    TEMP --> REL
    EVT --> REL
    REL --> MEM
    SEM --> TOPIC
    MEM --> TOPIC
    TEMP --> TOPIC
    TOPIC --> PSY
    TOPIC --> NET
    REL --> PSY
    PSY --> INT
    NET --> INT
    INT --> ART
```

On current protected main the workspace/evidence foundation, six-clock temporal values, Allen algebra/path-consistency, event ontology/membership, and PostgreSQL persistence through restore-integrity probes are implemented. The active-PR `prediction_contradiction` crate is the promotion-authority gate: call `refuse_promotion` before authorizing promotion of unmatched predicted mass. Remaining TDT/CHRONOS, topic, psychometric, and service boxes stay accepted-target.

## Evidence-to-analysis sequence

```mermaid
sequenceDiagram
    actor Analyst
    participant Evidence as Evidence ingestion
    participant Temporal as Temporal core
    participant Graph as Event/relation graph
    participant Semantic as Semantic unitizer
    participant Topic as Topic measurement
    participant Psych as ESEM/DSEM
    participant Artifact as Evidence artifact service

    Analyst->>Evidence: authorized immutable document/artifact
    Evidence->>Evidence: bound + hash + exact spans
    Evidence->>Temporal: document/event/assertion/availability clocks
    Temporal->>Temporal: expose typed clocks and interval primitives
    Temporal->>Graph: typed intervals/relations (target integration)
    Evidence->>Semantic: exact source spans
    Semantic->>Semantic: validated multilingual concept evidence
    Semantic->>Topic: sparse concept/native lexical channels
    Graph->>Topic: covariates/memberships/time structure
    Topic->>Psych: posterior coordinates/plausible values
    Psych->>Artifact: factors/paths/invariance/uncertainty
    Topic->>Artifact: topics/prevalence/correlation/uncertainty
    Graph->>Artifact: event/provenance graph
    Artifact-->>Analyst: accessible tables/graphs/manifest
```

PR #8 provides typed clock/interval primitives only. Persistence/corpus-split enforcement of historical cutoff eligibility and graph integration remain accepted-target rather than as-built leakage protection.

## Six-clock availability state rule

```mermaid
stateDiagram-v2
    [*] --> observed_source
    observed_source --> temporally_typed
    temporally_typed --> cutoff_eligible: available_time <= knowledge_cutoff
    temporally_typed --> withheld_future_evidence: available_time > knowledge_cutoff
    cutoff_eligible --> analysis_snapshot
    withheld_future_evidence --> [*]
    analysis_snapshot --> [*]
```

A later document may report an earlier event, but it cannot be inserted into an earlier historical analysis before its availability time. PR #8 implements the typed clock/interval primitives; persistence/split enforcement of `cutoff_eligible` remains accepted-target.

## Relation authority view

```mermaid
flowchart LR
    A[Input/event state t0] -->|forward transition| B[Process/event state t1]
    B -->|forward transition| C[Outcome/event state t2]
    C -. retrospective_report .-> A
    C -. cites/supports .-> B
```

Solid transition edges obey forward temporal constraints. Provenance/evidence edges may point backward but do not become reverse state transitions.

## Cross-classified membership view

```mermaid
flowchart TB
    SEG[Semantic span/document]
    SEG --> AUTHOR[Author]
    SEG --> DEPT[Department]
    SEG --> PROJECT[Project/opportunity]
    SEG --> ROLE[Time-varying entity role]
    SEG --> EVENT[Event episode]
    SEG --> LANG[Language/template/source]
    ROLE --> ORG[Organization/market]
    PROJECT --> ORG
```

Membership is not forced into a single hierarchy; weights and validity intervals allow multiple simultaneous memberships.

## Compute view

```mermaid
flowchart LR
    DATA[Sparse batches]
    CPU[CPU f64 reference]
    GPU[GPU streamed backend]
    VRAM[VRAM budget controller]
    PARITY[Parity/recovery gate]
    OUT[Accepted numerical artifacts]

    DATA --> CPU
    DATA --> VRAM
    VRAM --> GPU
    CPU --> PARITY
    GPU --> PARITY
    PARITY --> OUT
```

GPU may be absent or fall back to CPU. A GPU result is not accepted merely because a kernel completed.

## Implementation/dependency state view

```mermaid
stateDiagram-v2
    [*] --> workspace_foundation
    workspace_foundation --> immutable_evidence: protected_main
    immutable_evidence --> six_clock_temporal: PR_8_active_replacement
    six_clock_temporal --> interval_reasoning: Task_4_replay_required
    interval_reasoning --> event_relation_membership: accepted_target
    event_relation_membership --> persistence_and_splits: accepted_target
    persistence_and_splits --> topic_measurement: accepted_target
    topic_measurement --> gpu_and_model_selection: accepted_target
    gpu_and_model_selection --> event_intelligence_and_psychometrics: accepted_target
    event_intelligence_and_psychometrics --> visual_interpretation_release: accepted_target
```

### Legacy stack note

PR #5 is superseded/conflicted lineage for Task 3. PR #6 remains a legacy Draft carrying Task 4 implementation history but is based on that superseded lineage. Its unique behavior must be replayed onto PR #8 or the exact protected-main descendant after PR #8 merges; old checks/reviews do not transfer.

## Maintenance rule

Update these views when a scientific estimand, clock/interval semantics, relation authority, membership model, compute backend, persistence boundary, implementation lineage, or maturity changes. Active PRs become as-built only after protected-main integration and fresh required evidence.
