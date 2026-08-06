# ADR 0003: Relational Event Ontology and Multiple Membership

**Status:** Accepted  
**Date:** 2026-08-05

## Decision

Documents are not independent observations. TEPP represents document, passage, event, entity, revision, translation, evidence, and forward-transition graphs. Event instances are separate from fallible event mentions and retain agents, factors, products, places, subevents, arguments, confidence, and exact evidence.

Authors, departments, organizations, customers, partners, competitors, projects, opportunity pools, templates, languages, locations, and episodes form cross-classified, time-varying, multiple-membership assignments. Customer, partner, and competitor are contextual roles rather than permanent entity types.

## Consequences

Translation/revision/copy variants and episode members remain together in data splits. Higher-level conclusions use hierarchical or multiple-membership estimates instead of aggregating document-level associations atomistically. Relation absence is distinguished from unobserved relation status.

## Verification

Tests recover known relation graphs and membership effects, reject invalid role intervals and transition cycles, preserve observed versus inferred provenance, and measure relation precision/recall, graph stability, and duplicate-aware effective sample size.
