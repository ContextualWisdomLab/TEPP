# Changelog

All notable changes to TEPP are documented here. The format follows Keep a Changelog and semantic versioning once the first implementation release is cut.

## [Unreleased]

### Added

- `tepp_api` LineageWeave temporal-context contract (v1): cutoff-safe event eligibility, deterministic event-time ordering, explicit non-causal association/gap boundaries, HTTPS interchange construction, and loopback listener handling at `POST /v1/temporal-context`; read-only context requests no longer require the write-only idempotency header, and no causal inference or completed-result service is included.
- `tepp_api` LineageWeave consumer-scoped analysis-run ingress: versioned, credential-free requests use a published consumer identity and isolate idempotency by consumer, tenant workspace, and opaque caller key; the one-shot restack workflow is removed after the protected-main merge is verified.
- ADR 0018 records the consumer-scoped analysis-run ingress, its in-memory loopback maturity, and the persistence boundary required before production use.
- ADR 0020 records the credential-free bounded LineageWeave project-history service boundary and keeps source authorization with LineageWeave while TEPP owns temporal validation and deterministic projection.
- `tepp_api` project-history wire-size symmetry (ADR 0019): request and projection serialization enforce the shared 256 KiB limit, and generated projections fail closed before returning when their deterministic response would exceed it.
- `summarizes_edge` identity gate: summaries may point to earlier event time without becoming state transitions or reusing source-document identity; recovery tests outperform collapsing every summary to the source.
- `outcome_order` identity gate: `input_to` and `process_to` require strict forward event-time rank, while `outcome_of` remains non-transition provenance; recovery tests outperform collapsing every kind to `input_to`.
- `retrospective_edge` identity gate: retrospective reporting may point to earlier event time but cannot become a state transition or a translation; recovered reporting kinds match known truth at a higher computed rate than collapsing every report to a contemporaneous forward report (ADR 0002/0003).
- `payload_bound` identity gate: documents, serialized records, model checkpoints, and LLM outputs stay untrusted until identity, provenance, size, and depth validate; recovered accept/reject flags match known truth at a higher computed rate than accepting every payload (ADR 0008/0013).
- `inferred_status` identity gate: inferred relations cannot be promoted to observed evidence or to state transitions; recovered observed/inferred labels match known truth at a higher computed rate than treating every status as observed (ADR 0003).
- `support_edge` identity gate: support, contradiction, summary, and `outcome_of` edges cannot become state transitions; recovered evidential kinds match known truth at a higher computed rate than collapsing every kind to support (ADR 0002/0003).
- `system_clock` identity gate: event, assertion, document, availability, and knowledge-cutoff time cannot stand in for system time; recovered system stamps match known truth at a higher computed rate than treating every stamp as event time (ADR 0002).
- `event_clock` identity gate: assertion, system, document, and availability time cannot stand in for event/valid time; recovered event stamps match known truth at a higher computed rate than treating every stamp as assertion time (ADR 0002).
- Dependabot Rust toolchain updates now use a seven-day cooldown so newly published versions receive a bounded review window before automated proposals.
- `assertion_clock` identity gate: event, system, document, and availability time cannot stand in for assertion time; recovered assertion stamps match known truth at a higher computed rate than treating every stamp as event time (ADR 0002).
- `cutoff_clock` identity gate: event time, system time, and availability time cannot stand in for knowledge cutoff; recovered cutoff stamps match known truth at a higher computed rate than treating every stamp as availability time (ADR 0002).
- `available_clock` identity gate: event time and system time cannot stand in for availability time; recovered availability stamps match known truth at a higher computed rate than treating every stamp as system time (ADR 0002).
- `document_clocks` six-clock gate: a document analytical row cannot omit assertion time or document time, and event/system time cannot stand in for those clocks; recovered completeness flags match known truth at a higher computed rate than treating every row as complete (ADR 0002/0013).
- `revision_order` system-time gate: a higher document revision number cannot carry earlier or equal system time; recovered order flags match known truth at a higher computed rate than accepting every pair (ADR 0002/0013).
- `encrypted_mapping` purpose-bound AES-256-GCM envelope: source identities are sealed with an operating-system-generated nonce and analytical/key identifiers as authenticated associated data, with a 1 MiB resource bound, so analytical, log, and model-artifact purposes cannot recover plaintext; recovered identities match known truth at a higher computed rate than collapsing every mapping to one name. Persistence and KMS wait for a later migration (ADR 0009).
- `citation_edge` provenance gate: citation, translation, revision, and retrospective-report edges may point to the past but cannot become input-process-outcome transitions; recovered kinds match known truth at a higher computed rate than collapsing every edge to citation (ADR 0002/0003).
- `psychometric_fit` CPU `f64` ESEM/DSEM fit: exploratory OLS recovers known cross-loadings from admitted log-ratio or logistic-normal coordinates with computed RMSE below a zero-loading collapse; reverse or zero event-time lagged paths fail closed; a good global fit cannot reclassify formative or network constructs as reflective (ADR 0005). No new migration number (`#45` still owns `0007`).
- `subevent_containment` parent-window gate: a half-open subevent interval that starts before or ends after its parent cannot attach; recovered containment flags match known truth at a higher computed rate than accepting every child (ADR 0003).
- `prediction_contradiction` promotion gate: `temporal_core` Allen classification refuses `before`/`after` as contradiction and `meets`/`met_by` as unsupported adjacency; `refuse_promotion` and `require_observed_coverage` refuse partial overlap that leaves unmatched predicted mass; `refuse_contradiction_or_adjacency` is the weaker contradiction/adjacency filter only; evidence available after the knowledge cutoff is ineligible. Label agreement is not RMSE recovery (ADR 0002, ADR 0016). Canonical docs name the crate, not a pull-request number, as the landable authority; `scripts/validate_documentation.py` fail-closes on `landable coverage gate is PR #N` and inverted or paraphrased forms (`PR #N is the landable coverage gate`, `the landable gate is PR #N`, `coverage-authority landing PR #N`, `merge PR #N as the coverage-authority`) including drafts #93, #94, #97, #101, #102, #104, #108, #109, #111, and #112. The hourly queue lock also fail-closes when those drafts are omitted from Keep-unmerged sentences, when a Keep-unmerged sentence is negated, or when the naruon live-HTTP *subject* is not PR #107 with #87 and #105 kept unmerged.
- `provider_receipt` disclosure audit: a receipt records purpose and field codes sent to a model provider; source text, source identity, and blanket PII masking fail closed; recovered field codes match known truth at a higher computed rate than a collapsed set (ADR 0009).
- `tepp_api` corpus-split leakage-audit manifest v1: cutoff exclusion counts, relation-component and partition digests, governed link-kind vocabulary, and a canonical `SHA-256` that binds to `corpus_split_manifest` without exporting source text.
- `persistence_postgres` `audit_event` inserts call `operational_log::try_record` before SQL is rendered: author/customer/project source text, source identity, and blanket-mask grants cannot enter `INSERT INTO audit_event`; clear inspection still persists a validated action code (ADR 0009; ISO/IEC 29100:2024). No new migration number. `OperationalLogRecord::new` stays crate-private.
- `operational_log` source separation: `try_record` is the only recording API and inspects source text, source identity, and blanket-mask intent before creating a line; `OperationalLogRecord::new` is crate-private; a source-identity `&str` cannot become an `AnalyticalSubject`; privileged-export / identity-mapping / diagnosis action codes keep author, customer, and project memberships distinct; replayed lines match known truth at a higher computed rate than a collapsed single-action or collapsed-subject log (ADR 0009; ISO/IEC 29100:2024). The live docstring crate-root count is bound to `EXPECTED_CRATES` so the eleventh crate cannot fail a hard-coded `10`.
- `service_tls` production TLS bind gates: non-loopback binds require rustls PEM material, loopback HTTP is development-only, orchestrator live ports refuse loopback plaintext, and table-access host labels fail closed. `TlsBindRequest` Debug output redacts certificate and private-key PEM. Recovered bind decisions are computed from `authorize_production_tls` / `authorize_orchestrator_live_port` outputs and match known truth at a higher rate than a collapsed production grant (ADR 0011).
- `derived_sensitivity` inheritance: topic, factor, and relation artifacts keep the source sensitivity class; unknown kind codes fail closed on both `inherit_sensitivity` and `DerivedArtifact::try_new`; derivation and blanket PII masking cannot declassify to public; paired kind-and-class recovery matches known 3×3 synthetic truth at a higher computed rate than a public collapse (ADR 0009; GDPR Art. 4(1)/Recital 26; WP 136).
- `longitudinal_core` within/between decomposition: unit means stay between-unit components, occasion residuals stay within-unit change, and recovered components match known truth with lower computed RMSE than a grand-mean pooled collapse.
- `topic_lineage` global P0 topic identity: activity may become dormant or reactivated without minting a new identity, and recovered identities match known truth at a higher computed rate than mint-on-reactivate replacements.
- `interpretation_gateway` evidence-bounded LLM interpretations: proposals must cite at least one evidence span, remain hypothetical, cannot become estimator results or observed facts, and a cited interpreter records a lower computed unsupported-claim rate than uncited promotion.
- `model_selection` candidate-`K` gates: statistical candidates require `K >= 2` and finite held-out log-likelihood/complexity, a Pareto front excludes dominated alternatives, LLM votes cannot define the numerical optimum, and selected `K` recovers known truth with computed RMSE.
- `event_core` mention-confidence Brier score: known-truth binary outcomes recover a computed Brier of 0 for perfect forecasts and 0.25 for constant 0.5, with empty or mismatched streams failing closed.
- `membership_core` nested ICC: CPU `f64` unbalanced ANOVA recovers a known cluster ICC and refuses to treat cross-classified or multiple-membership designs as a single hierarchy (ADR 0003).
- `persistence_postgres` typed `text_segment` SQL: insert/lookup of exact UTF-8 half-open byte spans on the existing `0006` table, cutoff-eligible document reads (`available_time <= knowledge_cutoff`), and live recovery of a known `hello` span. No new migration number (`#45` still owns `0007`).
- Hourly contextual-orchestrator discovery records all provider models but routes OpenCode only through general-chat candidates, excluding embedding, image, reranker, transcription, moderation, safety, and other endpoint-only identifiers before price selection.
- Live `docs/product-technical-gap-baseline.md` mapping operator-visible gaps to
  protected-main maturity, exact current PR/issue state, stacked delivery order,
  and closure evidence; placeholder-only issues #161 and #162 were closed as
  queue hygiene. The documentation validator requires a dated UTC snapshot, a
  40-character protected-main SHA, an exact-head inventory matching the declared
  open-PR count, and operator-gap closure evidence, and it rejects affirmative
  queued-Checks-as-implemented-main claims even when wrapped across a line
  break, and it rejects an unrelated `not` in the same span (`queued Checks are
  not required; this PR is implemented-main`). Only never/do not/does not/
  cannot/must not plus promote/treat/make/mean counts as a promotion denial.
