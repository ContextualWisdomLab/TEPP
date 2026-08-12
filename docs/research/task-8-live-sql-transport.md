# Live SQL transport contracts (persistence follow-on)

## Scope

Extends Task 8 / ADR 0013 with:

1. `SqlSession` transport trait and recording offline implementation;
2. migration SQL statement splitting and ordered batch application;
3. parameterized document/audit SQL rendering for bitemporal tables;
4. `LiveDocumentRepository` over any `SqlSession`;
5. fail-closed `DATABASE_URL` configuration gate for future `SQLx` pool wiring.

A live PostgreSQL process is not required in CI. Pool construction is the remaining optional driver step.

## Authority

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

ISO/IEC. (2011). *ISO/IEC 9075-2:2011 Information technology — Database languages — SQL — Part 2: Foundation (SQL/Foundation)*. International Organization for Standardization.

## Verification

- unit tests for URL validation, empty batches, statement splitting, recording sessions, migration apply, insert/revise/audit SQL, and digest fail-closed paths;
- workspace line/branch coverage must remain complete.
