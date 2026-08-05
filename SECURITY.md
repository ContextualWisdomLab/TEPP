# Security Policy

## Supported status

TEPP is pre-alpha. No release is currently supported for production use. Security reports are nevertheless handled as confidential defects.

## Reporting

Do not disclose a suspected vulnerability in a public issue. Use GitHub private vulnerability reporting or contact the repository owner through an established private channel. Include affected commit, reproducible steps, impact, and the smallest safe proof of concept.

## Trust boundaries

- Uploaded reports, archives, metadata, hyperlinks, Unicode text, markup, event assertions, and document relations are untrusted.
- LLM outputs are untrusted and must pass exact-span, schema, size, depth, enum, provenance, and authorization checks.
- Documents cannot issue system instructions, enable tools, request network access, or alter model policy.
- Historical analysis is fail-closed when evidence availability time is unknown or exceeds the knowledge cutoff.
- Cross-tenant documents, embeddings, concept dictionaries, model artifacts, caches, and audit trails are isolated.
- Exported CSV/XLSX values are protected against formula injection; Office/PDF/SVG/HTML exports are sanitized and bounded.

## Secrets

Approved LLM live tests and autonomous development use the GitHub secret `NVIDIA_NIM_API_KEY`, mapped only to the minimum runtime variable needed by the selected provider. `COPILOT_GITHUB_TOKEN` is forbidden. Existing independent review-agent credentials are not renamed, copied, or repurposed.

Secrets must not appear in source, logs, prompts, traces, artifacts, test snapshots, issue text, or generated reports. Scheduled workflows fail closed when required credentials or inventory checks are unavailable.

## Supply chain

- Pin GitHub Actions by full commit SHA.
- Lock dependencies and verify licenses and advisories.
- Generate SBOM, provenance, checksums, and reproducible release evidence.
- Use least-privilege workflow permissions and concurrency controls.
- Reject mutable remote code, unverified installer scripts, and implicit network downloads in scientific tests.

## Scientific integrity as a security property

Parameter recovery, uncertainty coverage, temporal leakage prevention, multilingual invariance, group fairness, numerical parity, and causal-language restrictions are integrity boundaries. A result that silently violates them is treated as a security-relevant failure.
