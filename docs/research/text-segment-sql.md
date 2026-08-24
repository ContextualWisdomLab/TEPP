# Typed text-segment SQL (doctoring)

## Scope

Call `insert_text_segment` when a document must expose an exact observed
unit for membership, mention, or later semantic work. The adapter writes
the existing `0006` `text_segment` row: half-open UTF-8 byte offsets
`[start_byte, end_byte)`, tenant, document identity, system time, and
availability time. Historical reads use
`select_text_segments_for_document_as_of_sql` so a span whose
`available_time` is after the declared cutoff cannot enter a fit
(Jensen & Snodgrass, 1999).

This slice does **not** allocate `0007` or `0008`. It does not add the
accepted ERD foreign key from `text_segment.document_record_id` to
`document_record`, Unicode scalar columns, or `segment_type_code`. Those
remain later migrations.

## Authority

Exact character or byte positions are the durable way to point at a
substring without copying source text into every membership or mention
row (Bird & Liberman, 2001; Wilde & Duerst, 2008). Unicode text
segmentation defines language-appropriate units; TEPP stores the
resulting UTF-8 byte interval rather than a token string (Davis et al.,
2024). Availability versus cutoff is the historical-inclusion rule
already used for documents and splits (Jensen & Snodgrass, 1999).

Bird, S., & Liberman, M. (2001). A formal framework for linguistic
annotation. *Speech Communication, 33*(1–2), 23–60.
https://doi.org/10.1016/S0167-6393(00)00068-6

Wilde, E., & Duerst, M. (2008). *URI fragment identifiers for the
text/plain media type* (RFC 5147). Internet Engineering Task Force.
https://doi.org/10.17487/RFC5147

Davis, M., Iancu, L., & Whistler, K. (Eds.). (2024). *Unicode Standard
Annex #29: Unicode text segmentation*. Unicode Consortium.

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management.
*IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

## Verification

- contract tests recover the known `hello` span `[0, 5)` from
  `hello world` and refuse inverted, empty, and negative spans before
  SQL is rendered;
- cutoff selection SQL binds `available_time <= knowledge_cutoff`;
- live PostgreSQL CI inserts the known span, refuses an inverted
  adapter write, and proves the later `world` span is excluded at a
  February cutoff when `TEPP_LIVE_POSTGRES=1`.
