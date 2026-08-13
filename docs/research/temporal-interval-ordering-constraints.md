# Temporal interval ordering constraints (doctoring)

## Scope

Migration `0005` installs physical CHECK constraints that reject inverted temporal
windows on foundation tables while preserving open-ended and point intervals:

- `valid_to IS NULL OR valid_from <= valid_to` on `document_record`,
  `event_instance`, and `membership_assignment`;
- `system_to IS NULL OR system_from <= system_to` on bitemporal version tables; and
- `revision_number > 0` on `document_record`.

Equal bounds remain legal so instantaneous (point) intervals are not coerced
away. A later, more expressive physical design may replace these columns with
non-empty `tstzrange` windows per the ERD without relaxing the
definitely-later-start prohibition.

## Authority

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE
Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

Allen, J. F. (1983). Maintaining knowledge about temporal intervals.
*Communications of the ACM, 26*(11), 832–843.
https://doi.org/10.1145/182.358434

International Organization for Standardization. (2011). *Information
technology—Database languages—SQL—Part 2: Foundation (SQL/Foundation)*
(ISO/IEC Standard No. 9075-2:2011).

## Verification

- embedded catalog validation requires every multi-word constraint name and the
  open-ended/point-preserving predicates;
- fail-closed unit tests exercise missing constraint names and incomplete
  predicates; and
- live PostgreSQL CI proves inverted `valid_from`/`valid_to` and
  non-positive `revision_number` inserts fail closed when
  `TEPP_LIVE_POSTGRES=1`.
