---
title: TEPP
---

# TEPP

TEPP is the Temporal Event Psychometrics Platform: a multilingual, temporal, relational measurement system whose statistical and psychometric arithmetic is implemented in Rust.

## Start here

The repository is currently an actively developed Rust workspace rather than a supported commercial release. For the current executable surface, verification commands, and explicit claim boundaries, start with the [README](https://github.com/ContextualWisdomLab/TEPP#readme).

## Product and architecture

TEPP owns temporal/event semantics, evidence identity and availability, relational and membership structure, analysis-run composition, persistence boundaries, and psychometric recovery gates. Reusable response-family and LSIRM/MLSIRM/DLSJM numerical kernels remain in [fast-mlsirm](https://github.com/ContextualWisdomLab/fast-mlsirm), while model-provider execution and LLM orchestration remain in [contextual-orchestrator](https://github.com/ContextualWisdomLab/contextual-orchestrator).

The current delivery and bounded-context authority is documented in:

- [Product and technical gap baseline](https://github.com/ContextualWisdomLab/TEPP/blob/main/docs/product-technical-gap-baseline.md)
- [Domain context map](https://github.com/ContextualWisdomLab/TEPP/blob/main/docs/architecture/domain-context-map.md)
- [Temporal-dependence composition](https://github.com/ContextualWisdomLab/TEPP/blob/main/docs/architecture/temporal-dependence-composition.md)
- [Approved PRD](https://github.com/ContextualWisdomLab/TEPP/blob/main/docs/product/prd-v0.4-approved.md)
- [Standards and literature](https://github.com/ContextualWisdomLab/TEPP/blob/main/docs/research/standards-and-literature.md)
- [Releases](https://github.com/ContextualWisdomLab/TEPP/releases)

## Onboarding and verification

TEPP is fail-closed about scientific and release claims. Branch-local tests, open pull requests, planning documents, or queued checks are evidence for review; they do not make a capability shipped on the protected default branch. The README and repository documentation contain the supported local quality gates and current implementation boundaries.

For repository-grounded code and documentation questions, use [Ask DeepWiki](https://deepwiki.com/ContextualWisdomLab/TEPP).

This page is a public documentation landing source. GitHub Pages publication is a separate repository-facing state and must be verified live before it is claimed available.
