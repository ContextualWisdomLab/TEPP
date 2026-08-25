# ADR 0003 — Relational event ontology and time-varying multiple membership

**Decision status:** Accepted
**Implementation maturity:** partial — membership network and event mention/instance separation are implemented-main; episode membership containment, typed forward-only relation behavior, status gates, and cross-classified/multiple-membership refusal are active in this increment; full multilevel/MMMC estimators and remaining persistence remain accepted-target.
**Implementation maturity:** partial — membership networks, event mention/instance separation, and the protected-main forward-transition foundation are implemented-main; `support_edge`, `outcome_order`, `retrospective_edge`, `inferred_status`, `copy_identity`, `summarizes_edge`, `subevent_containment`, `location_membership`, `episode_membership`, and typed target kinds are covered by this active consolidation PR; full multilevel/MMMC estimators and remaining persistence remain accepted-target.
**Date:** 2026-08-05
**Implementation maturity:** partial — membership network, event mention/instance separation, inferred/evidential/retrospective status gates, summary/source identity separation, template-copy/source identity separation, typed forward-only relation graph, strict input-process-outcome ordering, nested ICC refusal, and subevent parent-window containment are implemented-main; full multilevel/MMMC estimators and remaining persistence remain accepted-target.
**Date:** 2026-08-24  
**Decision status:** Accepted  
**Implementation maturity:** partial — membership network/roles with Kish ESS and nested ICC (cross-classified/multiple-membership refusal), event mention/instance separation, the typed forward-only relation graph, and the copy/summary/outcome-order/support/inferred-status/retrospective-reporting/location identity gates are implemented-main; typed target-kind membership identity in `membership_target` is on PR #131; multilevel psychometric estimators and remaining persistence details follow ADR 0013 and [`docs/TRACEABILITY.md`](../TRACEABILITY.md) as accepted-target. The customer/competitor role-contradiction gate ships in `role_contradiction` on this PR. The absence-is-not-negative identity gate ships in `relation_absence` on this PR.  
**Supersedes:** None. ADR 0016 owns TDT/CHRONOS event-intelligence task semantics; this ADR remains authoritative for ontology, relation, role, and membership structure.

## Context

Documents, passages, events, entities, revisions, translations, projects, organizations, templates, authors, and time-varying roles are not independent observations. Treating each document as an atom can produce atomistic fallacy, inflate effective sample size, leak related variants across validation splits, and erase the fact that one observation can belong to several non-nested contexts at once (American Educational Research Association, American Psychological Association, & National Council on Measurement in Education, 2014; Fox & Glas, 2001).

Customer, partner, and competitor are especially contextual roles rather than permanent entity types. The same organization can occupy different roles across projects, events, markets, and time.

## Decision

TEPP represents document, passage, event, entity, revision, translation, evidence, and forward-transition relationships explicitly. Event instances are distinct from fallible event mentions and retain typed roles/arguments, agents, factors, products/outcomes, places, subevents, confidence, time, and exact evidence.

Authors, departments, organizations, customers, partners, competitors, projects, opportunity pools, templates, languages, locations, and episodes form cross-classified, time-varying, multiple-membership assignments. Memberships carry explicit weights where scientifically justified and governed validity intervals. Customer/partner/competitor are role assignments, not immutable entity classes.

An episode assignment is accepted only when its event-time window is contained by
the episode window. Equal boundaries are valid, while inverted windows and
windows that escape either boundary fail closed.

Observed relation evidence, inferred relations, and promoted transition edges remain distinct. Relation absence is not silently interpreted as evidence of no relationship.

This ADR names a **relational event ontology** and time-varying membership structure. It does not adopt, and TEPP does not implement, a statistical relational-event-model (REM) estimator family. Multilevel/non-independence measurement follows Fox and Glas (2001) and the clustered-observation discipline in the *Standards for Educational and Psychological Testing* (American Educational Research Association et al., 2014). Production multilevel IRT/ESEM/DSEM estimators remain accepted-target under ADR 0005.

## Non-goals

- do not force observations into one hierarchy;
- do not make every relation a transition or causal edge;
- do not treat event mentions as error-free event instances;
- do not duplicate TDT/CHRONOS prediction/tracking authority here; ADR 0016 governs that layer.

## Alternatives considered

1. **Independent-document model** — rejected because dependence, provenance, leakage, and higher-level structure disappear.
2. **Single hierarchy (`document -> author -> department -> company`)** — rejected because projects, customers, partners, episodes, templates, and other contexts are cross-classified and can have multiple simultaneous memberships.
3. **Relational event ontology plus weighted time-varying multiple membership** — accepted.

## Consequences

Translation/revision/copy variants and episode members can be kept together in relation-aware data splits. Higher-level conclusions use hierarchical/cross-classified/multiple-membership estimates rather than aggregating document associations atomistically. Entity roles can change over time without rewriting entity identity. Graph and psychometric estimators receive explicit relation/membership uncertainty rather than hidden preprocessing assumptions.

## Failure and recovery

Malformed role intervals, impossible transition cycles, invalid membership weights, broken evidence ownership, contradictory role assertions, or unknown relation status fail closed or remain unresolved according to the owning contract. Recovery corrects or augments the evidence/assignment while preserving the original provenance and prior artifact version.

## Security, privacy, and governance impact

Relationship graphs and membership assignments can be sensitive even when direct identifiers are opaque. Access and export follow ADR 0009. Untrusted LLM output may propose mentions/relations/roles but cannot promote them without deterministic schema, evidence, authorization, and scientific validation.

## Compatibility and migration

Relation and role vocabularies are versioned. New membership targets require typed references and updated exactly-one constraints rather than an untyped polymorphic identifier. Persistence and bitemporal validity are governed by ADR 0013. Event-intelligence APIs consume these semantics without redefining them under ADR 0016.

## Verification

Tests recover known event/relation graphs and membership effects, reject invalid role intervals and transition cycles, preserve observed versus inferred provenance, exercise cross-classified and multiple-membership structures, and measure relation precision/recall, graph stability, membership-effect recovery, leakage prevention, and duplicate-aware/effective sample-size behavior.

## Rollback and supersession

Rollback restores the previous ontology/relation/membership contract and revalidates dependent model artifacts. Supersede only through an ADR that preserves explicit evidence provenance and non-nested/multiple-membership semantics or deliberately changes the estimand with corresponding PRD and validation updates.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Fox, J.-P., & Glas, C. A. W. (2001). Bayesian estimation of a multilevel IRT model using Gibbs sampling. *Psychometrika, 66*(2), 271–288. https://doi.org/10.1007/BF02294839
