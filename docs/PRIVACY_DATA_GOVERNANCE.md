# TEPP Privacy and Data Governance

**Status:** Accepted target governance baseline; no certification claim.  
**Last reviewed:** 2026-08-13

## 1. Objective

TEPP must preserve analytical utility for psychometrics, temporal analysis, authorship, event provenance, and multiple-membership modeling while preventing uncontrolled disclosure or secondary use. Blanket PII masking is therefore not the default architecture. Controls are based on authorization, purpose, separation, encryption, minimization, retention, and audit.

## 2. Data classes

| Data class | Examples | Default handling |
|---|---|---|
| raw source evidence | reports, attachments, exact text spans | encrypted protected object; restricted access; immutable version identity |
| direct identity | person name, email, employee/customer identifiers | separate identity boundary where feasible; least-privilege access |
| contextual relationship | author, department, customer/partner/competitor role, project | treated as sensitive derived/relational data; time- and purpose-scoped |
| pseudonymous analytical ID | document/entity/event/model UUID | preferred in compute artifacts and logs |
| sensitive derived result | inferred topic, event, role, latent score, factor path | protected at least as strongly as source classification warrants |
| model/provider payload | approved evidence spans, schemas, prompts | minimum necessary disclosure; provider policy and region recorded |
| operational telemetry | latency, failures, resource metrics | no raw source/secret by default |
| audit/provenance evidence | actor/purpose/action/outcome/digests | append-oriented bounded evidence; avoid unnecessary raw content |

## 3. Authorization model

Every protected read, model submission, export, administrative action, and re-identification operation is evaluated using at least:

- tenant/resource scope;
- authenticated service/user identity;
- role and allowed action;
- declared processing purpose;
- allowed fields/source classes;
- valid time/lifetime of the grant;
- region/provider constraints where configured;
- export/re-identification privilege;
- auditable decision evidence.

Future persistent authorization entities must use descriptive multiword `snake_case` names such as `access_grant`, `purpose_binding`, `identity_mapping`, `retention_policy`, and `export_approval`.

## 4. Identity separation

When the scientific task does not require the direct identity string, TEPP should model with an opaque `entity_record_id` and keep the direct identity mapping in a separately encrypted/authorized store or host-owned identity service. The in-memory `encrypted_mapping` envelope seals source identity so analytical, log, and model-artifact purposes cannot recover plaintext. Persistence of that envelope waits for a later migration. The mapping may not be copied into model artifacts, ordinary logs, LLM prompts, or public exports merely for convenience.

Identity separation must not erase scientifically relevant time-varying memberships. The model may retain that opaque entity A authored document X and belonged to department Y at time T while the direct real-world name remains outside the ordinary compute artifact.

## 5. LLM disclosure policy

LLM use is optional and bounded. The default provider payload is evidence-minimized:

1. transmit only the exact spans/structured evidence required for the current semantic or interpretation task;
2. omit credentials, unrelated document sections, hidden metadata, and direct identifiers unless purpose and policy explicitly require them;
3. bind provider, model, region, prompt hash, reasoning level, evidence IDs, and retention policy to the run;
4. treat returned content as untrusted and subject to deterministic verification;
5. allow local/private provider profiles for deployments that prohibit external disclosure.

`tepp_api::minimize_provider_payload` is the fail-closed adapter for this policy: a time-bounded purpose grant is required, identity mappings cannot ride on a provider offer, and ordinary disclosure logs record purpose and field-class flags without source text. Re-identification uses `tepp_api::disclose_identity_mapping` and is limited to scientific validation with an explicit grant flag.

`NVIDIA_NIM_API_KEY` is a development/test credential boundary, not authorization to send unrestricted production data.

## 6. Retention, deletion, and legal hold

Retention is policy-driven per data class and purpose. Raw evidence, derived models, caches, provider payloads, exports, and audit records may have different retention periods. Deletion workflows distinguish:

- logical revocation and identity tombstones from active analysis (`cache_export_removal` does not drop a document from analysis);
- removal of mutable caches and exports;
- deletion/tombstone of identity mappings where legally permitted;
- immutable audit evidence that records the deletion action without retaining unnecessary raw PII;
- legal or contractual hold that prevents deletion and records its authority.

Historical model reproducibility cannot silently override an approved deletion obligation. If source deletion makes exact reproduction unavailable, TEPP records that limitation rather than restoring deleted data from an ungoverned copy. Migration `0007` (implemented-main via PR #45) persists `retention_policy`, `legal_hold`, `deletion_request`, and `evidence_tombstone` so an active hold blocks completed deletion and a tombstone blocks restore of the same document identity.

## 7. Tenant and service isolation

Standalone and MSA deployment profiles must preserve the same data-authority contract. No CWL service may read TEPP application tables directly. Integrations use versioned APIs/artifacts and explicit service identity. When PostgreSQL persistence is introduced, tenant-aware RLS/authorization and migration tests become release gates.

## 8. Data subject and customer operations

Where applicable to a deployment, the product/API should support auditable inventory, export, correction of authoritative metadata, processing restriction, purpose withdrawal, and deletion workflows. TEPP does not claim a universal legal basis; the host/deployer records jurisdiction- and contract-specific authority.

## 9. Logging and observability

Ordinary logs contain identifiers/digests sufficient for diagnosis without copying report text, credentials, raw model prompts, direct identity, or unrestricted relation graphs. Privileged forensic capture requires an explicit incident purpose, scoped access, retention, and audit trail.

## 10. Privacy validation

Required tests include cross-tenant denial, expired-purpose denial, re-identification-boundary checks, export authorization, provider payload minimization, raw-source log absence, deletion/retention behavior, audit replay, and derived-sensitive-data classification. Privacy controls must be tested with realistic author/customer/project/multiple-membership cases rather than only anonymous fixtures. The in-memory `provider_receipt` crate is the current disclosure-audit gate; persistence of receipts remains accepted-target.
Required tests include cross-tenant denial, expired-purpose denial, re-identification-boundary checks, export authorization, provider payload minimization, raw-source log absence, deletion/retention behavior, audit replay, and derived-sensitive-data classification. Privacy controls must be tested with realistic author/customer/project/multiple-membership cases rather than only anonymous fixtures. The in-memory `operational_log` crate is the current source-separation gate: `try_record` is the only recording API and inspects source text, source identity, and blanket-mask intent; a source-identity `&str` cannot become an analytical subject. `persistence_postgres` `audit_event` inserts call the same gate before SQL is rendered. Live HTTP and provider adapters remain accepted-target.
Required tests include cross-tenant denial, expired-purpose denial, re-identification-boundary checks, export authorization, provider payload minimization, raw-source log absence, deletion/retention behavior, audit replay, and derived-sensitive-data classification. Privacy controls must be tested with realistic author/customer/project/multiple-membership cases rather than only anonymous fixtures. The in-memory `derived_sensitivity` crate is the current inheritance gate; persistence of classifications remains accepted-target.
Required tests include cross-tenant denial, expired-purpose denial, re-identification-boundary checks, export authorization, provider payload minimization, raw-source log absence, deletion/retention behavior, audit replay, and derived-sensitive-data classification. Privacy controls must be tested with realistic author/customer/project/multiple-membership cases rather than only anonymous fixtures. The in-memory `provider_receipt` crate is the current disclosure-audit gate; persistence of receipts remains accepted-target.