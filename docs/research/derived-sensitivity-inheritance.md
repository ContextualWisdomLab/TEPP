# Derived sensitivity inheritance (doctoring)

## Scope

`derived_sensitivity` keeps topic, factor, and relation artifacts in the
source sensitivity class. Unknown kind codes fail closed. Derivation is
not declassification to public.
Blanket PII masking is not a declassification grant. Recovery is the
computed share of inherited classes that match known truth.

This slice does not persist classifications, score topics, or claim CSAP,
SOC 2, or legal sufficiency.

## Authority

### Normative TEPP contract

- `docs/adr/0009-purpose-bound-pii-governance.md` — derived
  relation/topic/factor data is not automatically non-sensitive.
- `docs/PRIVACY_DATA_GOVERNANCE.md` — derived-sensitive-data
  classification is a required privacy test.

### Supporting literature (read before this increment)

European Union. (2016). *Regulation (EU) 2016/679 of the European Parliament
and of the Council of 27 April 2016 on the protection of natural persons with
regard to the processing of personal data and on the free movement of such
data (General Data Protection Regulation)*. *Official Journal of the European
Union, L 119*, 1–88. https://eur-lex.europa.eu/eli/reg/2016/679/oj

Article 29 Data Protection Working Party. (2007). *Opinion 4/2007 on the
concept of personal data* (WP 136). European Commission.
https://ec.europa.eu/justice/article-29/documentation/opinion-recommendation/files/2007/wp136_en.pdf

International Organization for Standardization and International
Electrotechnical Commission. (2024). *Information technology—Security
techniques—Privacy framework* (ISO/IEC Standard No. 29100:2024).
https://www.iso.org/standard/85938.html

GDPR Article 4(1) defines personal data as *any information relating to* an
identified or identifiable natural person, including identification by
reference to factors specific to that person (European Union, 2016). Recital
26 states that the principles apply to any such information and that
pseudonymised data that can be attributed with additional information remain
data on an identifiable person; only information that cannot identify a
person is outside the principles (European Union, 2016). This increment
therefore cannot treat a topic, factor, or relation artifact as public merely
because it is computed.

WP 136 is the Article 29 Working Party's reading of that “relating to”
element. Information relates to a person when a content, purpose, *or* result
element is present; the three are alternative, not cumulative (Article 29
Data Protection Working Party, 2007, pp. 10–12). The opinion's medical-analysis
example treats *results* as relating to the patient, and the taxi-location
example treats derived monitoring that can change how a person is treated as
still relating to that person (Article 29 Data Protection Working Party,
2007). TEPP uses that test for inherited sensitivity: a topic proportion,
factor score, or relation edge that evaluates or can affect a source subject
keeps the source class. The opinion does not authorize a blanket mask as
declassification, and it does not certify TEPP.

ISO/IEC 29100:2024 is the current published privacy-framework edition; the
2011 edition and its 2018 amendment are withdrawn (International Organization
for Standardization & International Electrotechnical Commission, 2024). The
official catalogue abstract states that 29100 specifies privacy terminology,
actors/roles for PII processing, safeguarding considerations, and references
to known privacy principles. This increment cites that current edition as the
ICT privacy-framework pointer. It does **not** quote unread 2024 clause text
and does not treat the catalogue abstract as a declassification rule.

Local Zotero was not reachable in this environment (`127.0.0.1:23119`).

## Application

`inherit_sensitivity` and `DerivedArtifact::try_new` copy the source class
onto a closed kind (`topic`/`factor`/`relation`) and fail closed on unknown
kinds. `refuse_derivation_as_public` and
`refuse_blanket_mask_as_declassification` encode the WP 136 / GDPR reading
that computation and masking are not independent declassification decisions
(Article 29 Data Protection Working Party, 2007; European Union, 2016).
Recovery is a computed match rate of paired kind **and** class against known
truth, not an LLM judgment and not a class-only score that would treat a
factor as a recovered topic.

## Verification

- restricted/internal sources stay restricted/internal after inheritance;
- unknown kind codes fail closed on inheritance and on `try_new`;
- every topic/factor/relation × Restricted/Internal/Public pair recovers at
  rate 1.0; public collapse recovers only the three Public sources (1/3);
- reordering paired records preserves the recovery rate;
- a matching class on the wrong kind is not recovery;
- empty or length-mismatched recovery payloads fail closed.
