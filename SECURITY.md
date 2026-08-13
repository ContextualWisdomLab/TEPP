# Security Policy

## Supported status

TEPP is pre-alpha. No release is currently supported for production use. Security reports are nevertheless handled as confidential defects.

## Reporting

Do not disclose a suspected vulnerability in a public issue. Use GitHub private vulnerability reporting or contact the repository owner through an established private channel. Include affected commit, reproducible steps, impact, and the smallest safe proof of concept.

## Canonical security and privacy documents

- `docs/THREAT_MODEL.md` — assets, trust boundaries, abuse cases, scientific-integrity threats, verification.
- `docs/PRIVACY_DATA_GOVERNANCE.md` — purpose-bound PII/data handling, identity separation, retention/deletion, provider disclosure.
- `docs/COMPLIANCE_READINESS.md` — CSAP/SOC 2/ISO/NIST readiness mapping without certification claims.
- `docs/API_CONTRACT.md` — standalone/MSA authority and service integration boundaries.

## Trust boundaries

- Uploaded reports, archives, metadata, hyperlinks, Unicode text, markup, event assertions, and document relations are untrusted.
- LLM outputs are untrusted and must pass exact-span, schema, size, depth, enum, provenance, authorization, and scientific-support checks.
- Documents cannot issue system instructions, enable tools, request network access, alter model policy, or expand service/model authority.
- Historical analysis is fail-closed when evidence availability time is unknown or exceeds the knowledge cutoff.
- Cross-tenant documents, embeddings, concept dictionaries, model artifacts, caches, identity mappings, and audit trails are isolated.
- Exported CSV/XLSX values are protected against formula injection; Office/PDF/SVG/HTML exports are sanitized and bounded.
- Other CWL services never gain TEPP data authority through direct application-table access; integration is through explicit versioned APIs/artifacts and scoped service identity.

## PII and sensitive derived data

TEPP does not use blanket masking as its primary privacy control because names, authorship, organizational roles, customer/partner/competitor context, projects, and longitudinal identity may be required for valid cross-classified or multiple-membership measurement. Instead use purpose-bound authorization, opaque analytical identifiers, separately protected identity mapping, encryption, selective disclosure, tenant isolation, retention/deletion, and auditable privileged access.

Derived event, relation, topic, factor, membership, or inferred-sensitive data is not automatically safe merely because direct identity strings were removed. Provider/model payloads contain only the evidence required for the authorized task.

## Secrets

Approved LLM live tests and autonomous development use the GitHub secret `NVIDIA_NIM_API_KEY`, mapped only to the minimum runtime variable needed by the selected provider. `COPILOT_GITHUB_TOKEN` is forbidden. Existing independent review-agent credentials are not renamed, copied, or repurposed.

Secrets must not appear in source, logs, prompts, traces, artifacts, test snapshots, issue text, or generated reports. Scheduled workflows fail closed when required credentials or inventory checks are unavailable.

## Supply chain

- Pin GitHub Actions by full commit SHA.
- Inventory Actions registry identities against the protected-main tree and disable reviewed orphans; do not trust deleted YAML absence as proof the control plane is clean.
- Lock dependencies and verify licenses and advisories.
- Generate SBOM, provenance, checksums, and reproducible release evidence.
- Use least-privilege workflow permissions and concurrency controls.
- Reject mutable remote code, unverified installer scripts, and implicit network downloads in scientific tests.
- Treat checkpoints, model adapters, concept dictionaries, calibration artifacts, and generated schemas as versioned supply-chain inputs requiring identity/provenance validation.

## Scientific integrity as a security property

Parameter recovery, uncertainty coverage, temporal leakage prevention, multilingual invariance, group fairness, relation/membership provenance, numerical parity, compositional-coordinate correctness, and causal-language restrictions are integrity boundaries. A result that silently violates them is treated as a security-relevant failure even when the software process exits successfully.

## Assurance boundary

Repository controls are evidence inputs for future assurance. TEPP does not claim CSAP certification, SOC 2 attestation, ISO/IEC 42001 certification, or legal/regulatory compliance merely because controls are documented or tested here. Concrete deployment controls and independent assessment remain required.
