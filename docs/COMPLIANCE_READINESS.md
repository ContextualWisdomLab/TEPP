# TEPP Compliance and Assurance Readiness

**Status:** Accepted readiness baseline; this document does not claim certification or attestation.  
**Last reviewed:** 2026-08-10

## 1. Scope

TEPP is designed so a future deployment can produce evidence useful for CSAP, SOC 2, ISO/IEC 42001, ISO/IEC 23894, NIST AI RMF, and related organizational assurance programs. Repository controls are product evidence inputs only; actual certification/attestation depends on the deployed service, organization, contracts, infrastructure, operating history, and independent assessment.

## 2. Current authoritative references

- **CSAP:** KISA Cloud Security Assurance Program under the Korean Cloud Computing Act; service scope includes relevant systems, facilities, organizations, and supporting services. The current KISA program distinguishes IaaS, SaaS, DaaS and graded assurance categories.
- **SOC 2:** AICPA Trust Services Criteria cover Security, Availability, Processing Integrity, Confidentiality, and Privacy. TEPP maps controls/evidence but does not self-attest.
- **ISO/IEC 42001:2023:** AI management system requirements for establishing, implementing, maintaining, and continually improving responsible AI management.
- **ISO/IEC 23894:2023:** guidance for AI-specific risk management integrated into organizational risk management.
- **NIST AI RMF 1.0 / NIST AI 600-1:** voluntary AI risk-management framework and Generative AI profile. NIST states AI RMF 1.0 is under revision; TEPP keeps the published 1.0 normative until a successor is published.

## 3. Readiness matrix

| Control/evidence family | TEPP repository/product responsibility | Deployment/organization responsibility |
|---|---|---|
| asset/config inventory | versioned code, model, schema, artifact, dependency and provenance identities | infrastructure/service inventory, ownership, CMDB |
| access control | purpose/tenant/service-aware authorization contract; no ambient cross-service DB access | IAM, MFA/SSO, joiner/mover/leaver, privileged access review |
| confidentiality/privacy | encryption-capable boundaries, minimum-provider disclosure, separate identity mapping, bounded logs | KMS/HSM, key ownership, legal basis, privacy notices, regional policy |
| processing integrity | immutable evidence, strict validation, true-parameter recovery, temporal leakage prevention, CPU/GPU parity | operational change control, approval workflows, incident response |
| availability | bounded resource use, CPU fallback, explicit degraded states, recovery requirements | HA topology, capacity, DR, provider contracts, on-call |
| audit/provenance | immutable/digest-bound run/artifact evidence, versioned manifests | centralized log retention, SIEM, auditor access, evidence retention |
| vulnerability/supply chain | pinned Actions, dependency policy, SAST/security gates, SBOM/provenance | vulnerability-management SLA, image/runtime scanning, patch/change operation |
| AI governance | model/LLM boundary, measurement claims, human/scientific review escalation, model provenance | AI inventory, accountable owners, impact/risk acceptance, training/policy |
| data lifecycle | classification, retention/deletion contract, protected derived data | contract/legal retention, deletion approval, backups/legal holds |
| third parties | provider identity/region/model recorded in run contract | vendor due diligence, DPA/SLA, cross-border transfer assessment |
| change/release | exact-head CI/security/scientific/release gates, CHANGELOG, rollback evidence | production promotion approvals, segregation of duties, release operations |

## 4. CSAP-oriented engineering evidence

For a future Korean public-cloud SaaS deployment, design and acceptance evidence should cover at least:

- scoped service boundary and asset inventory;
- role/privilege separation and administrator accountability;
- tenant and data isolation;
- network/egress and external model-provider boundaries;
- secure development, vulnerability remediation, supply-chain evidence;
- encryption/key-management ownership;
- logging, time synchronization, audit and incident evidence;
- backup/restore and continuity;
- data deletion/return and subcontractor/provider governance;
- deployment-specific penetration and configuration assessment.

TEPP repository documentation cannot substitute for the KISA application guide, evaluation criteria, or independent evaluation in force at the time of certification.

## 5. SOC 2-oriented evidence

TEPP's strongest direct contributions are to Security and Processing Integrity: fail-closed validation, evidence provenance, exact-head quality gates, authorization contracts, numerical/scientific acceptance, change traceability, and supply-chain controls. Availability, Confidentiality, and Privacy become assessable only after concrete production architecture, operating controls, retention, incident history, and organizational ownership exist.

## 6. AI governance evidence

AI-related releases must preserve:

- intended use and non-goals;
- model/provider/version/prompt/config identities;
- data provenance and approved processing purpose;
- scientific/psychometric validation appropriate to the claim;
- language/time/group invariance or explicit limitations;
- uncertainty, abstention, and unsupported-claim behavior;
- human/scientific escalation criteria;
- monitoring for drift, integrity, privacy and provider change;
- rollback/replacement evidence.

## 7. Evidence maturity

Use these states:

- `implemented-main`: repository/product control integrated on protected main with current required evidence;
- `active-PR`: implemented on an unmerged branch only;
- `accepted-target`: approved architecture not yet implemented;
- `deployment-owned`: cannot be evidenced until a concrete deployment/organization exists;
- `external-assurance`: requires an independent auditor/certifier/assessment.

Never map a target architecture statement directly to `compliant`, `certified`, or `attested`.

## 8. Assurance pack target

A release candidate intended for enterprise procurement should eventually emit an evidence bundle containing release SHA/tag, SBOM, provenance, dependency/license status, security-scan results, scientific validation summary, model/data lineage, migration/rollback evidence, backup/restore evidence where applicable, accessibility evidence, change log, threat/privacy risk updates, and deployment-specific control mappings.