# Adopt authenticated central tool bootstrap

- Pin both TEPP reusable CI callers to corrected protected central supplier
  `3136c940235a3762b804bd99eb20fbe129e70c06`.
- Regenerate the six offline Git-object fixtures from that exact central commit
  and update caller/workflow SHA-256 bindings and the signed-object attribute.
- Replace obsolete cached-tool restore labels with actual private-root locked
  install argv and the correct LLVM plugin version probe; preserve pin, coverage,
  PostgreSQL, five-gate and corruption/ref/allowlist rejection policy.
- Private Python environments, exclusive Cargo/Rustup namespaces and hash-verified
  Rustup are supplier-owned; no cached executable restore or cache-hit install skip.
- This source adoption does not establish native bootstrap, hosted CI, runner
  access, GitHub App approval, or release acceptance. Rust/library source and
  numerical thresholds are unchanged.
