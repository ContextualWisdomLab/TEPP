# TEPP

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/ContextualWisdomLab/TEPP)

**Temporal Event Psychometrics Platform for multilingual, relational, time-aware measurement.**

TEPP measures documentary and event evidence as fallible observations of latent semantic, temporal, relational, and psychological structure. It preserves exact source evidence, multiple clocks, multilevel membership, measurement uncertainty, and relation history instead of flattening documents into independent rows or treating model output as scientific authority.

Production mathematical and psychometric arithmetic belongs in Rust. Statistical claims remain gated by explicit evidence and validation contracts; LLMs may assist interpretation or proposal generation but do not become numerical estimators or claim-promotion authority.

## Why TEPP

Many analytical systems lose the context needed to decide whether an apparent change is real: when evidence became available, which revision was observed, which language or template expressed it, which people or organizations belonged to which groups at the time, and whether a result is measurement drift rather than substantive change.

TEPP keeps those distinctions explicit.

| Need | What TEPP is designed to preserve |
| --- | --- |
| Multilingual measurement | Shared concepts with language-specific expression and evidence-bound alignment |
| Time-aware analysis | Event, assertion, document, system, availability, and knowledge-cutoff clocks |
| Relational evidence | Documents, spans, events, entities, revisions, translations, citations, memberships, and provenance |
| Multilevel structure | Cross-classified and multiple membership without collapsing observations to one group |
| Measurement uncertainty | Posterior-aware and invariance-aware contracts instead of treating latent estimates as error-free scores |
| Scientific governance | Separate validation evidence, interpretation, and scientific claim promotion |
| Reproducibility | Immutable evidence identities, deterministic contracts, versioned decisions, and explicit failure boundaries |

## Product boundary

TEPP owns **temporal/event measurement composition**: evidence/time semantics, longitudinal and relational measurement context, analysis-run contracts, scientific validation boundaries, and the event-aware structures required to measure change responsibly.

It does not absorb every adjacent capability:

