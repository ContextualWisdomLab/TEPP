# Append-only immutability triggers (doctoring)

## Scope

Migration `0004` adds defense-in-depth so identity and manifest tables cannot be rewritten after insert: least-privilege REVOKE of UPDATE/DELETE for `tepp_app_runtime`, plus BEFORE UPDATE OR DELETE triggers calling `reject_append_only_mutation`.

## Authority

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

ISO/IEC. (2011). *ISO/IEC 9075-2:2011 Information technology — Database languages — SQL — Part 2: Foundation (SQL/Foundation)*. International Organization for Standardization.

## Verification

- catalog validation requires function, multi-word triggers, and REVOKE statements for each identity table;
- live PostgreSQL CI proves UPDATE/DELETE on `reproducibility_manifest` fail closed when `TEPP_LIVE_POSTGRES=1`.
