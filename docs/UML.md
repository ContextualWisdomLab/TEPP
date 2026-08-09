# TEPP UML and Scientific Runtime Views

**Status:** Accepted diagrams aligned to PRD v0.4; as-built versus target maturity is explicit.  
**Last reviewed:** 2026-08-09

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

On current protected main only the workspace/evidence foundation is implemented. Temporal values are PR #5 active-PR; interval reasoning is PR #6 active-PR. Later boxes are accepted-target.

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
    Temporal->>Temporal: enforce cutoff/leakage semantics
    Temporal->>Graph: typed intervals/relations
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

A later document may report an earlier event, but it cannot be inserted into an earlier historical analysis before its availability time.

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
    workspace_foundation --> immutable_evidence: main
    immutable_evidence --> six_clock_temporal: PR_5_active
    six_clock_temporal --> interval_reasoning: PR_6_active_stacked
    interval_reasoning --> event_relation_membership: future
    event_relation_membership --> persistence_and_splits: future
    persistence_and_splits --> topic_measurement: future
    topic_measurement --> gpu_and_model_selection: future
    gpu_and_model_selection --> event_intelligence_and_psychometrics: future
    event_intelligence_and_psychometrics --> visual_interpretation_release: future
```

## Maintenance rule

Update these views when a scientific estimand, clock/interval semantics, relation authority, membership model, compute backend, persistence boundary, or implementation maturity changes. Active PRs become as-built only after protected-main integration and fresh required evidence.