- `checkpoint_authority` estimator gate: a model checkpoint remains an untrusted run artifact until identity, canonical `SHA-256`, and model-run provenance validate, and it cannot replace the CPU `f64` estimator or promote a scientific claim; recovered roles match known truth at a higher computed rate than collapsing every artifact to the estimator (ADR 0001/0014).
- `persistence_postgres` retention/deletion/legal-hold (migration `0007`): policy rows, legal holds that block completed deletion, evidence tombstones without raw-source restore, analysis exclusion only for `logical_revocation`/`identity_tombstone` (not `cache_export_removal`), and deletion requests bound to the cited retention policy's tenant/class/purpose.
- `event_core` now requires and retains `EventEvidenceLayer::PromotedTransition` when constructing an `EventInstance`; every other layer is rejected at the promotion boundary, and TDT story classification uses a caller-owned hash set for expected constant-time membership checks.
- `event_core` ADR 0016 evidence-status gates: TDT detections and CHRONOS predictions cannot admit a forward state transition; first-story detection scores miss/false-alarm rates against a known story stream (Allan 2002 task).
- `tepp_api` naruon live loopback HTTP/1.1 listener: `serve_one` installs a read/write deadline, requires a loopback `Host`, refuses `Transfer-Encoding` and NIM/proxy credential headers, parses `knowledge_cutoff` as RFC 3339 and refuses a future cutoff, keys analysis-run idempotency by tenant plus key, and proves both analysis-run and export POSTs over a real `TcpStream`. Not a production TLS/`$PORT` service (ADR 0011).
- `tepp_api` adaptive orchestration router (ADR 0010): versioned `direct`/`verify`/`committee`/`conductor`/`abstain` selection from CPU `f64` risk, ambiguity, evidence, and token-budget inputs; recorded stages, recursion, decomposition, access lists, and role-specific reasoning effort; fail-closed document-controlled policy/access/credentials; LLM plans remain proposals under deterministic statistical authority; comparable-budget ablation requires a direct baseline; credential-free contextual-orchestrator binding. Live NIM HTTP remains accepted-target.
- `tepp_api` purpose-bound provider-payload minimization: time-bounded `PurposeGrant` evaluation, fail-closed expired/not-yet-valid/inverted/cross-tenant/impossible-calendar denial, semantic UTC calendar validation, refusal to copy identity mappings into model-provider payloads or ordinary logs, preservation of opaque analytical identifiers and membership roles (no blanket PII mask), a separately authorized scientific re-identification path, and an internally bound FIPS 180-4 SHA-256 audit digest appended through `ReidentificationAuditSink` before disclosure.
- `persistence_postgres` backup/restore integrity: restored snapshots stay unusable until tenant, canonical `SHA-256`, knowledge-cutoff eligibility, temporal window order, and append-only triggers revalidate; SQL probes raise `restore integrity failed` (ADR 0013).
- `persistence_postgres` concurrent document-write stress: atomic revise `DO` block that requires exactly one open `system_to` close, SQLSTATE mapping onto `ConcurrentWriteConflict` / `DuplicateDocumentRecord`, and live multi-session insert/revise/append-only proofs. No new migration number.
- `tepp_api` naruon HTTP interchange: versioned `https` POST contracts for analysis-run create and modular export authorization that refuse table-access URLs, review/Copilot credential headers, reserved standard-header redefinition, principal-only export idempotency keys, and lexical inference claims (ADR 0011).
- `persistence_postgres` audit-event SQL contracts: append-only insert that refuses empty, oversized, or hostile `action_code` values before SQL is rendered.
- `network_analysis` compositional cluster gates: raw topic proportions cannot be treated as Euclidean coordinates; recovered clusters are scored with label-invariant pair precision and recall against known truth.
- `persistence_postgres` event-instance SQL contracts: bitemporal insert and as-known-at lookup that refuse inverted valid/system windows and hostile type/lifecycle labels before SQL is rendered.
- `persistence_postgres` event-mention SQL contracts: mention identity cannot equal the instance it supports; confidence must be finite and in `(0, 1]`.
- `persistence_postgres` event-relation SQL contracts: closed ERD transition/provenance vocabulary bound to `transition_edge`, fail-closed unknown types and transition self-loops, live insert of `causes`/`references`.
- `persistence_postgres` source-artifact SQL contracts: append-only insert and primary-key lookup that refuse non-canonical `SHA-256` digests, negative sizes, and hostile media-type or object-store labels before SQL is rendered; identical-identity retries are `ON CONFLICT DO NOTHING` plus a stored-row match assertion, and a same-id payload change fails closed as `ConflictingSourceArtifact`.
- `persistence_postgres` typed membership assignment (migration `0006`): `entity_record`, `project_record`, and `text_segment` plus exactly-one observed-unit and target constraints that replace the polymorphic `membership_target_id` stub, with SQL insert/lookup, fail-closed inverted-window and backslash-label refusal, and live proof that one document persists two entity memberships and one project membership.
- Actions workflow fleet auditor (`scripts/actions_workflow_fleet.py`): paginated registry inventory bound to the exact default-branch SHA/tree, classification of present/orphan/disabled/GitHub-dynamic identities, and fail-closed orphan disable that confirms GitHub's official `disabled_manually` state.
- `persistence_postgres` temporal interval ordering migration (`0005`): multi-word CHECK constraints on `document_record`, `event_instance`, and `membership_assignment` that reject inverted valid/system windows and non-positive document revisions while preserving open-ended NULL upper bounds and equal point bounds; catalog validation and live inverted-window proof.
- `persistence_postgres` append-only immutability migration (`0004`): `reject_append_only_mutation`, statement-level `BEFORE UPDATE OR DELETE OR TRUNCATE` triggers on identity/manifest tables, `REVOKE UPDATE`/`DELETE`/`TRUNCATE` from `tepp_app_runtime`, executable DDL/rollback contracts, and live representative mutation proof.
- `persistence_postgres` model-run artifact chain: migration `0003_model_run_artifact_chain` for append-only `corpus_split_manifest`, `model_run`, and `model_artifact` with FORCE RLS; SQL insert/lookup contracts and live repository methods binding runs to reproducibility manifests and optional splits.
- `persistence_postgres` append-only reproducibility-manifest SQL contracts and live repository methods (`insert_reproducibility_manifest`, digest/id lookup) with fail-closed SHA-256 and commit identity validation for `reproducibility_manifest`.
- `persistence_postgres` tenant row-level security: migration `0002_tenant_row_level_security`, `tepp_app_runtime` role, session GUC `tepp.current_tenant_record_id`, multi-word isolation policies with FORCE RLS, session helpers, contract validation, and live isolation proof under `TEPP_LIVE_POSTGRES=1`.
- `persistence_postgres` live PostgreSQL CI: `live-postgres` job with Postgres 16 service, `TEPP_LIVE_POSTGRES=1` gate, and integration coverage for pool open, foundation+RLS migrations, document insert/revise/as-of, audit SQL, and tenant isolation.
- Repository release evidence tooling: `scripts/release_evidence.py` generates CycloneDX 1.5 SBOM, exact-head provenance, and SHA-256 checksums from `Cargo.lock`/`Cargo.toml`, with fail-closed validation and CI generation on every quality gate.
- `persistence_postgres` `live-sqlx` feature: real `SQLx`/`PgPool` open/execute behind validated `DATABASE_URL` and `LiveSqlxPoolOptions`, with offline/live executor backends and CI coverage exclusion for the transport module.
- `persistence_postgres` live pool open gate: validated `LiveSqlxPoolOptions`, fail-closed `open_live_sqlx_pool` / `LiveSqlxPool` (`SqlSession`) with offline test backend; optional `live-sqlx` attaches real `SQLx`/`PgPool` after `DATABASE_URL` validation.
- `validation_core` recovery metrics: parameter matching, RMSE/bias with standard errors, interval coverage with Wilson bounds, relation-edge precision/recall, temporal-order accuracy, Monte Carlo summaries, and SE-aware acceptance gates with machine-readable reports.
- `tepp_api` versioned analysis-run DTOs, content-redacting error envelopes, reproducibility manifests, JSON-LD and GraphML export contracts, purpose-bound export authorization (no blanket PII masking), plus committed schemas/examples.
- `relation_graph` forward-only state-transition DAG with past-pointing provenance edges and cycle rejection.
- `tepp_simulation` deterministic truth-corpus generator with delayed reporting, multilevel memberships, method-effect variants, relation noise, and digest-bound truth manifests.
- `corpus_split` leakage-safe knowledge-cutoff snapshots, relation-connected co-partition groups, rolling-origin windows, and group-normalized ESS weight contracts.
- `persistence_postgres` live SQL port: `SqlSession` transport, migration batch applicator, document/audit SQL contracts, `LiveDocumentRepository`, and fail-closed `DATABASE_URL`/`LiveSqlxConfig` gate for SQLx pool wiring (optional `live-sqlx` driver attaches `PgPool`).
- `membership_core` Kish effective sample size, design effect, and group-normalized ESS helpers for multiple-membership estimation inputs.
- Credential-separated hourly NVIDIA NIM/OpenCode product-development workflow (issue #2): proposal, independent verification, and late Maintainer-App publication with `NVIDIA_NIM_API_KEY` only for model work.
- Hourly product-development queue gate now fails closed when either an open pull request or an open issue exists, including a second queue check immediately before publication.
- Documented modular naruon consumer contract for TEPP analysis-run and export surfaces, with a committed example request payload.
- Documented contextual-orchestrator interpretation port boundary and credential separation for TEPP.
- Foundation validation/release-readiness ledger at `docs/validation/temporal-event-foundation.md` tracking capability maturity and scientific acceptance gates.
- Research doctoring for multilevel/multiple-membership measurement and atomistic fallacy prevention.
- `persistence_postgres` bitemporal foundation: multi-word migration contracts, knowledge-cutoff eligibility, and in-memory as-known-at / as-valid-at document replay (live SQLx/PostgreSQL execution remains accepted-target).
- `event_core` mention/instance separation with explicit promotion, typed roles, event-time validity, and fail-closed mention-as-instance refusal.
- `membership_core` time-varying weighted multiple-membership network with contextual roles, event-time validity, and atomistic-fallacy prevention contracts.
- Bounded Allen interval algebra and path-consistency reasoner in `temporal_core` (Task 4; PR #9), with identity-isolated variables, resource budgets, inverse/composition, and conservative provenance.
- Approved Temporal Event Psychometrics Platform PRD v0.4 baseline.
- Canonical technical documentation map, TRD, UML/scientific runtime views, logical/planned ERD, scientific test strategy, operability/recovery guide, and requirements/research/evidence traceability with explicit implementation maturity.
- Whole-conversation documentation fitness assessment plus canonical API/MSA contract, threat model, privacy/data-governance contract, CSAP/SOC 2/ISO/NIST assurance-readiness mapping, and adaptive LLM orchestration/test-time-compute contract.
- ADR policy separating architectural **Decision status** from **Implementation maturity**, defining partial supersession, and making the ADR index the canonical decision-ownership map.
- ADR 0009 for purpose-bound PII governance without blanket masking, ADR 0010 for adaptive direct-versus-multi-agent LLM orchestration, and ADR 0011 for standalone/modular CWL service authority.
- ADR 0012 for Temporal Relational Shared-Latent Topic Measurement (TRSL-TM), global topic identity/backend compatibility/method-effect/model-selection authority.
- ADR 0013 for bitemporal persistence, immutable reproducibility/run/split manifests, relation-aware partitions, recovery, and PostgreSQL adapter authority.
- ADR 0014 separating accepted design, protected-main implementation, scientific/product claim promotion, and release evidence authority.
- ADR 0015 separating autonomous model proposal, deterministic verification, publication, independent review, and merge/release authority.
- ADR 0016 separating Event Ontology observation, TDT detection/tracking, CHRONOS schema prediction, symbolic temporal consistency, and promoted transition authority.
- Added APA 7 research traceability for ICLR 2026 TRINITY and Conductor, the 2026 Sakana Fugu technical report, ISO/IEC 42001:2023, ISO/IEC 23894:2023, NIST AI RMF/GAI Profile, AICPA Trust Services Criteria, and KISA CSAP guidance.
- Eight-phase delivery roadmap and Temporal/Event Foundation implementation plan.
- Immutable evidence, six-clock temporal semantics, interval reasoning, event ontology, typed relation graph, and time-varying multiple-membership contracts.
- Shared-latent multilingual topic measurement architecture with native lexical channels and language-profile validation.
- Longitudinal ESEM/DSEM and continuous-time structural modeling requirements.
- Rust-first CPU `f64`, multithreaded CPU, GPU, VRAM-adaptive streaming, and CPU/GPU parity requirements.
- Topic correlation, consensus clustering, TDT, CHRONOS, and evidence-grounded LLM interpretation requirements.
- APA 7th research traceability, source archive manifests, ADRs, governance, security, and contribution contracts.
- Hourly centralized PR-maintenance workflow and a documented requirement for a future credential-separated NVIDIA NIM/OpenCode product-development loop.
- Rust 1.97.1 virtual Cargo workspace with eleven explicit modular foundation crates.
- Repository contract, public-rustdoc, line-coverage, and nightly branch-coverage gates.
- Pinned `cargo-nextest` 0.9.140, `cargo-llvm-cov` 0.8.6, `cargo-deny` 0.19.7, and Coverage.py 7.15.2 quality tooling.
- Task 1 architecture decision and workspace-foundation validation report.
- Version-keyed, executable-only GitHub Actions caches for pinned Rust quality tools.
- Immutable `evidence_core` records with independent RFC 9562 `UUIDv7` identities, canonical `SHA-256` content digests, owned source bytes and UTF-8 text, exact byte/Unicode-scalar spans, and bounded page-layout coordinates.
- Strict versioned JSON wire contracts for artifacts, documents, exact spans, and nested page locations without exposing private domain storage.
- ADR 0008 and APA 7 doctoring for evidence identity, hashing, JSON interchange, UTF-8 boundaries, Unicode segmentation limits, and future W3C PROV integration.
- Same-run exact missing-line and missing-branch diagnostics for failed 100% Rust coverage gates.

### Changed

- The LineageWeave temporal-context read exchange no longer emits a fabricated
  `idempotency-key`; that header remains reserved for retryable write/export
  operations with a caller-owned operation key.
- `tepp_api` project-history requests and projections now share the strict
  `temporal_core` RFC 3339 parser and nominal `KnowledgeCutoff` boundary,
  rejecting unknown offsets and other timestamp forms that the transport
  parser could otherwise accept.
- Coverage validation now ignores LLVM rows for multiline call and iterator
  syntax that have no independently executable source coordinate, while
  retaining the authored-line 100% gate.
- Removed the temporary PR-155 review-repair workflows and source-fix helper after the bounded repair; subsequent changes use the normal reviewed branch path.
- Pinned Rust branch-coverage workflows to `nightly-2026-08-21`, which is newer than the workspace Rust 1.97.1 MSRV and avoids the previous nightly/MSRV mismatch.
- Applied the documented `sqlx_live.rs` authored-coverage exclusion to the hourly release gate so live-PostgreSQL success-path coverage is not reported as a false source failure.
- Removed unreachable duplicate Naruon host-control validation because the shared `require_nonempty` boundary already rejects C0/C1 controls; retained a C1 regression case alongside the existing C0 case.
- Kept one maturity row per capability in the traceability matrix while recording the active provider-receipt evidence without duplicating or downgrading existing capabilities.
- `tepp_api` corpus-split manifest validation now rejects governed link-kind arrays that are unsorted or duplicated, keeping untrusted JSON aligned with the schema's unique canonical representation and preventing equivalent audits from receiving different valid digests.
- Grounded `derived_sensitivity` doctoring on GDPR Article 4(1)/Recital 26 and WP29 Opinion 4/2007 (WP 136) as read from the official texts, and replaced the withdrawn ISO/IEC 29100:2011 use-limitation overclaim with the current 29100:2024 catalogue edition without quoting unread clause text.
- Added APA 7th method citations (Allen 1983; ISO 24617-1:2012; Hobbs & Pan 2017; Fox & Glas 2001; AERA/APA/NCME 2014; Blei & Lafferty 2006; Roberts et al. 2014, 2019; Chang & Blei 2009; Mimno et al. 2009; Asparouhov & Muthén 2009; Asparouhov et al. 2018; Marsh et al. 2014; Aitchison 1982; Allan 2002; Li et al. 2021; Anagnostopoulos et al. 2013) into ADRs 0002–0005, 0012, and 0016, plus TRACEABILITY/ARCHITECTURE/TRD method rows. Clarified that TRSL-TM is the product contract, STM-style logistic-normal is the reference family, ESEM/DSEM/TDT/CHRONOS remain accepted-target, and merged PRs #8/#9—not superseded drafts #5/#6—are the protected-main temporal lineage.
- Refreshed the live gap-baseline inventory to the 2026-08-24T05:41:54Z GitHub
  snapshot (118 open PRs / 48 drafts / 12 issues; protected-main
  `c45be17a9dbce95ef81cee230e9d128abc7160ac`), binding each operator-gap current
  head SHA to that exact-head register, including #201 `6afd650667e1` (RFC 5646
  cited once; first GAP-005 slice, not implemented-main), stacked drafts
  #202–#204, and #164 `ff2e645b1785` as the predecessor register head. Duplicate
 PR #179 remains closed. Stacked-merged heads and queued Checks are not
 implemented-main.
- Clarified ADR 0001 so it owns Rust-first numerical/reference-backend authority while ADR 0011 owns cross-service MSA/service authority.
- Clarified ADR 0006 so it owns GPU/VRAM and model-credential boundaries; ADR 0010 now owns LLM orchestration policy and ADR 0015 owns autonomous repository-write/review/merge authority.
- Expanded ADR 0002–0005 and 0009–0011 with explicit implementation maturity, alternatives, failure/recovery, compatibility/migration, verification, and rollback/supersession boundaries where they were previously implicit.

### Security

- Naruon interchange refuses `x-apikey`, `x-api_key`, and hyphenated `api-key` credential-header aliases, not only `x-api-key`.
- GitHub HTTPS fleet transport maps request, response, and close-path network exceptions to `upstream_unavailable` without leaking raw provider exception text.
- Disabled-state classification and live disable confirmation now accept GitHub's official `disabled_manually`, `disabled_fork`, `disabled_inactivity`, and `deleted` registry states so orphan bootstrap/repair identities can be retired without name-only heuristics.
- Prohibited `COPILOT_GITHUB_TOKEN` and reserved `NVIDIA_NIM_API_KEY` for approved LLM test and development workflows.
- Defined purpose-bound PII access, opaque analytical identifiers, separately protected identity mapping, selective model-provider disclosure, retention/deletion, and privileged audit controls instead of destructive blanket masking.
- Added explicit threat classes for temporal leakage, relation/membership poisoning, model/artifact poisoning, numerical divergence, cross-tenant disclosure, prompt injection, evidence substitution, resource exhaustion, and scientific-integrity failures.
- Removed the bootstrap branch's credential-co-resident OpenCode workflow: no model process may receive repository-write authority, and scheduled product development remains disabled until proposal, independent verification, and late publication authority are separated across fresh jobs.
- Removed completed bootstrap materializers, encoded payload fragments, readiness sentinels, and push probes from the reviewable tree.
- Required full-commit GitHub Action pins, minimum permissions, concurrency controls, immutable audit evidence, SBOM, and provenance.
- Kept ordinary Rust CI free of LLM and reviewer credentials and disabled persisted checkout credentials.
- Refused to cache mutable Cargo registry, Git source, or target trees; cached quality binaries are keyed and checked by exact version.
- Copied caller-provided source bytes and document text before acceptance and kept validated evidence fields private.
- Made empty, oversized, malformed-digest, invalid UTF-8-boundary, coordinate-mismatch, cross-document, nonfinite-geometry, and out-of-page evidence fail closed with content-redacting errors.
- Rejected malformed or extended wire payloads, unsupported schema versions, invalid identifiers and byte values, digest/content substitution, stale document ownership, and invalid nested geometry during reconstruction.

### Quality

- Required 100% production line and branch coverage and complete public API docstrings.
- Required true-parameter recovery, RMSE, bias, interval coverage, temporal leakage, graph recovery, invariance, and CPU/GPU parity evidence.
- Expanded documentation contracts to require the canonical threat/privacy/assurance/API/orchestration/fitness documents, ADR policy, and every numbered ADR present in the canonical index to remain indexed and structurally complete.
- Added deterministic validation that ADR files and the index have identical decision numbers and that every ADR declares valid decision status, implementation maturity, supersession scope, core decision sections, verification, and rollback behavior.
- Added 100% statement and branch coverage for the repository quality-gate scripts.
- Made a zero executable-code coverage denominator explicit for the skeleton-only slice rather than treating it as evidence of implemented behavior.
- Denied warnings, missing public documentation, and unsafe Rust across the workspace.
- Added known digest vectors, mutation detection, hostile multibyte Unicode, exact-coordinate, page-boundary, stable-error, and invalid-input regression tests for the first evidence slice.
- Added strict wire round trips, unknown-field and version rejection, digest reconstruction, configured-limit, hostile JSON, and generated multilingual span tests.

The repository has not yet cut a stable implementation release, so no compare reference is published for `[Unreleased]` yet.
