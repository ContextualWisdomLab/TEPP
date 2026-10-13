# Adopt authenticated central tool bootstrap

- Pin both TEPP reusable CI callers to central supplier commit
  `af81ef49683809ce8b302e078455e3960cf0acd1`.
- Regenerate the six offline Git-object fixtures from that exact central commit
  and update caller/workflow SHA-256 bindings and the signed-object attribute.
- Preserve five-gate custody and corruption/ref/allowlist rejection controls;
  private Python environments and hash-verified Rustup are supplier-owned.
- This source adoption does not establish native bootstrap, hosted CI, runner
  access, GitHub App approval, or release acceptance. Rust/library source is
  unchanged.
