# Retention, deletion, and legal hold (doctoring)

## Scope

Migration `0007` implements the persistence half of ADR 0009 / ADR 0013 lifecycle governance:

- retention is policy-driven per tenant, data class, and purpose;
- deletion is an append-only request, not a silent `DELETE` of identity tables;
- an active legal or contractual hold blocks completed deletion;
- a tombstone records an action digest without raw source text; and
- historical reproduction cannot restore tombstoned evidence from an ungoverned copy.

The increment does not claim GDPR, SOC 2, or CSAP sufficiency. Host/deployer authority remains outside TEPP.

## Authority

European Union. (2016). *Regulation (EU) 2016/679 of the European Parliament and of the Council of 27 April 2016 on the protection of natural persons with regard to the processing of personal data and on the free movement of such data (General Data Protection Regulation)*. Official Journal of the European Union, L 119, 1–88. https://eur-lex.europa.eu/eli/reg/2016/679/oj

National Institute of Standards and Technology. (2020). *NIST privacy framework: A tool for improving privacy through enterprise risk management, version 1.0*. https://doi.org/10.6028/NIST.CSWP.01162020

American Institute of Certified Public Accountants. (2023). *2017 Trust services criteria for security, availability, processing integrity, confidentiality, and privacy (with revised points of focus—2022)*. AICPA & CIMA.

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

GDPR Article 17 motivates erasure and restriction workflows, including the possibility that legal claims override erasure (European Union, 2016). The NIST Privacy Framework treats data processing, inventory, and disassociated/deleted data as risk-management outcomes rather than a single delete API (National Institute of Standards and Technology, 2020). AICPA privacy criteria require retention, disposal, and exception handling to be authorized and evidenced (American Institute of Certified Public Accountants, 2023). Bitemporal identity tables stay append-only so revocation is a later assertion, not a rewrite of valid/system history (Jensen & Snodgrass, 1999; National Academies of Sciences, Engineering, and Medicine, 2019).

## Verification

Defined deterministic verification includes:

- catalog validation requiring the four lifecycle tables, hold/restore functions, triggers, period/scope constraints, policy-succession contract, fixed trigger search paths, and tenant-session guards;
- unit/contract tests refusing hostile labels, non-positive periods, mismatched hold scope, completed deletion under an active hold, invalid tombstone digests, raw-source reproduction override, invalid policy successors, and deletion requests whose tenant/class/purpose do not match the cited retention policy; and
- an exact-head live PostgreSQL procedure that is intended to insert a hold, prove application/database completion refusal as `LegalHoldBlocksDeletion`, persist a `blocked_by_hold` request, supersede one active retention policy atomically, complete deletion of an unheld document, write a tombstone with `unavailable` reproduction (never the forbidden `available` token), and prove restore and analysis eligibility fail closed for `logical_revocation`/`identity_tombstone` only.

The live procedure is test code, not completed execution evidence. Its result remains pending until the unchanged PR head's live PostgreSQL job succeeds; queued, cancelled, predecessor-head, or local-only evidence does not promote this capability to implemented-main.