- [`fast-mlsirm`](https://github.com/ContextualWisdomLab/fast-mlsirm) owns reusable psychometric/static numerical kernels and model-family arithmetic that should not be duplicated here.
- [`contextual-orchestrator`](https://github.com/ContextualWisdomLab/contextual-orchestrator) owns model-provider execution, routing, credentials, and LLM orchestration.
- [`LineageWeave`](https://github.com/ContextualWisdomLab/LineageWeave) can preserve and render accepted lineage/evidence but is not TEPP's numerical authority.
- Consumer products such as Naruon own their workflow and decision semantics rather than becoming TEPP subdomains.

Cross-product integrations must use released, versioned contracts. An open upstream branch or model response is never treated as production truth.

## Current maturity

TEPP is an active technical platform under development, not a completed commercial release. Protected source contains a broad Rust-first contract foundation for evidence, temporal semantics, relations, membership, persistence, simulation, validation, API exchange, and selected analytical/recovery primitives. The approved PRD describes a larger target platform than the capabilities currently integrated and released.

In particular, do not infer from the architecture that TEPP already ships a complete ESEM/DSEM/continuous-time estimator suite, production operator workspace, supported GPU backend, or generally available service. Candidate behavior in open pull requests remains candidate evidence until it reaches protected source through normal governance and, where applicable, a versioned release.

The current product/technical gap and ownership evidence is maintained in [`docs/product-technical-gap-baseline.md`](docs/product-technical-gap-baseline.md).

## Start here

TEPP is currently consumed and developed from source; there is no generally available installation package or production endpoint advertised by this README.

For a source checkout, verify the repository contract before relying on a branch:

```bash
python3 scripts/check_workspace_contract.py
python3 scripts/check_docstrings.py
python3 -m coverage run --branch -m unittest discover -s tests/quality -p 'test_*.py'
python3 -m coverage report --fail-under=100 --show-missing

cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --doc --workspace --all-features
cargo doc --workspace --all-features --no-deps
cargo deny check
```

Stable Rust line coverage is measured with `cargo-llvm-cov`. Branch coverage uses a separately pinned nightly lane because Rust branch coverage remains an unstable compiler capability. A zero denominator is reported explicitly for a crate with no executable behavior; it must never hide uncovered production code.

If you are evaluating TEPP as an integrator rather than developing the workspace, start with the product boundary and released-contract evidence rather than individual crate names.

## Core concepts

### Evidence before interpretation

Source bytes, exact spans, identifiers, versions, provenance, and availability are first-class evidence. Derived fields and model outputs do not overwrite the observations that support them.

### Six-clock temporal semantics

TEPP distinguishes event/valid time, assertion time, document time, system time, availability time, and the analysis knowledge cutoff. Historical analyses exclude evidence that was not actually available by the declared cutoff.

### Relational and multilevel measurement

Documents and events can participate in revision, translation, citation, membership, project, organization, author, episode, and other time-varying relations. Membership can be cross-classified and multiple rather than forced into one hierarchy.

### Measurement is not labeling

A discovered topic, cluster, relation, or LLM explanation is not automatically a validated construct. Measurement invariance, uncertainty, known-truth recovery, model diagnostics, and external validity remain separate requirements appropriate to the claim being made.

### Temporal precedence is not causality

TEPP can preserve ordered events and analyze temporal association, but causal language requires an identified experimental, quasi-experimental, or defensible observational design. A successful API call, fit routine, or temporal ordering does not promote a causal claim.

## Architecture at a glance

```text
Documentary / event evidence
           |
           v
+-------------------------------+
| Evidence & temporal semantics |
| exact spans / provenance      |
| six clocks / cutoff rules     |
+---------------+---------------+
                |
                v
+-------------------------------+
| Relational measurement layer  |
| events / membership / lineage |
| multilingual measurement      |
+---------------+---------------+
                |
                v
+-------------------------------+
| Analysis & validation         |
| Rust numerical boundaries     |
| recovery / invariance gates   |
+---------------+---------------+
                |
                v
+-------------------------------+
| Evidence-grounded outputs     |
| interpretation / audit        |
| explicit claim authority      |
+-------------------------------+
```

Implementation crates are modular units inside these product responsibilities; crate count is not a customer capability metric and is intentionally not used as a maturity signal in this README.

## Scientific and security guardrails

TEPP is designed to fail closed when evidence, temporal identity, provenance, authorization, numerical validity, or claim-promotion requirements are absent or contradictory. The architecture deliberately separates statistical recovery from scientific acceptance and evidence-grounded interpretation from authoritative estimation.

Production mathematical and psychometric arithmetic is Rust-owned. Python can support repository validation, interoperability, and independent-oracle testing, but does not become the production numerical core.

Security, privacy, and access boundaries are part of the data contract: sensitive evidence is purpose-bound, consumer boundaries are explicit, and adjacent systems do not receive authority merely because they can call an API.

## Documentation map

Use the canonical documentation graph rather than treating this README as a complete technical specification:

- [`DOCUMENTATION.md`](DOCUMENTATION.md) — documentation authority and navigation.
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — system and bounded-context architecture.
- [`docs/product/prd-v0.4-approved.md`](docs/product/prd-v0.4-approved.md) — approved product requirements baseline.
- [`docs/architecture/domain-context-map.md`](docs/architecture/domain-context-map.md) — DDD ownership and dependency direction.
- [`docs/product-technical-gap-baseline.md`](docs/product-technical-gap-baseline.md) — current gaps, maturity, owners, and evidence limits.
- [`docs/adr/README.md`](docs/adr/README.md) — architecture decisions.
- [`docs/research/standards-and-literature.md`](docs/research/standards-and-literature.md) — standards and research basis.

## Contributing and support

Before changing product behavior, numerical authority, or cross-repository ownership, read [`AGENTS.md`](AGENTS.md), the architecture, applicable ADRs, and the product/technical gap baseline. Keep new behavior inside the bounded context that owns it, add executable evidence before promoting a claim, and update source, tests, documentation, and traceability together.

Use repository issues for reproducible product defects, standards gaps, scientific-contract gaps, and integration problems. An open PR is not support evidence for a capability until it is integrated and released under the repository's normal governance.

## License

TEPP source and documentation are licensed under the [MIT License](LICENSE). Third-party dependencies and external services retain their own licenses and terms; commercially incompatible inbound software must not be treated as acceptable merely because TEPP itself is MIT-licensed.
