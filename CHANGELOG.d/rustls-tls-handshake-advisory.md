# rustls TLS 1.3 handshake advisory repair

- The workspace `rustls` pin moves from `=0.23.43` to `=0.23.45`, clearing RUSTSEC-2026-0285 / GHSA-2mjx-qc3c-rqvc, whose affected TLS 1.3 path could accept handshake messages across encryption-level boundaries.
- The pin reaches TEPP through `service_tls` directly and through `sqlx-core`, so this is a repository-wide dependency-policy prerequisite rather than a leaf TLS-only change.
- `CVE-2025-61730` is intentionally not attributed to `rustls`; the RustSec advisory mentions it only as the functionally similar Go `crypto/tls` issue.
- This fragment preserves the security release note while #526 remains stacked on #523; the surviving root changelog can absorb it after the parent lands without carrying the former incorrect CVE attribution.
